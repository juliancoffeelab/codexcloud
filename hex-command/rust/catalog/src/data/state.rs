//! Owned battle snapshots with the established save and transport field shapes.

use super::*;
use serde::{Deserialize, Serialize};
include!(concat!(env!("OUT_DIR"), "/terrain_geometry.rs"));

/// Clockwise edge names for a pointy-top hex. These describe artwork only.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Tile {
    /// Coordinate address (`col,row`), used by commands and snapshots.
    pub id: String,
    /// Optional authored landmark identity; ordinary cells have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub col: i32,
    pub row: i32,
    pub terrain: String,
    /// Owning faction identifier, independent of its turn phase.
    pub owner: Option<String>,
    /// Road/village exits or opposite crossing edges; artwork only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<HexDirection>>,
    /// Short road spurs capped inside this tile rather than meeting another tile.
    #[serde(default, rename = "deadEnds", skip_serializing_if = "Option::is_none")]
    pub dead_ends: Option<Vec<HexDirection>>,
    /// Explicit complete wall edges; never inferred from neighbors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walls: Option<Vec<HexDirection>>,
    /// Reusable ground palette; independent of terrain and its gameplay rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// Immutable condition shared by snapshots; Arc keeps native catalogs thread-safe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog: Option<std::sync::Arc<ConditionRule>>,
    /// Terrain light anchored to this tile, using the same contextual rules as fog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<std::sync::Arc<TileLight>>,
}
impl Tile {
    pub fn hex(&self) -> Hex {
        Hex::new(self.col, self.row)
    }
    pub fn matches_reference(&self, reference: &TileRef) -> bool {
        match reference {
            TileRef::Location(name) => self.location.as_ref() == Some(name),
            TileRef::Hex(hex) => (self.col, self.row) == (hex.col, hex.row),
            TileRef::Binding(_) => false,
        }
    }

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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct GroundItem {
    pub tile: Hex,
    pub item: Item,
}

/// Treasury balances keyed by faction identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Gold(pub std::sync::Arc<std::collections::BTreeMap<String, i32>>);
impl Gold {
    pub fn two(player: i32, enemy: i32) -> Self {
        Self(std::sync::Arc::new(
            [("player".into(), player), ("enemy".into(), enemy)].into(),
        ))
    }
}
impl std::ops::Deref for Gold {
    type Target = std::collections::BTreeMap<String, i32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Gold {
    fn deref_mut(&mut self) -> &mut Self::Target {
        std::sync::Arc::make_mut(&mut self.0)
    }
}
impl std::ops::Index<&str> for Gold {
    type Output = i32;
    fn index(&self, id: &str) -> &i32 {
        &self.0[id]
    }
}
impl std::ops::IndexMut<&str> for Gold {
    fn index_mut(&mut self, id: &str) -> &mut i32 {
        std::sync::Arc::make_mut(&mut self.0)
            .get_mut(id)
            .expect("validated faction treasury")
    }
}
/// A complete owned battle snapshot suitable for persistence and transport.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
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
    pub armies: Option<[String; 2]>,
    /// Durable facts about conditions that have been satisfied at least once.
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
    /// Complete persisted combat PRNG state.
    pub combat_rng: u32,
    pub log: Vec<String>,
    pub tiles: Vec<Tile>,
    pub units: Vec<PublicUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignState>,
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_combat: Option<Option<LastCombat>>,
}
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<LastCombat>>, D::Error> {
    Option::<LastCombat>::deserialize(deserializer).map(Some)
}
