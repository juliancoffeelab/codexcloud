//! Numeric observations of Campaigner orders; no guard assignment or action scoring.
use crate::{
    ai::{context::BattleView, orders::*},
    data::*,
};
use hex_command_battle::RuntimeCommand as Command;
use std::rc::Rc;

pub const NAMES: [&str; 57] = [
    "goto_assigned",
    "goto_priority",
    "goto_turns_before",
    "goto_turns_after",
    "goto_reachable",
    "goto_at_before",
    "goto_at_after",
    "occupy_assigned",
    "occupy_priority",
    "occupy_turns_before",
    "occupy_turns_after",
    "occupy_reachable",
    "occupy_at_before",
    "occupy_at_after",
    "passage_assigned",
    "passage_priority",
    "passage_turns_before",
    "passage_turns_after",
    "passage_reachable",
    "passage_at_before",
    "passage_at_after",
    "passage_corridor_before",
    "passage_corridor_after",
    "escort_subject",
    "escort_guard_eligible",
    "escort_priority",
    "escort_guards_required",
    "escort_guards_nearby",
    "escort_distance_before",
    "escort_distance_after",
    "duty_progress_known",
    "duty_progress_fraction",
    "duty_hold_remaining_known",
    "duty_hold_turns_remaining",
    "terminal_arrival",
    "terminal_attack_target",
    "goto_unfinished",
    "goto_actionable",
    "occupy_unfinished",
    "occupy_actionable",
    "escort_unfinished",
    "escort_actionable",
    "passage_unfinished",
    "passage_actionable",
    "critical_unfinished",
    "protected_units",
    "pending_arrivals",
    "diplomatic_restraint",
    "withdrawal_protection",
    "protected_retreaters",
    "mediated_enemies",
    "nearest_enemy_withdrawing",
    "target_mediated",
    "withdrawal_lane_before",
    "withdrawal_lane_after",
    "deadline_known",
    "deadline_rounds_remaining",
];
pub const CHANNEL_NAMES: [&str; 12] = [
    "goto_actor",
    "goto_destination",
    "occupy_actor",
    "occupy_destination",
    "passage_actor",
    "passage_destination",
    "escort_subject",
    "escort_guard_count",
    "survive",
    "protected_retreating",
    "mediated_enemy",
    "withdrawal_lane",
];

fn priority(p: ObjectivePriority) -> f32 {
    match p {
        ObjectivePriority::Normal => 1. / 3.,
        ObjectivePriority::Important => 2. / 3.,
        ObjectivePriority::Critical => 1.,
    }
}
fn timing(progress: &ObjectiveProgress, x: &mut [f32]) {
    x[30..34].fill(0.);
    x[30] = (progress.total > 0) as u8 as f32;
    x[31] = progress.current as f32 / progress.total.max(1) as f32;
    if let Some(n) = progress.hold_turns_remaining {
        x[32] = 1.;
        x[33] = n as f32 / 30.;
    }
}
struct Route {
    kind: usize,
    id: String,
    actor: u32,
    destination: usize,
    priority: ObjectivePriority,
    costs: Rc<Vec<u16>>,
    corridor: Vec<bool>,
}
struct Escort {
    id: String,
    actor: u32,
    guards: usize,
    selector: UnitSelector,
    priority: ObjectivePriority,
    nearby: usize,
}
#[derive(Default)]
pub(super) struct Duties {
    routes: Vec<Route>,
    escorts: Vec<Escort>,
    globals: [f32; 15],
}
impl Duties {
    pub fn new(g: &BattleView<'_, '_>, orders: &TacticalOrders) -> Self {
        let mut result = Self::default();
        // The appended schema is campaign-only, making zero padding truthful for
        // legacy deathmatch corpora, including their old rebuild/hold targets.
        if !orders.campaign {
            return result;
        }
        for order in &orders.objectives {
            let (kind, actor, destination) = match &order.objective {
                TacticalObjective::GoTo { actor, destination } => (0, actor.0, Some(destination)),
                TacticalObjective::Occupy { actor, location } => (1, actor.0, Some(location)),
                TacticalObjective::ClearPassage { actor, destination } => {
                    (2, actor.0, Some(destination))
                }
                TacticalObjective::Escort {
                    actor,
                    guards,
                    selector,
                } => {
                    let Some(subject) = g.unit(actor.0) else {
                        continue;
                    };
                    let eligible: Vec<_> = g
                        .units()
                        .iter()
                        .filter(|u| {
                            u.id != actor.0
                                && g.belongs_to_faction(u, g.observer_faction())
                                && !u.is_leader()
                                && selector.accepts(g, u.kind)
                        })
                        .collect();
                    let nearby = eligible
                        .iter()
                        .filter(|u| g.board().distance(u.tile, subject.tile) <= 1)
                        .count();
                    let unmet = nearby < *guards;
                    let actionable = unmet && eligible.iter().any(|u| g.can_act(u.id) && !u.moved);
                    result.count(2, order.priority, unmet, actionable);
                    result.escorts.push(Escort {
                        id: order.id.clone(),
                        actor: actor.0,
                        guards: *guards,
                        selector: selector.clone(),
                        priority: order.priority,
                        nearby,
                    });
                    continue;
                }
            };
            let Some(unit) = g.unit(actor) else {
                continue;
            };
            let destination = g.site(&destination.unwrap().0);
            let costs = g.routes(unit.kind, &[destination]);
            let mut corridor = vec![];
            let (unmet, actionable) = if kind == 2 {
                let from = g.routes_from(unit.kind, unit.tile);
                let total = costs[unit.tile];
                corridor = (0..g.terrain().len())
                    .map(|tile| {
                        total != u16::MAX
                            && from[tile] != u16::MAX
                            && costs[tile] != u16::MAX
                            && u32::from(from[tile]) + u32::from(costs[tile]) == u32::from(total)
                    })
                    .collect();
                let blockers: Vec<_> = g
                    .units()
                    .iter()
                    .filter(|u| {
                        u.id != actor
                            && g.belongs_to_faction(u, g.observer_faction())
                            && corridor[u.tile]
                    })
                    .collect();
                (
                    !blockers.is_empty(),
                    blockers.iter().any(|u| g.can_act(u.id) && !u.moved),
                )
            } else {
                let incomplete = unit.tile != destination
                    || kind == 1
                        && orders
                            .progress
                            .iter()
                            .any(|p| p.id == order.id && !p.satisfied);
                (
                    incomplete,
                    unit.tile != destination
                        && costs[unit.tile] != u16::MAX
                        && g.can_act(actor)
                        && !unit.moved,
                )
            };
            result.count(
                if kind == 2 { 3 } else { kind },
                order.priority,
                unmet,
                actionable,
            );
            result.routes.push(Route {
                kind,
                id: order.id.clone(),
                actor,
                destination,
                priority: order.priority,
                costs,
                corridor,
            });
        }
        result.globals[9] = g
            .units()
            .iter()
            .filter(|u| g.belongs_to_faction(u, g.observer_faction()) && orders.protects(u.id))
            .count() as f32
            / 10.;
        result.globals[10] = orders.pending_arrivals as u8 as f32;
        result.globals[11] = orders.restrained as u8 as f32;
        result.globals[12] = orders.protect_withdrawal as u8 as f32;
        if orders.protect_withdrawal {
            for enemy in g
                .units()
                .iter()
                .filter(|u| g.hostile_to_faction(u, g.observer_faction()))
            {
                result.globals[13] += (enemy.retreating == Some(true)) as u8 as f32 / 10.;
                result.globals[14] +=
                    orders.mediated.contains(&UnitId(enemy.id)) as u8 as f32 / 10.;
            }
        }
        result
    }
    fn count(&mut self, kind: usize, p: ObjectivePriority, unmet: bool, actionable: bool) {
        self.globals[kind * 2] += unmet as u8 as f32 / 10.;
        self.globals[kind * 2 + 1] += actionable as u8 as f32 / 10.;
        self.globals[8] += (unmet && p == ObjectivePriority::Critical) as u8 as f32 / 10.;
    }
    pub fn encode(
        &self,
        g: &BattleView<'_, '_>,
        orders: &TacticalOrders,
        command: &Command,
        x: &mut [f32],
    ) {
        if !orders.campaign {
            return;
        }
        if let Some(round) = orders.deadline_round {
            x[55] = 1.;
            x[56] = round.saturating_sub(g.round()) as f32 / 30.;
        }
        x[36..51].copy_from_slice(&self.globals);
        if let Some(p) = orders
            .progress
            .iter()
            .filter(|p| !p.satisfied)
            .min_by_key(|p| (p.hold_turns_remaining.unwrap_or(u32::MAX),))
        {
            timing(p, x);
        }
        let (id, tile) = match command {
            Command::Move { unit, tile } => (unit.0, Some(tile.0)),
            Command::Attack { unit, .. } | Command::Advance { unit, .. } => (unit.0, None),
            _ => return,
        };
        let unit = g.unit(id).unwrap();
        let after = tile.unwrap_or(unit.tile);
        if orders.protect_withdrawal {
            x[51] = g
                .units()
                .iter()
                .filter(|u| g.hostile(unit, u))
                .min_by_key(|u| (g.board().distance(after, u.tile), u.id))
                .is_some_and(|u| u.retreating == Some(true)) as u8 as f32;
            if let Command::Attack { target, .. } = command {
                x[52] = orders.mediated.contains(target) as u8 as f32;
            }
            x[53] = orders
                .withdrawal_tiles
                .contains(&TileId(g.board().ids[unit.tile])) as u8 as f32;
            x[54] = orders
                .withdrawal_tiles
                .contains(&TileId(g.board().ids[after])) as u8 as f32;
        }
        let mut primary: Option<(&str, ObjectivePriority, u16)> = None;
        let mut consider = |name, p, distance| {
            if primary.is_none_or(|(_, old, d)| p > old || p == old && distance < d) {
                primary = Some((name, p, distance));
            }
        };
        for kind in 0..3 {
            if let Some(r) = self
                .routes
                .iter()
                .filter(|r| r.kind == kind && r.actor == id)
                .max_by_key(|r| (r.priority, std::cmp::Reverse(r.costs[after])))
            {
                let start = kind * 7;
                let movement = g.rules().units[unit.kind].r#move.max(1) as f32;
                x[start] = 1.;
                x[start + 1] = priority(r.priority);
                x[start + 2] = if r.costs[unit.tile] == u16::MAX {
                    0.
                } else {
                    r.costs[unit.tile] as f32 / movement / 10.
                };
                x[start + 3] = if r.costs[after] == u16::MAX {
                    0.
                } else {
                    r.costs[after] as f32 / movement / 10.
                };
                x[start + 4] = (r.costs[after] != u16::MAX) as u8 as f32;
                x[start + 5] = (unit.tile == r.destination) as u8 as f32;
                x[start + 6] = (after == r.destination) as u8 as f32;
                consider(r.id.as_str(), r.priority, r.costs[after]);
            }
        }
        for r in self.routes.iter().filter(|r| r.kind == 2 && r.actor != id) {
            x[21] = x[21].max(r.corridor[unit.tile] as u8 as f32);
            x[22] = x[22].max(r.corridor[after] as u8 as f32);
        }
        if let Some(e) = self
            .escorts
            .iter()
            .filter(|e| e.actor == id || !unit.is_leader() && e.selector.accepts(g, unit.kind))
            .max_by_key(|e| {
                (
                    e.priority,
                    std::cmp::Reverse(g.board().distance(unit.tile, g.unit(e.actor).unwrap().tile)),
                )
            })
        {
            let subject = g.unit(e.actor).unwrap();
            x[23] = (id == e.actor) as u8 as f32;
            x[24] = (id != e.actor) as u8 as f32;
            x[25] = priority(e.priority);
            x[26] = e.guards as f32 / 10.;
            x[27] = e.nearby as f32 / 10.;
            x[28] = g.board().distance(unit.tile, subject.tile) as f32 / 20.;
            x[29] = g.board().distance(after, subject.tile) as f32 / 20.;
            consider(
                e.id.as_str(),
                e.priority,
                g.board().distance(after, subject.tile),
            );
        }
        if let Some((id, _, _)) = primary {
            if let Some(p) = orders
                .progress
                .iter()
                .filter(|p| {
                    id == p.id
                        || id
                            .strip_prefix(&p.id)
                            .is_some_and(|tail| tail.starts_with(':'))
                })
                .max_by_key(|p| p.id.len())
            {
                timing(p, x);
            }
        }
        if let Command::Move { .. } = command {
            x[34] = orders.arrives(g, unit, after) as u8 as f32;
        }
        if let Command::Attack { target, .. } = command {
            x[35] = orders.terminal_targets.contains(target) as u8 as f32;
        }
    }
    pub fn paint(
        &self,
        g: &BattleView<'_, '_>,
        orders: &TacticalOrders,
        sites: &[usize],
        values: &mut [f32],
        cells: usize,
    ) {
        if !orders.campaign {
            return;
        }
        let mut mark = |channel: usize, tile: usize, value: f32| {
            let slot = (53 + channel) * cells + sites[tile];
            values[slot] = values[slot].max(value);
        };
        for r in &self.routes {
            mark(
                r.kind * 2,
                g.unit(r.actor).unwrap().tile,
                priority(r.priority),
            );
            mark(r.kind * 2 + 1, r.destination, priority(r.priority));
        }
        for e in &self.escorts {
            let tile = g.unit(e.actor).unwrap().tile;
            mark(6, tile, priority(e.priority));
            mark(7, tile, e.guards as f32 / 10.);
        }
        for u in g
            .units()
            .iter()
            .filter(|u| g.belongs_to_faction(u, g.observer_faction()) && orders.protects(u.id))
        {
            mark(8, u.tile, 1.);
        }
        if orders.protect_withdrawal {
            for enemy in g
                .units()
                .iter()
                .filter(|u| g.hostile_to_faction(u, g.observer_faction()))
            {
                if enemy.retreating == Some(true) {
                    mark(9, enemy.tile, 1.);
                }
                if orders.mediated.contains(&UnitId(enemy.id)) {
                    mark(10, enemy.tile, 1.);
                }
            }
            for &tile in &orders.withdrawal_tiles {
                mark(11, g.site(&tile.0), 1.);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::Game,
        training::{self, Dataset, FEATURE_NAMES, LEGACY_FEATURES},
    };

    #[test]
    fn diplomacy_filters_attacks_and_marks_protected_units_and_lanes() {
        let rules = Rules::load().unwrap();
        let original = Game::new(&rules, "mirror");
        let actor = original
            .units()
            .iter()
            .find(|u| u.phase == original.turn())
            .unwrap();
        let enemy = original
            .units()
            .iter()
            .find(|u| original.hostile(actor, u))
            .unwrap();
        let adjacent = (0..original.board().ids.len())
            .find(|&t| original.board().distance(actor.tile, t) == 1 && original.at(t).is_none())
            .unwrap();
        let enemy_id = UnitId(enemy.id);
        let actor_tile = TileId(original.board().ids[actor.tile]);
        let mut state = original.state();
        state
            .units
            .retain(|u| u.id == format!("u{}", actor.id) || u.id == format!("u{}", enemy.id));
        let target = state
            .units
            .iter_mut()
            .find(|u| u.faction != "player")
            .unwrap();
        target.tile = original.board().ids[adjacent];
        target.retreating = Some(true);
        let observe = |state: &State, orders: &TacticalOrders| {
            let game = Game::from_state(&rules, state.clone());
            training::scoped(
                Some(Dataset {
                    spatial: true,
                    ..Default::default()
                }),
                None,
                None,
                || training::candidates(game.view(), orders),
            )
            .0
        };
        let attacks = |choices: &training::Choices| {
            choices
                .commands
                .iter()
                .filter(|c| matches!(c, Command::Attack { .. }))
                .count()
        };
        let mut orders = TacticalOrders {
            campaign: true,
            ..Default::default()
        };
        assert!(attacks(&observe(&state, &orders)) > 0);
        orders.restrained = true;
        orders.protect_withdrawal = true;
        orders.mediated.push(enemy_id);
        orders.withdrawal_tiles.push(actor_tile);
        let protected = observe(&state, &orders);
        assert_eq!(attacks(&protected), 0);
        let feature = |name: &str| FEATURE_NAMES.iter().position(|&n| n == name).unwrap();
        assert_eq!(protected.features[0][feature("diplomatic_restraint")], 1.);
        assert_eq!(protected.features[0][feature("withdrawal_protection")], 1.);
        assert!(protected.features[0][feature("protected_retreaters")] > 0.);
        assert!(protected.features[0][feature("mediated_enemies")] > 0.);
        assert!(
            protected
                .features
                .iter()
                .any(|x| x[feature("withdrawal_lane_before")] == 1.)
        );
        assert!(
            protected
                .features
                .iter()
                .any(|x| x[feature("nearest_enemy_withdrawing")] == 1.)
        );
        let board = protected.spatial.unwrap().board;
        let cells = board.height * board.width;
        for channel in 62..65 {
            assert_eq!(
                board.values[channel * cells..(channel + 1) * cells]
                    .iter()
                    .filter(|&&v| v == 1.)
                    .count(),
                1
            );
        }
        let target = state
            .units
            .iter_mut()
            .find(|u| u.faction != "player")
            .unwrap();
        target.retreating = Some(false);
        // Keep the nonlethal case independent of current weapon balance.
        target.hp = i32::MAX;
        let nonlethal = observe(&state, &orders);
        assert!(attacks(&nonlethal) > 0);
        assert!(
            nonlethal
                .features
                .iter()
                .any(|x| x[feature("target_mediated")] == 1.)
        );
        state
            .units
            .iter_mut()
            .find(|u| u.faction != "player")
            .unwrap()
            .hp = 1;
        let lethal = observe(&state, &orders);
        orders.protect_withdrawal = false;
        assert!(attacks(&lethal) < attacks(&observe(&state, &orders)));
    }

    #[test]
    fn markers_bind_actors_and_expose_unfinished_work_on_end_turn() {
        let rules = Rules::load().unwrap();
        let game = Game::new(&rules, "mirror");
        let units: Vec<_> = game
            .units()
            .iter()
            .filter(|u| game.belongs_to_faction(u, game.observer_faction()))
            .collect();
        let actor = units[0];
        let guard = units[1];
        let destination = game
            .reachable(actor)
            .iter()
            .map(|v| v.0)
            .find(|&t| t != actor.tile && game.at(t).is_none())
            .unwrap();
        let tile = |t: usize| TileId(game.board().ids[t]);
        let mut orders = TacticalOrders {
            campaign: true,
            ..Default::default()
        };
        orders.objectives = vec![
            ObjectiveOrder {
                id: "route".into(),
                priority: ObjectivePriority::Critical,
                objective: TacticalObjective::GoTo {
                    actor: UnitId(actor.id),
                    destination: tile(destination),
                },
            },
            ObjectiveOrder {
                id: "hold".into(),
                priority: ObjectivePriority::Critical,
                objective: TacticalObjective::Occupy {
                    actor: UnitId(guard.id),
                    location: tile(guard.tile),
                },
            },
            ObjectiveOrder {
                id: "passage".into(),
                priority: ObjectivePriority::Important,
                objective: TacticalObjective::ClearPassage {
                    actor: UnitId(actor.id),
                    destination: tile(guard.tile),
                },
            },
            ObjectiveOrder {
                id: "route:escort".into(),
                priority: ObjectivePriority::Important,
                objective: TacticalObjective::Escort {
                    actor: UnitId(actor.id),
                    guards: 3,
                    selector: UnitSelector::AnyOf(
                        units
                            .iter()
                            .map(|u| UnitTypeId(rules.units[u.kind].id.clone()))
                            .collect(),
                    ),
                },
            },
        ];
        orders.constraints.push(TacticalConstraint::Survive {
            actor: UnitId(guard.id),
        });
        orders.progress.push(ObjectiveProgress {
            id: "hold".into(),
            current: 1,
            total: 3,
            satisfied: false,
            hold_turns_remaining: Some(2),
        });
        let observe = |o: &TacticalOrders| {
            training::scoped(
                Some(Dataset {
                    spatial: true,
                    ..Default::default()
                }),
                None,
                None,
                || training::candidates(game.view(), o),
            )
            .0
        };
        let choices = observe(&orders);
        let feature = |name: &str| FEATURE_NAMES.iter().position(|&n| n == name).unwrap();
        assert!(choices.features[0][feature("goto_unfinished")] > 0.);
        assert!(choices.features[0][feature("occupy_unfinished")] > 0.);
        assert!(choices.features[0][feature("duty_hold_remaining_known")] > 0.);
        let move_index = choices
            .commands
            .iter()
            .position(
                |c| matches!(c,Command::Move{unit,tile:t} if unit.0==actor.id && t.0==destination),
            )
            .unwrap();
        assert_eq!(choices.features[move_index][feature("goto_at_after")], 1.);
        assert_eq!(choices.features[move_index][feature("goto_priority")], 1.);
        let guard_index = choices
            .commands
            .iter()
            .position(|c| matches!(c,Command::Move{unit,..} if unit.0==guard.id))
            .unwrap();
        assert_eq!(choices.features[guard_index][feature("goto_assigned")], 0.);
        assert_eq!(
            choices.features[guard_index][feature("escort_guard_eligible")],
            1.
        );
        assert_eq!(
            choices.features[guard_index][feature("passage_corridor_before")],
            1.
        );
        let spatial = &choices.spatial.unwrap().board;
        let cells = spatial.height * spatial.width;
        assert!(spatial.values[61 * cells..62 * cells].contains(&1.));
        orders.campaign = false;
        let arena = observe(&orders);
        assert!(
            arena
                .features
                .iter()
                .all(|x| x[LEGACY_FEATURES..].iter().all(|&v| v == 0.))
        );
        assert!(
            arena.spatial.unwrap().board.values[53 * cells..]
                .iter()
                .all(|&v| v == 0.)
        );
    }
}
