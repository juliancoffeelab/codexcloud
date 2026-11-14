//! Neutral battle observations shared by native and browser hosts.
use crate::data::*;
use serde::{Deserialize, Serialize};

/// Per-command state fields. Terrain is carried separately and only when changed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "wasm", derive(hex_catalog_freeze_derive::BorrowFields))]
#[cfg_attr(feature = "wasm", borrow_from(crate::data::State))]
pub struct StateFrame {
    pub vision_groups: Vec<VisionGroupState>,
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
    pub turn: TurnPhase,
    pub gold: Gold,
    pub winner: Option<BattleResult>,
    pub next_id: u32,
    /// Complete persisted combat PRNG state.
    pub combat_rng: u32,
    pub log: Vec<String>,
    pub units: Vec<PublicUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignState>,
    #[serde(
        default,
        deserialize_with = "crate::data::present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_combat: Option<Option<LastCombat>>,
}
impl StateFrame {
    /// Exhaustive decomposition keeps new state fields from silently disappearing
    /// from the browser publication when the core save contract evolves.
    pub fn split(state: State) -> (Self, Vec<Tile>) {
        let State {
            vision_groups,
            factions,
            ground_items,
            armies,
            condition_memory,
            map_id,
            map_revision,
            cols,
            rows,
            round,
            turn,
            gold,
            winner,
            next_id,
            combat_rng,
            log,
            units,
            campaign,
            last_combat,
            tiles,
        } = state;
        (
            Self {
                vision_groups,
                factions,
                ground_items,
                armies,
                condition_memory,
                map_id,
                map_revision,
                cols,
                rows,
                round,
                turn,
                gold,
                winner,
                next_id,
                combat_rng,
                log,
                units,
                campaign,
                last_combat,
            },
            tiles,
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(feature = "wasm", derive(ocamljs_bindgen::EncodeProjection))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TerrainUpdate {
    Keep,
    Patch { tiles: Vec<Tile> },
    Replace { tiles: Vec<Tile> },
}
/// Transient wire publication; never a saved game. Applying it to the matching
/// prior revision reconstructs the complete immutable EngineView.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(feature = "wasm", derive(ocamljs_bindgen::EncodeProjection))]
#[serde(rename_all = "camelCase")]
pub struct DispatchUpdate {
    pub base_revision: u32,
    pub revision: u32,
    pub state: StateFrame,
    pub terrain: TerrainUpdate,
    pub presentation: crate::data::BattlefieldPresentation,
    pub combat: Option<CombatAnimation>,
    /// Optional current-revision inspection requested by the live adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspection: Option<BattleInspection>,
}
/// Immutable rule costs and board topology, loaded on entering a battlefield.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(map = "dict", borrow))]
pub struct BoardInspection {
    pub costs: std::collections::BTreeMap<String, std::collections::BTreeMap<String, Option<u16>>>,
    pub neighbors: std::collections::BTreeMap<String, Vec<Hex>>,
}

/// One resolved exchange for presentation; never retained in a checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct CombatAnimation {
    pub units: [crate::data::PublicUnit; 2],
    pub strikes: Vec<CombatStrike>,
}

/// The actual roll, modifiers and HP effects of one executed strike.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct CombatStrike {
    pub weapon: String,
    /// Stable presentation identity; weapon remains the display label.
    pub weapon_id: String,
    pub unit_id: String,
    pub roll_min: i32,
    pub roll_max: i32,
    pub raw: i32,
    pub modifier: i32,
    pub terrain: i32,
    pub damage: i32,
    pub effective: i32,
    pub drain: u8,
    pub healed: i32,
    pub thorns: i32,
    pub reflected: i32,
    pub health: [i32; 2],
}

/// Reachable destination and its finite accumulated movement cost.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct ReachableTile {
    pub tile: TileId,
    pub cost: u16,
}

/// Damage accounting for a single weapon against a positioned defender.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct DamageDetail {
    pub roll_min: i32,
    pub roll_max: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub expected: f64,
    /// Probability of each total, starting at minimum.
    pub probabilities: Vec<f64>,
    pub count: u8,
    pub total: i32,
    pub base: i32,
    pub damage_type: crate::data::DamageType,
    pub modifier: i32,
    pub modified: i32,
    pub protection: i32,
    pub exposure: i32,
    pub terrain: String,
    pub damage: i32,
}

/// Remaining health across all attack, healing and retaliation outcomes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct HpForecast {
    pub minimum: i32,
    pub maximum: i32,
    pub expected: f64,
}

/// A legal current-position attack and its complete display forecast.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct AttackDetails {
    pub attacker_health: HpForecast,
    pub defender_health: HpForecast,
    pub kill_chance: f64,
    pub death_chance: f64,
    pub retaliation_min: i32,
    pub retaliation_max: i32,
    pub thorn_damage: i32,
    pub healing: i32,
    pub retaliation_healing: i32,
    pub recommended: bool,
    pub target: String,
    pub attack: Weapon,
    pub response: Option<Weapon>,
    pub damage: i32,
    pub retaliation: i32,
    pub damage_detail: DamageDetail,
    pub retaliation_detail: Option<DamageDetail>,
}

// Explicit transport aliases select JS dictionaries without changing Rust's
// ordered-map types or its native, non-Wasm build.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(js_value("dict")))]
pub type RecruitmentTiles = std::collections::BTreeMap<String, Vec<Hex>>;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(js_value("dict")))]
pub type RecruitmentNames = std::collections::BTreeMap<String, Vec<String>>;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(js_value("dict")))]
pub type RecruitmentPrices =
    std::collections::BTreeMap<String, std::collections::BTreeMap<String, i32>>;

/// Legal action choices at one immutable browser publication.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct UnitActions {
    pub name: Option<String>,
    pub faction: crate::data::UnitFaction,
    pub unit: String,
    pub reachable: Vec<ReachableTile>,
    pub attacks: Vec<AttackDetails>,
    pub promotions: Vec<String>,
    pub recruitment: RecruitmentTiles,
}

/// Detached rule queries, computed in Rust once per state publication.
/// Rendering performs only lookups against this data, never engine calls.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct BattleInspection {
    pub story: Option<crate::data::ResolvedStory>,
    /// Current dialogue captions resolved from character IDs for display only.
    pub speakers: Vec<String>,
    pub objectives: Option<crate::data::CampaignObjectives>,
    pub revision: u32,
    pub units: Vec<UnitActions>,
    pub recruitment: RecruitmentNames,
    pub recruitment_prices: RecruitmentPrices,
    pub income: crate::data::Gold,
}

/// Mechanical local bounds for an individual actor; never an army-wide duty.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedOrder {
    pub actor: UnitId,
    pub area: Vec<TileId>,
    pub attack_radius: u16,
}

impl ScopedOrder {
    pub fn allows_move(&self, game: &crate::BattleView<'_, '_>, unit: &Unit, to: usize) -> bool {
        let area: Vec<_> = self.area.iter().map(|tile| game.site(&tile.0)).collect();
        if area.contains(&to) {
            return true;
        }
        if area.contains(&unit.tile) {
            return false;
        }
        let routes = game.routes(unit.kind, &area);
        routes[to] < routes[unit.tile]
    }
    pub fn allows_attack(
        &self,
        game: &crate::BattleView<'_, '_>,
        from: usize,
        target: usize,
    ) -> bool {
        self.area.iter().any(|tile| game.site(&tile.0) == from)
            && self
                .area
                .iter()
                .any(|tile| game.board().distance(game.site(&tile.0), target) <= self.attack_radius)
    }
}

/// Recruitment can use only the six neighboring hexes, in board order.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlacementTiles {
    tiles: [usize; 6],
    len: usize,
}
impl std::ops::Deref for PlacementTiles {
    type Target = [usize];
    fn deref(&self) -> &[usize] {
        &self.tiles[..self.len]
    }
}
impl FromIterator<usize> for PlacementTiles {
    fn from_iter<T: IntoIterator<Item = usize>>(tiles: T) -> Self {
        let mut result = Self::default();
        for tile in tiles {
            result.tiles[result.len] = tile;
            result.len += 1;
        }
        result
    }
}
impl IntoIterator for PlacementTiles {
    type Item = usize;
    type IntoIter = std::iter::Take<std::array::IntoIter<usize, 6>>;
    fn into_iter(self) -> Self::IntoIter {
        self.tiles.into_iter().take(self.len)
    }
}
