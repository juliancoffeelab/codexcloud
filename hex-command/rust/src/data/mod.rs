//! Shared current campaign catalog and transfer vocabulary.
pub use hex_command_catalog::data::*;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
extern crate alloc;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
include!(concat!(env!("OUT_DIR"), "/catalog_baked.rs"));
