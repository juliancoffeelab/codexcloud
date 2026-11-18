//! Authored native simulation and training workloads; no battle state or prepared mechanics.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// An arena roster, authored independently of story campaign recruitment.
pub struct ArmyRoster {
    pub id: Text,
    pub leader: Text,
    pub pool: Vec<Text>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A named arena workload over explicitly registered map variants.
pub struct ArenaScenario {
    pub id: Text,
    pub variants: Vec<ArenaVariant>,
    pub hold_turns: u32,
    pub territory: Vec<Hex>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArenaVariant {
    pub map: Text,
    pub map_revision: u32,
    pub forces: ArenaForces,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum ArenaForces {
    Selected { budget: i32 },
    Scenario,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArenaSettings {
    pub scenarios: Vec<ArenaScenario>,
    pub default_scenarios: Vec<Text>,
    pub default_max_rounds: u32,
    pub policies: Vec<Text>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CampaignSettings {
    pub escalation_flags: BTreeMap<Text, Text>,
    pub default_max_rounds: u32,
    pub diplomacy: Vec<Text>,
    pub policies: Vec<Text>,
    pub default_campaigns: Vec<Text>,
    pub campaigns: BTreeMap<Text, CampaignSimulation>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CampaignSimulation {
    pub diplomacy: Vec<Text>,
    pub routes: BTreeMap<Text, SimulationObjective>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingSettings {
    pub objectives: Vec<Text>,
    pub arenas: Vec<TrainingArena>,
    pub campaigns: BTreeMap<Text, Vec<TrainingCampaignVariant>>,
    pub opponents: Vec<Text>,
    pub recruiters: Vec<Text>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingArena {
    pub id: Text,
    pub pairs: Vec<TrainingArmies>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingArmies {
    pub player: Text,
    pub enemy: Text,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingCampaignVariant {
    pub objective: Text,
    pub diplomacy: Text,
}
/// Named simulator-only routes. Keys are authored stage map identifiers.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationObjective {
    pub stages: std::collections::BTreeMap<Text, SimulationStageObjective>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationSettings {
    pub rosters: Vec<ArmyRoster>,
    pub arena: ArenaSettings,
    pub campaign: CampaignSettings,
    pub training: TrainingSettings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SimulationStageObjective {
    pub script_conditions: BTreeMap<Text, When>,
    pub script_events: Vec<ScriptEvent>,
    #[serde(deserialize_with = "crate::schema::explicit_option")]
    pub ignore_optional_when: Option<Condition>,
}
