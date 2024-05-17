mod net;
mod nvs;

const INDEX_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/frontend/index.html"));

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("{}", INDEX_HTML);

    net::connect()?.stop()?;

    Ok(())
}
