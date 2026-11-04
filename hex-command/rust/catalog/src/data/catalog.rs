//! Runtime catalog vocabulary, separate from the authoring parser.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};
/// Relative document paths mapped to their complete UTF-8 JSON source.
pub type JsonBundle = BTreeMap<String, String>;

#[derive(Debug)]
/// A catalog error carrying the authoring document and a concrete diagnostic.
pub struct ContentError {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// An arena roster, authored independently of story campaign recruitment.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArmyRoster {
    pub id: String,
    pub leader: String,
    pub pool: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A generated arena landscape and its required territory hold duration.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct LandscapeConfig {
    pub id: String,
    pub hold_turns: u32,
    /// Optional authored scenario layout, shared with the playable map.
    pub map: Option<String>,
    /// Capture targets for this layout; omitted uses the generated-arena defaults.
    pub territory: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored arena dimensions and default per-match execution limits.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ArenaDefaults {
    pub landscapes: Vec<LandscapeConfig>,
    pub territory: Vec<String>,
    /// Minimum total gold value per side, including its leader.
    pub starting_budget: i32,
    pub default_max_rounds: u32,
    pub policies: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Campaign schedule dimensions and default attempt execution limits.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignDefaults {
    #[serde(default)]
    pub escalation_flags: BTreeMap<String, String>,
    pub default_max_rounds: u32,
    pub diplomacy: Vec<String>,
    #[serde(default)]
    pub objectives: BTreeMap<String, BTreeMap<String, SimulationObjective>>,
    pub policies: Vec<String>,
    pub campaigns: Vec<String>,
}

/// Named simulator-only routes. Keys are authored stage map identifiers.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SimulationObjective {
    pub stages: std::collections::BTreeMap<String, SimulationStageObjective>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SimulationStageObjective {
    #[serde(default)]
    pub script_conditions: BTreeMap<String, When>,
    #[serde(default)]
    pub script_events: Vec<ScriptEvent>,
    #[serde(default)]
    pub goals: Option<Vec<Goal>>,
    #[serde(default)]
    pub optional_goals: Option<Vec<Goal>>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub event_conditions: std::collections::BTreeMap<String, Condition>,
    #[serde(default)]
    pub ignore_optional_when: Option<Condition>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
    pub name: String,
    pub cost: u16,
    pub passable: bool,
    pub description: String,
    #[serde(default)]
    pub damage_modifiers: EnvironmentalModifiers,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored racial defaults inherited by every unit of that race.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct RaceDefinition {
    pub id: Race,
    pub name: String,
    pub terrain_modifiers: HashMap<TerrainRole, TerrainModifier>,
    pub damage_modifiers: DamageModifiers,
}

impl ContentError {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
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

pub fn unique<'a>(ids: impl IntoIterator<Item = &'a str>, path: &str) -> Result<(), ContentError> {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(ContentError::new(
                path,
                format!("duplicate identifier {id}"),
            ));
        }
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
