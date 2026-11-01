//! Planner commands keep indices; authored strings belong only to the wire.
use crate::data::UnitId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Move {
        unit: UnitId,
        tile: usize,
    },
    Attack {
        unit: UnitId,
        target: UnitId,
        weapon: usize,
    },
    Recruit {
        unit: UnitId,
        kind: usize,
        tile: usize,
    },
    Advance {
        unit: UnitId,
        kind: usize,
    },
    #[cfg_attr(not(any(feature = "wasm", test)), allow(dead_code))]
    EndTurn,
    SetUnitIntent {
        unit: UnitId,
        retreating: Option<bool>,
        rebuilding: Option<bool>,
        attacked: bool,
        record_retreat: bool,
    },
}
impl Action {
    pub fn is_attack(&self) -> bool {
        matches!(self, Self::Attack { .. })
    }
}
pub(crate) enum Operation {
    Wire(crate::data::Command),
    Planned(Action),
}
impl From<crate::data::Command> for Operation {
    fn from(command: crate::data::Command) -> Self {
        Self::Wire(command)
    }
}
impl From<Action> for Operation {
    fn from(command: Action) -> Self {
        Self::Planned(command)
    }
}
