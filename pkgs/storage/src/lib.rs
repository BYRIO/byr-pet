//! # NVS Storage Macro
//!
//! `nvs_storage` crate provides a procedural macro `NvsStorage` for automatically implementing the
//! serialization and deserialization of structures to the NVS (non-volatile storage) on ESP32 hardware.
//!
//! This macro simplifies the process of saving, loading, and removing structured data from NVS, 
//! by automatically generating the necessary methods for any struct it is applied to.
//!
//! ## Prerequisites
//!
//! Before using the macro-generated methods, ensure that the following crates are included in your
//! project and properly imported where the macro is used:
//!
//! ```rust
//! use anyhow; // For error handling with anyhow::Result
//! use esp_idf_svc; // For accessing NVS functionalities
//! ```
//!
//! These imports are crucial for the macro's generated methods to compile and function properly.
//!
//! ## Usage
//!
//! Apply the `NvsStorage` macro to any struct that you want to store in NVS. Make sure all the fields
//! in the struct can be serialized and deserialized by serde.
//!
//! ### Example
//!
//! Below is an example demonstrating how to define a struct and use the generated methods to 
//! manipulate data in NVS:
//!
//! ```rust
//! use nvs_storage::NvsStorage;
//! use serde::{Serialize, Deserialize};
//! use anyhow;
//! use esp_idf_svc;
//!
//! #[derive(Serialize, Deserialize, NvsStorage, Debug, Default)]
//! struct MyData {
//!     a: u32,
//!     b: f64,
//!     c: String,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let data = MyData { a: 42, b: 3.14, c: "Hello, NVS!".into() };
//!     // Save data to the default key
//!     data.save()?;
//!     // Load data from the default key
//!     let loaded_data = MyData::load()?;
//!     println!("Loaded data: {:?}", loaded_data);
//!
//!     // Save data to a custom key
//!     data.save_to("custom_key")?;
//!     // Load data from a custom key
//!     let custom_data = MyData::load_from("custom_key")?;
//!     println!("Loaded custom key data: {:?}", custom_data);
//!
//!     // Remove data from the default key
//!     MyData::remove()?;
//!     // Remove data from the custom key
//!     MyData::remove_from("custom_key")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! This example demonstrates basic operations: saving to and loading from both a default and a custom key,
//! as well as removing data. It shows how to apply the macro and use the methods it generates.


extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// This attribute macro generates methods to serialize and deserialize the marked struct
/// to and from NVS storage.
///
/// It automatically implements methods to save and load the instances of the struct using
/// bincode and ESP-IDF's NVS services.
#[proc_macro_derive(NvsStorage)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl #name {
            fn _save(&self, key: Option<&str>) -> anyhow::Result<()> {
                let mut storage = esp_idf_svc::nvs::EspNvs::new(
                    esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
                    stringify!(#name),
                    true
                )?;
                let encoded = bincode::serialize(self)?;
                storage.set_blob(key.unwrap_or("__default"), &encoded)?;
                Ok(())
            }

            fn _load(key: Option<&str>) -> anyhow::Result<Option<Self>>
            where
                Self: Sized,
            {
                let storage = esp_idf_svc::nvs::EspNvs::new(
                    esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
                    stringify!(#name),
                    false
                )?;
                let key = key.unwrap_or("__default");
                let mut buffer = vec![0u8; 100];
                Ok(match storage.get_blob(key, &mut buffer)? {
                    None => None,
                    Some(_) => Some(bincode::deserialize(&buffer)?)
                })
            }

            fn _remove(key: Option<&str>) -> anyhow::Result<bool> {
                let mut storage = esp_idf_svc::nvs::EspNvs::new(
                    esp_idf_svc::nvs::EspNvsPartition::<esp_idf_svc::nvs::NvsDefault>::take().unwrap(),
                    stringify!(#name),
                    true
                )?;
                Ok(storage.remove(key.unwrap_or("__default"))?)
            }

            fn load() -> anyhow::Result<Option<Self>>
            where
                Self: Sized,
            {
                Self::_load(None)
            }

            fn save(&self) -> anyhow::Result<()> {
                self._save(None)
            }

            fn load_from(key: &str) -> anyhow::Result<Option<Self>>
            where
                Self: Sized,
            {
                Self::_load(Some(key))
            }

            fn save_to(&self, key: &str) -> anyhow::Result<()> {
                self._save(Some(key))
            }

            fn remove() -> anyhow::Result<bool> {
                Self::_remove(None)
            }

            fn remove_from(key: &str) -> anyhow::Result<bool> {
                Self::_remove(Some(key))
            }
        }
    };

    TokenStream::from(expanded)
}