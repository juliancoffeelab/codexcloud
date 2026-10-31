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
    DropItem {
        unit: UnitId,
        item: String,
    },
    EndTurn,
    DismissDialogue,
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
/// Validation failures leave the entire battle unchanged.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandError {
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
    IllegalItem,
    IllegalIntent,
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CommandError {}

impl Command {
    pub(crate) fn is_attack(&self) -> bool {
        matches!(self, Self::Attack { .. })
    }
}
