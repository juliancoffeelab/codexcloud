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
    pub sampling_weights: BTreeMap<Text, f64>,
    pub map_weights: BTreeMap<Text, f64>,
    pub reward_profiles: BTreeMap<Text, TrainingRewardProfile>,
    pub adaptive_sampling: AdaptiveSamplingSettings,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdaptiveSamplingSettings {
    pub enabled: bool,
    pub completion_threshold: f64,
    pub minimum_multiplier: f64,
    pub ema_rate: f64,
    pub initial_completion_rate: f64,
    pub warmup_episodes: u32,
    pub update_every_episodes: u32,
}
impl AdaptiveSamplingSettings {
    pub fn valid(&self) -> bool {
        self.completion_threshold.is_finite()
            && (0.0..1.0).contains(&self.completion_threshold)
            && self.minimum_multiplier.is_finite()
            && self.minimum_multiplier > 0.0
            && self.minimum_multiplier <= 1.0
            && self.ema_rate.is_finite()
            && self.ema_rate > 0.0
            && self.ema_rate <= 1.0
            && self.initial_completion_rate.is_finite()
            && (0.0..=1.0).contains(&self.initial_completion_rate)
            && self.warmup_episodes > 0
            && self.update_every_episodes > 0
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingArena {
    pub id: Text,
    pub pairs: Vec<TrainingArmies>,
    pub sampling_group: Text,
    pub sampling_map: Text,
    pub condition_weight: f64,
    pub reward_profile: Text,
    /// None explicitly inherits the ordinary arena schedule.
    pub conditions: Option<TrainingArenaConditions>,
    #[serde(rename = "pretrain_imitator")]
    pub pretrain_imitator: PretrainImitator,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingArenaConditions {
    pub objectives: Vec<Text>,
    pub opponents: Vec<Text>,
    pub recruiters: Vec<Text>,
    pub first: Vec<Text>,
    pub recruit_limit: usize,
    pub max_rounds: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingRewardProfile {
    pub win: f64,
    pub chapter: f64,
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
    /// When omitted, inherit the authored global opponent pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponents: Option<Vec<Text>>,
    pub sampling_group: Text,
    pub sampling_map: Text,
    pub condition_weight: f64,
    pub reward_profile: Text,
    /// None inherits the global recruiters / policy recruitment limit.
    pub recruiters: Option<Vec<Text>>,
    pub recruit_limit: Option<usize>,
    #[serde(rename = "pretrain_imitator")]
    pub pretrain_imitator: PretrainImitator,
}
/// Pretraining-only action source. PPO never uses this selector.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum PretrainImitator {
    Named(Text),
    Path {
        path_to: crate::Hex,
    },
    Blocks {
        recruiter: Text,
        campaign: Text,
        planner: Text,
    },
}
impl PretrainImitator {
    pub fn valid(&self) -> bool {
        match self {
            Self::Named(name) => matches!(
                name.as_str(),
                "classic" | "queued" | "full" | "objective_progress"
            ),
            Self::Path { .. } => true,
            Self::Blocks {
                recruiter,
                campaign,
                planner,
            } => {
                matches!(
                    recruiter.as_str(),
                    "classic" | "balanced" | "situational" | "predictor"
                ) && campaign == "mission"
                    && matches!(planner.as_str(), "classic" | "queued" | "full")
            }
        }
    }
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
