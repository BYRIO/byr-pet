use twox_hash::XxHash64;
use std::hash::Hasher;

fn hash_type<T>() -> String {
    let mut hasher = XxHash64::default();
    hasher.write(std::any::type_name::<T>().as_bytes());
    let hash = hasher.finish();
    let hash = hash.to_string();
    (&hash[..15]).to_string()
}

fn _save<T>(data: T, key: Option<&str>) -> anyhow::Result<()>
where
    T: serde::Serialize + for<'a> serde::de::Deserialize<'a> ,
{
    let mut storage = esp_idf_svc::nvs::EspNvs::new(
        esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
        &hash_type::<T>(),
        true,
    )?;
    let encoded = bincode::serialize(&data)?;
    storage.set_blob(key.unwrap_or("__default"), &encoded)?;
    Ok(())
}

fn _load<T>(key: Option<&str>) -> anyhow::Result<Option<T>>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> ,
{
    match esp_idf_svc::nvs::EspNvs::new(
        esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
        &hash_type::<T>(),
        false,
    ) {
        Ok(storage) => {
            let key = key.unwrap_or("__default");
            match storage.blob_len(key)? {
                Some(len) => {
                    let mut buffer = vec![0u8; len];
                    match storage.get_blob(key, &mut buffer) {
                        Ok(Some(_)) => match bincode::deserialize::<T>(&buffer) {
                            Ok(data) => Ok(Some(data)),
                            Err(_) => Ok(None),
                        },
                        _ => Ok(None),
                    }
                }
                None => Ok(None),
            }
        }
        Err(err) => {
            if err.code() == esp_idf_svc::sys::ESP_ERR_NVS_NOT_FOUND {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
    }
}

fn _remove<T>(key: Option<&str>) -> anyhow::Result<bool>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> ,
{
    let mut storage = esp_idf_svc::nvs::EspNvs::new(
        esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
        &hash_type::<T>(),
        true,
    )?;
    Ok(storage.remove(key.unwrap_or("__default"))?)
}

#[allow(dead_code)]
pub fn load<T>() -> anyhow::Result<Option<T>>
where
    T: serde::Serialize + for<'a> serde::de::Deserialize<'a> ,
{
    _load(None)
}

#[allow(dead_code)]
pub fn save<T>(data: T) -> anyhow::Result<()>
where
    T: serde::Serialize + for<'a> serde::de::Deserialize<'a> ,
{
    _save(data, None)
}

#[allow(dead_code)]
pub fn load_from<T>(key: &str) -> anyhow::Result<Option<T>>
where
    T: serde::Serialize + for<'a> serde::de::Deserialize<'a> ,
{
    _load(Some(key))
}

#[allow(dead_code)]
pub fn save_to<T>(data: T, key: &str) -> anyhow::Result<()>
where
    T: serde::Serialize + for<'a> serde::de::Deserialize<'a> ,
{
    _save(data, Some(key))
}

#[allow(dead_code)]
pub fn remove<T>() -> anyhow::Result<bool>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> ,
{
    _remove::<T>(None)
}

#[allow(dead_code)]
pub fn remove_from<T>(key: &str) -> anyhow::Result<bool>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> ,
{
    _remove::<T>(Some(key))
}
