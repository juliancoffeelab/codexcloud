//! A bounded duty queue replaces the global unit search after every command.
use crate::ai::{TacticalRole, planner::QueueMode, planner::proposal::Plan, roles::role};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};

/// Turn-local queue cursor and deferred units.
pub struct UnitQueue {
    pending: Vec<UnitId>,
    cursor: usize,
    retry: Vec<UnitId>,
    second_pass: bool,
    active: Option<UnitId>,
    known_next_id: u32,
    dirty: bool,
    attempts: Vec<UnitId>,
}
fn priority_order(
    game: &BattleView<'_, '_>,
    mode: QueueMode,
    candidates: &super::candidates::Workspace,
    escort: &crate::ai::policies::escort::travel::Workspace,
) -> Vec<UnitId> {
    PriorityEvaluation(game).priority_order(mode, candidates, escort)
}
struct PriorityEvaluation<'view, 'game, 'catalog>(&'view BattleView<'game, 'catalog>);
impl<'view, 'game, 'catalog> std::ops::Deref for PriorityEvaluation<'view, 'game, 'catalog> {
    type Target = BattleView<'game, 'catalog>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl PriorityEvaluation<'_, '_, '_> {
    fn priority_order(
        &self,
        policy: QueueMode,
        cache: &super::candidates::Workspace,
        escort: &crate::ai::policies::escort::travel::Workspace,
    ) -> Vec<UnitId> {
        let goals = cache.goals();
        let clearing = cache.clearing();
        let foes: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.hostile_to_faction(u, self.observer_faction()))
            .collect();
        let mut ranked: Vec<_> = self
            .units()
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                self.belongs_to_faction(u, self.observer_faction())
                    && !u.attack_spent
                    && self.can_act(u.id)
            })
            .map(|(index, unit)| {
                let rule = &self.rules().units[unit.kind];
                let movement = if unit.moved { 0 } else { rule.r#move };
                let zero_cost = rule.costs.iter().any(|c| *c == Some(0));
                let range = self
                    .weapons(unit)
                    .map(|a| a.range as usize)
                    .max()
                    .unwrap_or(0);
                let reachable = (zero_cost
                    || cache.proximity()[unit.tile] as usize <= movement + range)
                    .then(|| self.threat_reach_blocked(unit, self.occupied_tiles().clone()));
                let geometry: Vec<_> = self
                    .weapons(unit)
                    .map(|a| self.attack_origins(a.category, a.range))
                    .collect();
                let mut offense = 0.0f64;
                for &foe in &foes {
                    for (weapon, attack) in self.weapons(unit).enumerate() {
                        if !zero_cost
                            && self.board().distance(unit.tile, foe.tile) as usize
                                > movement + attack.range as usize
                        {
                            continue;
                        }
                        let origin = reachable.as_ref().and_then(|reachable| {
                            let origins = &geometry[weapon][foe.tile];
                            if origins.contains(unit.tile) {
                                Some(unit.tile)
                            } else {
                                let first = reachable.first_intersection(origins)?;
                                if policy == QueueMode::CampaignBlock {
                                    self.reachable(unit)
                                        .iter()
                                        .map(|&(t, _)| t)
                                        .find(|&t| origins.contains(t))
                                } else {
                                    Some(first)
                                }
                            }
                        });
                        let Some(origin) = origin else {
                            continue;
                        };
                        let forecast = self.exchange(unit, foe, weapon, origin);
                        let response = forecast.damage[1];
                        let kill_chance = forecast.death[1];
                        let killed = kill_chance == 1.0;
                        let value = forecast.effective[0] * 2.0 - response * 1.8
                            + kill_chance * (12.0 + self.recruit_cost(foe, foe.kind) as f64 * 0.3)
                            + if killed && cache.orders().terminal_attack(foe) {
                                100.0
                            } else {
                                0.0
                            };
                        offense = offense.max(value);
                    }
                }
                let personal = goals.iter().any(|g| {
                    !g.id.starts_with("rebuild:")
                        && crate::ai::policies::objectives::can_pursue(self, unit, g)
                        && (g.actor.is_some() || g.family.is_some())
                });
                let urgency = goals
                    .iter()
                    .filter(|g| {
                        !g.id.starts_with("rebuild:")
                            && crate::ai::policies::objectives::can_pursue(self, unit, g)
                    })
                    .map(|g| {
                        let distance = g
                            .tiles
                            .iter()
                            .map(|t| self.board().distance(unit.tile, self.site(t)))
                            .min()
                            .unwrap_or(u16::MAX) as f64;
                        g.weight / (distance + 3.0)
                    })
                    .fold(0.0f64, f64::max);
                let mut score = offense + urgency;
                if escort.bound(unit.id) > 0.0 {
                    score += 40.0;
                }
                if clearing.contains_key(&unit.id) {
                    score += 120.0;
                }
                if policy == QueueMode::CampaignBlock && personal {
                    score += 20.0;
                }
                if role(self, unit.kind) == TacticalRole::Frontline && !unit.is_leader() {
                    score += 4.0;
                }
                if role(self, unit.kind) == TacticalRole::Healer {
                    for ally in self.units().iter().filter(|u| {
                        self.belongs_to_faction(u, self.observer_faction()) && u.id != unit.id
                    }) {
                        if self.board().distance(unit.tile, ally.tile) as usize > movement + 1 {
                            continue;
                        }
                        let heal = rule
                            .healing
                            .min(self.rules().units[ally.kind].hp - ally.hp)
                            .max(0) as f64;
                        score += heal
                            * if cache.orders().important(self, ally) {
                                3.0
                            } else {
                                0.5
                            };
                    }
                }
                if unit.is_leader() && !personal {
                    score -= 8.0;
                }
                if cache.orders().campaign
                    && self.unit_faction(unit) == "player"
                    && cache.orders().important(self, unit)
                    && !personal
                {
                    score -= 4.0;
                }
                (index, unit.id, score)
            })
            .collect();
        ranked.order_unstable_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        ranked.into_iter().map(|(_, id, _)| UnitId(id)).collect()
    }
}
#[derive(Default)]
pub(in crate::ai) struct Session {
    queue: Option<UnitQueue>,
    candidates: super::candidates::Workspace,
    search: super::search::Session,
}
impl Session {
    pub(in crate::ai) fn replace_goals(&mut self, goals: Vec<crate::ai::orders::Target>) {
        self.search.replace_goals(goals);
    }
}

pub(in crate::ai) fn next_plan<'a, 'catalog>(
    game: &'a BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    session: &mut Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    super::candidates::prepare(
        game,
        &mut session.candidates,
        &turn.orders.tactics,
        turn.coordination.revision(),
    );
    let policy = if turn.orders.tactics.campaign {
        QueueMode::CampaignBlock
    } else {
        QueueMode::Arena
    };
    if session.queue.is_none() {
        session.queue = Some(UnitQueue {
            pending: priority_order(game, policy, &session.candidates, &session.search.escort()),
            cursor: 0,
            retry: vec![],
            second_pass: false,
            active: None,
            known_next_id: game.next_id(),
            dirty: false,
            attempts: vec![],
        });
    }
    let mut queue = session.queue.take().unwrap();
    let result = next_plan_inner(game, phases, policy, session, turn, &mut queue, intents);
    session.queue = Some(queue);
    result
}
fn next_plan_inner<'a, 'catalog>(
    game: &'a BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    policy: QueueMode,
    session: &mut Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
    queue: &mut UnitQueue,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if queue.known_next_id != game.next_id() {
        let born: Vec<_> =
            priority_order(game, policy, &session.candidates, &session.search.escort())
                .into_iter()
                .filter(|id| id.0 >= queue.known_next_id)
                .collect();
        session.search.add_actions(born.len() * 3);
        queue.pending.splice(queue.cursor..queue.cursor, born);
        queue.known_next_id = game.next_id();
    }
    if queue.dirty {
        queue.pending = priority_order(game, policy, &session.candidates, &session.search.escort())
            .into_iter()
            .filter(|id| queue.attempts.iter().filter(|tried| *tried == id).count() < 3)
            .collect();
        queue.cursor = 0;
        queue.retry.clear();
        queue.second_pass = false;
        queue.active = None;
        queue.dirty = false;
    }
    loop {
        if session.search.exhausted() {
            return finish_intents(&session.candidates, intents);
        }
        let id = if let Some(id) = queue.active.take() {
            id
        } else if let Some(id) = queue.pending.get(queue.cursor).copied() {
            queue.cursor += 1;
            id
        } else if !queue.second_pass {
            queue.pending = std::mem::take(&mut queue.retry);
            queue.cursor = 0;
            queue.second_pass = true;
            continue;
        } else {
            return finish_intents(&session.candidates, intents);
        };
        let Some(unit) = game.unit(id.0).filter(|u| {
            game.belongs_to_faction(u, game.observer_faction())
                && !u.attack_spent
                && game.can_act(u.id)
        }) else {
            continue;
        };
        if queue.attempts.iter().filter(|tried| **tried == id).count() >= 3 {
            continue;
        }
        if unit.moved
            && !game
                .units()
                .iter()
                .filter(|u| game.hostile_to_faction(u, game.observer_faction()))
                .any(|target| {
                    game.weapons(unit)
                        .enumerate()
                        .any(|(weapon, _)| game.can_attack(unit, unit.tile, target, weapon))
                })
        {
            continue;
        }
        session.search.spend_action();
        queue.attempts.push(id);
        finish_intents(&session.candidates, intents);
        let mut plan = super::candidates::plan(
            game,
            phases,
            id.0,
            &mut session.candidates,
            session.search.candidate_shared(),
            super::candidates::EvaluationInput {
                seed: turn.seed,
                orders: &turn.orders.tactics,
                recruitment: turn.recruitment,
            },
        );
        if plan.is_none()
            && turn.orders.tactics.important(game, unit)
            && (policy == QueueMode::CampaignBlock
                || crate::ai::policies::objectives::proximity(
                    game,
                    game.unit_faction(unit),
                    unit.tile,
                ) <= 6)
        {
            plan = session.search.focused(game, phases, turn, id.0);
        }
        if plan.is_some() {
            queue.active = Some(id);
            queue.dirty = true;
            return plan;
        }
        if !queue.second_pass && !unit.moved && !queue.retry.contains(&id) {
            queue.retry.push(id);
        }
    }
}

fn finish_intents(
    cache: &super::candidates::Workspace,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if intents.is_empty() {
        intents.extend_from_slice(cache.rebuild_intents());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{PlanningWorkspace, Policy};
    use crate::logic::Game;
    #[test]
    fn campaign_rejects_arena_campaigner() {
        let rules = Rules::load().unwrap();
        let campaign =
            Game::start_campaign(&rules, "mirror-night", 0, vec![], 0, vec![], vec![], vec![]);
        let mut blocks = Policy::Balanced.blocks();
        blocks.campaign = crate::ai::CampaignBlock::Arena;
        let mut workspace = PlanningWorkspace::new(7, 0, None, crate::ai::Diplomacy::Unrestricted)
            .with_blocks(blocks);
        assert_eq!(
            crate::ai::choose_game_command(&campaign, Policy::Balanced, &mut workspace),
            Err(CommandError::BattleOnlyPolicy)
        );
    }
}

pub(super) struct Queued;
impl super::Planner for Queued {
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
            let (game, phases, actions, mut parts) = context.queued_parts();
            coordinated_step(game, phases, actions, &mut parts)
        })
    }
}

pub(super) fn prepare(
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    session: &mut Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
) {
    super::search::prepare(phases, &mut session.search, turn);
}
pub(super) fn choose<'a, 'catalog>(
    game: &'a BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    session: &mut Session,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Result<(), CommandError> {
    super::search::initialize(game, &mut session.search, turn);
    let mut intents = Vec::new();
    let plan = next_plan(game, phases, session, turn, &mut intents);
    super::search::publish_plan(game, turn, plan, &intents)
}

fn coordinated_step<'view, 'catalog>(
    game: &'view crate::ai::context::BattleView<'view, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'view, 'catalog>,
    actions: hex_command_battle::ActionForecastSource<'view, 'catalog>,
    parts: &mut crate::ai::runtime::PlannerParts<'_, Session>,
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
