//! Owned battle snapshots with the established save and transport field shapes.

use super::*;
use serde::{Deserialize, Serialize};
include!(concat!(env!("OUT_DIR"), "/terrain_geometry.rs"));

/// Clockwise edge names for a pointy-top hex. These describe artwork only.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum HexDirection {
    E,
    Se,
    Sw,
    W,
    Nw,
    Ne,
}
impl HexDirection {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A tile snapshot combining authored coordinates with current terrain and ownership.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
    /// Explicit complete wall edges; never inferred from neighbors.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walls: Option<Vec<HexDirection>>,
    /// Reusable ground palette; independent of terrain and its gameplay rules.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// Immutable condition shared by snapshots; Arc keeps native catalogs thread-safe.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog: Option<std::sync::Arc<ConditionRule>>,
}
impl Tile {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn valid_connections(&self) -> bool {
        if matches!(self.terrain.as_str(), "castle-wall" | "castle-bell") {
            if self.connections.is_some() || self.dead_ends.is_some() {
                return false;
            }
            let Some(walls) = &self.walls else {
                return false;
            };
            let mask = walls.iter().fold(0u8, |mask, edge| mask | edge.bit());
            if mask.count_ones() as usize != walls.len() {
                return false;
            }
            let variants = match self.terrain.as_str() {
                "castle-wall" => CASTLE_WALL_VARIANTS,
                "castle-bell" => CASTLE_BELL_VARIANTS,
                _ => unreachable!(),
            };
            return variants.contains(&mask);
        }
        if self.walls.is_some() {
            return false;
        }
        let Some(exits) = &self.connections else {
            return !matches!(
                self.terrain.as_str(),
                "signpost"
                    | "road"
                    | "village"
                    | "bridge"
                    | "ford"
                    | "landing"
                    | "ferry"
                    | "castle-wall"
                    | "castle-bell"
            ) && self.dead_ends.is_none();
        };
        let mask = exits.iter().fold(0u8, |mask, exit| mask | exit.bit());
        if (mask == 0
            && !matches!(
                self.terrain.as_str(),
                "village" | "castle-wall" | "castle-bell"
            ))
            || mask.count_ones() as usize != exits.len()
        {
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
            "road" => true,
            "signpost" => mask == 9,
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
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct GroundItem {
    pub tile: String,
    pub item: Item,
}

/// Treasury balances for the player and enemy.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Gold {
    pub player: i32,
    pub enemy: i32,
}
/// A complete owned battle snapshot suitable for persistence and transport.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<LastCombat>>, D::Error> {
    Option::<LastCombat>::deserialize(deserializer).map(Some)
}
