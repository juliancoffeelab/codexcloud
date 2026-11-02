//! Tactical dispatch and search styles, independent of recruitment choices.
use super::PlanningWorkspace;
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlannerKind {
    Classic,
    Full,
    Queued,
}
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
            Self::Classic => classic::Classic.plan(context, orders),
            Self::Full => full::Full.plan(context, orders),
            Self::Queued => queued::Queued.plan(context, orders),
        }
    }
}
fn drive(
    kind: PlannerKind,
    context: &mut crate::ai::context::TacticalContext<'_, '_>,
    orders: &crate::ai::orders::TacticalOrders,
) -> Result<crate::ai::orders::PlanningReport, CommandError> {
    context
        .workspace
        .orders
        .as_mut()
        .expect("turn orders")
        .tactics = orders.clone();
    context
        .workspace
        .orders
        .as_mut()
        .unwrap()
        .tactics
        .objectives
        .sort_by(|a, b| b.priority.cmp(&a.priority).then(a.id.cmp(&b.id)));
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
            unit.attacked,
        ))?;
    }
    while !context.workspace.finished
        || !context.workspace.commands.is_empty()
        || context.workspace.attack_continuation.is_some()
    {
        context.flush()?;
        if context.combat_boundary {
            break;
        }
        if context.workspace.finished || context.battle().winner().is_some() {
            break;
        }
        kind.choose(context.draft, context.workspace)?;
    }
    if !context.combat_boundary {
        context.flush()?;
    }
    Ok(context.workspace.report.clone())
}

impl PlannerKind {
    pub const fn queued(self) -> bool {
        match self {
            Self::Queued => true,
            Self::Classic | Self::Full => false,
        }
    }
    pub(super) fn choose(
        self,
        game: &BattleView<'_, '_>,
        workspace: &mut PlanningWorkspace,
    ) -> Result<(), CommandError> {
        super::cooperation::assign(game, workspace);
        super::cooperation::refresh_assignments(game, workspace);
        match self {
            Self::Classic => classic::prepare(game, workspace),
            Self::Full => full::prepare(game, workspace),
            Self::Queued => queued::prepare(game, workspace),
        }
        if super::cooperation::rescue(game, workspace)? {
            return Ok(());
        }
        match self {
            Self::Classic => classic::choose(game, workspace)?,
            Self::Full => full::choose(game, workspace)?,
            Self::Queued => queued::choose(game, workspace)?,
        }
        if workspace.finished {
            super::cooperation::fallback(game, workspace)?;
            super::cooperation::report(game, workspace);
        }
        Ok(())
    }
}

pub(in crate::ai) mod assignments;
pub(in crate::ai) mod candidates;
pub(in crate::ai) mod classic;
mod full;
pub(in crate::ai) mod queued;
pub(in crate::ai) mod safety;
pub(in crate::ai) mod search;

mod recruitment;
