use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattlefieldLighting {
    pub unlit_tiles: Vec<Hex>,
    pub lights: Vec<BattlefieldLight>,
}
/// Resolved current presentation; sent with views, never persisted as a cache.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattlefieldPresentation {
    pub lighting: BattlefieldLighting,
    pub fog: Vec<Hex>,
    /// Engine-resolved unit visibility, including concealment on visible terrain.
    pub visible_units: Vec<Text>,
    pub unit_appearances: Vec<UnitAppearance>,
}

pub trait TileLightPlacement {
    fn at(&self, col: i32, row: i32) -> BattlefieldLight;
}
impl TileLightPlacement for TileLight {
    fn at(&self, col: i32, row: i32) -> BattlefieldLight {
        BattlefieldLight {
            color: self.color,
            fixture: self.fixture,
            col,
            row,
            dx: self.dx,
            dy: self.dy,
            radius: self.radius,
            power: self.power,
            when: None,
        }
    }
}
