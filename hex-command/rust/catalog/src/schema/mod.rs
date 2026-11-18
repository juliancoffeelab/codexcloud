//! Private authored representation; exported vocabulary is explicitly selected at crate root.
mod abilities;
pub use abilities::*;
mod characters;
pub use characters::*;
mod conditions;
pub use conditions::*;
mod definitions;
pub use definitions::*;
mod effects;
pub use effects::*;
mod environment;
pub use environment::*;
mod events;
pub use events::*;
mod factions;
pub use factions::*;
mod geometry;
pub use geometry::*;
mod ids;
pub use ids::*;
mod lighting;
pub use lighting::*;
mod modifiers;
pub use modifiers::*;
mod perception;
pub use perception::*;
mod policy;
pub use policy::*;
mod scenario;
pub use scenario::*;
mod units;
pub use units::*;

pub(crate) fn explicit_option<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    serde::Deserialize::deserialize(d)
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(js_value("string")))]
pub type Text = String;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(js_value("string")))]
pub type TextSlice = str;
