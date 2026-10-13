//! Tactical dispatch and search styles, independent of recruitment choices.
use super::{
    PlanningWorkspace, Recorder, advance_command, intent_command, move_command,
    objectives::{self, actor_matches},
    priority,
};
use crate::{data::*, logic::Game};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Planner {
    Classic,
    Rush,
    Mixed,
    Queued,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GlobalStyle {
    Rush,
    Mixed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueueMode {
    Arena,
    CampaignBlock,
}
impl Planner {
    pub const fn queued(self) -> bool {
        match self {
            Self::Queued => true,
            Self::Classic | Self::Rush | Self::Mixed => false,
        }
    }
    fn global_style(self) -> GlobalStyle {
        match self {
            Self::Rush => GlobalStyle::Rush,
            Self::Mixed => GlobalStyle::Mixed,
            Self::Classic | Self::Queued => unreachable!("global planner dispatch"),
        }
    }
    pub(super) fn choose(
        self,
        game: &Game<'_>,
        workspace: &mut PlanningWorkspace,
    ) -> Result<(), CommandError> {
        let rules = game.rules;
        let planner = self;
        super::cooperation::assign(game, workspace);
        super::cooperation::refresh_assignments(game, workspace);
        if super::cooperation::rescue(game, workspace)? {
            return Ok(());
        }
        let orders = &workspace.orders.as_ref().expect("campaign orders").tactics;
        let restrained = orders.restrained;
        fn classic(planner: Planner) -> bool {
            match planner {
                Planner::Classic => true,
                Planner::Rush | Planner::Mixed | Planner::Queued => false,
            }
        }
        if classic(planner) {
            if workspace.tactical_remaining.is_none() {
                workspace.classic_objectives = orders.targets.clone();
                if !orders.campaign && workspace.goals.is_none() {
                    workspace
                        .classic_objectives
                        .retain(|g| g.id.starts_with("rebuild:"));
                }
                workspace.classic_ids = game.classic_order(&workspace.classic_objectives).into();
                workspace.tactical_remaining = Some(workspace.classic_ids.len());
                for unit in game
                    .units
                    .iter()
                    .filter(|u| u.side == game.turn && rules.units[u.kind].leader)
                {
                    let required = workspace.classic_objectives.iter().any(|g| {
                        g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, unit.id)
                    });
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
                game.units.iter().filter(|u| u.side == game.turn).count() * 3
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
                workspace.prepared_goals = workspace
                    .goals
                    .as_ref()
                    .map(|g| objectives::individual(g.clone()));
            }
            let plan = if planner.queued() {
                priority::next_plan(game, workspace, &mut intents)
            } else {
                game.best_plan::<true, false>(
                    planner.global_style(),
                    orders,
                    workspace.seed,
                    workspace.goals.as_deref(),
                    workspace.prepared_goals.as_deref(),
                    restrained,
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
