//! Resolved commands use board/catalog indices and stable shared weapon identities.
//! They are transient execution data, never a save or wire schema.
//! Numeric keys belong to the originating board and catalog (and their forks).
//! Owners must discard queued commands when replacing either scope.
use crate::data::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeCommand {
    Move {
        unit: UnitId,
        tile: TileIndex,
    },
    Attack {
        unit: UnitId,
        target: UnitId,
        weapon: WeaponKey,
    },
    Recruit {
        unit: UnitId,
        kind: UnitKind,
        tile: TileIndex,
    },
    Advance {
        unit: UnitId,
        kind: UnitKind,
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
        record_retreat: bool,
    },
}
impl RuntimeCommand {
    #[inline]
    pub fn is_attack(&self) -> bool {
        matches!(self, Self::Attack { .. })
    }
}
