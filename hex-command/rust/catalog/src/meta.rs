//! Required authored recruitment valuation profiles.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecruitmentWeights {
    pub offense: f64,
    pub endurance_reference_hp: f64,
    pub retaliation: f64,
    pub exchange_retaliation: f64,
    pub drain: f64,
    pub kill: f64,
    pub healing: f64,
    pub aura_thorns: f64,
    pub intrinsic_thorns: f64,
    pub future_wounds: f64,
    pub support_recipients: f64,
    pub healing_overlap: f64,
    pub aura_overlap: f64,
    pub future_enemies: f64,
    pub village_capture: f64,
    pub speed: f64,
    pub leader_pressure: f64,
    /// Explicit null leaves the combined village/movement bonus uncapped.
    #[serde(deserialize_with = "crate::schema::explicit_option")]
    pub movement_cap: Option<f64>,
    pub duplicate_penalty: f64,
    pub wait_penalty: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecruiterWeights {
    pub standard: RecruitmentWeights,
    pub aggressive_predictor: RecruitmentWeights,
}

/// Authored policy configuration, separate from composable mechanical definitions.
#[derive(Clone, Debug)]
pub struct CatalogueMeta {
    pub simulation: crate::SimulationSettings,
    pub recruiter_weights: RecruiterWeights,
}
