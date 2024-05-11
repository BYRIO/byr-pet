
use anyhow;
use esp_idf_svc;
use storage;

#[derive(storage::NvsStorage, serde::Serialize, serde::Deserialize, Debug, Default)]
struct Bar {
    a: u32,
    b: f64,
    c: String,
}


fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    match Bar::load()? {
        Some(bar) => {
            log::info!("Data found: {:?}", bar);
            log::info!("Removing data...");
            Bar::remove()?;
            log::info!("Data removed");
        }
        None => {
            log::info!("No data found, saving default data...");
            Bar {
                a: 42,
                b: 3.14,
                c: "Hello, World!".to_string(),
            }
            .save()?;
            log::info!("Data saved");
        }
    }

    Ok(())
}
