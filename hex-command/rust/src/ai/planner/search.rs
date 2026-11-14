use super::proposal::Plan;
use crate::ai::TacticalRole;
use crate::ai::orders::target as objective;
use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::ai::policies::escort::scoring as escort_scoring;
use crate::ai::policies::objectives::actor_matches;
use crate::ai::policies::tie_breaking::action_jitter;
use crate::ai::roles::role;
use crate::ai::{advance_command, intent_command, move_command};
use crate::sort::SharedSort;
use crate::{
    ai::context::BattleView,
    ai::policies::objectives::{TileOrder, individual},
    ai::policies::threat::Threats,
    data::*,
};
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
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
pub(super) fn best_plan<'a, 'catalog, const BOUNDED: bool, const FOCUSED: bool>(
    game: &'a BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    orders: &crate::ai::orders::TacticalOrders,
    escort: &crate::ai::policies::escort::travel::Workspace,
    seed: u32,
    prepared_goals: Option<&[Objective]>,
    routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
    commitments: &mut Commitments,
    intents: &mut Vec<(u32, Option<bool>)>,
    threat_workspace: &mut crate::ai::policies::threat::ArenaThreatWorkspace,
    focus: Option<u32>,
    recruitment: Option<crate::ai::policies::recruitment::EstimateInput<'_>>,
) -> Option<Plan> {
    Evaluation(game, phases).best_plan::<BOUNDED, FOCUSED>(
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
    )
}
struct Evaluation<'view, 'game, 'catalog>(
    &'view BattleView<'game, 'catalog>,
    hex_command_battle::PhaseForecasts<'game, 'catalog>,
);
impl<'view, 'game, 'catalog> std::ops::Deref for Evaluation<'view, 'game, 'catalog> {
    type Target = BattleView<'game, 'catalog>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'view, 'game, 'catalog> Evaluation<'view, 'game, 'catalog> {
    #[inline(always)]
    pub(super) fn best_plan<const BOUNDED: bool, const FOCUSED: bool>(
        &self,
        orders: &crate::ai::orders::TacticalOrders,
        escort: &crate::ai::policies::escort::travel::Workspace,
        seed: u32,
        prepared_goals: Option<&[Objective]>,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut Commitments,
        intents: &mut Vec<(u32, Option<bool>)>,
        threat_workspace: &mut crate::ai::policies::threat::ArenaThreatWorkspace,
        focus: Option<u32>,
        recruitment: Option<crate::ai::policies::recruitment::EstimateInput<'_>>,
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
        escort: &crate::ai::policies::escort::travel::Workspace,
        seed: u32,
        prepared_goals: Option<&[Objective]>,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut Commitments,
        intents: &mut Vec<(u32, Option<bool>)>,
        threat_workspace: &mut crate::ai::policies::threat::ArenaThreatWorkspace,
        focus: Option<u32>,
        recruitment: Option<crate::ai::policies::recruitment::EstimateInput<'_>>,
        _focused: bool,
    ) -> Option<Plan> {
        #[cfg(target_arch = "wasm32")]
        let focused = _focused;
        #[cfg(not(target_arch = "wasm32"))]
        let focused = FOCUSED;
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
            .filter(|u| {
                self.belongs_to_faction(u, self.observer_faction())
                    && self.can_act(u.id)
                    && u.is_leader()
            })
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
            .filter(|u| self.belongs_to_faction(u, self.observer_faction()) && self.can_act(u.id))
            .collect();
        if allies.iter().all(|u| u.attack_spent) {
            return None;
        }
        let foes: Vec<_> = self
            .units()
            .iter()
            .filter(|u| self.hostile_to_faction(u, self.observer_faction()))
            .collect();
        let lanes = orders.lanes(self);
        let mut threats =
            Threats::with_workspace(self.1, self.observer_faction(), threat_workspace)
                .with_orders(orders);
        let screens: Vec<_> = allies
            .iter()
            .filter(|u| role(self, u.kind) == TacticalRole::Frontline && !u.is_leader())
            .collect();
        let movable_screens = screens.iter().any(|u| !u.moved && !u.attack_spent);
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
        let tile_order_context = TileOrder::context(self, self.observer_faction(), true);
        let mut best: Option<Plan> = None;
        let mut best_order = (usize::MAX, usize::MAX, usize::MAX);
        // Score units with the strongest immediate opportunities first. This
        // only changes evaluation order: roster/tile/weapon ordinals still
        // decide all equal scores and bookkeeping remains unit-local.
        let mut roster: Vec<_> = allies
            .iter()
            .enumerate()
            .filter(|(_, u)| !u.attack_spent && (!focused || Some(u.id) == focus))
            .map(|(index, &unit)| {
                let before_risk = crate::ai::policies::threat::cap_exposure(
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
                                    * if unit.role.is_some_and(Role::is_escort) {
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
                                    * if unit.role.is_some_and(Role::is_escort) {
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
            let reserved =
                crate::ai::policies::objectives::reserved(self, original_unit, &objectives);
            let required = orders.moving_actor(original_unit.id)
                || escort_lanes.contains_key(&original_unit.id);
            let unchanged = (Vec::new(), original_unit.rebuilding);
            let (rebuilding, intent) = rebuilds.get(&original_unit.id).unwrap_or(&unchanged);
            let mut planned_unit = (*original_unit).clone();
            planned_unit.rebuilding = *intent;
            let unit = &planned_unit;
            let recruiting = unit.is_leader()
                && self
                    .recruitment_options_for_faction(self.observer_faction())
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
                        self.owns(tile, self.observer_faction())
                            && (original.hold != Some(true)
                                || self.at(tile).is_some_and(|o| {
                                    self.belongs_to_faction(o, self.observer_faction())
                                        && o.id != unit.id
                                }))
                    })
                {
                    continue;
                }
                if original.id.starts_with("rebuild:") && !actor_matches(&original.actor, unit.id) {
                    continue;
                }
                let pursuing = crate::ai::policies::objectives::can_pursue(self, &unit, original);
                let assisting =
                    !pursuing && (original.actor.is_some() || original.family.is_some());
                if !pursuing && !assisting {
                    continue;
                }
                let goal = if assisting {
                    Cow::Owned(crate::ai::policies::objectives::support_objective(
                        self, &unit, original,
                    ))
                } else {
                    Cow::Borrowed(original)
                };
                let tiles: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
                let route = if pursuing
                    && (original.actor.is_some() || original.family.is_some())
                    && crate::ai::policies::objectives::actor_route(original)
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
                    assigned: crate::ai::policies::objectives::assigned_duty(self, &unit, original),
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
            let escortee = escort_scoring::subject(orders, &unit);
            let improvement_weight = if escortee {
                0.0
            } else if important {
                3.0
            } else if unit.hp <= 4 {
                1.5
            } else {
                0.2
            };
            let danger_weight = if escortee {
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
                    if escortee {
                        // A loose bound preserves all winners under the bounded
                        // fifteen-point escort progress/arrival policy.
                        ceiling = add(ceiling, 15.0);
                    } else {
                        ceiling = add(ceiling, ((progress * d.goal.weight).upper() / 4.0).upper());
                    }
                    // Reaching an endpoint costs at least its current reverse
                    // route distance, including occupancy-aware escort routes.
                    if !escortee
                        && !unit.moved
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
                    - if !escortee && before_risk >= unit.hp {
                        if important { 40.0 } else { 5.0 }
                    } else {
                        0.0
                    })
                .lower();
                ceiling = add(ceiling, maximum_combat);
                ceiling = add(ceiling, (before_risk as f64 * improvement_weight).upper());
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
                    if escortee {
                        positioning += escort_scoring::progress(
                            d.before,
                            after,
                            d.goal.tiles.contains(&self.board().ids[unit.tile]),
                            d.goal.tiles.contains(&self.board().ids[tile]),
                        );
                    } else {
                        positioning += (d.before as f64 - after as f64) * d.goal.weight / 4.0
                            + ((d.goal.tiles.contains(&self.board().ids[tile]) as i32
                                - d.goal.tiles.contains(&self.board().ids[unit.tile]) as i32)
                                as f64)
                                * d.goal.weight
                                * 0.6;
                    }
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
                    && (self.owns(tile, self.observer_faction()) || self.at(tile).is_none());
                if healing_tile {
                    positioning += (self.rules().units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0;
                }
                if self.role_is(tile, TerrainRole::Village)
                    && self.captures(tile, self.observer_faction())
                {
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
                        - if !escortee && before_risk >= unit.hp {
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
                            * if unit.role.is_some_and(Role::is_escort) {
                                0.1
                            } else {
                                0.5
                            }
                    } else {
                        combat
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
                        && !crate::ai::policies::objectives::leader_may_advance(
                            self, &unit, tile, recruiting, required,
                        )
                    {
                        continue;
                    }
                    let incoming = if eliminated.is_none() {
                        *incoming_normal.get_or_insert_with(|| threats.at(&unit, tile, None))
                    } else {
                        threats.at(&unit, tile, eliminated)
                    };
                    let incoming =
                        crate::ai::policies::threat::cap_exposure(&unit, incoming, important);
                    let lethal = incoming >= unit.hp - retaliation;
                    if important && !finishing && !finishes_battle {
                        combat *= if unit.role.is_some_and(Role::is_escort) {
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
                        + (before_risk - incoming).max(0) as f64 * improvement_weight
                        + action_jitter(
                            seed,
                            self.round(),
                            unit.id,
                            self.board().positions[tile],
                            target.map(|target| target.id),
                        );
                    let withdrawing = self.faction_relation("player", self.unit_faction(&unit))
                        == Relation::Hostile
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
        for unit in self.units().iter().filter(|u| {
            self.belongs_to_faction(u, self.observer_faction()) && focus.is_none_or(|id| id == u.id)
        }) {
            if let Some(recruiting) =
                super::recruitment::plan(self, self.1, unit, orders, recruitment)
            {
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
    use hex_command_battle::RuntimeCommand as Command;
    #[test]
    fn whole_unit_bounds_preserve_plans_and_commitments_through_live_actions() {
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
        for mut game in [
            Game::new(&rules, "mirror"),
            Game::start_campaign(&rules, "mirror-night", 0, vec![], 0, vec![], vec![], vec![]),
        ] {
            let workspace = || {
                crate::ai::PlanningWorkspace::new(7, 1, None, crate::ai::Diplomacy::Unrestricted)
                    .with_blocks(crate::ai::Blocks {
                        planner: crate::ai::Planner::Full,
                        ..crate::ai::Policy::Balanced.blocks()
                    })
            };
            let mut actual = workspace();
            let mut left_routes = HashMap::default();
            let mut right_routes = HashMap::default();
            let mut left_commitments = Commitments::default();
            let mut right_commitments = Commitments::default();
            let mut left_threats = crate::ai::policies::threat::ArenaThreatWorkspace::default();
            let mut right_threats = crate::ai::policies::threat::ArenaThreatWorkspace::default();
            for _ in 0..24 {
                for _ in [()] {
                    let mission = crate::ai::campaigner::mission::MissionPolicy::new(
                        game.campaign_view(),
                        game.phase_forecasts(),
                    );
                    let input = crate::ai::campaigner::mission::CampaignInput {
                        seed: 0,
                        goals: None,
                        diplomacy: crate::ai::Diplomacy::Unrestricted,
                    };
                    let orders = crate::ai::CampaignBlock::Mission.campaign_orders(&mission, input);
                    let mut left_intents = vec![];
                    let mut right_intents = vec![];
                    let left = best_plan::<true, false>(
                        &game.view(),
                        game.phase_forecasts(),
                        &orders.tactics,
                        &crate::ai::policies::escort::travel::Workspace::default(),
                        7,
                        None,
                        &mut left_routes,
                        &mut left_commitments,
                        &mut left_intents,
                        &mut left_threats,
                        None,
                        None,
                    );
                    let right = best_plan::<false, false>(
                        &game.view(),
                        game.phase_forecasts(),
                        &orders.tactics,
                        &crate::ai::policies::escort::travel::Workspace::default(),
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
                    if game.winner().is_some() {
                        break;
                    }
                    game.execute_runtime(Command::EndTurn).unwrap();
                    actual = workspace();
                } else {
                    game.execute_runtime(command).unwrap();
                }
            }
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub(in crate::ai::planner) struct Commitments(HashMap<u32, String>);
impl Commitments {
    pub(super) fn get(&self, actor: &u32) -> Option<&String> {
        self.0.get(actor)
    }
    pub(super) fn insert(&mut self, actor: u32, target: String) {
        self.0.insert(actor, target);
    }
    pub(super) fn remove(&mut self, actor: &u32) {
        self.0.remove(actor);
    }
}
pub(super) struct CandidateShared<'a> {
    pub escort: &'a crate::ai::policies::escort::travel::Workspace,
    pub commitments: &'a mut Commitments,
    pub threats: &'a mut crate::ai::policies::threat::ArenaThreatWorkspace,
}
/// Search state is shared deliberately by Full evaluation and Queued's focused fallback.
#[derive(Default)]
pub(in crate::ai) struct Session {
    prepared_goals: Option<Vec<Objective>>,
    routes: HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
    commitments: Commitments,
    threats: crate::ai::policies::threat::ArenaThreatWorkspace,
    escort: crate::ai::policies::escort::travel::Workspace,
    remaining: Option<usize>,
}
impl Session {
    pub(in crate::ai) fn replace_goals(&mut self, goals: Vec<Objective>) {
        self.prepared_goals = Some(goals);
    }
    pub(super) fn exhausted(&self) -> bool {
        self.remaining == Some(0)
    }
    pub(super) fn add_actions(&mut self, count: usize) {
        *self.remaining.as_mut().expect("prepared action budget") += count;
    }
    pub(super) fn spend_action(&mut self) {
        *self.remaining.as_mut().expect("prepared action budget") -= 1;
    }
    pub(super) fn candidate_shared(&mut self) -> CandidateShared<'_> {
        CandidateShared {
            escort: &self.escort,
            commitments: &mut self.commitments,
            threats: &mut self.threats,
        }
    }
    pub(super) fn escort(&self) -> &crate::ai::policies::escort::travel::Workspace {
        &self.escort
    }
    pub(super) fn focused<'a, 'catalog>(
        &mut self,
        game: &'a BattleView<'a, 'catalog>,
        phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
        turn: &crate::ai::runtime::PlannerTurn<'_>,
        id: u32,
    ) -> Option<Plan> {
        best_plan::<true, true>(
            game,
            phases,
            &turn.orders.tactics,
            &self.escort,
            turn.seed,
            self.prepared_goals.as_deref(),
            &mut self.routes,
            &mut self.commitments,
            &mut Vec::new(),
            &mut self.threats,
            Some(id),
            turn.recruitment,
        )
    }
}

pub(super) fn prepare(
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    session: &mut Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
) {
    session
        .escort
        .prepare(phases, turn.report, &turn.orders.tactics);
}

pub(super) fn initialize(
    game: &BattleView<'_, '_>,
    session: &mut Session,
    turn: &crate::ai::runtime::PlannerTurn<'_>,
) {
    session.remaining.get_or_insert_with(|| {
        game.units()
            .iter()
            .filter(|unit| {
                game.belongs_to_faction(unit, game.observer_faction()) && game.can_act(unit.id)
            })
            .count()
            * 3
    });
    if session.prepared_goals.is_none() {
        session.prepared_goals = Some(individual(
            turn.orders
                .tactics
                .targets
                .iter()
                .filter(|target| !target.id.starts_with("rebuild:"))
                .cloned()
                .collect(),
        ));
    }
}

pub(super) fn choose<'a, 'catalog>(
    game: &'a BattleView<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
    session: &mut Session,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
) -> Result<(), CommandError> {
    initialize(game, session, turn);
    if session.exhausted() {
        *turn.finished = true;
        return Ok(());
    }
    session.spend_action();
    let mut intents = Vec::new();
    let plan = best_plan::<true, false>(
        game,
        phases,
        &turn.orders.tactics,
        &session.escort,
        turn.seed,
        session.prepared_goals.as_deref(),
        &mut session.routes,
        &mut session.commitments,
        &mut intents,
        &mut session.threats,
        None,
        turn.recruitment,
    );
    publish_plan(game, turn, plan, &intents)
}

pub(super) fn publish_plan(
    game: &BattleView<'_, '_>,
    turn: &mut crate::ai::runtime::PlannerTurn<'_>,
    plan: Option<Plan>,
    intents: &[(u32, Option<bool>)],
) -> Result<(), CommandError> {
    for (id, intent) in intents {
        let actor = game.unit(*id).unwrap();
        turn.commands.push_back(intent_command(
            actor,
            actor.retreating,
            *intent,
            actor.attack_spent,
        ));
    }
    if let Some(plan) = plan {
        let actor = game.unit(plan.unit).unwrap();
        let rebuilding = intents
            .iter()
            .find(|(id, _)| *id == plan.unit)
            .map_or(actor.rebuilding, |(_, intent)| *intent);
        if let Some(kind) = plan.advance {
            turn.commands.push_back(advance_command(plan.unit, kind));
        } else {
            if actor.tile != plan.tile {
                turn.commands.push_back(move_command(plan.unit, plan.tile));
                let retreating = plan.withdrawing.then_some(true);
                if actor.retreating != retreating {
                    turn.commands.push_back(intent_command(
                        actor,
                        retreating,
                        rebuilding,
                        actor.attack_spent,
                    ));
                }
            }
            if let (Some(target), Some(weapon)) = (plan.target, plan.weapon) {
                *turn.attack_continuation = Some((
                    UnitId(plan.unit),
                    UnitId(target),
                    game.weapon(actor, weapon).key(),
                ));
            }
        }
    } else {
        *turn.finished = true;
    }
    Ok(())
}
