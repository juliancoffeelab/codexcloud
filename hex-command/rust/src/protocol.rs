//! Canonical transfer contract shared by native/WASM hosts and generated TypeScript.
/// Transport options reuse the core policy and diplomacy vocabularies.
pub use crate::ai::{Diplomacy as AiDiplomacy, Policy as AiPolicy};
use crate::data::{Command, CommandError, Effects, State, TileId, UnitId, Weapon, WeaponId};
use serde::{Deserialize, Serialize};

/// Select a scenario or the first mission of an authored campaign.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Start {
    /// Start the requested revision, or the latest revision when omitted.
    Scenario {
        map: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "wasm", tsify(optional))]
        revision: Option<u32>,
    },
    /// Begin a campaign without incoming veterans or carried gold.
    Campaign { id: String },
}

/// Tagged failures from transport validation, lifecycle checks, or battle rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(untagged)]
pub enum EngineResult<T> {
    Success {
        #[cfg_attr(feature = "wasm", tsify(type = "true"))]
        ok: bool,
        value: T,
    },
    Failure {
        #[cfg_attr(feature = "wasm", tsify(type = "false"))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct EngineView {
    /// Advances only after a successfully dispatched command.
    pub revision: u32,
    /// Preserves the core snapshot's distinction between absent and null values.
    pub state: State,
}

/// Versioned checkpoint containing exact battle state and optional AI continuation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGame {
    /// The current checkpoint format is version 1.
    pub version: u32,
    /// Seed used when creating a new planning workspace.
    pub seed: u32,
    pub state: State,
    /// Older checkpoints omit this field and resume without an active planner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub planner: Option<SavedPlanner>,
}

/// Planner configuration, persistent workspace, and any command awaiting dispatch.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedPlanner {
    pub options: AiOptions,
    pub workspace: crate::ai::PlanningSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    /// Chosen but unapplied command, returned again after restoration.
    pub pending: Option<Command>,
}

/// Factory-local battle identity and its initial published snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct BattleHandle {
    pub id: u32,
    pub view: EngineView,
}

/// Coherent state publication and rule effects from one successful command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct DispatchResult {
    pub view: EngineView,
    pub effects: Effects,
}

/// Reachable destination and its finite accumulated movement cost.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct ReachableTile {
    pub tile: TileId,
    pub cost: u16,
}

/// Stable identities needed to preview an attack without mutating the battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase")]
pub struct AttackQuery {
    pub unit: UnitId,
    pub target: UnitId,
    pub weapon: WeaponId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    /// Hypothetical origin; omission uses the attacker's current tile.
    pub from: Option<TileId>,
}

/// Deterministic damage, retaliation, and the defender's selected response weapon.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct AttackPreview {
    pub damage: i32,
    pub retaliation: i32,
    pub response: Option<Weapon>,
}

/// Policy configuration retained for the current planning phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiOptions {
    pub policy: crate::ai::Policy,
    pub diplomacy: crate::ai::Diplomacy,
    /// Maximum recruit commands the planner may choose during this phase.
    pub recruitment_limit: usize,
}

/// Chosen command and the battle revision against which it must be dispatched.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct AiDecision {
    pub revision: u32,
    pub command: Command,
}

/// Authored scenario identity, revision, and board dimensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct MapSummary {
    pub id: String,
    pub revision: u32,
    pub cols: i32,
    pub rows: i32,
}

/// Authored campaign identity, display name, and mission count.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct CampaignSummary {
    pub id: String,
    pub name: String,
    pub missions: usize,
}

/// Display and recruitment metadata for one catalog unit type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct CatalogView {
    pub maps: Vec<MapSummary>,
    pub campaigns: Vec<CampaignSummary>,
    pub units: Vec<UnitSummary>,
    pub terrains: Vec<String>,
}

/// Optional browser-supplied catalog documents, keyed by their relative paths.
#[cfg(feature = "wasm")]
#[derive(Clone, Debug, Serialize, Deserialize, tsify::Tsify)]
#[serde(transparent)]
#[tsify(hashmap_as_object)]
pub struct CatalogDocuments(pub std::collections::BTreeMap<String, String>);

// The WASM ABI needs concrete names; the generic envelope remains shared.
// Transparent response types retain the existing plain-object wire format.
#[cfg(feature = "wasm")]
macro_rules! response {
    ($name:ident, $value:ty) => {
        #[doc = concat!("Typed WASM response carrying `", stringify!($value), "` or an engine error.")]
        #[derive(Clone, Debug, Serialize, Deserialize, tsify::Tsify)]
        #[serde(transparent)]
        #[tsify(missing_as_null, hashmap_as_object)]
        pub struct $name(pub EngineResult<$value>);

        impl From<Result<$value, EngineError>> for $name {
            fn from(result: Result<$value, EngineError>) -> Self {
                Self(result.into())
            }
        }
    };
}
#[cfg(feature = "wasm")]
response!(CatalogResponse, CatalogView);
#[cfg(feature = "wasm")]
response!(BattleResponse, BattleHandle);
#[cfg(feature = "wasm")]
response!(ViewResponse, EngineView);
#[cfg(feature = "wasm")]
response!(ReachableResponse, Vec<ReachableTile>);
#[cfg(feature = "wasm")]
response!(AttackResponse, AttackPreview);
#[cfg(feature = "wasm")]
response!(DispatchResponse, DispatchResult);
#[cfg(feature = "wasm")]
response!(AiResponse, AiDecision);
#[cfg(feature = "wasm")]
response!(SaveResponse, SavedGame);
#[cfg(feature = "wasm")]
response!(UnitResponse, ());

/// Damage accounting for a single weapon against a positioned defender.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase")]
pub struct DamageDetail {
    pub base: i32,
    pub damage_type: crate::data::DamageType,
    pub modifier: i32,
    pub modified: i32,
    pub protection: i32,
    pub exposure: i32,
    pub terrain: String,
    pub damage: i32,
}

/// A legal current-position attack and its complete display forecast.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase")]
pub struct AttackDetails {
    pub target: String,
    pub attack: Weapon,
    pub response: Option<Weapon>,
    pub damage: i32,
    pub retaliation: i32,
    pub damage_detail: DamageDetail,
    pub retaliation_detail: Option<DamageDetail>,
}

/// Legal action choices at one immutable browser publication.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct UnitActions {
    pub unit: String,
    pub reachable: Vec<ReachableTile>,
    pub attacks: Vec<AttackDetails>,
    pub promotions: Vec<String>,
    pub recruitment: std::collections::BTreeMap<String, Vec<String>>,
}

/// Detached rule queries, computed in Rust once per state publication.
/// Rendering performs only lookups against this data, never engine calls.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct BattleInspection {
    pub revision: u32,
    pub units: Vec<UnitActions>,
    pub recruitment: std::collections::BTreeMap<String, Vec<String>>,
    pub costs: std::collections::BTreeMap<String, std::collections::BTreeMap<String, Option<u16>>>,
    pub neighbors: std::collections::BTreeMap<String, Vec<String>>,
    pub income: crate::data::Gold,
}
#[cfg(feature = "wasm")]
response!(InspectionResponse, BattleInspection);
