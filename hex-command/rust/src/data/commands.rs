//! Typed battle commands, validation failures and observable feedback.

use super::*;
use serde::{Deserialize, Serialize};
/// A validated battle mutation expressed with stable authored identifiers.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Command {
    Move {
        unit: UnitId,
        tile: TileId,
    },
    Attack {
        unit: UnitId,
        target: UnitId,
        weapon: WeaponId,
    },
    Recruit {
        unit: UnitId,
        #[serde(rename = "type")]
        kind: UnitTypeId,
        tile: TileId,
    },
    Advance {
        unit: UnitId,
        #[serde(rename = "type")]
        kind: UnitTypeId,
    },
    EndTurn,
    DismissDialogue,
    NextMission,
    /// Continue with the browser's explicitly selected surviving recruits.
    RecallMission {
        units: Vec<UnitId>,
    },
    /// Restore this chapter's incoming veterans, gold and story checkpoint.
    RetryMission,
    PreviousDialogue,
    SkipDialogue,
    SetUnitIntent {
        unit: UnitId,
        retreating: Option<bool>,
        rebuilding: Option<bool>,
        attacked: bool,
        #[serde(default)]
        record_retreat: bool,
    },
}
/// Consequences returned after a command has been applied successfully.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effects {
    /// Ordered feedback derived from the command and resulting state.
    pub events: Vec<GameEvent>,
    pub added_unit: Option<UnitId>,
    pub damage: Option<i32>,
    pub retaliation: Option<i32>,
    pub winner: Option<Side>,
}
/// Validation failures leave the entire battle unchanged.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandError {
    CatalogMismatch,
    BattleOnlyPolicy,
    BattleFinished,
    UnknownUnit(UnitId),
    UnknownTile(TileId),
    UnknownType(UnitTypeId),
    UnknownWeapon(WeaponId),
    NoCampaign,
    NoNextMission,
    WrongTurn,
    IllegalMove,
    IllegalAttack,
    IllegalRecruit,
    IllegalAdvance,
    IllegalIntent,
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CommandError {}
/// A terrain entry is either impassable or spends a finite movement cost.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "cost", rename_all = "camelCase")]
pub enum Passability {
    Blocked,
    Passable(MovementCost),
}
/// Observable consequences of an accepted command. Hosts can update feedback
/// from these events while displaying the resulting immutable snapshot.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GameEvent {
    UnitMoved {
        unit: UnitId,
        from: TileId,
        to: TileId,
    },
    TileOwnerChanged {
        tile: TileId,
        previous: Option<Side>,
        owner: Option<Side>,
    },
    UnitAdded {
        unit: PublicUnit,
    },
    UnitDied {
        unit: PublicUnit,
    },
    UnitHealed {
        unit: UnitId,
        amount: i32,
        hp: i32,
    },
    UnitDamaged {
        unit: UnitId,
        amount: i32,
        hp: i32,
    },
    ExperienceChanged {
        unit: UnitId,
        previous: Option<u32>,
        experience: Option<u32>,
    },
    UnitAdvanced {
        unit: UnitId,
        previous: UnitTypeId,
        #[serde(rename = "type")]
        unit_type: UnitTypeId,
        hp: i32,
    },
    IntentChanged {
        unit: UnitId,
        retreating: Option<bool>,
        rebuilding: Option<bool>,
        attacked: bool,
    },
    GoldChanged {
        side: Side,
        previous: i32,
        gold: i32,
    },
    TurnBegan {
        side: Side,
        round: u32,
    },
    DialogueQueued {
        lines: Vec<Dialogue>,
    },
    DialogueAdvanced {
        index: usize,
    },
    MissionStarted {
        campaign: CampaignId,
        mission: usize,
    },
    CombatResolved {
        unit: UnitId,
        target: UnitId,
        weapon: WeaponId,
        damage: i32,
        retaliation: i32,
    },
    Victory {
        winner: Side,
    },
}
