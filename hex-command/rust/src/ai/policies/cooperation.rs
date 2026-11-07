//! Duty assignment, local rescue and defensive fallback; planner entry points opt in explicitly.
use crate::ai::orders::TargetKind as ObjectiveKind;
use crate::ai::orders::target as objective;
use crate::ai::{attack_command, move_command, orders::*, policies::threat::Threats};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
use hex_command_battle::{ActionForecastSource, PhaseForecasts};

const PER_ACTOR: usize = 32;
const PER_TURN: usize = 128;

#[derive(Clone, Debug, Default)]
pub(in crate::ai) struct State {
    assigned: bool,
    attempted: Vec<UnitId>,
    fallback: Vec<UnitId>,
    candidates: usize,
    actor_positions: Vec<(UnitId, TileId)>,
    revision: u32,
    exhausted: Vec<UnitId>,
}

impl State {
    pub(in crate::ai) fn revision(&self) -> u32 {
        self.revision
    }
}

pub(in crate::ai) fn assign(
    game: &BattleView<'_, '_>,
    state: &mut State,
    orders: &mut TacticalOrders,
    report: &mut PlanningReport,
) -> Option<Vec<Target>> {
    if state.assigned {
        return None;
    }
    state.assigned = true;
    let mut allocated = Vec::new();
    let mut targets = Vec::new();
    let mut actor_destinations = Vec::<(UnitId, TileId)>::new();
    let mut deferred = Vec::new();
    for order in &orders.objectives {
        match &order.objective {
            TacticalObjective::GoTo { actor, destination }
            | TacticalObjective::Occupy {
                actor,
                location: destination,
            } => {
                if actor_destinations
                    .iter()
                    .any(|(id, tile)| id == actor && tile != destination)
                {
                    deferred.push(order.id.clone());
                    report.duties.push(DutyReport {
                        id: order.id.clone(),
                        actor: Some(*actor),
                        status: PlanningStatus::Deferred,
                        reason: Some(PlanningReason::ConflictingDuty),
                        guards: vec![],
                        incoming: None,
                    });
                    continue;
                }
                actor_destinations.push((*actor, destination.clone()));
                let occupy = match order.objective {
                    TacticalObjective::Occupy { .. } => true,
                    TacticalObjective::GoTo { .. }
                    | TacticalObjective::Escort { .. }
                    | TacticalObjective::ClearPassage { .. } => false,
                };
                let mut target = orders
                    .targets
                    .iter()
                    .find(|g| g.id == order.id)
                    .cloned()
                    .unwrap_or_else(|| {
                        objective(
                            &order.id,
                            ObjectiveKind::Visit,
                            vec![destination.0.clone()],
                            24.0,
                        )
                    });
                target.actor = Some(format!("u{}", actor.0));
                target.family = None;
                target.kind = if occupy {
                    ObjectiveKind::Capture
                } else {
                    ObjectiveKind::Visit
                };
                target.hold = Some(occupy);
                target.tiles = vec![destination.0.clone()];
                targets.push(target);
            }
            TacticalObjective::Escort {
                actor,
                guards,
                selector,
            } => {
                let Some(unit) = game.unit(actor.0) else {
                    continue;
                };
                let mut eligible: Vec<_> = game
                    .units()
                    .iter()
                    .filter(|u| {
                        game.belongs_to_phase(u, game.turn())
                            && game.can_act(u.id)
                            && u.id != actor.0
                            && !allocated.contains(&u.id)
                            && !orders.moving_actor(u.id)
                            && !u.is_leader()
                            && selector.accepts(game, u.kind)
                    })
                    .collect();
                eligible.order_by_key(|u| (game.board().distance(u.tile, unit.tile), u.id));
                let selected: Vec<_> = eligible.into_iter().take(*guards).collect();
                for guard in &selected {
                    allocated.push(guard.id);
                }
                report.duties.push(DutyReport {
                    id: order.id.clone(),
                    actor: Some(*actor),
                    status: if selected.len() == *guards {
                        PlanningStatus::Fulfilled
                    } else {
                        PlanningStatus::Unmet
                    },
                    reason: (selected.len() < *guards)
                        .then_some(PlanningReason::InsufficientGuards),
                    guards: selected.iter().map(|u| UnitId(u.id)).collect(),
                    incoming: None,
                });
            }
            TacticalObjective::ClearPassage { .. } => {}
        }
    }
    orders
        .targets
        .retain(|g| !orders.objectives.iter().any(|order| order.id == g.id));
    orders
        .objectives
        .retain(|order| !deferred.contains(&order.id));
    orders
        .targets
        .extend(targets.into_iter().filter(|goal| !goal.tiles.is_empty()));
    Some(
        orders
            .targets
            .iter()
            .filter(|goal| !goal.id.starts_with("rebuild:"))
            .cloned()
            .collect(),
    )
}

pub(in crate::ai) fn refresh_assignments(
    game: &BattleView<'_, '_>,
    state: &mut State,
    orders: &mut TacticalOrders,
    report: &PlanningReport,
) -> Option<Vec<Target>> {
    let positions: Vec<_> = report
        .duties
        .iter()
        .filter(|r| !r.guards.is_empty())
        .filter_map(|r| {
            r.actor.and_then(|actor| {
                game.unit(actor.0)
                    .map(|u| (actor, TileId(game.board().ids[u.tile].clone())))
            })
        })
        .collect();
    if positions == state.actor_positions {
        return None;
    }
    state.actor_positions = positions;
    state.revision += 1;
    orders.targets.retain(|g| !g.tiles.is_empty());
    Some(
        orders
            .targets
            .iter()
            .filter(|goal| !goal.id.starts_with("rebuild:"))
            .cloned()
            .collect(),
    )
}

fn destination(game: &BattleView<'_, '_>, orders: &TacticalOrders, actor: u32) -> Option<usize> {
    orders
        .objectives
        .iter()
        .find_map(|order| match &order.objective {
            TacticalObjective::GoTo {
                actor: id,
                destination,
            } => (id.0 == actor).then(|| game.site(&destination.0)),
            TacticalObjective::Occupy {
                actor: id,
                location,
            } => (id.0 == actor).then(|| game.site(&location.0)),
            TacticalObjective::Escort { .. } | TacticalObjective::ClearPassage { .. } => None,
        })
}

fn safest(
    phases: PhaseForecasts<'_, '_>,
    orders: &TacticalOrders,
    id: u32,
) -> Option<(usize, i32)> {
    let game = phases.view();
    let unit = game.unit(id)?;
    let target = destination(game, orders, id);
    let mut threats = Threats::protected(phases, unit.side, orders);
    std::iter::once(unit.tile)
        .chain(game.reachable(unit).iter().map(|&(tile, _)| tile))
        .map(|tile| (tile, threats.at(unit, tile, None)))
        .min_by_key(|&(tile, incoming)| {
            (
                incoming,
                target.map_or(0, |t| game.board().distance(tile, t)),
                tile != unit.tile,
                tile,
            )
        })
}

/// Rescue only actors with predicted lethal exposure and no individually safe move.
pub(in crate::ai) fn rescue(
    game: &BattleView<'_, '_>,
    phases: PhaseForecasts<'_, '_>,
    actions: ActionForecastSource<'_, '_>,
    state: &mut State,
    orders: &TacticalOrders,
    report: &mut PlanningReport,
    queued: &mut std::collections::VecDeque<Command>,
) -> Result<bool, CommandError> {
    for constraint in &orders.constraints {
        let TacticalConstraint::Survive { actor } = constraint;
        if state.attempted.contains(actor) {
            continue;
        }
        let Some(unit) = game
            .unit(actor.0)
            .filter(|u| !u.attacked && game.can_act(u.id))
        else {
            continue;
        };
        state.attempted.push(*actor);
        let Some((_, exposure)) = safest(phases, orders, actor.0) else {
            continue;
        };
        if exposure < unit.hp {
            continue;
        }
        if state.candidates >= PER_TURN {
            break;
        }
        let mut helpers: Vec<_> = game
            .units()
            .iter()
            .filter(|u| {
                game.belongs_to_phase(u, game.turn())
                    && game.can_act(u.id)
                    && !u.attacked
                    && u.id != actor.0
                    && !u.is_leader()
                    && !orders.protects(u.id)
                    && !orders.moving_actor(u.id)
                    && game.board().distance(u.tile, unit.tile) <= 5
            })
            .collect();
        helpers.order_by_key(|u| (game.board().distance(u.tile, unit.tile), u.id));
        let helpers: Vec<_> = helpers.into_iter().take(2).map(|u| u.id).collect();
        let mut pending = std::collections::VecDeque::from([(
            actions.fork(),
            Vec::<Command>::new(),
            Vec::<u32>::new(),
        )]);
        let mut actor_candidates = 0;
        while let Some((board, commands, used)) = pending.pop_front() {
            if !commands.is_empty() {
                if let Some((tile, incoming)) = safest(board.phase_forecasts(), orders, actor.0) {
                    if board.winner() == Some(game.turn())
                        || incoming < board.unit(actor.0).unwrap().hp
                    {
                        queued.extend(commands);
                        if tile != board.unit(actor.0).unwrap().tile {
                            queued.push_back(move_command(&board, actor.0, tile));
                        }
                        report.duties.push(DutyReport {
                            id: format!("survive:u{}", actor.0),
                            actor: Some(*actor),
                            status: PlanningStatus::Fulfilled,
                            reason: None,
                            guards: used.iter().map(|&id| UnitId(id)).collect(),
                            incoming: Some(incoming),
                        });
                        return Ok(true);
                    }
                }
            }
            if used.len() >= 2 || actor_candidates >= PER_ACTOR || state.candidates >= PER_TURN {
                continue;
            }
            for helper in helpers.iter().filter(|id| !used.contains(id)) {
                if actor_candidates >= PER_ACTOR || state.candidates >= PER_TURN {
                    break;
                }
                let Some(ally) = board.unit(*helper) else {
                    continue;
                };
                let mut positions: Vec<_> = std::iter::once(ally.tile)
                    .chain(board.reachable(ally).iter().map(|&(tile, _)| tile))
                    .collect();
                positions.order_by_key(|&tile| (board.board().distance(tile, unit.tile), tile));
                let mut actions = Vec::new();
                for tile in positions {
                    for enemy in board
                        .units()
                        .iter()
                        .filter(|u| board.hostile_to_phase(u, game.turn()))
                    {
                        for weapon in board.attack_options(ally, enemy, tile) {
                            let response = board.maximum_retaliation(ally, enemy, weapon, tile);
                            if board.exchange(ally, enemy, weapon, tile).death[1] == 1.0
                                && response < ally.hp
                                && orders.attack_allowed(&board, ally, enemy, weapon, tile)
                            {
                                actions.push((tile, enemy.id, weapon));
                            }
                        }
                    }
                }
                actions.order_by_key(|&(tile, target, _)| {
                    (
                        board
                            .board()
                            .distance(board.unit(target).unwrap().tile, unit.tile),
                        tile != ally.tile,
                        tile,
                        target,
                    )
                });
                actions.truncate(8);
                for (tile, target, weapon) in actions {
                    if actor_candidates >= PER_ACTOR || state.candidates >= PER_TURN {
                        break;
                    }
                    actor_candidates += 1;
                    state.candidates += 1;
                    report.candidates += 1;
                    let mut draft = board.fork();
                    let mut next = commands.clone();
                    if tile != ally.tile {
                        let command = move_command(&draft, *helper, tile);
                        draft.execute(command.clone())?;
                        next.push(command);
                    }
                    let command = attack_command(&draft, *helper, target, weapon);
                    draft.execute(command.clone())?;
                    next.push(command);
                    let mut next_used = used.clone();
                    next_used.push(*helper);
                    pending.push_back((draft, next, next_used));
                }
            }
        }
        if actor_candidates >= PER_ACTOR || state.candidates >= PER_TURN {
            state.exhausted.push(*actor);
        }
    }
    Ok(false)
}

pub(in crate::ai) fn fallback(
    game: &BattleView<'_, '_>,
    phases: PhaseForecasts<'_, '_>,
    state: &mut State,
    orders: &TacticalOrders,
    report: &mut PlanningReport,
    queued: &mut std::collections::VecDeque<Command>,
) -> Result<bool, CommandError> {
    for constraint in &orders.constraints {
        let TacticalConstraint::Survive { actor } = constraint;
        if state.fallback.contains(actor) {
            continue;
        }
        state.fallback.push(*actor);
        let Some(unit) = game.unit(actor.0) else {
            continue;
        };
        if let Some((tile, incoming)) = safest(phases, orders, actor.0) {
            if tile != unit.tile && !unit.moved && !unit.attacked {
                queued.push_back(move_command(game, actor.0, tile));
                // Drain before choosing another fallback so legality uses updated occupancy.
                return Ok(true);
            }
            if incoming >= unit.hp {
                report.duties.push(DutyReport {
                    id: format!("survive:u{}", actor.0),
                    actor: Some(*actor),
                    status: PlanningStatus::Unmet,
                    reason: Some(
                        if state.candidates >= PER_TURN || state.exhausted.contains(actor) {
                            PlanningReason::SearchBudget
                        } else {
                            PlanningReason::UnsafeFallback
                        },
                    ),
                    guards: vec![],
                    incoming: Some(incoming),
                });
            }
        }
    }
    Ok(false)
}

/// A vacant destination alone says nothing about access to it. Check connected,
/// passable, unoccupied cells without forecast enemy pressure along the way.
fn passage_clear(
    game: &BattleView<'_, '_>,
    phases: PhaseForecasts<'_, '_>,
    orders: &TacticalOrders,
    unit: &Unit,
    destination: usize,
) -> bool {
    if unit.tile == destination {
        return true;
    }
    let route = game.routes(unit.kind, &[destination]);
    if route[unit.tile] == u16::MAX {
        return false;
    }
    let mut threats = Threats::protected(phases, unit.side, orders);
    let mut visited = vec![false; game.terrain().len()];
    visited[unit.tile] = true;
    let mut pending =
        std::collections::BinaryHeap::from([std::cmp::Reverse((route[unit.tile], unit.tile))]);
    while let Some(std::cmp::Reverse((_, tile))) = pending.pop() {
        for &next in &game.board().adjacent[tile] {
            if visited[next] || route[next] == u16::MAX || game.cost(unit.kind, next) == u16::MAX {
                continue;
            }
            visited[next] = true;
            if game.at(next).is_some_and(|other| other.id != unit.id)
                || threats.at(unit, next, None) > 0
            {
                continue;
            }
            if next == destination {
                return true;
            }
            pending.push(std::cmp::Reverse((route[next], next)));
        }
    }
    false
}

pub(in crate::ai) fn report(
    game: &BattleView<'_, '_>,
    phases: PhaseForecasts<'_, '_>,
    state: &State,
    orders: &CampaignOrders,
    report: &mut PlanningReport,
    finished: bool,
) {
    if !finished {
        return;
    }
    report
        .duties
        .retain(|r| r.id.ends_with(":escort") || r.reason == Some(PlanningReason::ConflictingDuty));
    for constraint in &orders.tactics.constraints {
        let TacticalConstraint::Survive { actor } = constraint;
        let (status, reason, incoming) = if let Some(unit) = game.unit(actor.0) {
            let mut threats = Threats::protected(phases, game.turn(), &orders.tactics);
            let incoming = threats.at(unit, unit.tile, None);
            (
                if incoming < unit.hp {
                    PlanningStatus::Fulfilled
                } else {
                    PlanningStatus::Unmet
                },
                (incoming >= unit.hp).then_some(
                    if state.candidates >= PER_TURN || state.exhausted.contains(actor) {
                        PlanningReason::SearchBudget
                    } else {
                        PlanningReason::UnsafeFallback
                    },
                ),
                Some(incoming),
            )
        } else {
            (PlanningStatus::Unmet, Some(PlanningReason::ActorLost), None)
        };
        report.duties.push(DutyReport {
            id: format!("survive:u{}", actor.0),
            actor: Some(*actor),
            status,
            reason,
            guards: vec![],
            incoming,
        });
    }
    for order in &orders.tactics.objectives {
        let (actor, location, passage) = match &order.objective {
            TacticalObjective::GoTo { actor, destination } => (*actor, destination, false),
            TacticalObjective::Occupy { actor, location } => (*actor, location, false),
            TacticalObjective::ClearPassage { actor, destination } => (*actor, destination, true),
            TacticalObjective::Escort { .. } => continue,
        };
        let unit = game.unit(actor.0);
        let achieved = unit.is_some_and(|u| {
            if passage {
                passage_clear(game, phases, &orders.tactics, u, game.site(&location.0))
            } else {
                game.board().ids[u.tile] == location.0
            }
        });
        report.duties.push(DutyReport {
            id: order.id.clone(),
            actor: Some(actor),
            status: if achieved {
                PlanningStatus::Fulfilled
            } else {
                PlanningStatus::Deferred
            },
            reason: if achieved {
                None
            } else if unit.is_none() {
                Some(PlanningReason::ActorLost)
            } else if unit.is_some_and(|u| u.attacked) {
                Some(PlanningReason::InactiveActor)
            } else {
                Some(if passage {
                    PlanningReason::BlockedAccess
                } else {
                    PlanningReason::NotReached
                })
            },
            guards: vec![],
            incoming: None,
        });
    }
    for (index, requirement) in orders.recruitment.requirements.iter().enumerate() {
        let deficit = requirement.deficit(game);
        report.duties.push(DutyReport {
            id: format!("recruit:{index}"),
            actor: None,
            status: if deficit == 0 {
                PlanningStatus::Fulfilled
            } else {
                PlanningStatus::Unmet
            },
            reason: (deficit > 0).then_some(PlanningReason::RecruitmentUnavailable),
            guards: vec![],
            incoming: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Hex;
    use crate::logic::Game;

    fn small_battle(rules: &Rules) -> Game<'_> {
        let mut state = Game::new(rules, "mirror").state();
        state.units.retain(|u| u.role == Some(Role::Leader));
        let actor = state
            .units
            .iter_mut()
            .find(|u| u.side == Side::Player)
            .unwrap();
        actor.tile = Hex::new(1, 3).into();
        actor.hp = 1;
        actor.moved = true;
        let mut threat = actor.clone();
        threat.id = format!("u{}", state.next_id);
        state.next_id += 1;
        threat.side = Side::Enemy;
        threat.r#type = "swordsman".into();
        threat.tile = Hex::new(2, 3).into();
        threat.hp = 1;
        threat.moved = false;
        threat.character = None;
        threat.role = None;
        state.units.push(threat);
        let mut helper = state.units[0].clone();
        helper.id = format!("u{}", state.next_id);
        state.next_id += 1;
        helper.side = Side::Player;
        helper.r#type = "archer".into();
        helper.tile = Hex::new(3, 3).into();
        helper.hp = rules.units[rules.types["archer"]].hp;
        helper.moved = false;
        helper.attacked = false;
        helper.character = None;
        helper.role = None;
        state.units.push(helper);
        Game::try_from_state(rules, state).unwrap()
    }
    struct Harness {
        state: State,
        orders: CampaignOrders,
        report: PlanningReport,
        commands: std::collections::VecDeque<Command>,
    }
    fn workspace(game: &Game<'_>) -> Harness {
        let actor = game
            .units()
            .iter()
            .find(|u| u.side == Side::Player && u.is_leader())
            .unwrap()
            .id;
        Harness {
            state: State::default(),
            orders: CampaignOrders {
                tactics: TacticalOrders {
                    constraints: vec![TacticalConstraint::Survive {
                        actor: UnitId(actor),
                    }],
                    ..TacticalOrders::default()
                },
                ..CampaignOrders::default()
            },
            report: PlanningReport::default(),
            commands: std::collections::VecDeque::new(),
        }
    }
    #[test]
    fn an_ally_removes_a_lethal_threat_before_the_protected_actor_acts() {
        let rules = Rules::load().unwrap();
        let mut game = small_battle(&rules);
        let mut workspace = workspace(&game);
        let actor = game
            .units()
            .iter()
            .find(|u| u.side == Side::Player && u.is_leader())
            .unwrap()
            .id;
        let orders = &workspace.orders.tactics;
        assert!(
            safest(game.phase_forecasts(), orders, actor).unwrap().1
                >= game.unit(actor).unwrap().hp
        );
        assert!(
            rescue(
                game.view(),
                game.phase_forecasts(),
                game.action_forecasts(),
                &mut workspace.state,
                &workspace.orders.tactics,
                &mut workspace.report,
                &mut workspace.commands
            )
            .unwrap()
        );
        assert!(workspace.report.candidates <= PER_ACTOR);
        while let Some(command) = workspace.commands.pop_front() {
            game.execute(command).unwrap();
        }
        assert!(
            safest(game.phase_forecasts(), &workspace.orders.tactics, actor)
                .unwrap()
                .1
                < game.unit(actor).unwrap().hp
        );
    }
    #[test]
    fn an_unsafe_fallback_is_reported_without_claiming_survival() {
        let rules = Rules::load().unwrap();
        let game = small_battle(&rules);
        let mut workspace = workspace(&game);
        let queued = fallback(
            game.view(),
            game.phase_forecasts(),
            &mut workspace.state,
            &workspace.orders.tactics,
            &mut workspace.report,
            &mut workspace.commands,
        )
        .unwrap();
        report(
            game.view(),
            game.phase_forecasts(),
            &workspace.state,
            &workspace.orders,
            &mut workspace.report,
            !queued,
        );
        assert!(workspace.report.duties.iter().any(|r| match r.status {
            PlanningStatus::Unmet => r.reason.as_ref().is_some_and(|reason| match reason {
                PlanningReason::UnsafeFallback => true,
                PlanningReason::InactiveActor
                | PlanningReason::ActorLost
                | PlanningReason::InsufficientGuards
                | PlanningReason::RecruitmentUnavailable
                | PlanningReason::BlockedAccess
                | PlanningReason::ConflictingDuty
                | PlanningReason::SearchBudget
                | PlanningReason::NotReached => false,
            }),
            PlanningStatus::Fulfilled | PlanningStatus::Deferred => false,
        }));
    }
}
