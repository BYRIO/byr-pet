
use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        prelude::Peripherals,
        peripheral
    },
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_hal::delay;
use log::info;
use std::fmt;

fn connect_wifi(
    config: NetConfig,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let (auth_method, ssid, pass) = match config {
        NetConfig::BuptPortal(_) => (AuthMethod::None, "BUPT-portal".to_string(), String::new()),
        NetConfig::NormalWifi(wifi) => (AuthMethod::WPA2Personal, wifi.ssid, wifi.password),
    };
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_iter(ssid.chars()),
        password: heapless::String::<64>::from_iter(pass.chars()),
        channel: None,
        auth_method,
        ..Default::default()
    }))?;

    info!("Starting wifi...");

    wifi.start()?;

    info!("Connecting wifi {}...", ssid);

    let delay: delay::Delay = Default::default();

    for retry in 0..10 {
        match wifi.connect() {
            Ok(_) => break,
            Err(e) => {
                info!("Failed to connect: {}, retrying...", e);
            }
        }
        delay.delay_ms(1000 * 10);
        if retry == 9 {
            bail!("Failed to connect to wifi");
        }
    }

    info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct BuptAccount {
    username: String,
    password: String,
}

impl fmt::Debug for BuptAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let password_length = self.password.len();
        let hidden_password = "*".repeat(password_length);
        f.debug_struct("BuptAccount")
            .field("username", &self.username)
            .field("password", &hidden_password)
            .finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Wifi {
    ssid: String,
    password: String,
}

impl fmt::Debug for Wifi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let password_length = self.password.len();
        let hidden_password = "*".repeat(password_length);
        f.debug_struct("Wifi")
            .field("ssid", &self.ssid)
            .field("password", &hidden_password)
            .finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
enum NetConfig {
    BuptPortal(BuptAccount),
    NormalWifi(Wifi),
}

pub fn connect() -> anyhow::Result<()> {
    let config: NetConfig = match crate::nvs::load::<NetConfig>()? {
        Some(config) => {
            log::info!("Loaded NetConfig: {:?}", &config);
            config
        }
        None => {
            unimplemented!()
        }
    };

    let _wifi = connect_wifi(
        config,
        Peripherals::take()?.modem,
        EspSystemEventLoop::take()?
    )?;
    
    Ok(())
}