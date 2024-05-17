mod net;
mod nvs;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    net::connect()?.stop()?;

    Ok(())
}
