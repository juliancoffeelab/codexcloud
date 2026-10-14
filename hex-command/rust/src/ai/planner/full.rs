//! Full tactical evaluation uses shared mechanical search, never another planner.

pub(super) struct Full;
impl super::Planner for Full {
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, crate::data::CommandError> {
        super::drive(super::PlannerKind::Full, context, orders)
    }
}
