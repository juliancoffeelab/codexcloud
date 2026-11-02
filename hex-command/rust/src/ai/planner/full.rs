//! Full tactical evaluation uses shared mechanical search, never another planner.

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

pub(super) fn prepare(
    game: &crate::ai::context::BattleView<'_, '_>,
    workspace: &mut crate::ai::PlanningWorkspace,
) {
    super::search::prepare(game, workspace);
}

pub(super) fn choose(
    game: &crate::ai::context::BattleView<'_, '_>,
    workspace: &mut crate::ai::PlanningWorkspace,
) -> Result<(), crate::data::CommandError> {
    super::search::choose(game, workspace, false)
}
