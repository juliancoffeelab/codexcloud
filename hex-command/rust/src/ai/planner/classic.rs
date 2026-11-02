pub(in crate::ai) mod escort;
use crate::ai::Recorder;
use crate::ai::intent_command;
use crate::ai::objectives::actor_matches;
use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::sort::SharedSort;
use crate::{
    ai::context::{BattleDraft, BattleView},
    ai::objectives::{TileOrder, individual},
    ai::planner::safety::Threats,
    data::*,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
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
    recruitment: HashMap<usize, (i32, i32)>,
    holding: bool,
    orders: &'game crate::ai::orders::TacticalOrders,
    escort: &'game escort::Workspace,
}
impl<'game, 'catalog> ClassicRules<'game, 'catalog> {
    pub fn new(
        g: &'game BattleView<'game, 'catalog>,
        u: &Unit,
        goals: &[Objective],
        _restrained: bool,
        orders: &'game crate::ai::orders::TacticalOrders,
        escort: &'game escort::Workspace,
        opportunity: Option<&crate::ai::recruiter::RecruitmentOpportunity>,
    ) -> Self {
        let important = orders.important(g, u);
        let threats = important.then(|| Threats::protected(g, u.side, orders));
        // Classic owns its movement penalties; only legal command execution is shared.
        let mut recruitment = HashMap::default();
        if let Some(opportunity) = opportunity.filter(|r| r.budget != Some(0)) {
            if u.is_leader() && !u.moved && !u.attacked {
                let destinations = orders.authored_destinations(
                    g,
                    u,
                    g.reachable(u).iter().map(|&(tile, _)| tile).collect(),
                );
                for tile in destinations
                    .into_iter()
                    .filter(|&tile| tile != u.tile && g.is_keep(tile) && g.owns(tile, u.side))
                {
                    let mut draft = g.fork();
                    if draft
                        .execute(crate::ai::move_command(g, u.id, tile))
                        .is_err()
                    {
                        continue;
                    }
                    let Ok((_, report)) = draft.recruit(opportunity) else {
                        continue;
                    };
                    if report.recruited.is_empty() {
                        continue;
                    }
                    let value: i32 = report
                        .recruited
                        .iter()
                        .filter_map(|id| draft.unit(id.0))
                        .map(|unit| draft.recruit_cost(unit, unit.kind))
                        .sum();
                    if let Some(actor) = draft.unit(u.id) {
                        let mut forecast = Threats::protected(&draft, u.side, orders);
                        recruitment.insert(tile, (forecast.at(actor, tile, None), value));
                    }
                }
            }
        }
        let yielding = orders.classic_clearing(g).remove(&u.id).unwrap_or_default();
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
            recruitment,
            holding,
            orders,
            escort,
        }
    }
    fn risk(&mut self, tile: usize, eliminated: Option<u32>) -> i32 {
        let risk = if eliminated.is_none() {
            self.recruitment.get(&tile).map(|&(risk, _)| risk)
        } else {
            None
        }
        .unwrap_or_else(|| {
            self.threats
                .as_mut()
                .map_or(0, |t| t.at(&self.unit, tile, eliminated))
        });
        risk.min(self.unit.hp.max(1))
    }
    fn penalty(&mut self, g: &BattleView, tile: usize) -> f64 {
        let finishing = self.orders.arrives(g, &self.unit, tile);
        -self
            .recruitment
            .get(&tile)
            .map_or(0.0, |&(_, value)| value as f64)
            - self.escort.position(self.unit.id, tile)
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
                let risk = self.risk(tile, None);
                if self.orders.protects(self.unit.id) {
                    risk as f64 * 40.0 / self.unit.hp.max(1) as f64
                        + if risk >= self.unit.hp { 40.0 } else { 0.0 }
                } else {
                    risk as f64 * 12.0 + if risk >= self.unit.hp { 40.0 } else { 0.0 }
                }
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
        let killed = g.exchange(&self.unit, target, weapon, from).death[1] == 1.0;
        if self.orders.protects(self.unit.id) {
            return true;
        }
        if retaliation >= self.unit.hp {
            return false;
        }
        if killed && !self.orders.campaign && target.is_leader() {
            return true;
        }
        let incoming = self.risk(from, (killed).then_some(target.id));
        incoming + retaliation < self.unit.hp
            && (self.unit.role != Some(Role::Courier) || incoming + retaliation == 0)
            && g.allied(u, &self.unit)
    }

    fn attack_value(&mut self, g: &BattleView, target: &Unit, weapon: usize, from: usize) -> f64 {
        let u = &self.unit;
        let exchange = g.exchange(u, target, weapon, from);
        let mut value = g.attack_score(u, target, from, Some(weapon))
            + self.escort.attack(
                u.id,
                target,
                g.preview(u, target, weapon, from).0,
                exchange.death[1],
            );
        if self.orders.protects(u.id) {
            let retaliation = g.maximum_retaliation(u, target, weapon, from);
            let hp = u.hp;
            let before = self.risk(from, None);
            let after =
                self.risk(from, (exchange.death[1] == 1.0).then_some(target.id)) + retaliation;
            // Compare attacking with waiting here, including the following enemy phase.
            value -= (after - before) as f64 * 40.0 / hp.max(1) as f64
                + ((after >= hp) as i32 - (before >= hp) as i32) as f64 * 40.0;
        }
        value
    }
}
impl BattleView<'_, '_> {
    pub fn enemy_danger(&self, u: &Unit, tile: usize) -> i32 {
        let mut target = u.clone();
        target.tile = tile;
        let mut first = 0;
        let mut second = 0;
        for foe in self.units().iter().filter(|o| self.hostile(&u, o)) {
            let damage = self
                .forecast_attack_options(foe, &target, foe.tile)
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
        let forecast = self.exchange(u, target, w, from);
        let retaliation = forecast.damage[1];
        (if target.is_leader() {
            20.0
        } else if target.role == Some(Role::Courier) {
            12.0
        } else {
            0.0
        }) + forecast.death[1] * 12.0
            + (self.rules().units[target.kind].hp - target.hp) as f64 * 0.3
            + forecast.effective[0]
            - retaliation * 0.7
            - self.death_chance(u, target, w, from) * 30.0
    }

    fn movement_goals(&self, u: &Unit, orders: &crate::ai::orders::TacticalOrders) -> Vec<usize> {
        let villages: Vec<_> = (0..self.terrain().len())
            .filter(|&t| self.role_is(t, TerrainRole::Village) && self.captures(t, u.side))
            .collect();
        let goals = if self.rules().units[u.kind].family == Family::Scout && !villages.is_empty() {
            villages
        } else {
            self.units()
                .iter()
                .filter(|o| self.hostile(&u, o))
                .map(|o| o.tile)
                .collect()
        };
        if u.is_leader() {
            if let Some(post) = &orders.leader_post {
                return vec![self.site(&post.0)];
            }
        }
        if !orders.movement_tiles.is_empty() {
            let contested: Vec<_> = orders
                .movement_tiles
                .iter()
                .map(|tile| self.site(&tile.0))
                .filter(|&tile| self.captures(tile, u.side))
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
            .filter(|u| self.belongs_to_phase(&u, self.turn()) && self.can_act(u.id))
            .map(|u| u.id)
            .collect();
        let clearing = self.classic_lanes(objectives);
        let order = |u: &Unit| {
            if u.is_leader() {
                return ClassicOrder::Leader;
            }
            let r = &self.rules().units[u.kind];
            match r.id.as_str() {
                "scout" | "wolf" => ClassicOrder::Scout,
                "archer" | "thorncaller" | "bonearcher" | "adept" => ClassicOrder::Ranged,
                "soldier" | "skeleton" | "warden" | "boar" => ClassicOrder::Frontline,
                _ => match r.family {
                    Family::Captives | Family::Scout | Family::Wolf | Family::Ghost => {
                        ClassicOrder::Scout
                    }
                    Family::Archer | Family::Thorncaller | Family::Bonearcher | Family::Adept => {
                        ClassicOrder::Ranged
                    }
                    Family::Skeleton | Family::Boar => ClassicOrder::Frontline,
                    Family::Commander
                    | Family::ZombieWalker
                    | Family::ZombieDecayCaster
                    | Family::Wildkeeper
                    | Family::Necromancer
                    | Family::Bear
                    | Family::GiantBat
                    | Family::GiantSpider
                    | Family::Harpooner
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
                        if r.range > 1 {
                            ClassicOrder::Ranged
                        } else {
                            ClassicOrder::Frontline
                        }
                    }
                },
            }
        };
        ids.order_by_key(|&id| order(self.unit(id).unwrap()));
        ids.order_by_key(|&id| {
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
        if !self.can_act(id) {
            return false;
        }
        let Some(u) = self.unit(id).cloned() else {
            return false;
        };
        let order = TileOrder::new(self, &u, false);
        let mut best: Option<(&Unit, usize, f64)> = None;
        for target in self.units().iter().filter(|t| self.hostile(&u, t)) {
            for weapon in self.attack_options(&u, target, u.tile) {
                if !rules.attack(self, &u, target, weapon, u.tile)
                    || (!desperate
                        && !rules.orders.protects(id)
                        && self.preview(&u, target, weapon, u.tile).1 >= u.hp)
                {
                    continue;
                }
                let score = rules.attack_value(self, target, weapon, u.tile);
                if rules.orders.protects(id) && score <= 0.0 {
                    continue;
                }
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
        let enemies: Vec<_> = self
            .units()
            .iter()
            .filter(|o| self.hostile(&u, o))
            .collect();
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
                self.role_is(t, TerrainRole::Village)
                    && self.owns(t, u.side)
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
        destinations.order_by(|&a, &b| {
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
        if !self.belongs_to_phase(&u, self.turn()) || u.attacked {
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
            .filter(|&t| {
                !u.is_leader()
                    || !rebuild.is_empty()
                    || self.leader_may_advance(&u, t, recruiting, required)
                    || (!campaign
                        && self.units().iter().any(|target| {
                            self.hostile(&u, target)
                                && target.is_leader()
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
                    if let Some(weapon) = self.best_attack(&u, target, t) {
                        opportunity = opportunity
                            .max(5.0 + rules.attack_value(self, target, weapon, t) / 5.0);
                    }
                }
            }
            let capture = if self.role_is(t, TerrainRole::Village) && self.captures(t, u.side) {
                if self.rules().units[u.kind].family == Family::Scout {
                    8.0
                } else {
                    5.0
                }
            } else {
                0.0
            };
            let heal = if u.hp <= 4 && self.role_is(t, TerrainRole::Village) && self.owns(t, u.side)
            {
                9.0
            } else {
                0.0
            };
            scores[t] = -distance
                + opportunity
                + capture
                + heal
                + if self.role_is(t, TerrainRole::Hill) {
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
        dest.order_by(|&a, &b| {
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
        escort: &escort::Workspace,
        restrained: bool,
        opportunity: Option<&crate::ai::recruiter::RecruitmentOpportunity>,
        recorder: &mut Recorder,
    ) {
        if self.winner().is_some() {
            return;
        }
        if !self.can_act(id) {
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
                        || g.tiles.iter().any(|t| self.captures(self.site(t), u.side)))
            })
            .collect();
        if eligible.iter().any(|g| self.assigned_duty(&u, g)) {
            eligible.retain(|g| self.assigned_duty(&u, g));
        }
        eligible.order_by(|a, b| {
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
        let clearing = orders.classic_clearing(self).remove(&u.id);
        let yielding = clearing.is_some();
        let movement = if let Some(lane) = clearing {
            let available: Vec<_> = self
                .reachable(&u)
                .iter()
                .copied()
                .map(|(tile, _)| tile)
                .filter(|tile| !reserved.contains(tile))
                .collect();
            let aside: Vec<_> = available
                .iter()
                .copied()
                .filter(|tile| !lane.contains(tile))
                .collect();
            // A packed corridor may have no side pocket. Keep the yielding
            // penalty, but allow moving along the corridor to free this tile.
            Some(if aside.is_empty() { available } else { aside })
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
        let rules = ClassicRules::new(
            source,
            &u,
            objectives,
            restrained,
            orders,
            escort,
            opportunity,
        );
        self.classic_act(id, movement, required, rules, recorder);
    }
}

pub(super) struct Classic;
impl super::Planner for Classic {
    fn recruits_after_move(&self) -> bool {
        true
    }
    fn plan(
        &mut self,
        context: &mut crate::ai::context::TacticalContext<'_, '_>,
        orders: &crate::ai::orders::TacticalOrders,
    ) -> Result<crate::ai::orders::PlanningReport, crate::data::CommandError> {
        super::drive(super::PlannerKind::Classic, context, orders)
    }
}

pub(super) fn prepare(game: &BattleView<'_, '_>, workspace: &mut crate::ai::PlanningWorkspace) {
    workspace.classic_escort.prepare(game, &workspace.report);
}

pub(super) fn choose(
    game: &BattleView<'_, '_>,
    workspace: &mut crate::ai::PlanningWorkspace,
) -> Result<(), CommandError> {
    let orders = &workspace.orders.as_ref().expect("campaign orders").tactics;
    let restrained = orders.restrained;
    if workspace.classic_revision != workspace.cooperation.revision
        && workspace.tactical_remaining.is_some()
    {
        workspace.classic_objectives = orders
            .targets
            .iter()
            .filter(|g| !g.id.starts_with("rebuild:"))
            .cloned()
            .collect();
        workspace
            .classic_objectives
            .extend(orders.rebuild_objectives(game));
        workspace.classic_revision = workspace.cooperation.revision;
    }
    if workspace.tactical_remaining.is_none() {
        workspace.classic_objectives = orders
            .targets
            .iter()
            .filter(|g| !g.id.starts_with("rebuild:"))
            .cloned()
            .collect();
        workspace
            .classic_objectives
            .extend(orders.rebuild_objectives(game));
        workspace.classic_revision = workspace.cooperation.revision;
        if workspace.goals.is_none() && (!orders.campaign || game.turn() == Side::Enemy) {
            workspace
                .classic_objectives
                .retain(|g| g.id.starts_with("rebuild:") || g.id.starts_with("group:"));
        }
        workspace.classic_ids = game.classic_order(&workspace.classic_objectives).into();
        workspace
            .classic_ids
            .make_contiguous()
            .order_by_key(|id| workspace.classic_escort.bound(*id) == 0.0);
        workspace.tactical_remaining = Some(workspace.classic_ids.len());
        for unit in game.units().iter().filter(|u| {
            game.belongs_to_phase(u, game.turn()) && game.can_act(u.id) && u.is_leader()
        }) {
            let required = orders.moving_actor(unit.id);
            let (_, intent) = orders.rebuild_plan(game, unit.id, required);
            if unit.rebuilding != intent {
                workspace.commands.push_back(intent_command(
                    unit,
                    unit.retreating,
                    intent,
                    unit.attacked,
                ));
            }
        }
        if !workspace.commands.is_empty() {
            return Ok(());
        }
    }
    if let Some(id) = workspace.classic_ids.pop_front() {
        let mut virtual_game = game.fork();
        let mut recorder = Recorder::new(game);
        virtual_game.classic_choose(
            game,
            id,
            &workspace.classic_objectives,
            orders,
            &workspace.classic_escort,
            restrained,
            workspace.recruitment_opportunity.as_ref(),
            &mut recorder,
        );
        recorder.flush_intents(&virtual_game);
        if let Some(error) = recorder.error {
            return Err(error);
        }
        workspace.commands.extend(recorder.commands);
    } else {
        workspace.finished = true;
    }
    Ok(())
}

impl BattleView<'_, '_> {
    fn classic_lanes(&self, goals: &[Objective]) -> HashMap<u32, HashSet<usize>> {
        let mut lanes: HashMap<u32, HashSet<usize>> = HashMap::default();
        fn required_corridor(goal: &Objective) -> bool {
            match goal.kind {
                ObjectiveKind::Visit | ObjectiveKind::Capture => {
                    goal.id.starts_with("required:") && goal.hold != Some(true)
                }
                ObjectiveKind::Defeat => false,
            }
        }
        let assigned = goals.iter().any(crate::ai::objectives::actor_route);
        for goal in goals
            .iter()
            .filter(|g| crate::ai::objectives::actor_route(g) || !assigned && required_corridor(g))
        {
            let Some(actor) = self
                .units()
                .iter()
                .filter(|u| {
                    self.belongs_to_phase(u, self.turn())
                        && (goal.actor.is_some()
                            || goal.family.is_some()
                            || !u.is_leader() && u.role != Some(Role::Courier))
                        && if goal.actor.is_some() {
                            actor_matches(&goal.actor, u.id)
                        } else {
                            self.can_pursue(u, goal)
                        }
                })
                .min_by_key(|u| {
                    let targets: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
                    self.routes(u.kind, &targets)[u.tile]
                })
            else {
                continue;
            };
            let targets: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
            if targets.contains(&actor.tile) {
                continue;
            }
            let mut penalty = 2;
            let mut route = self.occupied_routes(actor, &targets, true);
            let available = if actor.moved {
                0
            } else {
                self.rules().units[actor.kind].r#move as u16
            };
            if route[actor.tile] > available {
                let short = self.occupied_routes_with_penalty(actor, &targets, Some(0));
                let arrival_cost = targets
                    .iter()
                    .map(|&tile| self.cost(actor.kind, tile))
                    .min()
                    .unwrap_or(0);
                if short[actor.tile] <= available.saturating_add(arrival_cost) {
                    route = short;
                    penalty = 0;
                }
            }
            if route[actor.tile] == u16::MAX {
                continue;
            }
            // Equal route values are valid on zero-cost terrain. Search admissible
            // edges with visited parents, rather than requiring strict descent.
            let mut parents = vec![usize::MAX; self.terrain().len()];
            parents[actor.tile] = actor.tile;
            let mut pending = std::collections::VecDeque::from([actor.tile]);
            let mut endpoint = None;
            while let Some(current) = pending.pop_front() {
                if targets.contains(&current) {
                    endpoint = Some(current);
                    break;
                }
                let mut next: Vec<_> =
                    self.board().adjacent[current]
                        .iter()
                        .copied()
                        .filter(|&tile| {
                            if parents[tile] != usize::MAX || route[tile] == u16::MAX {
                                return false;
                            }
                            let occupied = self
                                .at(tile)
                                .is_some_and(|u| u.id != actor.id && self.allied(actor, u));
                            let entering = self
                                .cost(actor.kind, tile)
                                .saturating_add(if occupied { penalty } else { 0 });
                            route[tile].saturating_add(entering) <= route[current]
                        })
                        .collect();
                next.order_by_key(|&tile| route[tile].saturating_add(self.cost(actor.kind, tile)));
                for tile in next {
                    parents[tile] = current;
                    pending.push_back(tile);
                }
            }
            let Some(mut current) = endpoint else {
                continue;
            };
            let mut path = vec![current];
            while current != actor.tile {
                current = parents[current];
                path.push(current);
            }
            path.reverse();
            let mut spent = 0;
            for &tile in path.iter().skip(1) {
                spent += self.cost(actor.kind, tile) as usize;
                if spent > self.rules().units[actor.kind].r#move {
                    break;
                }
                if let Some(blocker) = self
                    .at(tile)
                    .filter(|u| self.allied(actor, u) && u.id != actor.id)
                {
                    lanes
                        .entry(blocker.id)
                        .or_default()
                        .extend(path.iter().copied());
                }
            }
        }
        lanes
    }
}

impl crate::ai::orders::TacticalOrders {
    fn classic_clearing(&self, game: &BattleView<'_, '_>) -> HashMap<u32, HashSet<usize>> {
        game.classic_lanes(&self.passage_targets())
    }
}
