//! One learned decision at a time; runtime still owns recruitment and combat boundaries.
use crate::{ai::orders::*, data::*};
pub(super) struct Learned;
impl super::Planner for Learned {
    fn recruits_after_move(&self) -> bool {
        true
    }
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &TacticalOrders,
    ) -> Result<PlanningReport, CommandError> {
        if !crate::training::configured() {
            return Err(CommandError::IllegalIntent);
        }
        super::prepare_priorities_and_recovery(context, orders)?;
        let observed = context.observation_orders().clone();
        super::drive(context, |context| {
            let choices = crate::training::candidates(context.battle(), &observed);
            let index = crate::training::select(&choices);
            let command = choices.commands[index].clone();
            if command == Command::EndTurn {
                *context.turn_parts().finished = true;
            } else {
                context.emit(command)?;
            }
            Ok(())
        })
    }
}
