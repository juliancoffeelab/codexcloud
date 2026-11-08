//! Bounded detailed evaluation: inexpensive candidates first, full safety second.
use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::ai::{
    TacticalRole,
    planner::proposal::Plan,
    policies::escort::scoring as escort_scoring,
    policies::objectives::{TileOrder, TileOrderContext, actor_matches},
    policies::threat::Threats,
    policies::tie_breaking::action_jitter,
    roles::role,
};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
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
pub(in crate::ai) struct Workspace {
    revision: u32,
    goals: Vec<Objective>,
    orders: crate::ai::orders::TacticalOrders,
    rebuild_intents: Vec<(u32, Option<bool>)>,
    blockers: Vec<bool>,
    proximity: Vec<u16>,
    proximity_roster: Vec<(u32, usize)>,
    frontline_support: Vec<f64>,
    front: u16,
    screens: usize,
    roster: Vec<GoalUnit>,
    owners: Vec<Option<String>>,
    terrain: Vec<usize>,
    gold: i32,
    side: Option<Side>,
    positions: Vec<(u32, usize, Option<bool>)>,
    clearing: FxHashMap<u32, FxHashSet<usize>>,
    lanes: FxHashSet<usize>,
    order: Option<TileOrderContext>,
    reserved: FxHashMap<u32, FxHashSet<usize>>,
    fields: FxHashMap<(usize, usize, bool), Rc<Field>>,
}
impl Workspace {
    pub(super) fn orders(&self) -> &crate::ai::orders::TacticalOrders {
        &self.orders
    }
    pub(super) fn goals(&self) -> &[Objective] {
        &self.goals
    }
    pub(super) fn proximity(&self) -> &[u16] {
        &self.proximity
    }
    pub(super) fn clearing(&self) -> &FxHashMap<u32, FxHashSet<usize>> {
        &self.clearing
    }
    pub(super) fn rebuild_intents(&self) -> &[(u32, Option<bool>)] {
        &self.rebuild_intents
    }
}
pub(super) struct EvaluationInput<'a> {
    pub seed: u32,
    pub orders: &'a crate::ai::orders::TacticalOrders,
    pub recruitment: Option<crate::ai::policies::recruitment::EstimateInput<'a>>,
}
pub(in crate::ai) fn prepare(
    game: &BattleView<'_, '_>,
    cache: &mut Workspace,
    orders: &crate::ai::orders::TacticalOrders,
    revision: u32,
) {
    let same = cache.side == Some(game.turn())
        && cache.revision == revision
        && cache.gold == game.gold_for_phase(game.turn())
        && cache.owners == *game.owners()
        && cache.terrain == *game.terrain()
        && cache.roster.len() == game.units().len()
        && cache.roster.iter().zip(game.units()).all(|(key, u)| {
            key.0 == u.id
                && key.1 == u.kind
                && key.2 == u.side
                && key.3 == (game.hostile_to_phase(u, game.turn())).then_some(u.tile)
                && key.4 == u.role
                && key.5 == u.rebuilding
                && key.6 == u.character
        });
    let spatial_same = cache.positions.len() == game.units().len()
        && cache
            .positions
            .iter()
            .zip(game.units())
            .all(|(key, u)| *key == (u.id, u.tile, u.retreating));
    if !same {
        cache.revision = revision;
        cache.side = Some(game.turn());
        cache.gold = game.gold_for_phase(game.turn());
        cache.owners.clone_from(&game.owners());
        cache.terrain.clone_from(&game.terrain());
        cache.orders = orders.clone();
        cache.roster.clear();
        cache.roster.extend(game.units().iter().map(|u| {
            (
                u.id,
                u.kind,
                u.side,
                (game.hostile_to_phase(u, game.turn())).then_some(u.tile),
                u.role,
                u.rebuilding,
                u.character.clone(),
            )
        }));

        cache.goals = orders.targets.clone();
        cache.goals.retain(|g| !g.id.starts_with("rebuild:"));
        cache
            .goals
            .extend(crate::ai::policies::objectives::individual(
                orders.rebuild_objectives(game),
            ));
        cache.blockers.clear();
        cache.blockers.resize(game.terrain().len(), false);
        for goal in cache
            .goals
            .iter()
            .filter(|g| g.weight >= 24.0 && g.kind == ObjectiveKind::Capture)
        {
            for site in &goal.tiles {
                cache.blockers[game.site(&site)] = true;
            }
        }
        cache.rebuild_intents.clear();
        for unit in game
            .units()
            .iter()
            .filter(|u| game.belongs_to_phase(u, game.turn()) && u.is_leader())
        {
            let required = orders.moving_actor(unit.id);
            let (_, intent) = orders.rebuild_plan(game, unit.id, required);
            if unit.rebuilding != intent {
                cache.rebuild_intents.push((unit.id, intent));
            }
        }
        cache.order = Some(TileOrder::context(game, game.turn(), true));
        cache.reserved.clear();
        cache.fields.clear();
    }
    let proximity_same = cache.proximity.len() == game.terrain().len()
        && cache.proximity_roster.iter().copied().eq(game
            .units()
            .iter()
            .filter(|u| game.hostile_to_phase(u, game.turn()))
            .map(|u| (u.id, u.tile)));
    if !proximity_same {
        cache.proximity_roster = game
            .units()
            .iter()
            .filter(|u| game.hostile_to_phase(u, game.turn()))
            .map(|u| (u.id, u.tile))
            .collect();
        cache.proximity = (0..game.terrain().len())
            .map(|t| crate::ai::policies::objectives::proximity(game, game.turn(), t))
            .collect();
    }
    if !same || !spatial_same {
        cache.frontline_support.clear();
        cache.frontline_support.resize(game.terrain().len(), 0.0);
        cache.front = 20;
        cache.screens = 0;
        for ally in game
            .units()
            .iter()
            .filter(|u| game.belongs_to_phase(u, game.turn()))
        {
            if role(game, ally.kind) == TacticalRole::Frontline && !ally.is_leader() {
                cache.front = if cache.screens == 0 {
                    cache.proximity[ally.tile]
                } else {
                    cache.front.min(cache.proximity[ally.tile])
                };
                cache.screens += 1;
            }
            if orders.important(game, ally) || role(game, ally.kind) != TacticalRole::Frontline {
                for &tile in &game.board().adjacent[ally.tile] {
                    if cache.proximity[tile] <= cache.proximity[ally.tile] {
                        cache.frontline_support[tile] += 1.5;
                    }
                }
            }
        }

        cache.positions = game
            .units()
            .iter()
            .map(|u| (u.id, u.tile, u.retreating))
            .collect();
        cache.clearing = orders.clearing(game);
        cache.lanes = orders.lanes(game);
    }
}

fn leader_may_advance(
    game: &BattleView<'_, '_>,
    unit: &Unit,
    tile: usize,
    recruiting: bool,
    required: bool,
    proximity: &[u16],
) -> bool {
    if tile == unit.tile || required || !game.units().iter().any(|u| game.hostile(&unit, u)) {
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
    let (count, front) = crate::ai::policies::objectives::ordinary(game, unit)
        .filter(|u| {
            u.hp as f64 >= game.rules().units[u.kind].hp as f64 / 2.0
                && game.board().distance(tile, u.tile) <= 2
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
pub(super) fn plan(
    game: &BattleView<'_, '_>,
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    id: u32,
    cache: &mut Workspace,
    shared: super::search::CandidateShared<'_>,
    input: EvaluationInput<'_>,
) -> Option<Plan> {
    let original = game.unit(id)?;
    let objectives = &cache.goals;
    let orders = input.orders;
    let required = orders.moving_actor(id) || cache.clearing.contains_key(&id);
    let (rebuilding, intent) = orders.rebuild_plan(game, id, required);
    let mut unit = original.clone();
    unit.rebuilding = intent;
    let important = orders.important(game, &unit);
    let escortee = escort_scoring::subject(orders, &unit);
    let category = role(game, unit.kind);
    let battle = !orders.campaign;
    let reserved = cache
        .reserved
        .entry(id)
        .or_insert_with(|| crate::ai::policies::objectives::reserved(game, &unit, objectives));
    let lanes = &cache.lanes;
    let clearing = cache.clearing.get(&id);
    let order = cache.order.as_ref().unwrap().for_unit(game, &unit);
    let mut duties = vec![];
    for (goal_index, original) in objectives.iter().enumerate() {
        if orders.assigned_elsewhere(id, original) {
            continue;
        }
        if original.id.starts_with("rebuild:") && !actor_matches(&original.actor, id) {
            continue;
        }
        if original.kind == ObjectiveKind::Capture
            && original.actor.is_none()
            && original.tiles.iter().all(|t| {
                let tile = game.site(t);
                game.owns(tile, unit.side)
                    && (original.hold != Some(true)
                        || game
                            .at(tile)
                            .is_some_and(|u| game.allied(&unit, u) && u.id != id))
            })
        {
            continue;
        }
        let pursuing = crate::ai::policies::objectives::can_pursue(game, &unit, original);
        if !pursuing && original.actor.is_none() && original.family.is_none() {
            continue;
        }
        let dynamic = pursuing
            && (original.actor.is_some() || original.family.is_some())
            && crate::ai::policies::objectives::actor_route(original);
        let field = cache
            .fields
            .entry((unit.kind, goal_index, pursuing))
            .or_insert_with(|| {
                let goal = if pursuing {
                    original.clone()
                } else {
                    crate::ai::policies::objectives::support_objective(game, &unit, original)
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
        let assigned = crate::ai::policies::objectives::assigned_duty(game, &unit, original);
        let priority = (route[unit.tile] as f64 + 3.0) / field.goal.weight;
        duties.push((field, route, assigned, priority));
    }
    if duties.iter().any(|d| d.2) {
        duties.retain(|d| d.2);
    }
    let committed = duties
        .iter()
        .position(|d| shared.commitments.get(&id) == Some(&d.0.goal.id));
    let duty = if let Some(index) = committed {
        Some(duties.swap_remove(index))
    } else {
        duties.order_by(|a, b| {
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
        shared.commitments.insert(id, d.0.goal.id.clone());
    } else {
        shared.commitments.remove(&id);
    }
    let allies: Vec<_> = game
        .units()
        .iter()
        .filter(|u| game.allied(&unit, u))
        .collect();
    let movement = if unit.moved {
        0
    } else {
        game.rules().units[unit.kind].r#move
    };
    let maximum_range = game
        .weapons(&unit)
        .map(|a| a.range as usize)
        .max()
        .unwrap_or(0);
    let zero_cost = game.rules().units[unit.kind]
        .costs
        .iter()
        .any(|c| *c == Some(0));
    let foes: Vec<_> = game
        .units()
        .iter()
        .filter(|u| {
            game.hostile(&unit, u)
                && (zero_cost
                    || game.board().distance(unit.tile, u.tile) as usize
                        <= movement + maximum_range)
        })
        .collect();
    let proximity_grid = &cache.proximity;
    let front = cache.front;
    let has_screens = cache.screens > 0;
    let holding = objectives.iter().any(|g| {
        g.hold == Some(true)
            && actor_matches(&g.actor, id)
            && g.tiles.contains(&game.board().ids[unit.tile])
    });
    let mut support = if category == TacticalRole::Frontline {
        cache.frontline_support.clone()
    } else {
        vec![0.0; game.terrain().len()]
    };
    if category == TacticalRole::Frontline && important {
        for &tile in &game.board().adjacent[unit.tile] {
            if proximity_grid[tile] <= proximity_grid[unit.tile] {
                support[tile] -= 1.5;
            }
        }
    }
    for ally in &allies {
        if ally.id == id {
            continue;
        }
        let aura = game.rules().units[unit.kind].thorn_aura;
        if aura > 0 {
            let other = allies
                .iter()
                .filter(|u| {
                    u.id != id && u.id != ally.id && game.board().distance(u.tile, ally.tile) == 1
                })
                .map(|u| game.rules().units[u.kind].thorn_aura)
                .max()
                .unwrap_or(0);
            let hits = game
                .units()
                .iter()
                .filter(|enemy| {
                    game.hostile(&unit, enemy)
                        && game.board().distance(enemy.tile, ally.tile)
                            <= game.rules().units[enemy.kind].r#move as u16 + 1
                })
                .flat_map(|enemy| {
                    game.weapons(enemy)
                        .filter(|w| w.category == Category::Melee)
                        .map(|w| w.count)
                })
                .max()
                .unwrap_or(0);
            for &tile in &game.board().adjacent[ally.tile] {
                if proximity_grid[ally.tile] <= 4 {
                    support[tile] += (aura - other).max(0) as f64 * f64::from(hits) * 2.0;
                }
            }
        }
        if category == TacticalRole::Healer {
            let other = allies
                .iter()
                .filter(|u| {
                    u.id != id && u.id != ally.id && game.board().distance(u.tile, ally.tile) == 1
                })
                .map(|u| game.rules().units[u.kind].healing)
                .max()
                .unwrap_or(0);
            let benefit = (game.rules().units[unit.kind].healing - other)
                .max(0)
                .min(game.rules().units[ally.kind].hp - ally.hp) as f64
                * 3.0;
            for &tile in &game.board().adjacent[ally.tile] {
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
        && unit.hp as f64 > game.rules().units[unit.kind].hp as f64 / 2.0
        && allies.iter().any(|u| {
            role(game, u.kind) == TacticalRole::Frontline
                && !u.is_leader()
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
            if escortee {
                value += escort_scoring::progress(
                    route[unit.tile],
                    route[tile],
                    field.tiles.contains(&unit.tile),
                    field.tiles.contains(&tile),
                );
            } else {
                value += (route[unit.tile] as f64 - route[tile] as f64) * goal.weight / 4.0;
                value += (field.tiles.contains(&tile) as i32
                    - field.tiles.contains(&unit.tile) as i32) as f64
                    * goal.weight
                    * 0.6;
            }
        }
        if holding && tile != unit.tile {
            value -= 24.0;
        }
        let healing = (game.role_is(tile, TerrainRole::Village)
            || game.role_is(tile, TerrainRole::Grove))
            && (game.owns(tile, unit.side) || game.at(tile).is_none());
        if healing {
            value += (game.rules().units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0;
        }
        if game.role_is(tile, TerrainRole::Village) && game.captures(tile, unit.side) {
            value += 3.0;
        }
        let proximity = proximity_grid[tile];
        if important
            && !unit.role.is_some_and(Role::is_escort)
            && !unit.is_leader()
            && !duty.as_ref().is_some_and(|d| d.2)
            && has_screens
            && proximity <= front
        {
            value -= 20.0;
        }
        if category != TacticalRole::Frontline && has_screens && proximity <= 4 {
            value += if proximity > front { 2.0 } else { -3.0 };
        }
        value += support[tile] + shared.escort.position(id, tile);
        value
    };
    let mut destinations = vec![unit.tile];
    destinations.extend(game.reachable(&unit).iter().map(|&(tile, _)| tile));
    let destinations = orders.authored_destinations(game, &unit, destinations);
    let wait_position = position(unit.tile);
    let recruiting = unit.is_leader()
        && game
            .recruitment_options(unit.side)
            .iter()
            .any(|&k| !game.placement_tiles(&unit, k, true).is_empty());
    let geometry: Vec<_> = if foes.is_empty() {
        vec![]
    } else {
        game.weapons(&unit)
            .map(|a| game.attack_origins(a.category, a.range))
            .collect()
    };
    let may_advance: Vec<_> = destinations
        .iter()
        .map(|&tile| {
            !unit.is_leader()
                || !rebuilding.is_empty()
                || leader_may_advance(game, &unit, tile, recruiting, required, proximity_grid)
        })
        .collect();
    let finishing_sites: Vec<_> = orders
        .terminal_arrivals
        .iter()
        .filter(|(actor, _)| actor.0 == id)
        .map(|(_, tile)| game.site(&tile.0))
        .collect();
    let width = 12;
    let mut candidates = Vec::with_capacity(width + 1);
    let mut ordinal = 0;
    let positions: Vec<_> = destinations
        .iter()
        .map(|&tile| {
            (
                position(tile),
                finishing_sites.contains(&tile) && orders.arrives(game, &unit, tile),
                (game.role_is(tile, TerrainRole::Village)
                    || game.role_is(tile, TerrainRole::Grove))
                    && (game.owns(tile, unit.side) || game.at(tile).is_none()),
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
                || !orders.attack_allowed(game, &unit, target, weapon, tile)
            {
                return;
            }
            let forecast = game.exchange(&unit, target, weapon, tile);
            let damage = forecast.damage[0].round() as i32;
            let retaliation = forecast.damage[1].round() as i32;
            let drained = forecast.healing[0].round() as i32;
            response = if important {
                game.maximum_retaliation(&unit, target, weapon, tile)
            } else {
                retaliation - drained
            };
            let kill_chance = forecast.death[1];
            let killed = kill_chance == 1.0;
            eliminated = killed.then_some(target.id);
            terminal = killed && orders.terminal_attack(target);
            combat = forecast.effective[0] * 2.0
                + kill_chance * (12.0 + game.recruit_cost(target, target.kind) as f64 * 0.3)
                - retaliation as f64 * 1.8
                + drained as f64 * 2.0
                + shared.escort.attack(id, target, damage, kill_chance);
            if orders.important(game, target) {
                combat += 4.0 + 14.0 * kill_chance;
            }
            if battle
                && killed
                && game.rules().units[unit.kind]
                    .experience_needed
                    .is_some_and(|needed| {
                        unit.experience.unwrap_or(0) + 6 * game.rules().units[target.kind].level
                            >= needed
                    })
            {
                combat += 3.0;
            }
            // Coordinate damage against an enemy blocking a required site.
            if !battle && cache.blockers[target.tile] {
                combat += 16.0;
            }
            if response >= unit.hp {
                if important && !orders.protects(unit.id) {
                    return;
                }
                combat -= 18.0;
            }
        }
        let finishing = finish_objective && response < unit.hp;
        if important && !finishing && !terminal {
            combat *= if unit.role.is_some_and(Role::is_escort) {
                0.1
            } else {
                0.5
            };
        }
        if unit.is_leader()
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
                input.seed,
                game.round(),
                id,
                game.board().positions[tile],
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
        candidates
            .order_unstable_by(|a, b| b.score.total_cmp(&a.score).then(a.order.cmp(&b.order)));
        candidates.truncate(width);
    };
    if battle {
        let mut by_tile = vec![usize::MAX; game.terrain().len()];
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
            for weapon in 0..game.weapons(&unit).count() {
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
                for weapon in 0..game.weapons(&unit).count() {
                    add(destination, Some(target), Some(weapon));
                }
            }
        }
    }
    if !orders.campaign
        && candidates.iter().all(|c| c.score <= 0.1)
        && game.promotion_options(&unit).is_empty()
        && game
            .units()
            .iter()
            .filter(|u| game.hostile(&unit, u))
            .all(|u| {
                !game.rules().units[u.kind]
                    .costs
                    .iter()
                    .any(|c| *c == Some(0))
                    && game.board().distance(unit.tile, u.tile) as usize
                        > game.rules().units[u.kind].r#move
                            + game.weapons(u).map(|a| a.range as usize).max().unwrap_or(0)
            })
    {
        return None;
    }
    let mut threats =
        Threats::with_workspace(phases, unit.side, shared.threats).with_orders(orders);
    let before = crate::ai::policies::threat::cap_exposure(
        &unit,
        threats.at(&unit, unit.tile, None),
        important,
    );
    let danger = if escortee {
        escort_scoring::EXPOSURE / unit.hp.max(1) as f64
    } else if orders.protects(unit.id) {
        // A finite cost for the fraction of remaining life at risk.
        40.0 / unit.hp.max(1) as f64
    } else if important {
        6.0
    } else if category == TacticalRole::Frontline {
        0.55
    } else {
        0.9
    };
    let wait_risk = before as f64 * danger
        + if !escortee && before >= unit.hp {
            if important { 40.0 } else { 5.0 }
        } else {
            0.0
        };
    let mut best = orders
        .promotion(game, &unit)
        .filter(|_| before < unit.hp)
        .map(|kind| Plan {
            unit: id,
            tile: unit.tile,
            score: 14.0,
            target: None,
            weapon: None,
            advance: Some(kind),
            withdrawing: false,
        });
    // Escort exposure already rewards a reduction by at most twenty points.
    let improvement_weight = if escortee {
        0.0
    } else if important {
        3.0
    } else if unit.hp <= 4 {
        1.5
    } else {
        0.2
    };
    for c in candidates {
        let upper = c.score + wait_risk + before.max(0) as f64 * improvement_weight;
        if upper <= 0.1 || best.as_ref().is_some_and(|p| p.score >= upper) {
            break;
        }
        let incoming = if c.terminal {
            0
        } else {
            threats.at(&unit, c.tile, c.eliminated)
        };
        let incoming = crate::ai::policies::threat::cap_exposure(&unit, incoming, important);
        let lethal = incoming >= unit.hp - c.response;
        let score = c.score + wait_risk
            - incoming as f64 * danger
            - if lethal && !c.terminal {
                if important { 40.0 } else { 5.0 }
            } else {
                0.0
            }
            + (before - incoming).max(0) as f64 * improvement_weight;
        if score <= 0.1 || best.as_ref().is_some_and(|p| p.score >= score) {
            continue;
        }
        let withdrawing = unit.side == Side::Enemy
            && c.target.is_none()
            && unit.hp < game.rules().units[unit.kind].hp
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
    // Protected campaign actors must compare escape moves even when the raw
    // shortlist contains a positive but dangerous attack. Ordinary arena units
    // retain their existing fallback and bounded search.
    if before > 0
        && ((orders.campaign && orders.protects(unit.id))
            || (best.is_none() && battle && important))
    {
        for (index, &tile) in destinations.iter().enumerate() {
            let (positioning, _, healing) = positions[index];
            if tile == unit.tile || !positioning.is_finite() {
                continue;
            }
            let incoming = crate::ai::policies::threat::cap_exposure(
                &unit,
                threats.at(&unit, tile, None),
                important,
            );
            if !may_advance[index]
                && incoming >= before
                && !healing
                && unit.retreating != Some(true)
            {
                continue;
            }
            let score = positioning - wait_position
                + action_jitter(
                    input.seed,
                    game.round(),
                    id,
                    game.board().positions[tile],
                    None,
                )
                + wait_risk
                - incoming as f64 * danger
                - if incoming >= unit.hp { 40.0 } else { 0.0 }
                + (before - incoming).max(0) as f64 * improvement_weight;
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
                    && unit.hp < game.rules().units[unit.kind].hp
                    && proximity_grid[tile] > proximity_grid[unit.tile]
                    && (incoming < before || healing || unit.retreating == Some(true)),
            });
        }
    }
    if let Some(recruiting) =
        super::recruitment::plan(game, phases, &unit, orders, input.recruitment)
    {
        if best.as_ref().is_none_or(|p| recruiting.score > p.score) {
            best = Some(recruiting);
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::PlanningWorkspace;
    use crate::logic::Game;
    #[test]
    fn shared_frames_follow_actions_and_turn_boundaries() {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::DismissDialogue
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::DropItem { .. }
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        let mut games = vec![
            Game::new(&rules, "mirror"),
            Game::start_campaign(&rules, "lantern-road", 0, vec![], 0, vec![], vec![], vec![]),
        ];
        for game in &mut games {
            let mut workspace =
                PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Unrestricted);
            let mut cache = Workspace::default();
            let mut previous_orders = None;
            let mut revision = 0;
            for _ in 0..24 {
                let command = crate::ai::choose_game_command(
                    game,
                    crate::ai::Policy::Balanced,
                    &mut workspace,
                )
                .unwrap();
                let view = game.view();
                let mission = crate::ai::campaigner::mission::MissionPolicy::new(
                    game.campaign_view(),
                    game.phase_forecasts(),
                );
                if game.winner().is_some() {
                    break;
                }
                let authored_orders;
                let orders = if let Some(orders) = workspace.orders() {
                    orders
                } else {
                    authored_orders = crate::ai::CampaignBlock::Mission.campaign_orders(
                        &mission,
                        crate::ai::campaigner::mission::CampaignInput {
                            goals: None,
                            diplomacy: crate::ai::Diplomacy::Unrestricted,
                        },
                    );
                    &authored_orders
                };
                let current_orders = serde_json::to_value(&orders.tactics).unwrap();
                if previous_orders.as_ref() != Some(&current_orders) {
                    revision += 1;
                    previous_orders = Some(current_orders);
                }
                prepare(&view, &mut cache, &orders.tactics, revision);
                let mut expected_goals = orders.tactics.targets.clone();
                expected_goals.retain(|g| !g.id.starts_with("rebuild:"));
                expected_goals.extend(crate::ai::policies::objectives::individual(
                    orders.tactics.rebuild_objectives(&game.view()),
                ));
                assert_eq!(
                    serde_json::to_value(&cache.goals).unwrap(),
                    serde_json::to_value(expected_goals).unwrap()
                );
                assert_eq!(
                    cache.proximity,
                    (0..game.terrain().len())
                        .map(|tile| crate::ai::policies::objectives::proximity(
                            &game.view(),
                            game.turn(),
                            tile
                        ))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    cache.clearing,
                    crate::ai::policies::objectives::actor_lanes(&game.view(), &cache.goals)
                );
                assert_eq!(
                    cache.lanes,
                    mission.withdrawal_lanes(game.turn(), orders.tactics.restrained)
                );
                for unit in game
                    .units()
                    .iter()
                    .filter(|u| u.side == game.turn() && u.is_leader())
                {
                    for tile in 0..game.terrain().len() {
                        assert_eq!(
                            leader_may_advance(
                                &game.view(),
                                unit,
                                tile,
                                true,
                                false,
                                &cache.proximity
                            ),
                            crate::ai::policies::objectives::leader_may_advance(
                                &game.view(),
                                unit,
                                tile,
                                true,
                                false
                            )
                        );
                    }
                }
                let fresh: Vec<_> = (0..game.terrain().len())
                    .map(|tile| {
                        game.units()
                            .iter()
                            .filter(|ally| {
                                ally.side == game.turn()
                                    && (mission.essential(ally)
                                        || role(&game.view(), ally.kind) != TacticalRole::Frontline)
                                    && game.board().distance(tile, ally.tile) == 1
                                    && cache.proximity[tile] <= cache.proximity[ally.tile]
                            })
                            .count() as f64
                            * 1.5
                    })
                    .collect();
                assert_eq!(cache.frontline_support, fresh);
                if game.winner().is_some() {
                    break;
                }
                let boundary = is_command_endturn(&command);
                game.execute(command).unwrap();
                if boundary {
                    workspace =
                        PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Unrestricted);
                }
            }
        }
    }
}
