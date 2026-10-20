use crate::ai::Recorder;
use crate::ai::objectives::actor_matches;
use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::{
    ai::context::{BattleDraft, BattleView},
    ai::objectives::{TileOrder, individual},
    ai::planner::safety::Threats,
    data::*,
};
use rustc_hash::FxHashSet as HashSet;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Retreat {
    Idle,
    Retreated,
    Trapped,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ClassicOrder {
    Scout,
    Ranged,
    Frontline,
    Leader,
}
pub(in crate::ai) struct ClassicRules<'game, 'catalog> {
    pub unit: Unit,
    pub goals: Vec<Objective>,
    reserved: HashSet<usize>,
    yielding: HashSet<usize>,
    lanes: HashSet<usize>,
    important: bool,
    threats: Option<Threats<'game, 'catalog>>,
    minimum: i32,
    holding: bool,
    orders: &'game crate::ai::orders::TacticalOrders,
    escort: &'game crate::ai::escort::Workspace,
}
impl<'game, 'catalog> ClassicRules<'game, 'catalog> {
    pub fn new(
        g: &'game BattleView<'game, 'catalog>,
        u: &Unit,
        goals: &[Objective],
        _restrained: bool,
        orders: &'game crate::ai::orders::TacticalOrders,
        escort: &'game crate::ai::escort::Workspace,
    ) -> Self {
        let important = orders.important(g, u);
        let mut threats = important.then(|| Threats::protected(g, u.side, orders));
        let minimum = if let Some(t) = &mut threats {
            let current = t.at(u, u.tile, None);
            if current == 0 {
                0
            } else {
                current.min(t.minimum(u, g.reachable(u).iter().copied().map(|(tile, _)| tile)))
            }
        } else {
            0
        };
        let yielding = orders.clearing(g).remove(&u.id).unwrap_or_default();
        let holding = !yielding.contains(&u.tile)
            && goals.iter().any(|o| {
                o.hold == Some(true)
                    && actor_matches(&o.actor, u.id)
                    && o.tiles.contains(&g.board().ids[u.tile])
            });
        Self {
            unit: u.clone(),
            goals: goals.to_vec(),
            reserved: g.reserved(u, goals),
            yielding,
            lanes: orders.lanes(g),
            important,
            threats,
            minimum,
            holding,
            orders,
            escort,
        }
    }
    fn risk(&mut self, tile: usize, eliminated: Option<u32>) -> i32 {
        self.threats
            .as_mut()
            .map_or(0, |t| t.at(&self.unit, tile, eliminated))
    }
    fn penalty(&mut self, g: &BattleView, tile: usize) -> f64 {
        let finishing = self.orders.arrives(g, &self.unit, tile);
        -self.escort.position(self.unit.id, tile)
            + (if self.reserved.contains(&tile) {
                160.0
            } else {
                0.0
            })
            + (if self.yielding.contains(&tile) {
                80.0
            } else {
                0.0
            })
            + (if self.lanes.contains(&tile) {
                100.0
            } else {
                0.0
            })
            + (if self.holding && tile != self.unit.tile {
                100.0
            } else {
                0.0
            })
            + if finishing {
                -1000.0
            } else {
                self.risk(tile, None) as f64 * 12.0
            }
    }
    fn movement(&mut self, g: &BattleView, tile: usize) -> bool {
        !self.important || self.orders.arrives(g, &self.unit, tile) || {
            let risk = self.risk(tile, None);
            risk < self.unit.hp || risk <= self.minimum
        }
    }
    fn attack(
        &mut self,
        g: &BattleView,
        u: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> bool {
        if !self
            .orders
            .attack_allowed(g, &self.unit, target, weapon, from)
        {
            return false;
        }
        if !self.important {
            return true;
        }
        let retaliation = g.maximum_retaliation(&self.unit, target, weapon, from);
        let killed = g.guaranteed_kill(&self.unit, target, weapon);
        if self.orders.protects(self.unit.id) && retaliation > 0 {
            return false;
        }
        if retaliation >= self.unit.hp {
            return false;
        }
        if killed && !self.orders.campaign && g.rules().units[target.kind].leader {
            return true;
        }
        let incoming = self.risk(from, (killed).then_some(target.id));
        incoming + retaliation < self.unit.hp
            && (self.unit.role != Some(Role::Courier) || incoming + retaliation == 0)
            && u.side == self.unit.side
    }
}
impl BattleView<'_, '_> {
    pub fn enemy_danger(&self, u: &Unit, tile: usize) -> i32 {
        let mut target = u.clone();
        target.tile = tile;
        let mut first = 0;
        let mut second = 0;
        for foe in self.units().iter().filter(|o| o.side != u.side) {
            let damage = self
                .attack_options(foe, &target, foe.tile)
                .iter()
                .map(|&w| self.damage(foe, &target, w))
                .max()
                .unwrap_or(0);
            if damage > first {
                second = first;
                first = damage;
            } else if damage > second {
                second = damage;
            }
        }
        first + second
    }
    fn attack_score(&self, u: &Unit, target: &Unit, from: usize, weapon: Option<usize>) -> f64 {
        let Some(w) = weapon.or_else(|| self.best_attack(u, target, from)) else {
            return -100.0;
        };
        let (_, retaliation, _) = self.preview(u, target, w, from);
        (if self.rules().units[target.kind].leader {
            20.0
        } else if target.role == Some(Role::Courier) {
            12.0
        } else {
            0.0
        }) + self.kill_chance(u, target, w) * 12.0
            + (self.rules().units[target.kind].hp - target.hp) as f64 * 0.3
            + self.effective_damage(u, target, w)
            - retaliation as f64 * 0.7
            - self.death_chance(u, target, w, from) * 30.0
    }

    fn movement_goals(&self, u: &Unit, orders: &crate::ai::orders::TacticalOrders) -> Vec<usize> {
        let villages: Vec<_> = (0..self.terrain().len())
            .filter(|&t| self.terrain_is(t, Terrain::Village) && self.owners()[t] != Some(u.side))
            .collect();
        let goals = if self.rules().units[u.kind].family == Family::Scout && !villages.is_empty() {
            villages
        } else {
            self.units()
                .iter()
                .filter(|o| o.side != u.side)
                .map(|o| o.tile)
                .collect()
        };
        if self.rules().units[u.kind].leader {
            if let Some(post) = &orders.leader_post {
                return vec![self.site(&post.0)];
            }
        }
        if !orders.movement_tiles.is_empty() {
            let contested: Vec<_> = orders
                .movement_tiles
                .iter()
                .map(|tile| self.site(&tile.0))
                .filter(|&tile| self.owners()[tile] != Some(u.side))
                .collect();
            if !contested.is_empty() {
                return contested;
            }
            if !orders.defensive_tiles.is_empty() {
                return orders
                    .movement_tiles
                    .iter()
                    .map(|tile| self.site(&tile.0))
                    .collect();
            }
        }
        goals
    }

    pub(in crate::ai) fn classic_order(&self, objectives: &[Objective]) -> Vec<u32> {
        let mut ids: Vec<_> = self
            .units()
            .iter()
            .filter(|u| u.side == self.turn())
            .map(|u| u.id)
            .collect();
        let clearing = self.actor_lanes(objectives);
        let order = |u: &Unit| {
            let r = &self.rules().units[u.kind];
            match r.id.as_str() {
                "scout" | "wolf" => ClassicOrder::Scout,
                "archer" | "thorncaller" | "bonearcher" | "adept" => ClassicOrder::Ranged,
                "soldier" | "skeleton" | "warden" | "boar" => ClassicOrder::Frontline,
                "commander" | "wildkeeper" | "necromancer" => ClassicOrder::Leader,
                _ => match r.family {
                    Family::Scout | Family::Wolf => ClassicOrder::Scout,
                    Family::Archer | Family::Thorncaller | Family::Bonearcher | Family::Adept => {
                        ClassicOrder::Ranged
                    }
                    Family::Skeleton | Family::Boar => ClassicOrder::Frontline,
                    Family::Commander | Family::Wildkeeper | Family::Necromancer => {
                        ClassicOrder::Leader
                    }
                    Family::Harpooner
                    | Family::HorseArcher
                    | Family::SkeletonRider
                    | Family::SkeletonAxeman
                    | Family::FoxSpearguard
                    | Family::Treant
                    | Family::MerfolkMacer
                    | Family::Reefguard
                    | Family::Rider
                    | Family::SkeletonMage
                    | Family::Spearman
                    | Family::Swordsman
                    | Family::Tidekeeper
                    | Family::Wavemender => {
                        if r.leader {
                            ClassicOrder::Leader
                        } else if r.range > 1 {
                            ClassicOrder::Ranged
                        } else {
                            ClassicOrder::Frontline
                        }
                    }
                },
            }
        };
        ids.sort_by_key(|&id| order(self.unit(id).unwrap()));
        ids.sort_by_key(|&id| {
            let u = self.unit(id).unwrap();
            !self.reserved(u, &objectives).contains(&u.tile) && !clearing.contains_key(&id)
        });
        ids
    }
}

impl BattleDraft<'_, '_> {
    fn try_attack(
        &mut self,
        id: u32,
        desperate: bool,
        rules: &mut ClassicRules<'_, '_>,
        recorder: &mut Recorder,
    ) -> bool {
        let Some(u) = self.unit(id).cloned() else {
            return false;
        };
        let order = TileOrder::new(self, &u, false);
        let mut best: Option<(&Unit, usize, f64)> = None;
        for target in self.units().iter().filter(|t| t.side != u.side) {
            for weapon in self.attack_options(&u, target, u.tile) {
                if !rules.attack(self, &u, target, weapon, u.tile)
                    || (!desperate && self.preview(&u, target, weapon, u.tile).1 >= u.hp)
                {
                    continue;
                }
                let score = self.attack_score(&u, target, u.tile, Some(weapon))
                    + rules.escort.attack(
                        id,
                        target,
                        self.preview(&u, target, weapon, u.tile).0,
                        self.kill_chance(&u, target, weapon),
                    );
                if best.as_ref().is_none_or(|(b, _, s)| {
                    score > *s || (score == *s && order.compare(self, target.tile, b.tile).is_lt())
                }) {
                    best = Some((target, weapon, score));
                }
            }
        }
        if let Some((target, weapon, _)) = best {
            recorder.attack(self, id, target.id, weapon);
            true
        } else {
            false
        }
    }
    fn retreat(
        &mut self,
        id: u32,
        rules: &mut ClassicRules<'_, '_>,
        recorder: &mut Recorder,
    ) -> Retreat {
        let u = self.unit(id).unwrap().clone();
        let enemies: Vec<_> = self.units().iter().filter(|o| o.side != u.side).collect();
        let retaliations: Vec<_> = enemies
            .iter()
            .flat_map(|target| {
                self.attack_options(&u, target, u.tile)
                    .into_iter()
                    .map(|w| self.preview(&u, target, w, u.tile).1)
            })
            .collect();
        if enemies.is_empty() || retaliations.iter().any(|&d| d < u.hp) {
            self.units_mut()
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .retreating = None;
            return Retreat::Idle;
        }
        let trapped = !retaliations.is_empty();
        let villages: Vec<_> = (0..self.terrain().len())
            .filter(|&t| {
                self.terrain_is(t, Terrain::Village)
                    && self.owners()[t] == Some(u.side)
                    && self.at(t).is_none_or(|o| o.id == id)
                    && self.enemy_danger(&u, t) == 0
            })
            .collect();
        let routes = (!villages.is_empty()).then(|| self.routes(u.kind, &villages));
        let recovering = u.retreating == Some(true)
            && u.hp < self.rules().units[u.kind].hp
            && routes.as_ref().is_some_and(|r| r[u.tile] != u16::MAX);
        if !trapped && !recovering {
            drop(routes);
            self.units_mut()
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .retreating = None;
            return Retreat::Idle;
        }
        if recovering && villages.contains(&u.tile) {
            drop(routes);
            self.units_mut()
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .attacked = true;
            return Retreat::Retreated;
        }
        let mut destinations: Vec<_> = std::iter::once(u.tile)
            .chain(self.reachable(&u).iter().copied().map(|(t, _)| t))
            .filter(|&t| rules.movement(self, t))
            .collect();
        let mut scores = vec![f64::NEG_INFINITY; self.terrain().len()];
        for &t in &destinations {
            let separation = enemies
                .iter()
                .map(|e| self.board().distance(t, e.tile))
                .min()
                .unwrap()
                .min(8);
            scores[t] = -self.enemy_danger(&u, t) as f64 * 20.0
                + separation as f64 * 2.0
                + if villages.contains(&t) { 20.0 } else { 0.0 }
                - routes
                    .as_ref()
                    .map_or(0.0, |r| if r[t] == u16::MAX { 0.0 } else { r[t] as f64 })
                - rules.penalty(self, t);
        }
        drop(routes);
        let order = TileOrder::new(self, &u, true);
        destinations.sort_by(|&a, &b| {
            scores[b]
                .total_cmp(&scores[a])
                .then_with(|| order.compare(self, a, b))
        });
        if let Some(&t) = destinations.first() {
            if t != u.tile && scores[u.tile] != f64::NEG_INFINITY && scores[t] > scores[u.tile] {
                recorder.move_unit(self, id, t);
                let index = self.index(id).unwrap();
                self.units_mut()[index].retreating = Some(true);
                self.units_mut()[index].attacked = true;
                recorder.mark_retreat(id);
                self.record(format!(
                    "{} is retreating.",
                    u.name
                        .unwrap_or_else(|| self.rules().units[u.kind].name.clone())
                ));
                return Retreat::Retreated;
            }
        }
        self.units_mut()
            .iter_mut()
            .find(|unit| unit.id == id)
            .unwrap()
            .retreating = None;
        if trapped {
            Retreat::Trapped
        } else {
            Retreat::Idle
        }
    }
    pub(in crate::ai) fn classic_act(
        &mut self,
        id: u32,
        movement: Option<Vec<usize>>,
        required: bool,
        mut rules: ClassicRules<'_, '_>,
        recorder: &mut Recorder,
    ) {
        if self.winner().is_some() {
            return;
        }
        let Some(mut u) = self.unit(id).cloned() else {
            return;
        };
        if u.side != self.turn() || u.attacked {
            return;
        }
        if let Some(promotion) = rules.orders.promotion(self, &u) {
            recorder.advance(self, id, promotion);
            return;
        }
        let (rebuild, intent) = rules.orders.rebuild_plan(self, id, required);
        let index = self.index(id).unwrap();
        self.units_mut()[index].rebuilding = intent;
        u = self.unit(id).unwrap().clone();
        let retreat = self.retreat(id, &mut rules, recorder);
        if retreat == Retreat::Retreated {
            return;
        }
        if rebuild.is_empty()
            && rules.penalty(self, u.tile) < 100.0
            && self.try_attack(id, retreat == Retreat::Trapped, &mut rules, recorder)
        {
            return;
        }
        let goals = if !rebuild.is_empty() {
            rebuild.clone()
        } else {
            movement.unwrap_or_else(|| self.movement_goals(&u, rules.orders))
        };
        if goals.is_empty() {
            return;
        }
        let assigned = rules.goals.iter().any(|g| {
            (actor_matches(&g.actor, id) || g.family.is_some() && self.can_pursue(&u, g))
                && crate::ai::objectives::actor_route(g)
        });
        let routes = if assigned {
            Some(std::rc::Rc::new(self.occupied_routes(&u, &goals, false)))
        } else {
            (self.map_id() == "river" || rules.orders.campaign || !rebuild.is_empty())
                .then(|| self.routes(u.kind, &goals))
        };
        let recruiting = self
            .recruitment_options(u.side)
            .iter()
            .any(|&k| !self.placement_tiles(&u, k, true).is_empty());
        let campaign = rules.orders.campaign;
        let mut dest: Vec<_> = std::iter::once(u.tile)
            .chain(self.reachable(&u).iter().copied().map(|(t, _)| t))
            .filter(|&t| rules.movement(self, t))
            .filter(|&t| {
                !self.rules().units[u.kind].leader
                    || !rebuild.is_empty()
                    || self.leader_may_advance(&u, t, recruiting, required)
                    || (!campaign
                        && self.units().iter().any(|target| {
                            target.side != u.side
                                && self.rules().units[target.kind].leader
                                && self
                                    .attack_options(&u, target, t)
                                    .iter()
                                    .any(|&w| self.preview(&u, target, w, t).0 >= target.hp)
                        }))
            })
            .collect();
        dest = rules.orders.authored_destinations(self, &u, dest);
        let mut scores = vec![f64::NEG_INFINITY; self.terrain().len()];
        for &t in &dest {
            let distance = routes.as_ref().map_or_else(
                || {
                    goals
                        .iter()
                        .map(|&g| self.board().distance(t, g))
                        .min()
                        .unwrap() as f64
                },
                |r| {
                    if r[t] == u16::MAX {
                        1000.0
                    } else {
                        r[t] as f64
                    }
                },
            );
            let mut opportunity: f64 = 0.0;
            for target in self.units() {
                if self
                    .attack_options(&u, target, t)
                    .into_iter()
                    .any(|w| rules.attack(self, &u, target, w, t))
                {
                    let escort_bonus = self.best_attack(&u, target, t).map_or(0.0, |weapon| {
                        rules.escort.attack(
                            id,
                            target,
                            self.preview(&u, target, weapon, t).0,
                            self.kill_chance(&u, target, weapon),
                        )
                    });
                    opportunity = opportunity
                        .max(5.0 + (self.attack_score(&u, target, t, None) + escort_bonus) / 5.0);
                }
            }
            let capture =
                if self.terrain_is(t, Terrain::Village) && self.owners()[t] != Some(u.side) {
                    if self.rules().units[u.kind].family == Family::Scout {
                        8.0
                    } else {
                        5.0
                    }
                } else {
                    0.0
                };
            let heal = if u.hp <= 4
                && self.terrain_is(t, Terrain::Village)
                && self.owners()[t] == Some(u.side)
            {
                9.0
            } else {
                0.0
            };
            scores[t] = -distance
                + opportunity
                + capture
                + heal
                + if self.terrain_is(t, Terrain::Hill) {
                    0.25
                } else {
                    0.0
                }
                - self.enemy_danger(&u, t) as f64 * 0.5
                + if rebuild.contains(&t) { 40.0 } else { 0.0 }
                - rules.penalty(self, t);
        }
        drop(routes);
        let order = TileOrder::new(self, &u, true);
        dest.sort_by(|&a, &b| {
            scores[b]
                .total_cmp(&scores[a])
                .then_with(|| order.compare(self, a, b))
        });
        if let Some(&t) = dest.first() {
            if t != u.tile {
                recorder.move_unit(self, id, t);
            }
            if self.winner().is_none() {
                self.try_attack(id, false, &mut rules, recorder);
            }
        }
    }
    pub(in crate::ai) fn classic_choose(
        &mut self,
        source: &BattleView<'_, '_>,
        id: u32,
        objectives: &[Objective],
        orders: &crate::ai::orders::TacticalOrders,
        escort: &crate::ai::escort::Workspace,
        restrained: bool,
        recorder: &mut Recorder,
    ) {
        if self.winner().is_some() {
            return;
        }
        let Some(u) = self.unit(id).cloned() else {
            return;
        };
        let individual = individual(objectives.to_vec());
        let tile_order = TileOrder::new(self, &u, true);
        let nearest = |g: &Objective| {
            g.tiles
                .iter()
                .map(|t| self.site(t))
                .min_by(|&a, &b| {
                    self.board()
                        .distance(u.tile, a)
                        .cmp(&self.board().distance(u.tile, b))
                        .then_with(|| tile_order.compare(self, a, b))
                })
                .unwrap()
        };
        let mut eligible: Vec<_> = individual
            .iter()
            .filter(|g| {
                self.can_pursue(&u, g)
                    && (g.kind != ObjectiveKind::Capture
                        || g.hold == Some(true)
                        || g.tiles
                            .iter()
                            .any(|t| self.owners()[self.site(t)] != Some(u.side)))
            })
            .collect();
        if eligible.iter().any(|g| self.assigned_duty(&u, g)) {
            eligible.retain(|g| self.assigned_duty(&u, g));
        }
        eligible.sort_by(|a, b| {
            let score = |g: &Objective| {
                g.tiles
                    .iter()
                    .map(|t| self.board().distance(u.tile, self.site(t)))
                    .min()
                    .unwrap() as f64
                    / g.weight
            };
            score(a)
                .total_cmp(&score(b))
                .then_with(|| tile_order.compare(self, nearest(a), nearest(b)))
        });
        let best = eligible.first().copied();
        let reserved = self.reserved(&u, objectives);
        let clearing = orders.clearing(self).remove(&u.id);
        let yielding = clearing.is_some();
        let movement = if let Some(lane) = clearing {
            Some(
                self.reachable(&u)
                    .iter()
                    .copied()
                    .map(|(tile, _)| tile)
                    .filter(|tile| !lane.contains(tile) && !reserved.contains(tile))
                    .collect(),
            )
        } else if reserved.contains(&u.tile) {
            Some(
                self.board().adjacent[u.tile]
                    .iter()
                    .copied()
                    .filter(|t| !reserved.contains(t))
                    .collect(),
            )
        } else {
            best.map(|g| g.tiles.iter().map(|t| self.site(t)).collect())
        };
        let required = orders.moving_actor(id) || yielding;
        let rules = ClassicRules::new(source, &u, objectives, restrained, orders, escort);
        self.classic_act(id, movement, required, rules, recorder);
    }
}

pub(super) struct Classic;
impl super::Planner for Classic {
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, crate::data::CommandError> {
        super::drive(super::PlannerKind::Classic, context, orders)
    }
}
