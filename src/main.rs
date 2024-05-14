mod net;
mod nvs;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut wifi = net::connect()?;
    std::thread::sleep(std::time::Duration::from_secs(60));
    wifi.stop()?;
    log::info!("Wifi stopped");

    Ok(())
}
