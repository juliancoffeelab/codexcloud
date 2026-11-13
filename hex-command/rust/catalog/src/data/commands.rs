//! Typed battle commands, validation failures and observable feedback.

use super::*;
use serde::{Deserialize, Serialize};
/// A validated battle mutation expressed with stable authored identifiers.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
        #[serde(rename = "attackSpent")]
        attack_spent: bool,
        #[serde(default)]
        record_retreat: bool,
    },
}
/// Validation failures leave the entire battle unchanged.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn is_attack(&self) -> bool {
        matches!(self, Self::Attack { .. })
    }
}
