//! Tactical dispatch and search styles, independent of recruitment choices.
use super::{
    PlanningWorkspace, Recorder, advance_command, intent_command, move_command,
    objectives::{self},
    priority,
};
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlannerKind {
    Classic,
    #[serde(alias = "rush", alias = "mixed")]
    Full,
    Queued,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueueMode {
    Arena,
    CampaignBlock,
}
pub(crate) trait Planner {
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, CommandError>;
}
impl Planner for PlannerKind {
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
            unit.side == context.battle().turn()
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
        if context.workspace.finished || context.battle().winner().is_some() {
            break;
        }
        kind.choose(context.draft, context.workspace)?;
    }
    context.flush()?;
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
        let rules = game.rules();
        let planner = self;
        super::cooperation::assign(game, workspace);
        super::cooperation::refresh_assignments(game, workspace);
        workspace.escort.prepare(game, &workspace.report);
        if super::cooperation::rescue(game, workspace)? {
            return Ok(());
        }
        let orders = &workspace.orders.as_ref().expect("campaign orders").tactics;
        let restrained = orders.restrained;
        fn classic(planner: PlannerKind) -> bool {
            match planner {
                PlannerKind::Classic => true,
                PlannerKind::Full | PlannerKind::Queued => false,
            }
        }
        if classic(planner) {
            if workspace.classic_revision != workspace.cooperation.revision
                && workspace.tactical_remaining.is_some()
            {
                workspace.classic_objectives = orders
                    .targets
                    .iter()
                    .filter(|g| !g.id.starts_with("rebuild:"))
                    .cloned()
                    .collect();
                workspace
                    .classic_objectives
                    .extend(orders.rebuild_objectives(game));
                workspace.classic_revision = workspace.cooperation.revision;
            }
            if workspace.tactical_remaining.is_none() {
                workspace.classic_objectives = orders
                    .targets
                    .iter()
                    .filter(|g| !g.id.starts_with("rebuild:"))
                    .cloned()
                    .collect();
                workspace
                    .classic_objectives
                    .extend(orders.rebuild_objectives(game));
                workspace.classic_revision = workspace.cooperation.revision;
                if workspace.goals.is_none() && (!orders.campaign || game.turn() == Side::Enemy) {
                    workspace
                        .classic_objectives
                        .retain(|g| g.id.starts_with("rebuild:"));
                }
                workspace.classic_ids = game.classic_order(&workspace.classic_objectives).into();
                workspace
                    .classic_ids
                    .make_contiguous()
                    .sort_by_key(|id| workspace.escort.bound(*id) == 0.0);
                workspace.tactical_remaining = Some(workspace.classic_ids.len());
                for unit in game
                    .units()
                    .iter()
                    .filter(|u| u.side == game.turn() && rules.units[u.kind].leader)
                {
                    let required = orders.moving_actor(unit.id);
                    let (_, intent) = orders.rebuild_plan(game, unit.id, required);
                    if unit.rebuilding != intent {
                        workspace.commands.push_back(intent_command(
                            unit,
                            unit.retreating,
                            intent,
                            unit.attacked,
                        ));
                    }
                }
                if !workspace.commands.is_empty() {
                    return Ok(());
                }
            }
            if let Some(id) = workspace.classic_ids.pop_front() {
                let mut virtual_game = game.fork();
                let mut recorder = Recorder::new(game);
                virtual_game.classic_choose(
                    game,
                    id,
                    &workspace.classic_objectives,
                    orders,
                    &workspace.escort,
                    restrained,
                    &mut recorder,
                );
                recorder.flush_intents(&virtual_game);
                if let Some(error) = recorder.error {
                    return Err(error);
                }
                workspace.commands.extend(recorder.commands);
            } else {
                workspace.finished = true;
            }
        } else {
            let remaining = workspace.tactical_remaining.get_or_insert_with(|| {
                game.units()
                    .iter()
                    .filter(|u| u.side == game.turn())
                    .count()
                    * 3
            });
            if !planner.queued() {
                if *remaining == 0 {
                    workspace.finished = true;
                    return Ok(());
                }
                *remaining -= 1;
            }
            let mut intents = Vec::new();
            if workspace.prepared_goals.is_none() {
                workspace.prepared_goals = workspace.orders.as_ref().map(|orders| {
                    objectives::individual(
                        orders
                            .tactics
                            .targets
                            .iter()
                            .filter(|target| !target.id.starts_with("rebuild:"))
                            .cloned()
                            .collect(),
                    )
                });
            }
            let plan = if planner.queued() {
                priority::next_plan(game, workspace, &mut intents)
            } else {
                game.best_plan::<true, false>(
                    orders,
                    &workspace.escort,
                    workspace.seed,
                    workspace.prepared_goals.as_deref(),
                    &mut workspace.routes,
                    &mut workspace.commitments,
                    &mut intents,
                    &mut workspace.threats,
                    None,
                )
            };
            for (id, intent) in &intents {
                let actor = game.unit(*id).unwrap();
                workspace.commands.push_back(intent_command(
                    actor,
                    actor.retreating,
                    *intent,
                    actor.attacked,
                ));
            }
            if let Some(plan) = plan {
                let actor = game.unit(plan.unit).unwrap();
                let rebuilding = intents
                    .iter()
                    .find(|(id, _)| *id == plan.unit)
                    .map_or(actor.rebuilding, |(_, intent)| *intent);
                if let Some(kind) = plan.advance {
                    workspace
                        .commands
                        .push_back(advance_command(game, plan.unit, kind));
                } else {
                    if actor.tile != plan.tile {
                        workspace
                            .commands
                            .push_back(move_command(game, plan.unit, plan.tile));
                        let retreating = plan.withdrawing.then_some(true);
                        if actor.retreating != retreating {
                            workspace.commands.push_back(intent_command(
                                actor,
                                retreating,
                                rebuilding,
                                actor.attacked,
                            ));
                        }
                    }
                    if let (Some(target), Some(weapon)) = (plan.target, plan.weapon) {
                        workspace.attack_continuation = Some((
                            UnitId(plan.unit),
                            UnitId(target),
                            WeaponId(rules.units[actor.kind].attacks[weapon].id.clone()),
                        ));
                    }
                }
            } else {
                workspace.finished = true;
            }
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
pub(in crate::ai) mod escort;
mod full;
pub(in crate::ai) mod queued;
pub(in crate::ai) mod safety;
pub(in crate::ai) mod search;
