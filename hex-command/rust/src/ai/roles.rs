use super::TacticalRole;
use crate::ai::context::BattleView;
use crate::data::UnitRole;
pub(super) fn role(g: &BattleView<'_, '_>, kind: usize) -> TacticalRole {
    match g.unit_definition(kind).role() {
        UnitRole::Healer => TacticalRole::Healer,
        UnitRole::Ranger => TacticalRole::Ranged,
        UnitRole::Warrior | UnitRole::Flank => TacticalRole::Frontline,
    }
}
pub(super) fn recruit_role(g: &BattleView<'_, '_>, kind: usize) -> UnitRole {
    g.unit_definition(kind).role()
}
