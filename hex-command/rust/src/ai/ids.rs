//! Owned planner identifiers stay inline; public wire commands retain their IDs.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct TileKey(
    #[cfg_attr(feature = "wasm", tsify(type = "string"))] pub smallstr::SmallString<[u8; 16]>,
);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct TypeKey(
    #[cfg_attr(feature = "wasm", tsify(type = "string"))] pub smallstr::SmallString<[u8; 64]>,
);
pub type TileId = TileKey;
pub type UnitTypeId = TypeKey;
#[allow(non_snake_case)]
pub fn TileId(value: impl AsRef<str>) -> TileKey {
    TileKey(value.as_ref().into())
}
#[allow(non_snake_case)]
pub fn UnitTypeId(value: impl AsRef<str>) -> TypeKey {
    TypeKey(value.as_ref().into())
}
impl From<&str> for TileKey {
    fn from(value: &str) -> Self {
        TileId(value)
    }
}
impl From<String> for TileKey {
    fn from(value: String) -> Self {
        TileId(value)
    }
}
impl From<&str> for TypeKey {
    fn from(value: &str) -> Self {
        UnitTypeId(value)
    }
}
impl From<String> for TypeKey {
    fn from(value: String) -> Self {
        UnitTypeId(value)
    }
}
