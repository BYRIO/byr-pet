use core::convert::TryInto;
use std::{
    net::{Ipv4Addr, UdpSocket},
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use serde_json::json;

use embedded_svc::http::Headers;

use esp_idf_hal::delay;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    http::{
        server::{EspHttpConnection, EspHttpServer, Request},
        Method,
    },
    io::Write,
    ipv4::{self, RouterConfiguration},
    ipv4::{Mask, Subnet},
    netif::{EspNetif, NetifConfiguration, NetifStack},
    wifi::{
        self, AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, EspWifi,
    },
};

use include_dir::{include_dir, Dir};

use log::*;

use crate::net::bupt;

static FRONTEND: Dir = include_dir!("$OUT_DIR/frontend");

const STACK_SIZE: usize = 10240;
const SSID: &str = "BYR-pet";
// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 71, 1);
const IP_STRING: &str = "192.168.71.1";

const MIME_TYPES: &[(&str, &str)] = &[
    ("html", "text/html"),
    ("js", "application/javascript"),
    ("css", "text/css"),
];

fn read_body_to_string(req: &mut Request<&mut EspHttpConnection>) -> anyhow::Result<String> {
    let mut body = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = req.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..bytes_read]);
    }

    Ok(String::from_utf8(body)?)
}

fn check_host_and_log<'a, 'b>(
    req: Request<&'a mut EspHttpConnection<'b>>,
) -> anyhow::Result<Option<Request<&'a mut EspHttpConnection<'b>>>> {
    log::info!(
        "HTTP {:?} - {}{}",
        req.method(),
        req.host().unwrap_or("Unknown"),
        req.uri()
    );
    if req.host() != Some(IP_STRING) {
        req.into_response(
            302,
            None,
            &[("Location", format!("http://{}/", IP_STRING).as_str())],
        )?;
        return Ok(None);
    }
    Ok(Some(req))
}

pub fn main() -> anyhow::Result<Box<EspWifi<'static>>> {
    let wifi = setup_ap()?;

    let mut dns = DnsServer::new(IP);
    dns.start()?;

    let mut http = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        uri_match_wildcard: true,
        ..Default::default()
    })?;

    let semaphore = Arc::new((Mutex::new(()), Condvar::new()));
    let semaphore1 = Arc::clone(&semaphore);

    http.fn_handler::<anyhow::Error, _>("/", Method::Get, |req| {
        if let Some(req) = check_host_and_log(req)? {
            match FRONTEND.get_file("index.html") {
                Some(file) => {
                    req.into_response(200, None, &[("Content-Type", "text/html")])?
                        .write_all(file.contents())?;
                }
                None => {
                    req.into_response(404, None, &[])?;
                }
            }
        }
        Ok(())
    })?;

    http.fn_handler::<anyhow::Error, _>("/login", Method::Post, move |req| {
        if let Some(mut req) = check_host_and_log(req)? {
            let body = read_body_to_string(&mut req)?;

            if req.header("Content-Type") == Some("application/x-www-form-urlencoded") {
                let mut username = None;
                let mut password = None;
                for pair in body.split('&') {
                    let mut pair = pair.split('=');
                    let key = pair.next().ok_or(anyhow::anyhow!("Invalid body"))?;
                    let value = pair.next().ok_or(anyhow::anyhow!("Invalid body"))?;
                    let key = urlencoding::decode(key)?;
                    let value = urlencoding::decode(value)?;
                    match key.as_ref() {
                        "username" => username = Some(value),
                        "password" => password = Some(value),
                        _ => {}
                    }
                }
                let config = bupt::BuptAccount {
                    username: username
                        .ok_or(anyhow::anyhow!("Missing username"))?
                        .to_string(),
                    password: password
                        .ok_or(anyhow::anyhow!("Missing password"))?
                        .to_string(),
                };
                match bupt::login(&config) {
                    Ok(_) => {
                        req.into_ok_response()?
                            .write_all(json!({"code": 0}).to_string().as_bytes())?;
                        let (_lock, cvar) = &*semaphore1;
                        crate::nvs::save(super::NetConfig::BuptPortal(config)).map_err(|x| {
                            log::error!("Failed to save account: {:?} / {}", x, x);
                            x
                        })?;
                        cvar.notify_all();
                    }
                    Err(e) => {
                        req.into_ok_response()?.write_all(
                            json!({"code": 1, "message": e.to_string()})
                                .to_string()
                                .as_bytes(),
                        )?;
                    }
                }
            } else {
                log::info!("Invalid Content-Type");
                req.into_response(400, None, &[])?;
            }
        }
        Ok(())
    })?;

    http.fn_handler::<anyhow::Error, _>("*", Method::Get, |req| {
        if let Some(req) = check_host_and_log(req)? {
            match FRONTEND.get_file(req.uri().trim_start_matches('/')) {
                Some(file) => {
                    let ext = req.uri().split('.').last().unwrap_or("");
                    let mime = MIME_TYPES
                        .iter()
                        .find(|(ext_, _)| ext == *ext_)
                        .map(|(_, mime)| *mime)
                        .unwrap_or("application/octet-stream");
                    req.into_response(200, None, &[("Content-Type", mime)])?
                        .write_all(file.contents())?;
                }
                None => {
                    req.into_response(404, None, &[])?;
                }
            }
        }
        Ok(())
    })?;

    log::info!("Now visit http://{} to login", IP);
    let (lock, cvar) = &*semaphore;
    drop(cvar.wait(lock.lock().unwrap()).unwrap());

    Ok(wifi)
}

fn setup_ap() -> anyhow::Result<Box<EspWifi<'static>>> {
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = crate::nvs::nvs();

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;

    esp_wifi.swap_netif_ap(EspNetif::new_with_conf(&NetifConfiguration {
        key: "WIFI_AP_DEF_BYR_PET".try_into().unwrap(),
        description: "ap".try_into().unwrap(),
        route_priority: 10,
        ip_configuration: ipv4::Configuration::Router(RouterConfiguration {
            subnet: Subnet {
                gateway: IP,
                mask: Mask(24),
            },
            dhcp_enabled: true,
            dns: Some(IP),
            ..Default::default()
        }),
        stack: NetifStack::Ap,
        custom_mac: None,
    })?)?;

    #[cfg(feature = "random_mac")]
    {
        use esp_idf_svc::wifi::WifiDeviceId;
        let mac = super::generate_random_mac();
        log::info!("Generated random MAC: {:02X?}", mac);
        esp_wifi.set_mac(WifiDeviceId::Sta, mac)?;
        log::info!(
            "Set MAC address to {:02X?}",
            esp_wifi.get_mac(WifiDeviceId::Sta)?
        );
    }

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop)?;

    let wifi_configuration = wifi::Configuration::Mixed(
        ClientConfiguration {
            ssid: heapless::String::<32>::try_from("BUPT-portal").unwrap(),
            auth_method: AuthMethod::None,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: SSID.try_into().unwrap(),
            auth_method: AuthMethod::None,
            channel: CHANNEL,
            ..Default::default()
        },
    );
    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;

    let delay: delay::Delay = Default::default();
    for retry in 0..10 {
        match wifi.connect() {
            Ok(_) => break,
            Err(e) => {
                log::warn!(
                    "Failed to connect wifi: {}, will retry after 10 seconds...",
                    e
                );
            }
        }
        delay.delay_ms(1000 * 10);
        if retry == 9 {
            log::error!("Retry limit exceeded");
            anyhow::bail!("Failed to connect to wifi");
        } else {
            log::info!("Retrying...");
        }
    }
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Connected to BUPT-portal: {:?}", ip_info);
    info!("Created Wi-Fi with WIFI_SSID `{}`", SSID);

    Ok(Box::new(esp_wifi))
}

const DNS_MAX_LEN: usize = 512;

struct DnsServer {
    ip: Ipv4Addr,
    handle: Option<thread::JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl DnsServer {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self {
            ip,
            handle: None,
            stop_tx: None,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel();
        self.stop_tx = Some(tx);

        let ip = self.ip;
        log::info!("Dns Server started");

        let handle = thread::spawn(move || {
            let udp_server = UdpSocket::bind("0.0.0.0:53").expect("Could not bind to address");
            udp_server
                .set_read_timeout(Some(Duration::from_secs(1)))
                .expect("Could not set read timeout");
            let mut buf = [0u8; DNS_MAX_LEN];

            loop {
                if rx.try_recv().is_ok() {
                    break;
                }

                match udp_server.recv_from(&mut buf) {
                    Ok((len, addr)) if len >= 12 => {
                        log::debug!("Received UDP packet from {}", addr);
                        let mut response = Vec::with_capacity(DNS_MAX_LEN);
                        response.extend_from_slice(&buf[..len]);
                        response[2] |= 0x80;
                        response[3] |= 0x80;

                        // Type A, Class IN
                        if buf[len - 4..len] == [0x00, 0x01, 0x00, 0x01] {
                            log::debug!("DNS A query from {}", addr);
                            response[7] = 1;
                            response.extend_from_slice(&[
                                0xc0,
                                0x0c, // Name (pointer to the domain name in the question section)
                                0x00,
                                0x01, // Type (A)
                                0x00,
                                0x01, // Class (IN)
                                0x00,
                                0x00,
                                0x00,
                                0x0A, // TTL (10 seconds)
                                0x00,
                                0x04, // Data length (4 bytes for IPv4 address)
                                ip.octets()[0],
                                ip.octets()[1],
                                ip.octets()[2],
                                ip.octets()[3], // IP address
                            ]);
                        } else {
                            response[7] = 0;
                        }

                        udp_server
                            .send_to(&response, addr)
                            .expect("Failed to send response");
                    }
                    Ok(_) => {
                        log::warn!("Received UDP packet with invalid length");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
            }
        });

        self.handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) {
        if self.stop_tx.is_none() {
            return;
        }
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        log::info!("Dns Server stopped");
        self.handle = None;
    }
}

impl Drop for DnsServer {
    fn drop(&mut self) {
        self.stop();
    }
}
