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
        super::prepare_priorities_and_recovery(context, orders)?;
        super::drive(context, |context| {
            let (game, phases, actions, mut parts) = context.full_parts();
            coordinated_step(game, phases, actions, &mut parts)
        })
    }
}

pub(super) fn prepare<'a, 'catalog>(
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    session: &mut super::search::Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
) {
    super::search::prepare(phases, session, turn);
}
pub(super) fn choose<'a, 'catalog>(
    game: &'a crate::ai::context::BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    session: &mut super::search::Session,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Result<(), crate::data::CommandError> {
    super::search::choose(game, phases, session, turn)
}

fn coordinated_step<'view, 'catalog>(
    game: &'view crate::ai::context::BattleView<'view, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'view, 'catalog>,
    actions: hex_command_battle::ActionForecastSource<'view, 'catalog>,
    parts: &mut crate::ai::runtime::PlannerParts<'_, super::search::Session>,
) -> Result<(), crate::data::CommandError> {
    if let Some(goals) = super::prepare_coordination(game, &mut parts.turn) {
        parts.session.replace_goals(goals);
    }
    prepare(phases, parts.session, &parts.turn);
    if super::rescue_coordination(game, phases, actions, &mut parts.turn)? {
        return Ok(());
    }
    choose(game, phases, parts.session, &mut parts.turn)?;
    super::finish_coordination(game, phases, &mut parts.turn)
}
