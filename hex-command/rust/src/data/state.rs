//! Owned battle snapshots with the established save and transport field shapes.

use super::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static ROAD_DEAD_ENDS: LazyLock<Vec<(u8, u8)>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../pixel/art/terrain/road/dead-ends.json"
    ))
    .expect("registered native road caps")
});

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
    /// Road/village exits or opposite crossing edges; artwork only.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<HexDirection>>,
    /// Short road spurs capped inside this tile rather than meeting another tile.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, rename = "deadEnds", skip_serializing_if = "Option::is_none")]
    pub dead_ends: Option<Vec<HexDirection>>,
    /// Reusable ground palette; independent of terrain and its gameplay rules.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// Condition that clears this tile's authored fog.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog: Option<ConditionRule>,
}
impl Tile {
    pub fn valid_connections(&self) -> bool {
        let Some(exits) = &self.connections else {
            return self.dead_ends.is_none();
        };
        let mask = exits.iter().fold(0u8, |mask, exit| mask | exit.bit());
        if mask == 0 || mask.count_ones() as usize != exits.len() {
            return false;
        }
        if let Some(ends) = &self.dead_ends {
            let capped = ends.iter().fold(0u8, |mask, end| mask | end.bit());
            if self.terrain != "road"
                || capped == 0
                || capped.count_ones() as usize != ends.len()
                || capped & mask != 0
                || !ROAD_DEAD_ENDS.contains(&(mask, capped))
            {
                return false;
            }
        }
        match self.terrain.as_str() {
            "road" | "signpost" => true,
            "village" => mask.count_ones() <= 3,
            "bridge" | "ford" => matches!(mask, 9 | 18 | 36),
            "landing" | "ferry" => matches!(mask, 1 | 8),
            _ => false,
        }
    }
}
/// An item left on a battlefield tile. Ownership passes only when collected.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundItem {
    pub tile: String,
    pub item: Item,
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
    pub factions: Vec<Faction>,
    pub ground_items: Vec<GroundItem>,
    /// Selected skirmish factions; campaigns use their authored recruitment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub armies: Option<[String; 2]>,
    /// Durable facts about conditions that have been satisfied at least once.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub condition_memory: Vec<String>,
    pub map_id: String,
    pub map_revision: u32,
    pub cols: i32,
    pub rows: i32,
    pub round: u32,
    pub turn: Side,
    pub gold: Gold,
    pub winner: Option<Side>,
    pub next_id: u32,
    /// Persisted combat PRNG state; absent legacy saves start from a stable seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub combat_rng: Option<u32>,
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
