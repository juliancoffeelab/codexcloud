//! Canonical transfer contract shared by native/WASM hosts and generated TypeScript.
/// Transport options reuse the core policy and diplomacy vocabularies.
pub use crate::ai::{Diplomacy as AiDiplomacy, Policy as AiPolicy};
use crate::data::{Command, CommandError, Effects, State, TileId, UnitId, Weapon, WeaponId};
use serde::{Deserialize, Serialize};

/// Select a scenario or the first mission of an authored campaign.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Start {
    /// Start the requested revision, or the latest revision when omitted.
    Scenario {
        map: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "bindings", ts(optional))]
        revision: Option<u32>,
    },
    /// Begin a campaign without incoming veterans or carried gold.
    Campaign { id: String },
}

/// Tagged failures from transport validation, lifecycle checks, or battle rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum EngineError {
    InvalidRequest {
        message: String,
    },
    Content {
        path: String,
        message: String,
    },
    InvalidSave {
        message: String,
    },
    UnknownMap {
        map: String,
    },
    UnknownCampaign {
        id: String,
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

/// Every operation returns data or a tagged error, without throwing for rule failures.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(untagged)]
pub enum EngineResult<T> {
    Success {
        #[cfg_attr(feature = "bindings", ts(type = "true"))]
        ok: bool,
        value: T,
    },
    Failure {
        #[cfg_attr(feature = "bindings", ts(type = "false"))]
        ok: bool,
        error: EngineError,
    },
}
impl<T> From<Result<T, EngineError>> for EngineResult<T> {
    fn from(result: Result<T, EngineError>) -> Self {
        match result {
            Ok(value) => Self::Success { ok: true, value },
            Err(error) => Self::Failure { ok: false, error },
        }
    }
}

/// Detached battle state paired with the revision required by subsequent requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct EngineView {
    /// Advances only after a successfully dispatched command.
    pub revision: u32,
    /// Preserves the core snapshot's distinction between absent and null values.
    pub state: State,
}

/// Versioned checkpoint containing exact battle state and optional AI continuation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGame {
    /// The current checkpoint format is version 1.
    pub version: u32,
    /// Seed used when creating a new planning workspace.
    pub seed: u32,
    pub state: State,
    /// Older checkpoints omit this field and resume without an active planner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "bindings", ts(optional))]
    pub planner: Option<SavedPlanner>,
}

/// Planner configuration, persistent workspace, and any command awaiting dispatch.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedPlanner {
    pub options: AiOptions,
    pub workspace: crate::ai::PlanningSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "bindings", ts(optional))]
    /// Chosen but unapplied command, returned again after restoration.
    pub pending: Option<Command>,
}

/// Factory-local battle identity and its initial published snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct BattleHandle {
    pub id: u32,
    pub view: EngineView,
}

/// Coherent state publication and rule effects from one successful command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct DispatchResult {
    pub view: EngineView,
    pub effects: Effects,
}

/// Reachable destination and its finite accumulated movement cost.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct ReachableTile {
    pub tile: TileId,
    pub cost: u16,
}

/// Stable identities needed to preview an attack without mutating the battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct AttackQuery {
    pub unit: UnitId,
    pub target: UnitId,
    pub weapon: WeaponId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "bindings", ts(optional))]
    /// Hypothetical origin; omission uses the attacker's current tile.
    pub from: Option<TileId>,
}

/// Deterministic damage, retaliation, and the defender's selected response weapon.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct AttackPreview {
    pub damage: i32,
    pub retaliation: i32,
    pub response: Option<Weapon>,
}

/// Policy configuration retained for the current planning phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiOptions {
    pub policy: AiPolicy,
    pub diplomacy: AiDiplomacy,
    /// Maximum recruit commands the planner may choose during this phase.
    pub recruitment_limit: usize,
}

/// Chosen command and the battle revision against which it must be dispatched.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct AiDecision {
    pub revision: u32,
    pub command: Command,
}

/// Authored scenario identity, revision, and board dimensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct MapSummary {
    pub id: String,
    pub revision: u32,
    pub cols: i32,
    pub rows: i32,
}

/// Authored campaign identity, display name, and mission count.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct CampaignSummary {
    pub id: String,
    pub name: String,
    pub missions: usize,
}

/// Display and recruitment metadata for one catalog unit type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct UnitSummary {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub movement: usize,
    pub cost: i32,
    pub leader: bool,
    pub attacks: Vec<Weapon>,
    pub advances_to: Vec<String>,
}

/// Owned catalog metadata available before a battle is created.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
pub struct CatalogView {
    pub maps: Vec<MapSummary>,
    pub campaigns: Vec<CampaignSummary>,
    pub units: Vec<UnitSummary>,
    pub terrains: Vec<String>,
}

/// Emit one module, with all referenced Rust declarations included exactly once.
#[cfg(feature = "bindings")]
pub fn typescript() -> String {
    use std::collections::BTreeMap;
    use ts_rs::{TS, TypeVisitor};
    struct Declarations(BTreeMap<String, String>);
    impl TypeVisitor for Declarations {
        fn visit<T: TS + 'static + ?Sized>(&mut self) {
            if T::output_path().is_none() {
                return;
            }
            let name = T::ident();
            if self.0.contains_key(&name) {
                return;
            }
            self.0.insert(name, format!("export {}", T::decl()));
            T::visit_dependencies(self);
        }
    }
    let mut declarations = Declarations(BTreeMap::new());
    declarations.visit::<Start>();
    // The absence/null override on State.last_combat intentionally uses a TS name.
    declarations.visit::<crate::data::LastCombat>();
    declarations.visit::<EngineResult<()>>();
    declarations.visit::<BattleHandle>();
    declarations.visit::<DispatchResult>();
    declarations.visit::<SavedGame>();
    declarations.visit::<ReachableTile>();
    declarations.visit::<AttackQuery>();
    declarations.visit::<AttackPreview>();
    declarations.visit::<AiOptions>();
    declarations.visit::<AiDecision>();
    declarations.visit::<CatalogView>();
    let mut output = String::from("// Generated by rust/src/bin/export-types.rs. Do not edit.\n\n");
    for declaration in declarations.0.into_values() {
        output.push_str("// prettier-ignore\n");
        output.push_str(&declaration);
        output.push('\n');
    }
    output
}
