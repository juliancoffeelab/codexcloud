//! Tactical dispatch and search styles, independent of recruitment choices.
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};
// Select the variants before ocamljsify expands; it does not filter cfg variants.
macro_rules! planner_kind {
    ($($extra:ident,)*) => {
        #[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum PlannerKind {
            $($extra,)*
            Classic,
            Full,
            Queued,
        }
    };
}
#[cfg(all(feature = "training", not(target_arch = "wasm32")))]
planner_kind!(Learned,);
#[cfg(not(all(feature = "training", not(target_arch = "wasm32"))))]
planner_kind!();
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueueMode {
    Arena,
    CampaignBlock,
}
pub(crate) trait Planner {
    fn recruits_after_move(&self) -> bool {
        false
    }
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, CommandError>;
}
impl Planner for PlannerKind {
    fn recruits_after_move(&self) -> bool {
        true
    }
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, CommandError> {
        match self {
            #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
            Self::Learned => learned::Learned.plan(context, orders),
            Self::Classic => classic::Classic.plan(context, orders),
            Self::Full => full::Full.plan(context, orders),
            Self::Queued => queued::Queued.plan(context, orders),
        }
    }
}
/// Shared preparation deliberately selected by the current adapters.
fn prepare_priorities_and_recovery(
    context: &mut crate::ai::context::TacticalContext<'_, '_>,
    orders: &crate::ai::orders::TacticalOrders,
) -> Result<(), CommandError> {
    {
        let turn = context.turn_parts();
        turn.orders.tactics = orders.clone();
        turn.orders
            .tactics
            .objectives
            .order_by(|a, b| b.priority.cmp(&a.priority));
    }
    let recovered: Vec<_> = context
        .battle()
        .units()
        .iter()
        .filter(|unit| {
            context
                .battle()
                .belongs_to_phase(unit, context.battle().turn())
                && context.battle().can_act(unit.id)
                && unit.retreating == Some(true)
                && unit.hp == context.battle().rules().units[unit.kind].hp
        })
        .cloned()
        .collect();
    for unit in recovered {
        context.emit(super::intent_command(
            &unit,
            None,
            unit.rebuilding,
            unit.attack_spent,
        ))?;
    }
    Ok(())
}

/// Mechanical command/boundary loop. The adapter supplies its selected-session step.
fn drive<'game, 'catalog>(
    context: &mut crate::ai::context::TacticalContext<'game, 'catalog>,
    mut step: impl FnMut(
        &mut crate::ai::context::TacticalContext<'game, 'catalog>,
    ) -> Result<(), CommandError>,
) -> Result<crate::ai::orders::PlanningReport, CommandError> {
    loop {
        let work = {
            let turn = context.turn_parts();
            !*turn.finished || !turn.commands.is_empty() || turn.attack_continuation.is_some()
        };
        if !work {
            break;
        }
        context.flush()?;
        if context.is_boundary() {
            break;
        }
        let finished = *context.turn_parts().finished;
        if finished || context.battle().winner().is_some() {
            break;
        }
        step(context)?;
    }
    if !context.is_boundary() {
        context.flush()?;
    }
    Ok(context.turn_parts().report.clone())
}

/// Explicit coordination subscription used by the three current planner adapters.
fn prepare_coordination(
    game: &BattleView<'_, '_>,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Option<Vec<crate::ai::orders::Target>> {
    let update = crate::ai::policies::cooperation::assign(
        game,
        turn.coordination,
        &mut turn.orders.tactics,
        turn.report,
    );
    crate::ai::policies::cooperation::refresh_assignments(
        game,
        turn.coordination,
        &mut turn.orders.tactics,
        turn.report,
    )
    .or(update)
}
fn rescue_coordination(
    game: &BattleView<'_, '_>,
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    actions: hex_command_battle::ActionForecastSource<'_, '_>,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Result<bool, CommandError> {
    crate::ai::policies::cooperation::rescue(
        game,
        phases,
        actions,
        turn.coordination,
        &turn.orders.tactics,
        turn.report,
        turn.commands,
    )
}
fn finish_coordination(
    game: &BattleView<'_, '_>,
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Result<(), CommandError> {
    if *turn.finished {
        if crate::ai::policies::cooperation::fallback(
            game,
            phases,
            turn.coordination,
            &turn.orders.tactics,
            turn.report,
            turn.commands,
        )? {
            *turn.finished = false;
        }
        crate::ai::policies::cooperation::report(
            game,
            phases,
            turn.coordination,
            turn.orders,
            turn.report,
            *turn.finished,
        );
    }
    Ok(())
}
impl PlannerKind {
    pub const fn queued(self) -> bool {
        match self {
            Self::Queued => true,
            #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
            Self::Learned => false,
            Self::Classic | Self::Full => false,
        }
    }
}

pub(in crate::ai) mod candidates;
pub(in crate::ai) mod classic;
mod full;
pub(in crate::ai) mod queued;
pub(in crate::ai) mod search;

mod recruitment;

mod proposal;

#[cfg(all(feature = "training", not(target_arch = "wasm32")))]
pub(crate) mod learned;
