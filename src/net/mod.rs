mod bupt;
mod provisioning;

use anyhow::{bail, Result};
use esp_idf_hal::delay;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{peripheral, prelude::Peripherals},
    log::set_target_level,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use std::fmt;

fn connect_wifi_with_config(
    config: NetConfig,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let nvs = EspDefaultNvsPartition::take()?;
    let mut bupt_account = None;
    let (auth_method, ssid, pass) = match config {
        NetConfig::BuptPortal(account) => {
            bupt_account = Some(account);
            (AuthMethod::None, "BUPT-portal".to_string(), String::new())
        }
        NetConfig::NormalWifi(wifi) => (AuthMethod::WPA2Personal, wifi.ssid, wifi.password),
    };
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;

    #[cfg(feature = "random_mac")]
    {
        use esp_idf_svc::wifi::WifiDeviceId;
        let mac = generate_random_mac();
        log::info!("Generated random MAC: {:02X?}", mac);
        esp_wifi.set_mac(WifiDeviceId::Sta, mac)?;
        log::info!(
            "Set MAC address to {:02X?}",
            esp_wifi.get_mac(WifiDeviceId::Sta)?
        );
    }

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_iter(ssid.chars()),
        password: heapless::String::<64>::from_iter(pass.chars()),
        channel: None,
        auth_method,
        ..Default::default()
    }))?;

    log::info!("Starting wifi...");
    wifi.start()?;
    log::info!("Connecting wifi {}...", ssid);
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
            bail!("Failed to connect to wifi");
        } else {
            log::info!("Retrying...");
        }
    }

    log::info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wifi DHCP info: {:?}", ip_info);

    if let Some(account) = bupt_account {
        for retry in 0..10 {
            match bupt::login(account.clone()) {
                Ok(_) => break,
                Err(e) => {
                    log::warn!(
                        "Failed to login to BUPT-portal: {}, will retry after 10 seconds...",
                        e
                    );
                }
            }
            delay.delay_ms(1000 * 10);
            if retry == 9 {
                log::error!("Retry limit exceeded");
                bail!("Failed to login to BUPT-portal");
            } else {
                log::info!("Retrying...");
            }
        }
    }
    Ok(Box::new(esp_wifi))
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
    BuptPortal(bupt::BuptAccount),
    NormalWifi(Wifi),
}

#[cfg(feature = "random_mac")]
pub fn generate_random_mac() -> [u8; 6] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut mac = [0u8; 6];
    rng.fill(&mut mac);
    mac[0] &= 0xFE;
    mac[0] &= 0xFD;
    mac
}

pub fn connect() -> Result<Box<EspWifi<'static>>> {
    set_target_level("wifi", log::LevelFilter::Warn)?;
    set_target_level("wifi_init", log::LevelFilter::Warn)?;

    match crate::nvs::load::<NetConfig>()? {
        Some(config) => {
            log::info!("Loaded NetConfig: {:?}", &config);
            connect_wifi_with_config(
                config,
                Peripherals::take()?.modem,
                EspSystemEventLoop::take()?,
            )
        }
        None => provisioning::main(),
    }
}
