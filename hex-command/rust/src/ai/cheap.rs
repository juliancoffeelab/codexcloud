//! Bounded detailed evaluation: inexpensive candidates first, full safety second.
use super::{
    PlanningWorkspace, TacticalRole,
    objectives::{TileOrder, TileOrderContext, actor_matches, individual},
    roles::role,
    tactics::{Plan, action_jitter},
    threats::Threats,
};
use crate::{data::*, logic::Game};
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

struct Field {
    goal: Objective,
    tiles: Vec<usize>,
    route: Option<Rc<Vec<u16>>>,
}
type GoalUnit = (
    u32,
    usize,
    Side,
    Option<usize>,
    Option<Role>,
    Option<bool>,
    Option<String>,
);
#[derive(Default)]
pub(super) struct Workspace {
    pub(super) goals: Vec<Objective>,
    pub(super) rebuild_intents: Vec<(u32, Option<bool>)>,
    blockers: Vec<bool>,
    pub(super) proximity: Vec<u16>,
    proximity_roster: Vec<(u32, usize)>,
    frontline_support: Vec<f64>,
    front: u16,
    screens: usize,
    roster: Vec<GoalUnit>,
    owners: Vec<Option<Side>>,
    terrain: Vec<usize>,
    marks: Vec<String>,
    escalated: bool,
    gold: i32,
    side: Option<Side>,
    positions: Vec<(u32, usize, Option<bool>)>,
    pub(super) clearing: FxHashMap<u32, FxHashSet<usize>>,
    lanes: FxHashSet<usize>,
    order: Option<TileOrderContext>,
    reserved: FxHashMap<u32, FxHashSet<usize>>,
    fields: FxHashMap<(usize, usize, bool), Rc<Field>>,
}
pub(super) fn prepare(game: &Game<'_>, workspace: &mut PlanningWorkspace) {
    let effective_restraint = restrained(game, workspace);
    let cache = &mut workspace.cheap;
    let marks = game
        .campaign
        .as_ref()
        .map_or(&[][..], |c| c.marks.as_slice());
    let escalated = game.campaign.as_ref().is_some_and(|c| c.escalated());
    let same = cache.side == Some(game.turn)
        && cache.gold == game.gold[game.turn.index()]
        && cache.owners == game.owners
        && cache.terrain == game.terrain
        && cache.marks == marks
        && cache.escalated == escalated
        && cache.roster.len() == game.units.len()
        && cache.roster.iter().zip(&game.units).all(|(key, u)| {
            key.0 == u.id
                && key.1 == u.kind
                && key.2 == u.side
                && key.3 == (u.side != game.turn).then_some(u.tile)
                && key.4 == u.role
                && key.5 == u.rebuilding
                && key.6 == u.name
        });
    let spatial_same = cache.positions.len() == game.units.len()
        && cache
            .positions
            .iter()
            .zip(&game.units)
            .all(|(key, u)| *key == (u.id, u.tile, u.retreating));
    if !same {
        cache.side = Some(game.turn);
        cache.gold = game.gold[game.turn.index()];
        cache.owners.clone_from(&game.owners);
        cache.terrain.clone_from(&game.terrain);
        cache.marks.clear();
        cache.marks.extend_from_slice(marks);
        cache.escalated = escalated;
        cache.roster.clear();
        cache.roster.extend(game.units.iter().map(|u| {
            (
                u.id,
                u.kind,
                u.side,
                (u.side != game.turn).then_some(u.tile),
                u.role,
                u.rebuilding,
                u.name.clone(),
            )
        }));

        cache.goals = policy_goals(game, workspace.goals.as_deref());
        cache.blockers.clear();
        cache.blockers.resize(game.terrain.len(), false);
        for goal in cache
            .goals
            .iter()
            .filter(|g| g.weight >= 24.0 && g.kind == ObjectiveKind::Capture)
        {
            for site in &goal.tiles {
                cache.blockers[game.site(site)] = true;
            }
        }
        cache.rebuild_intents.clear();
        for unit in game
            .units
            .iter()
            .filter(|u| u.side == game.turn && game.rules.units[u.kind].leader)
        {
            let required = cache
                .goals
                .iter()
                .any(|g| g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, unit.id));
            let (_, intent) = game.leader_rebuild_plan(unit.id, required);
            if unit.rebuilding != intent {
                cache.rebuild_intents.push((unit.id, intent));
            }
        }
        cache.order = Some(TileOrder::context(game, game.turn, true));
        cache.reserved.clear();
        cache.fields.clear();
    }
    let proximity_same = cache.proximity.len() == game.terrain.len()
        && cache.proximity_roster.iter().copied().eq(game
            .units
            .iter()
            .filter(|u| u.side != game.turn)
            .map(|u| (u.id, u.tile)));
    if !proximity_same {
        cache.proximity_roster = game
            .units
            .iter()
            .filter(|u| u.side != game.turn)
            .map(|u| (u.id, u.tile))
            .collect();
        cache.proximity = (0..game.terrain.len())
            .map(|t| game.proximity(game.turn, t))
            .collect();
    }
    if !same || !spatial_same {
        cache.frontline_support.clear();
        cache.frontline_support.resize(game.terrain.len(), 0.0);
        cache.front = 20;
        cache.screens = 0;
        for ally in game.units.iter().filter(|u| u.side == game.turn) {
            if role(game, ally.kind) == TacticalRole::Frontline
                && !game.rules.units[ally.kind].leader
            {
                cache.front = if cache.screens == 0 {
                    cache.proximity[ally.tile]
                } else {
                    cache.front.min(cache.proximity[ally.tile])
                };
                cache.screens += 1;
            }
            if game.essential(ally) || role(game, ally.kind) != TacticalRole::Frontline {
                for &tile in &game.board.adjacent[ally.tile] {
                    if cache.proximity[tile] <= cache.proximity[ally.tile] {
                        cache.frontline_support[tile] += 1.5;
                    }
                }
            }
        }

        cache.positions = game
            .units
            .iter()
            .map(|u| (u.id, u.tile, u.retreating))
            .collect();
        cache.clearing = game.escort_lanes(&cache.goals);
        cache.lanes = game.withdrawal_lanes(game.turn, effective_restraint);
    }
}

fn policy_goals(game: &Game<'_>, provided: Option<&[Objective]>) -> Vec<Objective> {
    let mut goals = game.policy_objectives(provided);
    let escort_finish = provided.is_none()
        && game.turn == Side::Player
        && game.mission.as_ref().is_some_and(|mission| {
            (mission.refuge.is_none() || game.campaign.as_ref().is_some_and(|c| c.escalated()))
                && mission.goals.iter().any(|g| {
                    !game.goal_done(g) && matches!(g, Goal::Escort { .. } | Goal::Escape { .. })
                })
                && mission.goals.iter().all(|g| {
                    game.goal_done(g) || matches!(g, Goal::Escort { .. } | Goal::Escape { .. })
                })
        });
    if escort_finish {
        for goal in goals.iter_mut().filter(|g| g.id == "opposing-leader") {
            goal.weight = 48.0;
        }
    }
    individual(goals)
}
pub(super) fn heavy_campaign(game: &Game<'_>) -> bool {
    game.campaign.as_ref().is_some_and(|state| {
        game.rules.campaigns[&state.id]
            .missions
            .iter()
            .any(|mission| {
                mission
                    .goals
                    .iter()
                    .any(|g| matches!(g, Goal::Defeat { .. }))
                    && mission
                        .goals
                        .iter()
                        .any(|g| matches!(g, Goal::Seal { .. }) && g.unit_type().is_none())
            })
    })
}
pub(super) fn crisis(game: &Game<'_>) -> bool {
    if heavy_campaign(game) {
        return game.turn == Side::Player;
    }
    game.turn == Side::Player
        && game.campaign.is_some()
        && game.units.iter().any(|u| {
            u.side == Side::Player
                && game.essential(u)
                && u.hp * 2 <= game.rules.units[u.kind].hp
                && game.proximity(u.side, u.tile) <= 5
        })
}

pub(super) fn restrained(game: &Game<'_>, workspace: &PlanningWorkspace) -> bool {
    workspace.diplomacy == super::Diplomacy::Restrained
        || !heavy_campaign(game)
            && game.turn == Side::Player
            && game.mission.as_ref().is_some_and(|m| m.refuge.is_some())
            && game.campaign.as_ref().is_some_and(|c| !c.escalated())
}
pub(super) fn terminal_attack(game: &Game<'_>, unit: &Unit, target: &Unit) -> bool {
    if !game.rules.units[target.kind].leader {
        return false;
    }
    if game.campaign.is_none() {
        return true;
    }
    unit.side == Side::Player
        && game.mission.as_ref().is_some_and(|mission| {
            mission
                .goals
                .iter()
                .all(|g| game.goal_done(g) || matches!(g, Goal::Defeat { .. }))
        })
        && game
            .units
            .iter()
            .filter(|u| u.side != unit.side && game.rules.units[u.kind].leader)
            .all(|u| u.id == target.id)
}
fn leader_may_advance(
    game: &Game<'_>,
    unit: &Unit,
    tile: usize,
    recruiting: bool,
    required: bool,
    proximity: &[u16],
) -> bool {
    if tile == unit.tile || required || !game.units.iter().any(|u| u.side != unit.side) {
        return true;
    }
    let before = proximity[unit.tile];
    let after = proximity[tile];
    if unit.rebuilding == Some(true) && recruiting && before > 2 {
        return false;
    }
    if after >= before {
        return true;
    }
    let (count, front) = game
        .ordinary(unit)
        .filter(|u| {
            u.hp as f64 >= game.rules.units[u.kind].hp as f64 / 2.0
                && game.board.distance(tile, u.tile) <= 2
        })
        .fold((0, false), |(count, front), ally| {
            (count + 1, front || proximity[ally.tile] <= after)
        });
    count >= 2 && front
}
struct Candidate {
    tile: usize,
    target: Option<u32>,
    weapon: Option<usize>,
    score: f64,
    response: i32,
    eliminated: Option<u32>,
    terminal: bool,
    healing: bool,
    order: usize,
}
pub(super) fn plan(game: &Game<'_>, id: u32, workspace: &mut PlanningWorkspace) -> Option<Plan> {
    let original = game.unit(id)?;
    let objectives = &workspace.cheap.goals;
    let restrained = restrained(game, workspace);
    let required = objectives
        .iter()
        .any(|g| g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, id));
    let (rebuilding, intent) = game.leader_rebuild_plan(id, required);
    let mut unit = original.clone();
    unit.rebuilding = intent;
    let important = game.essential(&unit);
    let category = role(game, unit.kind);
    let battle = game.campaign.is_none();
    let reserved = workspace
        .cheap
        .reserved
        .entry(id)
        .or_insert_with(|| game.reserved(&unit, objectives));
    let lanes = &workspace.cheap.lanes;
    let clearing = workspace.cheap.clearing.get(&id);
    let order = workspace
        .cheap
        .order
        .as_ref()
        .unwrap()
        .for_unit(game, &unit);
    let mut duties = vec![];
    for (goal_index, original) in objectives.iter().enumerate() {
        if original.id.starts_with("rebuild:") && !actor_matches(&original.actor, id) {
            continue;
        }
        if original.kind == ObjectiveKind::Capture
            && original.actor.is_none()
            && original.tiles.iter().all(|t| {
                let tile = game.site(t);
                game.owners[tile] == Some(unit.side)
                    && (original.hold != Some(true)
                        || game
                            .at(tile)
                            .is_some_and(|u| u.side == unit.side && u.id != id))
            })
        {
            continue;
        }
        let pursuing = game.can_pursue(&unit, original);
        if !pursuing && original.actor.is_none() && original.family.is_none() {
            continue;
        }
        let dynamic = pursuing
            && original.actor.is_some()
            && matches!(original.kind, ObjectiveKind::Escort | ObjectiveKind::Escape);
        let field = workspace
            .cheap
            .fields
            .entry((unit.kind, goal_index, pursuing))
            .or_insert_with(|| {
                let goal = if pursuing {
                    original.clone()
                } else {
                    game.support_objective(&unit, original)
                };
                let tiles: Vec<_> = goal.tiles.iter().map(|t| game.site(t)).collect();
                let route = (!dynamic && !tiles.is_empty()).then(|| game.routes(unit.kind, &tiles));
                Rc::new(Field { goal, tiles, route })
            })
            .clone();
        if field.tiles.is_empty() {
            continue;
        }
        let route = if dynamic {
            let occupied = game.occupied_routes(&unit, &field.tiles, false);
            if occupied[unit.tile] == u16::MAX {
                game.routes(unit.kind, &field.tiles)
            } else {
                Rc::new(occupied)
            }
        } else {
            field.route.as_ref().unwrap().clone()
        };
        if route[unit.tile] == u16::MAX {
            continue;
        }
        let assigned = pursuing && (original.actor.is_some() || original.family.is_some());
        let priority = (route[unit.tile] as f64 + 3.0) / field.goal.weight;
        duties.push((field, route, assigned, priority));
    }
    if duties.iter().any(|d| d.2) {
        duties.retain(|d| d.2);
    }
    let committed = duties
        .iter()
        .position(|d| workspace.commitments.get(&id) == Some(&d.0.goal.id));
    let duty = if let Some(index) = committed {
        Some(duties.swap_remove(index))
    } else {
        duties.sort_by(|a, b| {
            a.3.total_cmp(&b.3).then_with(|| {
                order.compare(
                    game,
                    game.site(&a.0.goal.tiles[0]),
                    game.site(&b.0.goal.tiles[0]),
                )
            })
        });
        duties.into_iter().next()
    };
    if let Some(d) = &duty {
        workspace.commitments.insert(id, d.0.goal.id.clone());
    } else {
        workspace.commitments.remove(&id);
    }
    let allies: Vec<_> = game.units.iter().filter(|u| u.side == unit.side).collect();
    let movement = if unit.moved {
        0
    } else {
        game.rules.units[unit.kind].r#move
    };
    let maximum_range = game.rules.units[unit.kind]
        .attacks
        .iter()
        .map(|a| a.range as usize)
        .max()
        .unwrap_or(0);
    let zero_cost = game.rules.units[unit.kind]
        .costs
        .iter()
        .any(|c| *c == Some(0));
    let foes: Vec<_> = game
        .units
        .iter()
        .filter(|u| {
            u.side != unit.side
                && (zero_cost
                    || game.board.distance(unit.tile, u.tile) as usize <= movement + maximum_range)
        })
        .collect();
    let proximity_grid = &workspace.cheap.proximity;
    let front = workspace.cheap.front;
    let has_screens = workspace.cheap.screens > 0;
    let holding = objectives
        .iter()
        .any(|g| actor_matches(&g.actor, id) && g.tiles.contains(&game.board.ids[unit.tile]));
    let mut support = if category == TacticalRole::Frontline {
        workspace.cheap.frontline_support.clone()
    } else {
        vec![0.0; game.terrain.len()]
    };
    if category == TacticalRole::Frontline && important {
        for &tile in &game.board.adjacent[unit.tile] {
            if proximity_grid[tile] <= proximity_grid[unit.tile] {
                support[tile] -= 1.5;
            }
        }
    }
    for ally in &allies {
        if ally.id == id {
            continue;
        }
        if category == TacticalRole::Healer {
            let other = allies
                .iter()
                .filter(|u| {
                    u.id != id && u.id != ally.id && game.board.distance(u.tile, ally.tile) == 1
                })
                .map(|u| game.rules.units[u.kind].healing)
                .max()
                .unwrap_or(0);
            let benefit = (game.rules.units[unit.kind].healing - other)
                .max(0)
                .min(game.rules.units[ally.kind].hp - ally.hp) as f64
                * 3.0;
            for &tile in &game.board.adjacent[ally.tile] {
                support[tile] += benefit;
                if battle
                    && role(game, ally.kind) == TacticalRole::Frontline
                    && proximity_grid[ally.tile] <= 4
                {
                    support[tile] += if proximity_grid[tile] > proximity_grid[ally.tile] {
                        3.0
                    } else {
                        0.5
                    };
                }
            }
        }
    }
    if battle
        && category == TacticalRole::Healer
        && unit.hp as f64 > game.rules.units[unit.kind].hp as f64 / 2.0
        && allies.iter().any(|u| {
            role(game, u.kind) == TacticalRole::Frontline
                && !game.rules.units[u.kind].leader
                && !u.moved
                && !u.attacked
        })
    {
        for value in &mut support {
            *value *= 0.25;
        }
    }
    let on_reserved = reserved.contains(&unit.tile);
    let position = |tile: usize| {
        let mut value = if on_reserved { 80.0 } else { 0.0 }
            - if reserved.contains(&tile) { 160.0 } else { 0.0 }
            - if lanes.contains(&tile) { 100.0 } else { 0.0 };
        if clearing.as_ref().is_some_and(|lane| !lane.contains(&tile)) {
            value += 40.0;
        }
        if let Some((field, route, _, _)) = &duty {
            let goal = &field.goal;
            if route[tile] == u16::MAX {
                return f64::NEG_INFINITY;
            }
            value += (route[unit.tile] as f64 - route[tile] as f64) * goal.weight / 4.0;
            value += (field.tiles.contains(&tile) as i32 - field.tiles.contains(&unit.tile) as i32)
                as f64
                * goal.weight
                * 0.6;
        }
        if holding && tile != unit.tile {
            value -= 24.0;
        }
        let healing = (game.terrain_is(tile, Terrain::Village)
            || game.terrain_is(tile, Terrain::Grove))
            && (game.owners[tile] == Some(unit.side) || game.at(tile).is_none());
        if healing {
            value += (game.rules.units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0;
        }
        if game.terrain_is(tile, Terrain::Village) && game.owners[tile] != Some(unit.side) {
            value += 3.0;
        }
        let proximity = proximity_grid[tile];
        if important
            && unit.role != Some(Role::Courier)
            && !game.rules.units[unit.kind].leader
            && !duty.as_ref().is_some_and(|d| d.2)
            && has_screens
            && proximity <= front
        {
            value -= 20.0;
        }
        if category != TacticalRole::Frontline && has_screens && proximity <= 4 {
            value += if proximity > front { 2.0 } else { -3.0 };
        }
        value += support[tile];
        value
    };
    let mut destinations = vec![unit.tile];
    destinations.extend(game.reachable(&unit).iter().map(|&(tile, _)| tile));
    let destinations = game.authored_destinations(&unit, destinations);
    let wait_position = position(unit.tile);
    let recruiting = game.rules.units[unit.kind].leader
        && game
            .recruitment_options(unit.side)
            .iter()
            .any(|&k| !game.placement_tiles(&unit, k, true).is_empty());
    let geometry: Vec<_> = if foes.is_empty() {
        vec![]
    } else {
        game.rules.units[unit.kind]
            .attacks
            .iter()
            .map(|a| game.attack_origins(a.category, a.range))
            .collect()
    };
    let may_advance: Vec<_> = destinations
        .iter()
        .map(|&tile| {
            !game.rules.units[unit.kind].leader
                || !rebuilding.is_empty()
                || leader_may_advance(game, &unit, tile, recruiting, required, proximity_grid)
        })
        .collect();
    let finishing_sites: Vec<_> =
        if unit.role == Some(Role::Courier) || unit.role == Some(Role::Leader) {
            objectives
                .iter()
                .filter(|g| {
                    actor_matches(&g.actor, id)
                        && matches!(g.kind, ObjectiveKind::Escort | ObjectiveKind::Escape)
                })
                .flat_map(|g| g.tiles.iter().map(|t| game.site(t)))
                .collect()
        } else {
            vec![]
        };
    let width = 12;
    let mut candidates = Vec::with_capacity(width + 1);
    let mut ordinal = 0;
    let positions: Vec<_> = destinations
        .iter()
        .map(|&tile| {
            (
                position(tile),
                finishing_sites.contains(&tile) && game.finishes_objective(&unit, tile, objectives),
                (game.terrain_is(tile, Terrain::Village) || game.terrain_is(tile, Terrain::Grove))
                    && (game.owners[tile] == Some(unit.side) || game.at(tile).is_none()),
            )
        })
        .collect();
    let mut add = |destination: usize, target: Option<&Unit>, weapon: Option<usize>| {
        let tile = destinations[destination];
        let (positioning, finish_objective, healing) = positions[destination];
        if !positioning.is_finite() {
            return;
        }
        ordinal += 1;
        if target.is_none() && tile == unit.tile {
            return;
        }
        let mut combat = 0.0;
        let mut response = 0;
        let mut eliminated = None;
        let mut terminal = false;
        if let (Some(target), Some(weapon)) = (target, weapon) {
            if !geometry[weapon][target.tile].contains(tile)
                || !game.diplomacy_allows(&unit, target, weapon, tile, restrained)
            {
                return;
            }
            let (damage, retaliation, _) = game.preview(&unit, target, weapon, tile);
            response = retaliation;
            let killed = damage >= target.hp;
            eliminated = killed.then_some(target.id);
            terminal = killed && terminal_attack(game, &unit, target);
            combat = damage.min(target.hp) as f64 * 2.0
                + if killed {
                    12.0 + game.rules.units[target.kind].cost as f64 * 0.3
                } else {
                    0.0
                }
                - response as f64 * 1.8;
            if game.essential(target) {
                combat += if killed { 18.0 } else { 4.0 };
            }
            if battle
                && killed
                && game.rules.units[unit.kind]
                    .experience_needed
                    .is_some_and(|needed| {
                        unit.experience.unwrap_or(0) + 6 * game.rules.units[target.kind].level
                            >= needed
                    })
            {
                combat += 3.0;
            }
            // Coordinate damage against an enemy blocking a required site.
            if !battle && workspace.cheap.blockers[target.tile] {
                combat += 16.0;
            }
            if response >= unit.hp {
                if important {
                    return;
                }
                combat -= 18.0;
            }
        }
        let finishing = finish_objective && response < unit.hp;
        if important && !finishing && !terminal {
            combat *= if unit.role == Some(Role::Courier) {
                0.1
            } else {
                0.5
            };
        }
        if game.rules.units[unit.kind].leader
            && !finishing
            && !terminal
            && rebuilding.is_empty()
            && !may_advance[destination]
        {
            return;
        }
        let score = positioning + combat + if finishing || terminal { 100.0 } else { 0.0 }
            - wait_position
            + action_jitter(
                workspace.seed,
                game.round,
                id,
                &game.board.ids[tile],
                target.map(|u| u.id),
            );
        let candidate = Candidate {
            tile,
            target: target.map(|u| u.id),
            weapon,
            score,
            response,
            eliminated,
            terminal: finishing || terminal,
            healing,
            order: ordinal,
        };
        if battle {
            if candidates.iter().any(|old: &Candidate| {
                old.tile == tile
                    && old.eliminated == eliminated
                    && old.terminal == candidate.terminal
                    && old.score >= score
                    && old.response <= response
            }) {
                return;
            }
            candidates.retain(|old| {
                !(old.tile == tile
                    && old.eliminated == eliminated
                    && old.terminal == candidate.terminal
                    && score >= old.score
                    && response <= old.response
                    && (score > old.score || response < old.response))
            });
        }
        if candidates.len() == width
            && candidates.last().is_some_and(|last: &Candidate| {
                last.score > score || (last.score == score && last.order < ordinal)
            })
        {
            return;
        }
        candidates.push(candidate);
        candidates.sort_unstable_by(|a, b| b.score.total_cmp(&a.score).then(a.order.cmp(&b.order)));
        candidates.truncate(width);
    };
    if battle {
        let mut by_tile = vec![usize::MAX; game.terrain.len()];
        for (index, &tile) in destinations.iter().enumerate() {
            by_tile[tile] = index;
        }
        let top = |indices: &[usize]| {
            let mut best: Vec<usize> = Vec::with_capacity(4 + 1);
            for &index in indices {
                if !positions[index].0.is_finite() {
                    continue;
                }
                let offset = best.partition_point(|&old| positions[old].0 >= positions[index].0);
                if offset < 4 {
                    best.insert(offset, index);
                    best.truncate(4);
                }
            }
            best
        };
        for index in top(&(0..destinations.len())
            .filter(|&i| destinations[i] != unit.tile)
            .collect::<Vec<_>>())
        .into_iter()
        {
            add(index, None, None);
        }
        for &target in &foes {
            for weapon in 0..game.rules.units[unit.kind].attacks.len() {
                let mut origins = vec![];
                geometry[weapon][target.tile].for_each(|tile| {
                    let index = by_tile[tile];
                    if index != usize::MAX {
                        origins.push(index);
                    }
                });
                for index in top(&origins).into_iter() {
                    add(index, Some(target), Some(weapon));
                }
            }
        }
    } else {
        for destination in 0..destinations.len() {
            add(destination, None, None);
            for &target in &foes {
                for weapon in 0..game.rules.units[unit.kind].attacks.len() {
                    add(destination, Some(target), Some(weapon));
                }
            }
        }
    }
    if game.campaign.is_none()
        && candidates.iter().all(|c| c.score <= 0.1)
        && game.promotion_options(&unit).is_empty()
        && game.units.iter().filter(|u| u.side != unit.side).all(|u| {
            !game.rules.units[u.kind].costs.iter().any(|c| *c == Some(0))
                && game.board.distance(unit.tile, u.tile) as usize
                    > game.rules.units[u.kind].r#move
                        + game.rules.units[u.kind]
                            .attacks
                            .iter()
                            .map(|a| a.range as usize)
                            .max()
                            .unwrap_or(0)
        })
    {
        return None;
    }
    let mut threats = Threats::with_workspace(game, unit.side, &mut workspace.threats);
    let before = threats.at(&unit, unit.tile, None);
    let danger = if important {
        6.0
    } else if category == TacticalRole::Frontline {
        0.55
    } else {
        0.9
    };
    let wait_risk = before as f64 * danger
        + if before >= unit.hp {
            if important { 40.0 } else { 5.0 }
        } else {
            0.0
        };
    let mut best = game
        .promotion_options(&unit)
        .first()
        .filter(|_| before < unit.hp)
        .map(|&kind| Plan {
            unit: id,
            tile: unit.tile,
            score: 14.0,
            target: None,
            weapon: None,
            advance: Some(kind),
            withdrawing: false,
        });
    let mut minimum = None;
    for c in candidates {
        let upper = c.score
            + wait_risk
            + before.max(0) as f64
                * if important {
                    3.0
                } else if unit.hp <= 4 {
                    1.5
                } else {
                    0.2
                };
        if upper <= 0.1 || best.as_ref().is_some_and(|p| p.score >= upper) {
            break;
        }
        let incoming = if c.terminal {
            0
        } else {
            threats.at(&unit, c.tile, c.eliminated)
        };
        let lethal = incoming >= unit.hp - c.response;
        if important && lethal && incoming > 0 && !c.terminal {
            let least = *minimum
                .get_or_insert_with(|| threats.minimum(&unit, destinations.iter().copied()));
            if least < unit.hp || incoming + c.response > least || incoming >= before {
                continue;
            }
        }
        let score = c.score + wait_risk
            - incoming as f64 * danger
            - if lethal && !c.terminal {
                if important { 40.0 } else { 5.0 }
            } else {
                0.0
            }
            + (before - incoming).max(0) as f64
                * if important {
                    3.0
                } else if unit.hp <= 4 {
                    1.5
                } else {
                    0.2
                };
        if score <= 0.1 || best.as_ref().is_some_and(|p| p.score >= score) {
            continue;
        }
        let withdrawing = unit.side == Side::Enemy
            && c.target.is_none()
            && unit.hp < game.rules.units[unit.kind].hp
            && proximity_grid[c.tile] > proximity_grid[unit.tile]
            && (incoming < before || c.healing || unit.retreating == Some(true));
        best = Some(Plan {
            unit: id,
            tile: c.tile,
            score,
            target: c.target,
            weapon: c.weapon,
            advance: None,
            withdrawing,
        });
    }
    // A raw shortlist can hide retreat tiles behind attractive unsafe attacks.
    // Reuse the same forecast to search moves only when an essential actor has
    // no positive plan under pressure; ordinary units keep the bounded search.
    if best.is_none() && battle && important && before > 0 {
        for (index, &tile) in destinations.iter().enumerate() {
            let (positioning, _, healing) = positions[index];
            if tile == unit.tile || !positioning.is_finite() {
                continue;
            }
            let incoming = threats.at(&unit, tile, None);
            if !may_advance[index]
                && incoming >= before
                && !healing
                && unit.retreating != Some(true)
            {
                continue;
            }
            if incoming >= unit.hp && incoming > 0 {
                let least = *minimum
                    .get_or_insert_with(|| threats.minimum(&unit, destinations.iter().copied()));
                if least < unit.hp || incoming > least || incoming >= before {
                    continue;
                }
            }
            let score = positioning - wait_position
                + action_jitter(workspace.seed, game.round, id, &game.board.ids[tile], None)
                + wait_risk
                - incoming as f64 * danger
                - if incoming >= unit.hp { 40.0 } else { 0.0 }
                + (before - incoming).max(0) as f64 * 3.0;
            if score <= 0.1 || best.as_ref().is_some_and(|p| p.score >= score) {
                continue;
            }
            best = Some(Plan {
                unit: id,
                tile,
                score,
                target: None,
                weapon: None,
                advance: None,
                withdrawing: unit.side == Side::Enemy
                    && unit.hp < game.rules.units[unit.kind].hp
                    && proximity_grid[tile] > proximity_grid[unit.tile]
                    && (incoming < before || healing || unit.retreating == Some(true)),
            });
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_frames_follow_actions_and_turn_boundaries() {
        let rules = Rules::load().unwrap();
        let mut games = vec![
            Game::new(&rules, "mirror"),
            Game::start_campaign(&rules, "lantern-road", 0, vec![], 0, vec![], vec![], vec![]),
        ];
        for game in &mut games {
            let mut workspace =
                PlanningWorkspace::new(7, 1, None, super::super::Diplomacy::Pragmatic);
            for _ in 0..24 {
                prepare(game, &mut workspace);
                assert_eq!(
                    serde_json::to_value(&workspace.cheap.goals).unwrap(),
                    serde_json::to_value(policy_goals(game, None)).unwrap()
                );
                assert_eq!(
                    workspace.cheap.proximity,
                    (0..game.terrain.len())
                        .map(|tile| game.proximity(game.turn, tile))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    workspace.cheap.clearing,
                    game.escort_lanes(&workspace.cheap.goals)
                );
                assert_eq!(
                    workspace.cheap.lanes,
                    game.withdrawal_lanes(game.turn, restrained(game, &workspace))
                );
                for unit in game
                    .units
                    .iter()
                    .filter(|u| u.side == game.turn && game.rules.units[u.kind].leader)
                {
                    for tile in 0..game.terrain.len() {
                        assert_eq!(
                            leader_may_advance(
                                game,
                                unit,
                                tile,
                                true,
                                false,
                                &workspace.cheap.proximity
                            ),
                            game.leader_may_advance(unit, tile, true, false)
                        );
                    }
                }
                let fresh: Vec<_> = (0..game.terrain.len())
                    .map(|tile| {
                        game.units
                            .iter()
                            .filter(|ally| {
                                ally.side == game.turn
                                    && (game.essential(ally)
                                        || role(game, ally.kind) != TacticalRole::Frontline)
                                    && game.board.distance(tile, ally.tile) == 1
                                    && workspace.cheap.proximity[tile]
                                        <= workspace.cheap.proximity[ally.tile]
                            })
                            .count() as f64
                            * 1.5
                    })
                    .collect();
                assert_eq!(workspace.cheap.frontline_support, fresh);
                let command = super::super::choose_game_command(
                    &rules,
                    game,
                    super::super::Policy::Priority,
                    &mut workspace,
                )
                .unwrap();
                if game.winner.is_some() {
                    break;
                }
                let boundary = matches!(command, Command::EndTurn);
                game.execute(command).unwrap();
                if boundary {
                    workspace =
                        PlanningWorkspace::new(7, 1, None, super::super::Diplomacy::Pragmatic);
                }
            }
        }
    }
}
