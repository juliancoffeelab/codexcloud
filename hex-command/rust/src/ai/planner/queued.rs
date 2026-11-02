//! A bounded duty queue replaces the global unit search after every command.
use crate::ai::{PlanningWorkspace, TacticalRole, planner::QueueMode, roles::role, tactics::Plan};
use crate::{ai::context::BattleView, data::*};

/// Turn-local queue cursor and deferred units.
pub struct UnitQueue {
    pub pending: Vec<UnitId>,
    pub cursor: usize,
    pub retry: Vec<UnitId>,
    pub second_pass: bool,
    pub active: Option<UnitId>,
    pub known_next_id: u32,
    pub dirty: bool,
    pub attempts: Vec<UnitId>,
}
impl BattleView<'_, '_> {
    fn priority_order(
        &self,
        policy: QueueMode,
        cache: &crate::ai::cheap::Workspace,
        escort: &crate::ai::search_escort::Workspace,
    ) -> Vec<UnitId> {
        let goals = &cache.goals;
        let clearing = &cache.clearing;
        let foes: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.hostile_to_phase(u, self.turn()))
            .collect();
        let mut ranked: Vec<_> = self
            .units()
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                self.belongs_to_phase(u, self.turn()) && !u.attacked && self.can_act(u.id)
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
                    || cache.proximity[unit.tile] as usize <= movement + range)
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
                            + if killed && cache.orders.terminal_attack(foe) {
                                100.0
                            } else {
                                0.0
                            };
                        offense = offense.max(value);
                    }
                }
                let personal = goals.iter().any(|g| {
                    !g.id.starts_with("rebuild:")
                        && self.can_pursue(unit, g)
                        && (g.actor.is_some() || g.family.is_some())
                });
                let urgency = goals
                    .iter()
                    .filter(|g| !g.id.starts_with("rebuild:") && self.can_pursue(unit, g))
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
                    for ally in self
                        .units()
                        .iter()
                        .filter(|u| self.belongs_to_phase(u, self.turn()) && u.id != unit.id)
                    {
                        if self.board().distance(unit.tile, ally.tile) as usize > movement + 1 {
                            continue;
                        }
                        let heal = rule
                            .healing
                            .min(self.rules().units[ally.kind].hp - ally.hp)
                            .max(0) as f64;
                        score += heal
                            * if cache.orders.important(self, ally) {
                                3.0
                            } else {
                                0.5
                            };
                    }
                }
                if unit.is_leader() && !personal {
                    score -= 8.0;
                }
                if cache.orders.campaign
                    && unit.side == Side::Player
                    && cache.orders.important(self, unit)
                    && !personal
                {
                    score -= 4.0;
                }
                (index, unit.id, score)
            })
            .collect();
        ranked.sort_unstable_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        ranked.into_iter().map(|(_, id, _)| UnitId(id)).collect()
    }
}
pub(in crate::ai) fn next_plan(
    game: &BattleView<'_, '_>,
    workspace: &mut PlanningWorkspace,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    crate::ai::cheap::prepare(game, workspace);
    let policy = if workspace.orders.as_ref().unwrap().tactics.campaign {
        QueueMode::CampaignBlock
    } else {
        QueueMode::Arena
    };
    if workspace.priority.is_none() {
        workspace.priority = Some(UnitQueue {
            pending: game.priority_order(policy, &workspace.cheap, &workspace.search_escort),
            cursor: 0,
            retry: vec![],
            second_pass: false,
            active: None,
            known_next_id: game.next_id(),
            dirty: false,
            attempts: vec![],
        });
    }
    let mut queue = workspace.priority.take().unwrap();
    let result = next_plan_inner(game, policy, workspace, &mut queue, intents);
    workspace.priority = Some(queue);
    result
}
fn next_plan_inner(
    game: &BattleView<'_, '_>,
    policy: QueueMode,
    workspace: &mut PlanningWorkspace,
    queue: &mut UnitQueue,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if queue.known_next_id != game.next_id() {
        let born: Vec<_> = game
            .priority_order(policy, &workspace.cheap, &workspace.search_escort)
            .into_iter()
            .filter(|id| id.0 >= queue.known_next_id)
            .collect();
        *workspace.tactical_remaining.as_mut().unwrap() += born.len() * 3;
        queue.pending.splice(queue.cursor..queue.cursor, born);
        queue.known_next_id = game.next_id();
    }
    if queue.dirty {
        queue.pending = game
            .priority_order(policy, &workspace.cheap, &workspace.search_escort)
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
        if workspace.tactical_remaining == Some(0) {
            return finish_intents(workspace, intents);
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
            return finish_intents(workspace, intents);
        };
        let Some(unit) = game
            .unit(id.0)
            .filter(|u| game.belongs_to_phase(u, game.turn()) && !u.attacked && game.can_act(u.id))
        else {
            continue;
        };
        if queue.attempts.iter().filter(|tried| **tried == id).count() >= 3 {
            continue;
        }
        if unit.moved
            && !game
                .units()
                .iter()
                .filter(|u| game.hostile_to_phase(u, game.turn()))
                .any(|target| {
                    game.weapons(unit)
                        .enumerate()
                        .any(|(weapon, _)| game.can_attack(unit, unit.tile, target, weapon))
                })
        {
            continue;
        }
        *workspace.tactical_remaining.as_mut().unwrap() -= 1;
        queue.attempts.push(id);
        finish_intents(workspace, intents);
        let mut plan = crate::ai::cheap::plan(game, id.0, workspace);
        if plan.is_none()
            && workspace
                .orders
                .as_ref()
                .unwrap()
                .tactics
                .important(game, unit)
            && (policy == QueueMode::CampaignBlock || game.proximity(unit.side, unit.tile) <= 6)
        {
            let mut ignored = vec![];
            plan = game.best_plan::<true, true>(
                &workspace.orders.as_ref().unwrap().tactics,
                &workspace.search_escort,
                workspace.seed,
                workspace.prepared_goals.as_deref(),
                &mut workspace.routes,
                &mut workspace.commitments,
                &mut ignored,
                &mut workspace.threats,
                Some(id.0),
            );
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
    workspace: &PlanningWorkspace,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if intents.is_empty() {
        intents.extend_from_slice(&workspace.cheap.rebuild_intents);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::Policy;
    use crate::logic::Game;
    #[test]
    fn campaign_rejects_arena_campaigner() {
        let rules = Rules::load().unwrap();
        let campaign = Game::start_campaign(
            &rules,
            "borrowed-tide",
            0,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
        );
        let mut blocks = Policy::Balanced.blocks();
        blocks.campaign = crate::ai::CampaignBlock::Arena;
        let mut workspace =
            PlanningWorkspace::new(7, 0, None, crate::ai::Diplomacy::Pragmatic).with_blocks(blocks);
        assert_eq!(
            crate::ai::choose_game_command(&campaign, Policy::Balanced, &mut workspace),
            Err(CommandError::BattleOnlyPolicy)
        );
    }
}

pub(super) struct Queued;
impl super::Planner for Queued {
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, crate::data::CommandError> {
        super::drive(super::PlannerKind::Queued, context, orders)
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
    super::search::choose(game, workspace, true)
}
