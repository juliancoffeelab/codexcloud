use super::*;
use serde::{Deserialize, Serialize};
/// Stable unit identity independent of roster order.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitId(pub u32);
/// Geometric command address, separate from authored references.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TileId(pub Hex);
impl From<Hex> for TileId {
    fn from(value: Hex) -> Self {
        Self(value)
    }
}
/// Authored weapon identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponId(pub Text);
impl From<&str> for WeaponId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Kind of objective selected by the planner.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveKind {
    Capture,
    Defeat,
    Visit,
    Escort,
    Escape,
    Vigil,
}
impl ObjectiveKind {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Defeat => "defeat",
            Self::Visit => "visit",
            Self::Escort => "escort",
            Self::Escape => "escape",
            Self::Vigil => "vigil",
        }
    }
}
