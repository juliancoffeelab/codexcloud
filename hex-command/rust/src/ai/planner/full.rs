//! Full owns global candidate selection and deliberately shares named evaluation policies.

pub(super) struct Full;
impl super::Planner for Full {
    fn recruits_after_move(&self) -> bool {
        true
    }
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, crate::data::CommandError> {
        super::drive(super::PlannerKind::Full, context, orders)
    }
}

pub(super) fn prepare(game:&crate::ai::context::BattleView<'_, '_>, phases:hex_command_battle::PhaseForecasts<'_, '_>, session:&mut super::search::Session, turn:&crate::ai::runtime::PlannerTurn<'_>) {
    super::search::prepare(game,phases,session,turn);
}
pub(super) fn choose(game:&crate::ai::context::BattleView<'_, '_>, phases:hex_command_battle::PhaseForecasts<'_, '_>, session:&mut super::search::Session, turn:&mut crate::ai::runtime::PlannerTurn<'_>) -> Result<(),crate::data::CommandError> {
    super::search::choose(game,phases,session,turn)
}
