//! Owned battle snapshots with the established save and transport field shapes.

use super::*;
use serde::{Deserialize, Serialize};

/// Clockwise edge names for a pointy-top hex. These describe artwork only.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HexDirection {
    E,
    Se,
    Sw,
    W,
    Nw,
    Ne,
}
impl HexDirection {
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A tile snapshot combining authored coordinates with current terrain and ownership.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub id: String,
    pub col: i32,
    pub row: i32,
    pub terrain: String,
    pub owner: Option<Side>,
    /// Decorative road exits on plains, or the two opposite crossing edges.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<HexDirection>>,
}
impl Tile {
    pub fn valid_connections(&self) -> bool {
        let Some(exits) = &self.connections else {
            return true;
        };
        let mask = exits.iter().fold(0u8, |mask, exit| mask | exit.bit());
        if mask == 0 || mask.count_ones() as usize != exits.len() {
            return false;
        }
        match self.terrain.as_str() {
            "plains" => true,
            "bridge" | "ford" => matches!(mask, 9 | 18 | 36),
            _ => false,
        }
    }
}
/// Treasury balances for the player and enemy.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gold {
    pub player: i32,
    pub enemy: i32,
}
/// A complete owned battle snapshot suitable for persistence and transport.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub map_id: String,
    pub map_revision: u32,
    pub cols: i32,
    pub rows: i32,
    pub round: u32,
    pub turn: Side,
    pub gold: Gold,
    pub winner: Option<Side>,
    pub next_id: u32,
    pub log: Vec<String>,
    pub tiles: Vec<Tile>,
    pub units: Vec<PublicUnit>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignState>,
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "wasm", tsify(optional, type = "LastCombat | null"))]
    pub last_combat: Option<Option<LastCombat>>,
}
fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<LastCombat>>, D::Error> {
    Option::<LastCombat>::deserialize(deserializer).map(Some)
}
