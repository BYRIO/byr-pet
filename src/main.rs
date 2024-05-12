mod nvs;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct Foo {
    a: u32,
    b: f64,
    c: String,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    match nvs::load::<Foo>()? {
        Some(data) => {
            log::info!("Loaded data: {:?}", data);
            nvs::remove::<Foo>()?;
            log::info!("Removed data");
        }
        None => {
            log::info!("No data found");
            let data = Foo {
                a: 42,
                b: 3.14,
                c: "Hello, NVS!".into(),
            };
            nvs::save(data)?;
            log::info!("Saved data");
        }
    }

    Ok(())
}
