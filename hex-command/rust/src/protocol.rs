//! Canonical transfer contract shared by native/WASM hosts and generated TypeScript.
/// Transport options reuse the core policy and diplomacy vocabularies.
pub use crate::ai::{Diplomacy as AiDiplomacy, Policy as AiPolicy};
use crate::data::{Command, CommandError, State, TileId, Weapon};
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "wasm", tsify(optional))]
        armies: Option<[String; 2]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "wasm", tsify(optional))]
        first: Option<crate::data::Side>,
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
    pub presentation: crate::data::BattlefieldPresentation,
}

/// Current checkpoint data. Hosts version their save envelope and migrate before restore.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGame {
    /// Seed used when creating a new planning workspace.
    pub seed: u32,
    pub state: State,
}

/// Factory-local battle identity and its initial published snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct BattleHandle {
    pub id: u32,
    pub view: EngineView,
}

/// Coherent state publication from one successful command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct DispatchResult {
    pub view: EngineView,
    pub combat: Option<CombatAnimation>,
}

/// One resolved exchange for presentation; never retained in a checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct CombatAnimation {
    pub units: [crate::data::PublicUnit; 2],
    pub strikes: Vec<CombatStrike>,
}

/// The actual roll, modifiers and HP effects of one executed strike.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct CombatStrike {
    pub side: crate::data::Side,
    pub weapon: String,
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct ReachableTile {
    pub tile: TileId,
    pub cost: u16,
}

/// Policy configuration retained for the current planning phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiOptions {
    /// Optional independent blocks; omitted callers use the policy preset.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<crate::ai::Blocks>,
    pub policy: crate::ai::Policy,
    pub diplomacy: crate::ai::Diplomacy,
    /// Explicit recruitment override; omitted calls use the policy's default.
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recruitment_limit: Option<usize>,
}

/// Chosen command and the battle revision against which it must be dispatched.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct AiDecision {
    pub revision: u32,
    pub command: Command,
}

/// Owned, resolved catalog data available before a battle is created.
/// Browser code formats this metadata; all rule composition stays in Rust.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[serde(rename_all = "camelCase")]
pub struct CatalogView {
    pub units: Vec<crate::data::UnitRule>,
    pub terrain: Vec<crate::data::TerrainDefinition>,
    pub races: Vec<crate::data::RaceDefinition>,
    pub maps: Vec<crate::data::MapDefinition>,
    pub campaigns: Vec<crate::data::Campaign>,
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct HpForecast {
    pub minimum: i32,
    pub maximum: i32,
    pub expected: f64,
}

/// A legal current-position attack and its complete display forecast.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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

/// Legal action choices at one immutable browser publication.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
pub struct UnitActions {
    pub faction: crate::data::UnitFaction,
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
