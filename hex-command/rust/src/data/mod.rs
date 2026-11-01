//! Catalog models and loading shared with the build-time compiler.
pub use hex_command_catalog::data::*;

#[cfg(feature = "wasm")]
extern crate alloc;
#[cfg(feature = "wasm")]
include!(concat!(env!("OUT_DIR"), "/catalog_baked.rs"));

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Unit {
    pub id: u32,
    pub kind: usize,
    pub side: Side,
    pub tile: usize,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    pub experience: Option<u32>,
    pub name: Option<std::rc::Rc<str>>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
