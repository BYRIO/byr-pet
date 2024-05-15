use core::convert::TryInto;
use std::{
    net::{Ipv4Addr, UdpSocket},
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use embedded_svc::http::Headers;

use esp_idf_hal::delay;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::{server::EspHttpServer, Method},
    io::Write,
    ipv4::{Mask, Subnet},
    nvs::EspDefaultNvsPartition,
    wifi::{
        self, AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, EspWifi,
    },
};
use esp_idf_svc::{
    hal::prelude::Peripherals,
    ipv4::{self, RouterConfiguration},
    netif::{EspNetif, NetifConfiguration, NetifStack},
};

use log::*;

const STACK_SIZE: usize = 10240;
const SSID: &str = "BYR-pet";
// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 71, 1);

pub fn main() -> anyhow::Result<Box<EspWifi<'static>>> {
    let wifi = setup_ap()?;

    let mut dns = DnsServer::new(IP);
    dns.start()?;

    let mut http = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        uri_match_wildcard: true,
        ..Default::default()
    })?;

    // Here we use a counter to simulate the provisioning process
    let count = Arc::new((Mutex::new(0), Condvar::new()));
    let count1 = Arc::clone(&count);

    http.fn_handler::<anyhow::Error, _>("/", Method::Get, move |req| {
        log::info!("HTTP GET {}{}", req.host().unwrap_or("Unknown"), req.uri());
        if req.host() != Some(format!("{}", IP).as_str()) {
            req.into_response(
                302,
                None,
                &[("Location", format!("http://{}/", IP).as_str())],
            )?;
        } else {
            let (lock, cvar) = &*count1;
            let mut counter = lock.lock().unwrap();
            *counter += 1;
            if *counter >= 10 {
                cvar.notify_all();
            } else {
                log::info!("Counter: {}/10, refresh the page to continue", *counter);
            }
            req.into_ok_response()?.write_all(
                format!(
                    "<DOCTYPE html>
                <html>
                    <head>
                        <title>BYR-pet</title>
                        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
                    </head>
                    <body>
                        <h1>Welcome to BYR-pet</h1>
                        <p>Count: {}/10</p>
                    </body>
                </html>",
                    *counter
                )
                .as_bytes(),
            )?
        }
        Ok(())
    })?;

    http.fn_handler::<anyhow::Error, _>("*", Method::Get, |req| {
        log::info!("HTTP GET {}{}", req.host().unwrap_or("Unknown"), req.uri());
        req.into_response(
            302,
            None,
            &[("Location", format!("http://{}/", IP).as_str())],
        )?;
        Ok(())
    })?;

    log::info!("Now visit http://{} 10 times to continue", IP);
    let (lock, cvar) = &*count;
    let counter = lock.lock().unwrap();
    let _c = cvar.wait(counter).unwrap();
    log::info!("Count reached 10");

    Ok(wifi)
}

fn setup_ap() -> anyhow::Result<Box<EspWifi<'static>>> {
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

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
