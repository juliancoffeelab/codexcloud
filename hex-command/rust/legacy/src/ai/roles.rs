use super::TacticalRole;
use crate::data::UnitRole;
use crate::logic::Game;
pub(super) fn role(g: &Game<'_>, kind: usize) -> TacticalRole {
    match g.rules.units[kind].role {
        UnitRole::Healer => TacticalRole::Healer,
        UnitRole::Ranger => TacticalRole::Ranged,
        UnitRole::Warrior | UnitRole::Flank => TacticalRole::Frontline,
    }
}
pub(super) fn recruit_role(g: &Game<'_>, kind: usize) -> UnitRole {
    g.rules.units[kind].role
}
