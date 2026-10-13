//! A bounded duty queue replaces the global unit search after every command.
use super::{PlanningWorkspace, Policy, TacticalRole, roles::role, tactics::Plan};
use crate::{data::*, logic::Game};
use serde::{Deserialize, Serialize};

/// Persisted queue cursor and deferred units; derived scoring caches stay private.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrioritySnapshot {
    pub pending: Vec<UnitId>,
    pub cursor: usize,
    pub retry: Vec<UnitId>,
    pub second_pass: bool,
    pub active: Option<UnitId>,
    pub known_next_id: u32,
    pub dirty: bool,
    pub attempts: Vec<UnitId>,
}
impl Game<'_> {
    fn priority_order(&self, policy: Policy, cache: &super::cheap::Workspace) -> Vec<UnitId> {
        let goals = &cache.goals;
        let clearing = &cache.clearing;
        let foes: Vec<_> = self.units.iter().filter(|u| u.side != self.turn).collect();
        let mut ranked: Vec<_> = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.side == self.turn && !u.attacked)
            .map(|(index, unit)| {
                let rule = &self.rules.units[unit.kind];
                let movement = if unit.moved { 0 } else { rule.r#move };
                let zero_cost = rule.costs.iter().any(|c| *c == Some(0));
                let range = rule
                    .attacks
                    .iter()
                    .map(|a| a.range as usize)
                    .max()
                    .unwrap_or(0);
                let reachable = (zero_cost
                    || cache.proximity[unit.tile] as usize <= movement + range)
                    .then(|| self.threat_reach_blocked(unit, self.occupied_tiles.clone()));
                let geometry: Vec<_> = rule
                    .attacks
                    .iter()
                    .map(|a| self.attack_origins(a.category, a.range))
                    .collect();
                let mut offense = 0.0f64;
                for &foe in &foes {
                    for (weapon, attack) in rule.attacks.iter().enumerate() {
                        if !zero_cost
                            && self.board.distance(unit.tile, foe.tile) as usize
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
                                if policy == Policy::Priority {
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
                        let (damage, response, _) = self.preview(unit, foe, weapon, origin);
                        let killed = damage >= foe.hp;
                        let value = damage.min(foe.hp) as f64 * 2.0 - response as f64 * 1.8
                            + if killed {
                                12.0 + self.rules.units[foe.kind].cost as f64 * 0.3
                            } else {
                                0.0
                            }
                            + if killed && super::cheap::terminal_attack(self, unit, foe) {
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
                            .map(|t| self.board.distance(unit.tile, self.site(t)))
                            .min()
                            .unwrap_or(u16::MAX) as f64;
                        g.weight / (distance + 3.0)
                    })
                    .fold(0.0f64, f64::max);
                let mut score = offense + urgency;
                if clearing.contains_key(&unit.id) {
                    score += 120.0;
                }
                if policy == Policy::Priority && personal {
                    score += 20.0;
                }
                if role(self, unit.kind) == TacticalRole::Frontline && !rule.leader {
                    score += 4.0;
                }
                if role(self, unit.kind) == TacticalRole::Healer {
                    for ally in self
                        .units
                        .iter()
                        .filter(|u| u.side == self.turn && u.id != unit.id)
                    {
                        if self.board.distance(unit.tile, ally.tile) as usize > movement + 1 {
                            continue;
                        }
                        let heal = rule
                            .healing
                            .min(self.rules.units[ally.kind].hp - ally.hp)
                            .max(0) as f64;
                        score += heal * if self.essential(ally) { 3.0 } else { 0.5 };
                    }
                }
                if rule.leader && !personal {
                    score -= 8.0;
                }
                if self.campaign.is_some()
                    && unit.side == Side::Player
                    && self.essential(unit)
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
pub(super) fn next_plan(
    game: &Game<'_>,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if policy == Policy::Priority && super::cheap::crisis(game) {
        let remaining = workspace.tactical_remaining.as_mut().unwrap();
        if *remaining == 0 {
            super::cheap::prepare(game, workspace);
            return finish_intents(workspace, intents);
        }
        *remaining -= 1;
        return game.best_plan::<true, false>(
            Policy::Mixed,
            workspace.seed,
            workspace.goals.as_deref(),
            workspace.prepared_goals.as_deref(),
            super::cheap::restrained(game, workspace),
            &mut workspace.routes,
            &mut workspace.commitments,
            intents,
            &mut workspace.threats,
            None,
        );
    }
    super::cheap::prepare(game, workspace);
    let policy = if game.campaign.is_none() {
        Policy::Battler
    } else {
        policy
    };
    if workspace.priority.is_none() {
        workspace.priority = Some(PrioritySnapshot {
            pending: game.priority_order(policy, &workspace.cheap),
            cursor: 0,
            retry: vec![],
            second_pass: false,
            active: None,
            known_next_id: game.next_id,
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
    game: &Game<'_>,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
    queue: &mut PrioritySnapshot,
    intents: &mut Vec<(u32, Option<bool>)>,
) -> Option<Plan> {
    if queue.known_next_id != game.next_id {
        let born: Vec<_> = game
            .priority_order(policy, &workspace.cheap)
            .into_iter()
            .filter(|id| id.0 >= queue.known_next_id)
            .collect();
        *workspace.tactical_remaining.as_mut().unwrap() += born.len() * 3;
        queue.pending.splice(queue.cursor..queue.cursor, born);
        queue.known_next_id = game.next_id;
    }
    if queue.dirty {
        queue.pending = game
            .priority_order(policy, &workspace.cheap)
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
            .filter(|u| u.side == game.turn && !u.attacked)
        else {
            continue;
        };
        if queue.attempts.iter().filter(|tried| **tried == id).count() >= 3 {
            continue;
        }
        if unit.moved
            && !game
                .units
                .iter()
                .filter(|u| u.side != game.turn)
                .any(|target| {
                    game.rules.units[unit.kind]
                        .attacks
                        .iter()
                        .enumerate()
                        .any(|(weapon, _)| game.can_attack(unit, unit.tile, target, weapon))
                })
        {
            continue;
        }
        *workspace.tactical_remaining.as_mut().unwrap() -= 1;
        queue.attempts.push(id);
        finish_intents(workspace, intents);
        let mut plan = super::cheap::plan(game, id.0, workspace);
        if plan.is_none()
            && game.essential(unit)
            && (policy == Policy::Priority || game.proximity(unit.side, unit.tile) <= 6)
        {
            let mut ignored = vec![];
            plan = game.best_plan::<true, true>(
                super::Policy::Mixed,
                workspace.seed,
                workspace.goals.as_deref(),
                workspace.prepared_goals.as_deref(),
                super::cheap::restrained(game, workspace),
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
    #[test]
    fn complex_assaults_use_full_planning_after_snapshot_restoration() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "ashen-oath", 0, vec![], 0, vec![], vec![], vec![]);
        assert!(super::super::cheap::heavy_campaign(&game));
        let mut priority = PlanningWorkspace::new(7, 1, None, super::super::Diplomacy::Pragmatic);
        let mut mixed = PlanningWorkspace::new(7, 1, None, super::super::Diplomacy::Pragmatic);
        for _ in 0..6 {
            let command =
                super::super::choose_game_command(&rules, &game, Policy::Priority, &mut priority)
                    .unwrap();
            assert_eq!(
                command,
                super::super::choose_game_command(&rules, &game, Policy::Mixed, &mut mixed)
                    .unwrap()
            );
            let snapshot = priority.snapshot();
            snapshot.validate(&rules, &game.state()).unwrap();
            priority = PlanningWorkspace::from_snapshot(
                serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap(),
            );
            if matches!(command, Command::EndTurn) {
                break;
            }
            game.execute(command).unwrap();
        }
    }
    #[test]
    fn queues_roundtrip_and_reject_invalid_cursors_and_campaign_battlers() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let mut workspace = PlanningWorkspace::new(7, 0, None, super::super::Diplomacy::Pragmatic);
        for _ in 0..8 {
            let previous = workspace.snapshot();
            previous.validate(&rules, &game.state()).unwrap();
            let mut restored = PlanningWorkspace::from_snapshot(
                serde_json::from_str(&serde_json::to_string(&previous).unwrap()).unwrap(),
            );
            let command =
                super::super::choose_game_command(&rules, &game, Policy::Priority, &mut workspace)
                    .unwrap();
            assert_eq!(
                command,
                super::super::choose_game_command(&rules, &game, Policy::Priority, &mut restored)
                    .unwrap()
            );
            let saved = workspace.snapshot();
            saved.validate(&rules, &game.state()).unwrap();
            if let Some(queue) = &saved.priority {
                assert!(queue.pending.len() <= game.units.len());
                let mut invalid = saved.clone();
                invalid.priority.as_mut().unwrap().cursor = queue.pending.len() + 1;
                assert!(invalid.validate(&rules, &game.state()).is_err());
            }
            if matches!(command, Command::EndTurn) {
                break;
            }
            game.execute(command).unwrap();
            workspace = PlanningWorkspace::from_snapshot(
                serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap(),
            );
        }
        let mut campaign = Game::start_campaign(
            &rules,
            "borrowed-tide",
            0,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
        );
        let mut workspace = PlanningWorkspace::new(7, 0, None, super::super::Diplomacy::Pragmatic);
        assert_eq!(
            super::super::choose_game_command(&rules, &campaign, Policy::Battler, &mut workspace),
            Err(CommandError::BattleOnlyPolicy)
        );
        for policy in Policy::ALL.into_iter().filter(|p| p.legacy().is_some()) {
            assert_eq!(
                super::super::choose_game_command(&rules, &campaign, policy, &mut workspace),
                Err(CommandError::ReferenceOnlyPolicy)
            );
            let mut snapshot = workspace.snapshot();
            snapshot.policy = Some(policy);
            assert!(snapshot.validate(&rules, &campaign.state()).is_err());
        }
        let normal = campaign.policy_objectives(None);
        let normal_weight = normal
            .iter()
            .find(|g| g.id == "opposing-leader")
            .unwrap()
            .weight;
        let sites: Vec<_> = campaign
            .mission
            .as_ref()
            .unwrap()
            .goals
            .iter()
            .filter(|g| !matches!(g, Goal::Escort { .. } | Goal::Escape { .. }))
            .flat_map(Goal::sites)
            .map(|s| campaign.site(&s))
            .collect();
        for site in sites {
            campaign.owners[site] = Some(Side::Player);
        }
        super::super::cheap::prepare(&campaign, &mut workspace);
        assert!(
            workspace
                .cheap
                .goals
                .iter()
                .find(|g| g.id == "opposing-leader")
                .unwrap()
                .weight
                > normal_weight
        );
        assert!(
            workspace
                .cheap
                .goals
                .iter()
                .any(|g| g.kind == ObjectiveKind::Escort && g.actor.is_some())
        );
        let leader = campaign
            .units
            .iter()
            .position(|u| u.side == Side::Player && rules.units[u.kind].leader)
            .unwrap();
        let enemy = campaign
            .units
            .iter()
            .position(|u| u.side == Side::Enemy)
            .unwrap();
        let tile = campaign.board.adjacent[campaign.units[leader].tile]
            .iter()
            .copied()
            .find(|&tile| campaign.at(tile).is_none())
            .unwrap();
        campaign.units[enemy].tile = tile;
        campaign.units[leader].hp = rules.units[campaign.units[leader].kind].hp / 2;
        campaign.roster_changed();
        assert!(super::super::cheap::crisis(&campaign));
    }
}
