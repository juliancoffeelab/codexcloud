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
        attack_spent: bool,
        record_retreat: bool,
    },
}
impl RuntimeCommand {
    #[inline]
    pub fn is_attack(&self) -> bool {
        matches!(self, Self::Attack { .. })
    }
}

/// A planner's suggestion carries no authority over the live position. Only the
/// live executor admits it and applies real movement/contact and campaign rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionProposal(RuntimeCommand);
impl From<RuntimeCommand> for ActionProposal {
    fn from(command: RuntimeCommand) -> Self {
        Self(command)
    }
}
impl ActionProposal {
    pub fn into_command(self) -> RuntimeCommand {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalOutcome {
    Applied,
    Replan,
    Finished,
}
