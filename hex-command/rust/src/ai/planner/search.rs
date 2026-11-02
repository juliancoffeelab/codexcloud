pub(in crate::ai) mod escort;
use crate::ai::TacticalRole;
use crate::ai::objectives::actor_matches;
use crate::ai::orders::target as objective;
use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::ai::roles::role;
use crate::ai::{advance_command, intent_command, move_command};
use crate::sort::SharedSort;
use crate::{
    ai::context::BattleView,
    ai::objectives::{TileOrder, individual},
    ai::planner::safety::Threats,
    data::*,
};
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
pub struct Random(pub u32);
impl Random {
    pub fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut v = (self.0 ^ (self.0 >> 15)).wrapping_mul(1 | self.0);
        v ^= v.wrapping_add((v ^ (v >> 7)).wrapping_mul(61 | v));
        (v ^ (v >> 14)) as f64 / 4294967296.0
    }
}
pub(in crate::ai) fn jitter(seed: u32, key: &str) -> f64 {
    let mut h = seed;
    for c in key.encode_utf16() {
        h = (h ^ c as u32).wrapping_mul(16777619);
    }
    h as f64 / 4294967296.0 * 0.05
}
fn hash_number(mut hash: u32, mut number: u32) -> u32 {
    let mut digits = [0; 10];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for &byte in &digits[start..] {
        hash = (hash ^ byte as u32).wrapping_mul(16777619);
    }
    hash
}
/// Hash the action key without allocating formatted strings.
pub(in crate::ai) fn action_jitter(
    seed: u32,
    round: u32,
    unit: u32,
    tile: &str,
    target: Option<u32>,
) -> f64 {
    let byte = |hash: u32, byte: u8| (hash ^ byte as u32).wrapping_mul(16777619);
    let mut hash = hash_number(seed, round);
    hash = byte(byte(hash, b':'), b'u');
    hash = hash_number(hash, unit);
    hash = byte(hash, b':');
    for character in tile.encode_utf16() {
        hash = (hash ^ character as u32).wrapping_mul(16777619);
    }
    hash = byte(hash, b':');
    if let Some(target) = target {
        hash = hash_number(byte(hash, b'u'), target);
    } else {
        for character in b"move" {
            hash = byte(hash, *character);
        }
    }
    hash as f64 / 4294967296.0 * 0.05
}
/// Exact zero operations need no outward adjustment. Avoid introducing the
/// smallest subnormal into otherwise ordinary score arithmetic: those operands
/// are particularly expensive on the native CPUs used for full simulations.
trait ScoreBound {
    fn upper(self) -> Self;
    fn lower(self) -> Self;
}
impl ScoreBound for f64 {
    #[inline]
    fn upper(self) -> Self {
        if self == 0.0 { self } else { self.next_up() }
    }
    #[inline]
    fn lower(self) -> Self {
        if self == 0.0 { self } else { self.next_down() }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::ai) struct Plan {
    pub(in crate::ai) unit: u32,
    pub(in crate::ai) tile: usize,
    pub(in crate::ai) score: f64,
    pub(in crate::ai) target: Option<u32>,
    pub(in crate::ai) weapon: Option<usize>,
    pub(in crate::ai) advance: Option<usize>,
    pub(in crate::ai) withdrawing: bool,
}
impl BattleView<'_, '_> {
    #[inline(always)]
    pub(in crate::ai) fn best_plan<const BOUNDED: bool, const FOCUSED: bool>(
        &self,
        orders: &crate::ai::orders::TacticalOrders,
        escort: &crate::ai::search_escort::Workspace,
        seed: u32,
        prepared_goals: Option<&[Objective]>,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut HashMap<u32, String>,
        intents: &mut Vec<(u32, Option<bool>)>,
        threat_workspace: &mut crate::ai::threats::ArenaThreatWorkspace,
        focus: Option<u32>,
        recruitment: Option<&crate::ai::recruiter::RecruitmentOpportunity>,
    ) -> Option<Plan> {
        #[cfg(target_arch = "wasm32")]
        {
            self.best_plan_impl::<BOUNDED, false>(
                orders,
                escort,
                seed,
                prepared_goals,
                routes,
                commitments,
                intents,
                threat_workspace,
                focus,
                recruitment,
                FOCUSED,
            )
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.best_plan_impl::<BOUNDED, FOCUSED>(
                orders,
                escort,
                seed,
                prepared_goals,
                routes,
                commitments,
                intents,
                threat_workspace,
                focus,
                recruitment,
                false,
            )
        }
    }
    #[cfg_attr(target_arch = "wasm32", inline(never))]
    fn best_plan_impl<const BOUNDED: bool, const FOCUSED: bool>(
        &self,
        orders: &crate::ai::orders::TacticalOrders,
        escort: &crate::ai::search_escort::Workspace,
        seed: u32,
        prepared_goals: Option<&[Objective]>,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut HashMap<u32, String>,
        intents: &mut Vec<(u32, Option<bool>)>,
        threat_workspace: &mut crate::ai::threats::ArenaThreatWorkspace,
        focus: Option<u32>,
        recruitment: Option<&crate::ai::recruiter::RecruitmentOpportunity>,
        _focused: bool,
    ) -> Option<Plan> {
        #[cfg(target_arch = "wasm32")]
        let focused = _focused;
        #[cfg(not(target_arch = "wasm32"))]
        let focused = FOCUSED;
        let side = self.turn();
        let objectives: Cow<'_, [Objective]> = if let Some(prepared) = prepared_goals {
            let rebuilding = orders.rebuild_objectives(self);
            if rebuilding.is_empty() {
                Cow::Borrowed(prepared)
            } else {
                Cow::Owned(
                    prepared
                        .iter()
                        .cloned()
                        .chain(individual(rebuilding))
                        .collect(),
                )
            }
        } else {
            Cow::Owned(orders.targets.clone())
        };
        let escort_lanes = orders.clearing(self);
        let rebuilds: HashMap<_, _> = self
            .units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, side) && u.is_leader())
            .map(|u| (u.id, orders.moving_actor(u.id)))
            .map(|(id, required)| {
                (
                    id,
                    orders.rebuild_plan(self, id, required || escort_lanes.contains_key(&id)),
                )
            })
            .collect();
        for (&id, (_, intent)) in &rebuilds {
            if self.unit(id).unwrap().rebuilding != *intent {
                intents.push((id, *intent));
            }
        }
        // Commit bookkeeping in roster order, as policy_objectives and the
        // original planning loop visited leaders and units in that order.
        intents.order_by_key(|(id, _)| self.index(*id).unwrap());
        let allies: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, side) && self.can_act(u.id))
            .collect();
        if allies.iter().all(|u| u.attacked) {
            return None;
        }
        let foes: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.hostile_to_phase(u, side))
            .collect();
        let lanes = orders.lanes(self);
        let mut threats = Threats::with_workspace(self, side, threat_workspace).with_orders(orders);
        let screens: Vec<_> = allies
            .iter()
            .filter(|u| role(self, u.kind) == TacticalRole::Frontline && !u.is_leader())
            .collect();
        let movable_screens = screens.iter().any(|u| !u.moved && !u.attacked);
        let board = &self.board();
        // These distances and possible targets depend only on this planning
        // snapshot, not on the unit or candidate move currently being scored.
        let foe_ranges: Vec<_> = (0..board.ids.len())
            .map(|tile| {
                foes.iter()
                    .map(|u| board.distance(tile, u.tile))
                    .min()
                    .unwrap_or(20)
            })
            .collect();
        let range_to_foe = |tile: usize| foe_ranges[tile];
        let maximum_range = allies
            .iter()
            .flat_map(|u| self.weapons(u))
            .map(|a| a.range as u16)
            .max()
            .unwrap_or(0);
        let range = maximum_range as usize;
        let capacity = (3 * range * (range + 1) * foes.len()).min(board.ids.len() * foes.len());
        let mut nearby_foes = Vec::with_capacity(capacity);
        let mut nearby_offsets = Vec::with_capacity(board.ids.len() + 1);
        // One compact target array replaces a separately allocated list per tile.
        // Each slice retains the original foe roster order.
        for tile in 0..board.ids.len() {
            nearby_offsets.push(nearby_foes.len());
            for &foe in &foes {
                let distance = board.distance(tile, foe.tile);
                if distance > 0 && distance <= maximum_range {
                    nearby_foes.push(foe);
                }
            }
        }
        nearby_offsets.push(nearby_foes.len());
        let protected: Vec<_> = {
            allies
                .iter()
                .filter(|u| {
                    orders.important(self, u) || role(self, u.kind) != TacticalRole::Frontline
                })
                .collect()
        };
        let tile_order_context = TileOrder::context(self, side, true);
        let mut best: Option<Plan> = None;
        let mut best_order = (usize::MAX, usize::MAX, usize::MAX);
        // Score units with the strongest immediate opportunities first. This
        // only changes evaluation order: roster/tile/weapon ordinals still
        // decide all equal scores and bookkeeping remains unit-local.
        let mut roster: Vec<_> = allies
            .iter()
            .enumerate()
            .filter(|(_, u)| !u.attacked && (!focused || Some(u.id) == focus))
            .map(|(index, &unit)| {
                let before_risk = super::recruitment::exposure(
                    unit,
                    threats.at(unit, unit.tile, None),
                    orders.important(self, unit),
                );
                let important = orders.important(self, unit);
                let category = role(self, unit.kind);
                let danger = if important {
                    9.0
                } else if category == TacticalRole::Frontline {
                    0.75
                } else {
                    1.1
                };
                let movement = if unit.moved {
                    0
                } else {
                    self.rules().units[unit.kind].r#move
                };
                let terminal_objective = orders
                    .terminal_arrivals
                    .iter()
                    .any(|(actor, _)| actor.0 == unit.id);
                let zero_cost = self.rules().units[unit.kind]
                    .costs
                    .iter()
                    .any(|c| *c == Some(0));
                let mut offense = 0.0f64;
                let mut maximum_combat = 0.0f64;
                for &target in &foes {
                    for (weapon, attack) in self.weapons(unit).enumerate() {
                        let distant = board.distance(unit.tile, target.tile) as usize
                            > movement + attack.range as usize;
                        if distant && !zero_cost {
                            continue;
                        }
                        let kill_chance = self.potential_kill_chance(unit, target, weapon);
                        let killed = kill_chance == 1.0;
                        let kill_value = kill_chance
                            * (12.0 + self.recruit_cost(target, target.kind) as f64 * 0.3);
                        let essential_value = if orders.important(self, target) {
                            4.0 + 14.0 * kill_chance
                        } else {
                            0.0
                        };
                        let value = self.potential_damage(unit, target, weapon) * 2.0
                            + kill_value
                            + essential_value;
                        let terminal_attack = killed && orders.terminal_attack(target);
                        if !distant {
                            offense = offense.max(if terminal_attack {
                                value + 100.0
                            } else if important {
                                value
                                    * if unit.role == Some(Role::Courier) {
                                        0.1
                                    } else {
                                        0.5
                                    }
                            } else {
                                value
                            });
                        }
                        if BOUNDED {
                            // Dropping retaliation and death penalties gives an upper bound.
                            // A kill may also award the three-point promotion bonus.
                            let mut bound = (self.potential_damage(unit, target, weapon) * 2.0
                                + kill_value.upper())
                            .upper();
                            bound = (bound + essential_value).upper();
                            if killed {
                                bound = (bound + 3.0).upper();
                            }
                            if terminal_attack {
                                bound = (bound + 100.0).upper();
                            } else if important && !terminal_objective {
                                bound = (bound
                                    * if unit.role == Some(Role::Courier) {
                                        0.1
                                    } else {
                                        0.5
                                    })
                                .upper();
                            }
                            maximum_combat = maximum_combat.max(bound);
                        }
                    }
                }
                (
                    index,
                    unit,
                    before_risk,
                    offense + before_risk as f64 * danger,
                    maximum_combat,
                    terminal_objective,
                )
            })
            .collect();
        roster.order_unstable_by(|a, b| b.3.total_cmp(&a.3).then(a.0.cmp(&b.0)));
        for (unit_order, original_unit, before_risk, _, maximum_combat, terminal_objective) in
            roster
        {
            let category = role(self, original_unit.kind);
            let important = orders.important(self, original_unit);
            if let Some(promotion) = orders.promotion(self, original_unit) {
                let order = (unit_order, 0, 0);
                if before_risk < original_unit.hp
                    && best
                        .as_ref()
                        .is_none_or(|b| b.score < 14.0 || b.score == 14.0 && order < best_order)
                {
                    best_order = order;
                    best = Some(Plan {
                        unit: original_unit.id,
                        tile: original_unit.tile,
                        score: 14.0,
                        target: None,
                        weapon: None,
                        advance: Some(promotion),
                        withdrawing: false,
                    });
                }
            }
            let reserved = self.reserved(original_unit, &objectives);
            let required = orders.moving_actor(original_unit.id)
                || escort_lanes.contains_key(&original_unit.id);
            let unchanged = (Vec::new(), original_unit.rebuilding);
            let (rebuilding, intent) = rebuilds.get(&original_unit.id).unwrap_or(&unchanged);
            let mut planned_unit = (*original_unit).clone();
            planned_unit.rebuilding = *intent;
            let unit = &planned_unit;
            let recruiting = unit.is_leader()
                && self
                    .recruitment_options(side)
                    .iter()
                    .any(|&k| !self.placement_tiles(&unit, k, true).is_empty());
            let mut rebuild_goals = vec![];
            if !rebuilding.is_empty() {
                let mut g = objective(
                    "rebuild",
                    ObjectiveKind::Capture,
                    rebuilding
                        .iter()
                        .map(|&t| self.board().ids[t].clone())
                        .collect(),
                    80.0,
                );
                g.actor = Some(format!("u{}", unit.id));
                rebuild_goals.push(g);
            }
            let goals: &[Objective] = if rebuilding.is_empty() {
                &objectives
            } else {
                &rebuild_goals
            };
            struct Duty<'a> {
                goal: Cow<'a, Objective>,
                route: std::rc::Rc<Vec<u16>>,
                before: u16,
                assigned: bool,
                priority: f64,
            }
            let mut duties = Vec::with_capacity(goals.len());
            for original in goals {
                if orders.assigned_elsewhere(unit.id, original) {
                    continue;
                }
                if original.kind == ObjectiveKind::Capture
                    && original.actor.is_none()
                    && original.tiles.iter().all(|t| {
                        let tile = self.site(t);
                        self.owns(tile, side)
                            && (original.hold != Some(true)
                                || self.at(tile).is_some_and(|o| {
                                    self.belongs_to_phase(o, side) && o.id != unit.id
                                }))
                    })
                {
                    continue;
                }
                if original.id.starts_with("rebuild:") && !actor_matches(&original.actor, unit.id) {
                    continue;
                }
                let pursuing = self.can_pursue(&unit, original);
                let assisting =
                    !pursuing && (original.actor.is_some() || original.family.is_some());
                if !pursuing && !assisting {
                    continue;
                }
                let goal = if assisting {
                    Cow::Owned(self.support_objective(&unit, original))
                } else {
                    Cow::Borrowed(original)
                };
                let tiles: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
                let route = if pursuing
                    && (original.actor.is_some() || original.family.is_some())
                    && crate::ai::objectives::actor_route(original)
                {
                    let occupied = self.occupied_routes(unit, &tiles, false);
                    if occupied[unit.tile] == u16::MAX {
                        self.routes(unit.kind, &tiles)
                    } else {
                        std::rc::Rc::new(occupied)
                    }
                } else {
                    routes
                        .entry((unit.kind, tiles.clone()))
                        .or_insert_with(|| self.routes(unit.kind, &tiles))
                        .clone()
                };
                let before = route[unit.tile];
                if before == u16::MAX || goal.tiles.is_empty() {
                    continue;
                }
                duties.push(Duty {
                    priority: (before as f64 + 3.0) / goal.weight,
                    assigned: self.assigned_duty(&unit, original),
                    goal,
                    route,
                    before,
                });
            }
            if duties.iter().any(|d| d.assigned) {
                duties.retain(|d| d.assigned);
            }
            let order = tile_order_context.for_unit(self, &unit);
            let nearest = |goal: &Objective| {
                goal.tiles
                    .iter()
                    .map(|t| self.site(t))
                    .min_by(|&a, &b| {
                        self.board()
                            .distance(unit.tile, a)
                            .cmp(&self.board().distance(unit.tile, b))
                            .then_with(|| order.compare(self, a, b))
                    })
                    .unwrap()
            };
            let committed = duties
                .iter()
                .position(|d| commitments.get(&unit.id) == Some(&d.goal.id));
            let duty = if let Some(index) = committed {
                Some(duties.swap_remove(index))
            } else {
                duties.order_by(|a, b| {
                    a.priority
                        .total_cmp(&b.priority)
                        .then_with(|| order.compare(self, nearest(&a.goal), nearest(&b.goal)))
                });
                duties.into_iter().next()
            };
            if let Some(d) = &duty {
                commitments.insert(unit.id, d.goal.id.clone());
            } else {
                commitments.remove(&unit.id);
            }
            let danger_weight = if orders.protects(unit.id) {
                // A finite cost for the fraction of remaining life at risk.
                40.0 / unit.hp.max(1) as f64
            } else if important {
                6.0
            } else if category == TacticalRole::Frontline {
                0.55
            } else {
                0.9
            };

            if BOUNDED && best.is_some() {
                // Bound the whole unit before discovering destinations or scoring tiles.
                // Every omitted term is a penalty. Round each operation outward so
                // floating-point ties still reach the original ordinal comparison.
                let add = |a: f64, b: f64| (a + b).upper();
                let on_reserved = reserved.contains(&unit.tile);
                let mut ceiling = if on_reserved { 80.0 } else { 0.0 };
                ceiling = add(ceiling, escort.bound(unit.id));
                if escort_lanes.contains_key(&unit.id) {
                    ceiling = add(ceiling, 40.0);
                }
                if let Some(d) = &duty {
                    let movement = if unit.moved {
                        0
                    } else {
                        self.rules().units[unit.kind].r#move
                    };
                    let progress = d.before.min(movement.min(u16::MAX as usize) as u16) as f64;
                    ceiling = add(ceiling, ((progress * d.goal.weight).upper() / 4.0).upper());
                    // Reaching an endpoint costs at least its current reverse
                    // route distance, including occupancy-aware escort routes.
                    if !unit.moved
                        && d.before as usize <= movement
                        && !d.goal.tiles.contains(&self.board().ids[unit.tile])
                    {
                        ceiling = add(ceiling, (d.goal.weight * 0.6).upper());
                    }
                }
                ceiling = add(
                    ceiling,
                    (self.rules().units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0,
                );
                ceiling = add(ceiling, 3.0);
                {
                    if category == TacticalRole::Healer {
                        let healing = self.rules().units[unit.kind].healing;
                        for ally in &allies {
                            if ally.id == unit.id {
                                continue;
                            }
                            ceiling = add(
                                ceiling,
                                healing
                                    .min(self.rules().units[ally.kind].hp - ally.hp)
                                    .max(0) as f64
                                    * 3.0,
                            );
                            if role(self, ally.kind) == TacticalRole::Frontline
                                && range_to_foe(ally.tile) <= 4
                            {
                                ceiling = add(ceiling, 3.0);
                            }
                        }
                    }
                    if category != TacticalRole::Frontline && !screens.is_empty() {
                        ceiling = add(ceiling, 2.0);
                    }
                    if category == TacticalRole::Frontline {
                        for protected in &protected {
                            if protected.id != unit.id {
                                ceiling = add(ceiling, 1.5);
                            }
                        }
                    }
                }
                if terminal_objective {
                    ceiling = add(ceiling, 100.0);
                }
                let minimum_position = if on_reserved { -80.0 } else { 0.0 }
                    - if lanes.contains(&unit.tile) {
                        100.0
                    } else {
                        0.0
                    }
                    - if true
                        && category != TacticalRole::Frontline
                        && !screens.is_empty()
                        && range_to_foe(unit.tile) <= 4
                    {
                        3.0
                    } else {
                        0.0
                    };
                let minimum_wait =
                    (minimum_position - (before_risk as f64 * danger_weight).upper()).lower();
                let minimum_wait = (minimum_wait
                    - if before_risk >= unit.hp {
                        if important { 40.0 } else { 5.0 }
                    } else {
                        0.0
                    })
                .lower();
                ceiling = add(ceiling, maximum_combat);
                ceiling = add(
                    ceiling,
                    (before_risk as f64
                        * if important {
                            3.0
                        } else if unit.hp <= 4 {
                            1.5
                        } else {
                            0.2
                        })
                    .upper(),
                );
                ceiling = add(ceiling, 0.05);
                ceiling = (ceiling - minimum_wait).upper();
                if ceiling < best.as_ref().unwrap().score.max(0.1) {
                    continue;
                }
            }
            let reachable = self.reachable(original_unit);
            let mut destinations = Vec::with_capacity(reachable.len() + 1);
            destinations.push(original_unit.tile);
            destinations.extend(reachable.iter().copied().map(|(t, _)| t));
            destinations = orders.authored_destinations(self, original_unit, destinations);
            let other_healing: Vec<_> = if category == TacticalRole::Healer {
                allies
                    .iter()
                    .map(|ally| {
                        allies
                            .iter()
                            .filter(|o| {
                                o.id != unit.id
                                    && o.id != ally.id
                                    && self.board().distance(o.tile, ally.tile) == 1
                            })
                            .map(|o| self.rules().units[o.kind].healing)
                            .max()
                            .unwrap_or(0)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let mut wait_score = f64::NEG_INFINITY;
            let mut tiles = vec![unit.tile];
            tiles.extend(destinations.iter().copied().filter(|&t| t != unit.tile));
            for (tile_order, tile) in tiles.into_iter().enumerate() {
                let mut positioning = if reserved.contains(&unit.tile) {
                    80.0
                } else {
                    0.0
                } - if reserved.contains(&tile) { 160.0 } else { 0.0 }
                    - if lanes.contains(&tile) { 100.0 } else { 0.0 };
                positioning += escort.position(unit.id, tile);
                if escort_lanes
                    .get(&unit.id)
                    .is_some_and(|lane| !lane.contains(&tile))
                {
                    positioning += 40.0;
                }
                if let Some(d) = &duty {
                    let after = d.route[tile];
                    if after == u16::MAX {
                        continue;
                    }
                    positioning += (d.before as f64 - after as f64) * d.goal.weight / 4.0
                        + ((d.goal.tiles.contains(&self.board().ids[tile]) as i32
                            - d.goal.tiles.contains(&self.board().ids[unit.tile]) as i32)
                            as f64)
                            * d.goal.weight
                            * 0.6;
                }
                if rebuilding.contains(&unit.tile)
                    && tile != unit.tile
                    && threats.at(&unit, tile, None) >= before_risk
                {
                    continue;
                }
                if tile != unit.tile
                    && objectives.iter().any(|g| {
                        g.hold == Some(true)
                            && actor_matches(&g.actor, unit.id)
                            && g.tiles.contains(&self.board().ids[unit.tile])
                    })
                {
                    positioning -= 24.0;
                }
                let healing_tile = (self.role_is(tile, TerrainRole::Village)
                    || self.role_is(tile, TerrainRole::Grove))
                    && (self.owns(tile, side) || self.at(tile).is_none());
                if healing_tile {
                    positioning += (self.rules().units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0;
                }
                if self.role_is(tile, TerrainRole::Village) && self.captures(tile, side) {
                    positioning += 3.0;
                }
                {
                    if category == TacticalRole::Healer {
                        let healing = self.rules().units[unit.kind].healing;
                        let mut value = 0.0;
                        for (ally_index, ally) in allies.iter().enumerate() {
                            if ally.id == unit.id || self.board().distance(tile, ally.tile) != 1 {
                                continue;
                            }
                            let other = other_healing[ally_index];
                            value += (healing - other)
                                .max(0)
                                .min(self.rules().units[ally.kind].hp - ally.hp)
                                as f64
                                * 3.0;
                            if role(self, ally.kind) == TacticalRole::Frontline
                                && range_to_foe(ally.tile) <= 4
                            {
                                value += if range_to_foe(tile) > range_to_foe(ally.tile) {
                                    3.0
                                } else {
                                    0.5
                                };
                            }
                        }
                        positioning += value
                            * if movable_screens
                                && unit.hp as f64 > self.rules().units[unit.kind].hp as f64 / 2.0
                            {
                                0.25
                            } else {
                                1.0
                            };
                    }
                    if category != TacticalRole::Frontline
                        && !screens.is_empty()
                        && range_to_foe(tile) <= 4
                    {
                        let front = screens.iter().map(|u| range_to_foe(u.tile)).min().unwrap();
                        positioning += if range_to_foe(tile) > front {
                            2.0
                        } else {
                            -3.0
                        };
                    }
                    if category == TacticalRole::Frontline {
                        for protected in &protected {
                            if protected.id != unit.id
                                && self.board().distance(tile, protected.tile) == 1
                                && range_to_foe(tile) <= range_to_foe(protected.tile)
                            {
                                positioning += 1.5;
                            }
                        }
                    }
                }
                if tile == unit.tile {
                    wait_score = positioning
                        - before_risk as f64 * danger_weight
                        - if before_risk >= unit.hp {
                            if important { 40.0 } else { 5.0 }
                        } else {
                            0.0
                        };
                }
                if !destinations.contains(&tile) {
                    continue;
                }
                let objective_finish = orders.arrives(self, &unit, tile);
                let choices = std::iter::once((None, None)).chain(
                    nearby_foes[nearby_offsets[tile]..nearby_offsets[tile + 1]]
                        .iter()
                        .flat_map(|&target| {
                            (0..self.weapons(unit).count())
                                .filter_map(move |weapon| Some((Some(target), Some(weapon))))
                        }),
                );
                let mut incoming_normal = None;
                for (choice_order, (target, weapon)) in choices.enumerate() {
                    if let (Some(target), Some(weapon)) = (target, weapon) {
                        if !self.can_attack(unit, tile, target, weapon)
                            || !orders.attack_allowed(self, unit, target, weapon, tile)
                        {
                            continue;
                        }
                    }
                    if target.is_none() && tile == unit.tile {
                        continue;
                    }
                    let mut combat = 0.0;
                    let mut retaliation = 0;
                    let mut eliminated = None;
                    let mut finishes_battle = false;
                    if let (Some(target), Some(weapon)) = (target, weapon) {
                        let forecast = self.exchange(unit, target, weapon, tile);
                        let damage = forecast.damage[0].round() as i32;
                        let response = forecast.damage[1].round() as i32;
                        retaliation = if important {
                            self.maximum_retaliation(unit, target, weapon, tile)
                        } else {
                            response
                        };
                        let kill_chance = forecast.death[1];
                        let killed = kill_chance == 1.0;
                        eliminated = killed.then_some(target.id);
                        finishes_battle = killed && orders.terminal_attack(target);
                        combat = forecast.effective[0] * 2.0
                            + kill_chance
                                * (12.0 + self.recruit_cost(target, target.kind) as f64 * 0.3)
                            - retaliation as f64 * 1.8
                            + escort.attack(unit.id, target, damage, kill_chance);
                        if orders.important(self, target) {
                            combat += 4.0 + 14.0 * kill_chance;
                        }
                        if killed
                            && self.rules().units[unit.kind].experience_needed.is_some_and(
                                |needed| {
                                    unit.experience.unwrap_or(0)
                                        + 6 * self.rules().units[target.kind].level
                                        >= needed
                                },
                            )
                        {
                            combat += 3.0;
                        }
                        if retaliation >= unit.hp {
                            if important && !orders.protects(unit.id) {
                                continue;
                            }
                            combat -= 18.0;
                        }
                    }
                    let finishing = objective_finish && retaliation < unit.hp;
                    // Even zero incoming damage cannot rescue a candidate below
                    // this bound. Keep strict winner/tie ordering unchanged.
                    let weighted_combat = if important && !finishing && !finishes_battle {
                        combat
                            * if unit.role == Some(Role::Courier) {
                                0.1
                            } else {
                                0.5
                            }
                    } else {
                        combat
                    };
                    let improvement_weight = if important {
                        3.0
                    } else if unit.hp <= 4 {
                        1.5
                    } else {
                        0.2
                    };
                    let ceiling = positioning
                        + if finishing || finishes_battle {
                            100.0
                        } else {
                            0.0
                        }
                        + weighted_combat
                        + before_risk as f64 * improvement_weight
                        + 0.05
                        - wait_score;
                    let ceiling = ceiling.next_up();
                    if ceiling <= best.as_ref().map_or(0.1, |b| b.score.max(0.1)) {
                        continue;
                    }
                    if unit.is_leader()
                        && !finishing
                        && !finishes_battle
                        && rebuilding.is_empty()
                        && !self.leader_may_advance(&unit, tile, recruiting, required)
                    {
                        continue;
                    }
                    let incoming = if eliminated.is_none() {
                        *incoming_normal.get_or_insert_with(|| threats.at(&unit, tile, None))
                    } else {
                        threats.at(&unit, tile, eliminated)
                    };
                    let incoming = super::recruitment::exposure(&unit, incoming, important);
                    let lethal = incoming >= unit.hp - retaliation;
                    if important && !finishing && !finishes_battle {
                        combat *= if unit.role == Some(Role::Courier) {
                            0.1
                        } else {
                            0.5
                        };
                    }
                    let score = positioning
                        + if finishing || finishes_battle {
                            100.0
                        } else {
                            0.0
                        }
                        + combat
                        - if finishes_battle {
                            0.0
                        } else {
                            incoming as f64 * danger_weight
                        }
                        - if lethal && !finishes_battle {
                            if important { 40.0 } else { 5.0 }
                        } else {
                            0.0
                        }
                        + (before_risk - incoming).max(0) as f64
                            * if important {
                                3.0
                            } else if unit.hp <= 4 {
                                1.5
                            } else {
                                0.2
                            }
                        + action_jitter(
                            seed,
                            self.round(),
                            unit.id,
                            &self.board().ids[tile],
                            target.map(|target| target.id),
                        );
                    let withdrawing = unit.side == Side::Enemy
                        && target.is_none()
                        && unit.hp < self.rules().units[unit.kind].hp
                        && range_to_foe(tile) > range_to_foe(unit.tile)
                        && (incoming < before_risk
                            || healing_tile
                            || unit.retreating == Some(true));
                    let mut plan = Plan {
                        unit: unit.id,
                        tile,
                        target: target.map(|t| t.id),
                        weapon,
                        score,
                        advance: None,
                        withdrawing,
                    };
                    plan.score -= wait_score;
                    let order = (unit_order, tile_order + 1, choice_order);
                    if plan.score > 0.1
                        && best.as_ref().is_none_or(|b| {
                            plan.score > b.score || plan.score == b.score && order < best_order
                        })
                    {
                        best_order = order;
                        best = Some(plan);
                    }
                }
            }
        }
        for unit in self
            .units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, side) && focus.is_none_or(|id| id == u.id))
        {
            if let Some(recruiting) = super::recruitment::plan(self, unit, orders, recruitment) {
                if best.as_ref().is_none_or(|p| recruiting.score > p.score) {
                    best = Some(recruiting);
                }
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;
    #[test]
    fn whole_unit_bounds_preserve_plans_and_commitments_through_live_actions() {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                crate::logic::Command::EndTurn => true,
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
        for mut game in [
            Game::new(&rules, "mirror"),
            Game::start_campaign(
                &rules,
                "borrowed-tide",
                0,
                vec![],
                0,
                vec![],
                vec![],
                vec![],
            ),
        ] {
            let workspace = || {
                crate::ai::PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Pragmatic)
                    .with_blocks(crate::ai::Blocks {
                        planner: crate::ai::Planner::Full,
                        ..crate::ai::Policy::Balanced.blocks()
                    })
            };
            let mut actual = workspace();
            let mut left_routes = HashMap::default();
            let mut right_routes = HashMap::default();
            let mut left_commitments = HashMap::default();
            let mut right_commitments = HashMap::default();
            let mut left_threats = crate::ai::threats::ArenaThreatWorkspace::default();
            let mut right_threats = crate::ai::threats::ArenaThreatWorkspace::default();
            for _ in 0..24 {
                for _ in [()] {
                    let mut left_intents = vec![];
                    let mut right_intents = vec![];
                    let left = crate::ai::context::BattleView::new(&game).best_plan::<true, false>(
                        &crate::ai::CampaignBlock::Mission
                            .campaign_orders(
                                &game,
                                &crate::ai::PlanningWorkspace::new(
                                    7,
                                    1,
                                    None,
                                    crate::ai::Diplomacy::Pragmatic,
                                ),
                            )
                            .tactics,
                        &crate::ai::search_escort::Workspace::default(),
                        7,
                        None,
                        &mut left_routes,
                        &mut left_commitments,
                        &mut left_intents,
                        &mut left_threats,
                        None,
                        None,
                    );
                    let right = crate::ai::context::BattleView::new(&game)
                        .best_plan::<false, false>(
                            &crate::ai::CampaignBlock::Mission
                                .campaign_orders(
                                    &game,
                                    &crate::ai::PlanningWorkspace::new(
                                        7,
                                        1,
                                        None,
                                        crate::ai::Diplomacy::Pragmatic,
                                    ),
                                )
                                .tactics,
                            &crate::ai::search_escort::Workspace::default(),
                            7,
                            None,
                            &mut right_routes,
                            &mut right_commitments,
                            &mut right_intents,
                            &mut right_threats,
                            None,
                            None,
                        );
                    assert_eq!(left, right);
                    assert_eq!(left_intents, right_intents);
                    assert_eq!(left_commitments, right_commitments);
                }
                let command =
                    crate::ai::choose_game_command(&game, crate::ai::Policy::Balanced, &mut actual)
                        .unwrap();
                if is_command_endturn(&command) {
                    if game.winner.is_some() {
                        break;
                    }
                    game.end_turn();
                    actual = workspace();
                } else {
                    game.execute(command).unwrap();
                }
            }
        }
    }
    use super::{Random, action_jitter, jitter};
    #[test]
    fn action_jitter_matches_formatted_utf16_keys() {
        for seed in [0, 19, u32::MAX] {
            for (round, unit) in [(0, 0), (30, 12), (u32::MAX, u32::MAX)] {
                for tile in ["3,2", "😀", "-10,100"] {
                    for target in [None, Some(0), Some(u32::MAX)] {
                        let key = format!(
                            "{round}:u{unit}:{tile}:{}",
                            target.map_or("move".into(), |id| format!("u{id}"))
                        );
                        assert_eq!(
                            action_jitter(seed, round, unit, tile, target),
                            jitter(seed, &key)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn seeded_random_is_reproducible_and_bounded() {
        for seed in [0, 1, u32::MAX] {
            let mut left = Random(seed);
            let mut right = Random(seed);
            for _ in 0..32 {
                let value = left.next();
                assert_eq!(value, right.next());
                assert!((0.0..1.0).contains(&value));
            }
        }
    }
}

pub(super) fn prepare(game: &BattleView<'_, '_>, workspace: &mut crate::ai::PlanningWorkspace) {
    workspace.search_escort.prepare(
        game,
        &workspace.report,
        &workspace.orders.as_ref().unwrap().tactics,
    );
}

pub(super) fn choose(
    game: &BattleView<'_, '_>,
    workspace: &mut crate::ai::PlanningWorkspace,
    queued: bool,
) -> Result<(), CommandError> {
    let orders = &workspace.orders.as_ref().expect("campaign orders").tactics;
    let remaining = workspace.tactical_remaining.get_or_insert_with(|| {
        game.units()
            .iter()
            .filter(|u| game.belongs_to_phase(u, game.turn()) && game.can_act(u.id))
            .count()
            * 3
    });
    if !queued {
        if *remaining == 0 {
            workspace.finished = true;
            return Ok(());
        }
        *remaining -= 1;
    }
    let mut intents = Vec::new();
    if workspace.prepared_goals.is_none() {
        workspace.prepared_goals = workspace.orders.as_ref().map(|orders| {
            individual(
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
    let plan = if queued {
        crate::ai::priority::next_plan(game, workspace, &mut intents)
    } else {
        game.best_plan::<true, false>(
            orders,
            &workspace.search_escort,
            workspace.seed,
            workspace.prepared_goals.as_deref(),
            &mut workspace.routes,
            &mut workspace.commitments,
            &mut intents,
            &mut workspace.threats,
            None,
            workspace.recruitment_opportunity.as_ref(),
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
                    WeaponId(game.weapon(actor, weapon).id.clone()),
                ));
            }
        }
    } else {
        workspace.finished = true;
    }
    Ok(())
}
