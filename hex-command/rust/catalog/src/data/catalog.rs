//! Runtime catalog vocabulary, separate from the authoring parser.
use super::*;
use crate::data::Text;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};
/// Relative document paths mapped to their complete UTF-8 JSON source.
pub type JsonBundle = BTreeMap<Text, Text>;

#[derive(Debug)]
/// A catalog error carrying the authoring document and a concrete diagnostic.
pub struct ContentError {
    pub path: Text,
    pub message: Text,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// An arena roster, authored independently of story campaign recruitment.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArmyRoster {
    pub id: Text,
    pub leader: Text,
    pub pool: Vec<Text>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A named arena workload over explicitly registered map variants.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArenaScenario {
    pub id: Text,
    pub variants: Vec<ArenaVariant>,
    pub hold_turns: u32,
    pub territory: Vec<Hex>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArenaVariant {
    pub map: Text,
    pub forces: ArenaForces,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum ArenaForces {
    Selected { budget: i32 },
    Scenario,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArenaDefaults {
    pub scenarios: Vec<ArenaScenario>,
    pub default_scenarios: Vec<Text>,
    pub default_max_rounds: u32,
    pub policies: Vec<Text>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignDefaults {
    #[serde(default)]
    pub escalation_flags: BTreeMap<Text, Text>,
    pub default_max_rounds: u32,
    pub diplomacy: Vec<Text>,
    pub policies: Vec<Text>,
    pub default_campaigns: Vec<Text>,
    pub campaigns: BTreeMap<Text, CampaignSimulation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignSimulation {
    pub diplomacy: Vec<Text>,
    pub routes: BTreeMap<Text, SimulationObjective>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TrainingDefaults {
    pub objectives: Vec<Text>,
    pub arenas: Vec<TrainingArena>,
    pub campaigns: BTreeMap<Text, Vec<TrainingCampaignVariant>>,
    pub opponents: Vec<Text>,
    pub recruiters: Vec<Text>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TrainingArena {
    pub id: Text,
    pub pairs: Vec<TrainingArmies>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TrainingArmies {
    pub player: Text,
    pub enemy: Text,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TrainingCampaignVariant {
    pub objective: Text,
    pub diplomacy: Text,
}

/// Named simulator-only routes. Keys are authored stage map identifiers.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SimulationObjective {
    pub stages: std::collections::BTreeMap<Text, SimulationStageObjective>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SimulationStageObjective {
    #[serde(default)]
    pub script_conditions: BTreeMap<Text, When>,
    #[serde(default)]
    pub script_events: Vec<ScriptEvent>,
    #[serde(default)]
    pub goals: Option<Vec<Goal>>,
    #[serde(default)]
    pub optional_goals: Option<Vec<Goal>>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub event_conditions: std::collections::BTreeMap<Text, Condition>,
    #[serde(default)]
    pub ignore_optional_when: Option<Condition>,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
/// Category and damage-type contributions; both apply to an incoming hit.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct EnvironmentalModifiers {
    pub melee: i32,
    pub ranged: i32,
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A race or unit deviation from base terrain movement and damage rules.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TerrainModifier {
    #[serde(default)]
    pub cost: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passable: Option<bool>,
    #[serde(default)]
    pub damage_modifiers: EnvironmentalModifiers,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored terrain defaults, also exposed to the browser for display.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TerrainDefinition {
    pub role: TerrainRole,
    pub id: Terrain,
    pub name: Text,
    pub cost: u16,
    pub passable: bool,
    pub description: Text,
    #[serde(default)]
    pub damage_modifiers: EnvironmentalModifiers,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored racial defaults inherited by every unit of that race.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct RaceDefinition {
    pub id: Race,
    pub name: Text,
    pub terrain_modifiers: HashMap<TerrainRole, TerrainModifier>,
    pub damage_modifiers: DamageModifiers,
}

impl ContentError {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn new(path: impl Into<Text>, message: impl Into<Text>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ContentError {}

#[cfg_attr(not(target_arch = "wasm32"), inline)]

pub fn identifier(id: &str, path: &str) -> Result<(), ContentError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err(ContentError::new(
            path,
            format!("invalid identifier {id:?}"),
        ));
    }
    Ok(())
}

#[cfg_attr(not(target_arch = "wasm32"), inline)]

pub fn unique<T: Eq + std::hash::Hash + std::fmt::Display>(
    ids: impl IntoIterator<Item = T>,
    path: &str,
) -> Result<(), ContentError> {
    let mut seen = HashSet::new();
    for id in ids {
        if seen.contains(&id) {
            return Err(ContentError::new(
                path,
                format!("duplicate identifier {id}"),
            ));
        }
        seen.insert(id);
    }
    Ok(())
}
#[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
#[path = "catalog_loader.rs"]
mod loader;
#[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
pub use loader::content_fingerprint;
#[cfg(all(
    feature = "wasm",
    any(not(target_arch = "wasm32"), feature = "runtime-catalog")
))]
pub use loader::embedded_bundle;
