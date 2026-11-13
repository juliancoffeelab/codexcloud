//! Owned simulation configurations and complete per-phase match records.
use crate::{
    ai::{Blocks, Diplomacy, Policy},
    data::*,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Arena victory rule; existing reports retain their territory default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArenaObjective {
    #[default]
    Territory,
    Deathmatch,
}
impl ArenaObjective {
    fn is_territory(&self) -> bool {
        *self == Self::Territory
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// One race/policy pairing, initiative and seed on an arena landscape.
pub struct ArenaConfig {
    #[serde(default, skip_serializing_if = "ArenaObjective::is_territory")]
    pub objective: ArenaObjective,
    pub seed: u32,
    pub armies: Armies,
    pub landscape: String,
    pub player: Policy,
    pub enemy: Policy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_blocks: Option<Blocks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemy_blocks: Option<Blocks>,
    pub first: TurnPhase,
    pub max_rounds: u32,
    /// Explicit override for both sides; None uses each policy's default.
    pub recruit_limit: Option<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Simulation roster identifiers for the two seats.
pub struct Armies {
    pub player: String,
    pub enemy: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// One campaign attempt with complete player and enemy policies.
pub struct CampaignConfig {
    pub seed: u32,
    pub diplomacy: Diplomacy,
    #[serde(default = "authored_objective")]
    pub objective: String,
    pub campaign: String,
    pub player: Policy,
    pub enemy: Policy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_blocks: Option<Blocks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemy_blocks: Option<Blocks>,
    pub max_rounds: u32,
    /// Explicit override for both sides; None uses each policy's default.
    pub recruit_limit: Option<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
/// Typed native runner input shared by scheduling, replay and profiling clients.
pub enum MatchConfig {
    Campaign(CampaignConfig),
    Arena(ArenaConfig),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// A value associated with each player/enemy seat.
pub struct Sides<T> {
    pub player: T,
    pub enemy: T,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Capturable tile ownership at a phase boundary.
pub struct Ownership {
    pub tile: Hex,
    pub owner: Option<String>,
}
/// Tile identities are shared once per chapter; boundary snapshots copy only owners.
#[derive(Clone, Debug)]
pub struct OwnershipSnapshot {
    pub tiles: Arc<Vec<Hex>>,
    pub owners: Vec<Option<String>>,
}
impl Serialize for OwnershipSnapshot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        #[derive(Serialize)]
        struct Entry<'a> {
            tile: &'a Hex,
            owner: Option<&'a str>,
        }
        let mut sequence = serializer.serialize_seq(Some(self.owners.len()))?;
        for (tile, owner) in self.tiles.iter().zip(&self.owners) {
            sequence.serialize_element(&Entry {
                tile,
                owner: owner.as_deref(),
            })?;
        }
        sequence.end()
    }
}
impl<'de> Deserialize<'de> for OwnershipSnapshot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<Ownership>::deserialize(deserializer)?;
        let owners = values.iter().map(|entry| entry.owner.clone()).collect();
        let tiles = Arc::new(values.into_iter().map(|entry| entry.tile).collect());
        Ok(Self { tiles, owners })
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Complete actions, combats, casualties and roster/objective state for one phase.
pub struct Phase {
    #[serde(default, rename = "leaderAudit", skip_serializing_if = "Vec::is_empty")]
    pub leader_audit: Vec<super::leader_audit::LeaderAudit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning: Option<crate::ai::orders::TurnDiagnostics>,
    pub side: TurnPhase,
    pub round: u32,
    #[serde(flatten)]
    pub records: Records,
    pub living: Sides<usize>,
    pub health: Sides<i32>,
    pub gold: Gold,
    pub before: Vec<Arc<PublicUnit>>,
    pub units: Vec<Arc<PublicUnit>>,
    pub ownership: Arc<OwnershipSnapshot>,
    pub goals: Vec<Progress>,
    pub objectives: Vec<Objective>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignState>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Territory holding and optional visits used to decide an arena outcome.
pub struct ArenaScore {
    pub controlled: usize,
    pub held: u32,
    pub optional_visits: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Complete arena trace including initial and final battle snapshots.
pub struct ArenaResult {
    pub config: ArenaConfig,
    pub winner: Option<BattleResult>,
    pub reason: String,
    pub rounds: usize,
    pub phases: Vec<Phase>,
    pub initial: State,
    pub r#final: State,
    pub objectives: Sides<ArenaScore>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Complete trace for one reached chapter of a campaign attempt.
pub struct MissionResult {
    #[serde(default)]
    pub escalated: bool,
    pub id: String,
    pub winner: Option<BattleResult>,
    pub round: u32,
    pub goals: Vec<Progress>,
    pub phases: Vec<Phase>,
    pub initial: State,
    pub r#final: State,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Campaign completion and the ordered chapters actually reached.
pub struct CampaignResult {
    pub config: CampaignConfig,
    pub completed: bool,
    pub missions: Vec<MissionResult>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
/// Complete native result before journal/report compaction.
pub enum MatchResult {
    Arena(ArenaResult),
    Campaign(CampaignResult),
}

pub fn authored_objective() -> String {
    "authored".into()
}
