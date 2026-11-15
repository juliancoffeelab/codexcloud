//! Canonical transfer contract shared by native/WASM hosts and generated OCaml bindings.
/// Transport options reuse the core policy and diplomacy vocabularies.
pub use crate::ai::{Diplomacy as AiDiplomacy, Policy as AiPolicy};
use crate::data::Text;
use crate::data::{Command, CommandError, State};
use serde::{Deserialize, Serialize};

/// Select a scenario or the first mission of an authored campaign.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Start {
    /// Start the requested revision, or the latest revision when omitted.
    Scenario {
        map: Text,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        revision: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        armies: Option<[Text; 2]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first: Option<crate::data::TurnPhase>,
    },
    /// Begin a campaign without incoming veterans or carried gold.
    Campaign { id: Text },
}

/// Tagged failures from transport validation, lifecycle checks, or battle rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum EngineError {
    InvalidRequest {
        message: Text,
    },
    Content {
        path: Text,
        message: Text,
    },
    InvalidSave {
        message: Text,
    },
    UnknownMap {
        map: Text,
    },
    UnknownCampaign {
        id: Text,
    },
    UnknownBattle,
    Disposed,
    /// The caller's expected revision differs from the current battle revision.
    StaleRevision {
        expected: u32,
        actual: u32,
    },
    Command {
        error: CommandError,
    },
}
impl From<CommandError> for EngineError {
    fn from(error: CommandError) -> Self {
        Self::Command { error }
    }
}

/// Detached battle state paired with the revision required by subsequent requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct EngineView {
    /// Advances only after a successfully dispatched command.
    pub revision: u32,
    /// Preserves the core snapshot's distinction between absent and null values.
    pub state: State,
    pub presentation: crate::data::BattlefieldPresentation,
}

/// Current checkpoint data. Hosts version their save envelope and migrate before restore.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGame {
    /// Seed used when creating a new planning workspace.
    pub seed: u32,
    pub state: State,
}

/// A data-only backward change; this never records a command or planner state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum UndoChange {
    Set {
        path: Vec<Text>,
        value: serde_json::Value,
    },
    Remove {
        path: Vec<Text>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UndoStep {
    pub round: u32,
    pub unit_id: Text,
    pub changes: Vec<UndoChange>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndoValidation {
    pub state: State,
    /// Oldest first; validation walks backwards from the current state.
    pub history: Vec<UndoStep>,
}

/// Factory-local battle identity and its initial published snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct BattleHandle {
    pub id: u32,
    pub view: EngineView,
}

/// Coherent state publication from one successful command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct DispatchResult {
    pub view: EngineView,
    pub combat: Option<CombatAnimation>,
}

pub use hex_command_battle::model::{
    AttackDetails, BattleInspection, BoardInspection, CombatAnimation, CombatStrike, DamageDetail,
    DispatchUpdate, HpForecast, ReachableTile, StateFrame, TerrainUpdate, UnitActions,
};

/// Policy configuration retained for the current planning phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiOptions {
    /// Optional independent blocks; omitted callers use the policy preset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<crate::ai::Blocks>,
    pub policy: crate::ai::Policy,
    pub diplomacy: crate::ai::Diplomacy,
    /// Explicit recruitment override; omitted calls use the policy's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recruitment_limit: Option<usize>,
}

/// Chosen command and the battle revision against which it must be dispatched.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
pub struct AiDecision {
    pub revision: u32,
    pub command: Command,
}

/// Owned, resolved catalog data available before a battle is created.
/// Browser code formats this metadata; all rule composition stays in Rust.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[serde(rename_all = "camelCase")]
pub struct CatalogView {
    pub items: Vec<crate::data::Item>,
    pub units: Vec<crate::data::UnitRule>,
    pub terrain: Vec<crate::data::TerrainDefinition>,
    pub races: Vec<crate::data::RaceDefinition>,
    pub maps: Vec<crate::data::MapDefinition>,
    pub campaigns: Vec<crate::data::Campaign>,
}

#[cfg(feature = "wasm")]
pub use hex_command_battle::model::BorrowedStateFrame;
