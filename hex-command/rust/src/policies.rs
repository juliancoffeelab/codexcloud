use crate::{
    engine::Game,
    model::*,
    objectives::{TileOrder, individual, objective},
    threats::Threats,
};
use std::collections::HashMap;
pub struct Random(pub u32);
impl Random {
    pub fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut v = (self.0 ^ (self.0 >> 15)).wrapping_mul(1 | self.0);
        v ^= v.wrapping_add((v ^ (v >> 7)).wrapping_mul(61 | v));
        (v ^ (v >> 14)) as f64 / 4294967296.0
    }
}
fn jitter(seed: u32, key: &str) -> f64 {
    let mut h = seed;
    for c in key.encode_utf16() {
        h = (h ^ c as u32).wrapping_mul(16777619);
    }
    h as f64 / 4294967296.0 * 0.05
}
fn role(g: &Game, k: usize) -> usize {
    let r = &g.rules.units[k];
    if r.healing > 0 {
        2
    } else if r.range > 1 {
        1
    } else {
        0
    }
}
fn recruit_role(g: &Game, k: usize) -> usize {
    let r = &g.rules.units[k];
    if r.healing > 0 {
        3
    } else if r.r#move >= 5 && r.attacks[0].category == Category::Melee {
        1
    } else if r.range > 1 {
        2
    } else {
        0
    }
}
#[derive(Clone)]
struct Plan {
    unit: u32,
    tile: usize,
    score: f64,
    target: Option<u32>,
    weapon: Option<usize>,
    advance: Option<usize>,
    withdrawing: bool,
}
impl Game {
    pub fn policy_recruit(
        &mut self,
        policy: &str,
        seed: u32,
        limit: usize,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) {
        let side = self.turn;
        let goals = goals.map_or_else(|| self.objectives(side), |g| g.to_vec());
        let lanes = self.withdrawal_lanes(side, restrained);
        for _ in 0..limit {
            let Some(leader) = self
                .units
                .iter()
                .find(|u| u.side == side && self.rules.units[u.kind].leader)
                .cloned()
            else {
                return;
            };
            let pool = self.recruitment_options(side);
            let placement_allowed = |kind, tile| {
                let u = Unit {
                    id: 0,
                    kind,
                    tile,
                    side,
                    hp: self.rules.units[kind].hp,
                    moved: true,
                    attacked: true,
                    experience: None,
                    name: None,
                    role: None,
                    retreating: None,
                    rebuilding: None,
                };
                !self.reserved(&u, &goals).contains(&tile) && !lanes.contains(&tile)
            };
            if policy == "classic" {
                if pool.is_empty() {
                    continue;
                }
                let archer = self.kind("archer");
                let swordsman = self.kind("swordsman");
                let kind = if self.campaign.is_some()
                    || !pool.contains(&archer)
                    || !pool.contains(&swordsman)
                {
                    pool[(self.round as usize - 1) % pool.len()]
                } else if self
                    .units
                    .iter()
                    .any(|u| u.side == side && self.rules.units[u.kind].family == "archer")
                {
                    swordsman
                } else {
                    archer
                };
                let mut tiles: Vec<_> = self
                    .placement_tiles(&leader, kind, true)
                    .into_iter()
                    .filter(|&t| placement_allowed(kind, t))
                    .collect();
                let order = TileOrder::new(self, &leader, false);
                tiles.sort_by(|&a, &b| order.compare(self, a, b));
                if let Some(&tile) = tiles.first() {
                    self.recruit(leader.id, kind, tile);
                }
                continue;
            }
            let allies: Vec<_> = self
                .units
                .iter()
                .filter(|u| u.side == side && !self.rules.units[u.kind].leader)
                .collect();
            let opponents: Vec<_> = self.units.iter().filter(|u| u.side != side).collect();
            let mut counts = [0; 4];
            for ally in &allies {
                counts[recruit_role(self, ally.kind)] += 1;
            }
            let shares = [0.35, 0.15, 0.35, 0.15];
            let total_share: f64 = (0..4)
                .filter(|&r| pool.iter().any(|&k| recruit_role(self, k) == r))
                .map(|r| shares[r])
                .sum();
            let mut choices = vec![];
            for kind in pool {
                if policy == "rush" && role(self, kind) != 0 {
                    continue;
                }
                if !self
                    .placement_tiles(&leader, kind, policy != "mixed")
                    .iter()
                    .any(|&t| placement_allowed(kind, t))
                {
                    continue;
                }
                let r = &self.rules.units[kind];
                let category = recruit_role(self, kind);
                let deficiency = shares[category] / total_share * (allies.len() + 1) as f64
                    - counts[category] as f64;
                let recruit = Unit {
                    id: 0,
                    kind,
                    side,
                    tile: leader.tile,
                    hp: r.hp,
                    moved: true,
                    attacked: true,
                    experience: None,
                    name: None,
                    role: None,
                    retreating: None,
                    rebuilding: None,
                };
                let effectiveness = if opponents.is_empty() {
                    r.damage as f64
                } else {
                    opponents
                        .iter()
                        .map(|target| {
                            r.attacks
                                .iter()
                                .enumerate()
                                .filter(|(_, w)| {
                                    w.category
                                        == if category >= 2 {
                                            Category::Ranged
                                        } else {
                                            Category::Melee
                                        }
                                })
                                .map(|(w, _)| self.damage(&recruit, target, w))
                                .max()
                                .unwrap_or(0) as f64
                        })
                        .sum::<f64>()
                        / opponents.len() as f64
                };
                let efficiency = (r.hp as f64 * 0.35 + effectiveness * 1.5) / r.cost as f64;
                let score = if policy == "rush" {
                    efficiency * 5.0 + r.hp as f64 * 0.15
                } else {
                    deficiency * 8.0
                        + efficiency * 2.0
                        + if category == 0 && counts[0] == 0 {
                            6.0
                        } else {
                            0.0
                        }
                        + if category == 3 && counts[3] == 0 {
                            3.0
                        } else {
                            0.0
                        }
                };
                choices.push((
                    kind,
                    score + jitter(seed, &format!("recruit:{}:{}", self.round, r.id)),
                ));
            }
            choices.sort_by(|a, b| b.1.total_cmp(&a.1));
            let Some(&(preferred, _)) = choices.first() else {
                return;
            };
            if self.rules.units[preferred].cost > self.gold[side.index()] && self.income(side) > 0 {
                if (recruit_role(self, preferred) == 0 && counts[0] == 0)
                    || self.rules.units[preferred].cost - self.gold[side.index()]
                        <= self.income(side) * 2
                {
                    return;
                }
            }
            let Some(&(kind, _)) = choices
                .iter()
                .find(|&&(k, _)| self.rules.units[k].cost <= self.gold[side.index()])
            else {
                return;
            };
            let mut tiles: Vec<_> = self
                .placement_tiles(&leader, kind, true)
                .into_iter()
                .filter(|&t| placement_allowed(kind, t))
                .collect();
            let proximity = |t| {
                opponents
                    .iter()
                    .map(|u| self.board.distance(t, u.tile))
                    .min()
                    .unwrap_or(0)
            };
            tiles.sort_by(|&a, &b| {
                if role(self, kind) == 0 {
                    proximity(a).cmp(&proximity(b))
                } else {
                    proximity(b).cmp(&proximity(a))
                }
            });
            self.recruit(leader.id, kind, tiles[0]);
        }
    }
    fn best_plan(
        &mut self,
        policy: &str,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
        routes: &mut HashMap<(usize, Vec<usize>), Vec<u16>>,
        commitments: &mut HashMap<u32, String>,
    ) -> Option<Plan> {
        let side = self.turn;
        let objectives = individual(self.policy_objectives(goals));
        let allies: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side == side)
            .cloned()
            .collect();
        if allies.iter().all(|u| u.attacked) {
            return None;
        }
        let foes: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side != side)
            .cloned()
            .collect();
        let lanes = self.withdrawal_lanes(side, restrained);
        let mut threats = Threats::new(self, side);
        let screens: Vec<_> = allies
            .iter()
            .filter(|u| role(self, u.kind) == 0 && !self.rules.units[u.kind].leader)
            .collect();
        let movable_screens = screens.iter().any(|u| !u.moved && !u.attacked);
        let board = self.board.clone();
        let range_to_foe = |tile| {
            foes.iter()
                .map(|u| board.distance(tile, u.tile))
                .min()
                .unwrap_or(20)
        };
        let mut best: Option<Plan> = None;
        for original_unit in &allies {
            if original_unit.attacked {
                continue;
            }
            let category = role(self, original_unit.kind);
            let important = self.essential(original_unit);
            let before_risk = threats.at(original_unit, original_unit.tile, None);
            if let Some(&promotion) = self.promotion_options(original_unit).first() {
                if before_risk < original_unit.hp && best.as_ref().is_none_or(|b| b.score < 14.0) {
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
            let mut destinations = vec![original_unit.tile];
            destinations.extend(self.reachable(original_unit).into_iter().map(|(t, _)| t));
            destinations = self.authored_destinations(original_unit, destinations);
            let reserved = self.reserved(original_unit, &objectives);
            let required = objectives.iter().any(|g| {
                g.kind == "escape" && g.actor.as_ref() == Some(&format!("u{}", original_unit.id))
            });
            let rebuilding = self.leader_rebuild(original_unit.id, required);
            let unit = self.unit(original_unit.id).unwrap().clone();
            let minimum_risk = if important {
                destinations
                    .iter()
                    .map(|&t| threats.at(&unit, t, None))
                    .min()
                    .unwrap_or(i32::MAX)
            } else {
                0
            };
            let recruiting = self.rules.units[unit.kind].leader
                && self
                    .recruitment_options(side)
                    .iter()
                    .any(|&k| !self.placement_tiles(&unit, k, true).is_empty());
            let mut rebuild_goals = vec![];
            if !rebuilding.is_empty() {
                let mut g = objective(
                    "rebuild",
                    "capture",
                    rebuilding
                        .iter()
                        .map(|&t| self.board.ids[t].clone())
                        .collect(),
                    80.0,
                );
                g.actor = Some(format!("u{}", unit.id));
                rebuild_goals.push(g);
            }
            let goals = if rebuilding.is_empty() {
                &objectives
            } else {
                &rebuild_goals
            };
            struct Duty {
                goal: Objective,
                route: Vec<u16>,
                before: u16,
                assigned: bool,
                priority: f64,
            }
            let mut duties = vec![];
            for original in goals {
                if original.kind == "capture"
                    && original.actor.is_none()
                    && original.tiles.iter().all(|t| {
                        let tile = self.site(t);
                        self.owners[tile] == Some(side)
                            && (original.hold != Some(true)
                                || self
                                    .at(tile)
                                    .is_some_and(|o| o.side == side && o.id != unit.id))
                    })
                {
                    continue;
                }
                if original.id.starts_with("rebuild:")
                    && original.actor.as_ref() != Some(&format!("u{}", unit.id))
                {
                    continue;
                }
                let pursuing = self.can_pursue(&unit, original);
                let assisting =
                    !pursuing && (original.actor.is_some() || original.family.is_some());
                if !pursuing && !assisting {
                    continue;
                }
                let goal = if assisting {
                    self.support_objective(&unit, original)
                } else {
                    original.clone()
                };
                let tiles: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
                let route = routes
                    .entry((unit.kind, tiles.clone()))
                    .or_insert_with(|| self.routes(unit.kind, &tiles));
                let before = route[unit.tile];
                if before == u16::MAX || goal.tiles.is_empty() {
                    continue;
                }
                duties.push(Duty {
                    priority: (before as f64 + 3.0) / goal.weight,
                    assigned: !assisting && (original.actor.is_some() || original.family.is_some()),
                    goal,
                    route: route.clone(),
                    before,
                });
            }
            if duties.iter().any(|d| d.assigned) {
                duties.retain(|d| d.assigned);
            }
            let order = TileOrder::new(self, &unit, true);
            let nearest = |goal: &Objective| {
                goal.tiles
                    .iter()
                    .map(|t| self.site(t))
                    .min_by(|&a, &b| {
                        self.board
                            .distance(unit.tile, a)
                            .cmp(&self.board.distance(unit.tile, b))
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
                duties.sort_by(|a, b| {
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
            let danger_weight = if important {
                6.0
            } else if category == 0 {
                if policy == "rush" { 0.35 } else { 0.55 }
            } else {
                0.9
            };
            let mut unit_plans = vec![];
            let mut wait_score = f64::NEG_INFINITY;
            let mut tiles = vec![unit.tile];
            tiles.extend(destinations.iter().copied().filter(|&t| t != unit.tile));
            for tile in tiles {
                let mut positioning = if reserved.contains(&unit.tile) {
                    80.0
                } else {
                    0.0
                } - if reserved.contains(&tile) { 160.0 } else { 0.0 }
                    - if lanes.contains(&tile) { 100.0 } else { 0.0 };
                if let Some(d) = &duty {
                    let after = d.route[tile];
                    if after == u16::MAX {
                        continue;
                    }
                    positioning += (d.before as f64 - after as f64) * d.goal.weight / 4.0
                        + ((d.goal.tiles.contains(&self.board.ids[tile]) as i32
                            - d.goal.tiles.contains(&self.board.ids[unit.tile]) as i32)
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
                        g.actor.as_ref() == Some(&format!("u{}", unit.id))
                            && g.tiles.contains(&self.board.ids[unit.tile])
                    })
                {
                    positioning -= 24.0;
                }
                let healing_tile = (self.terrain_is(tile, "village")
                    || self.terrain_is(tile, "grove"))
                    && (self.owners[tile] == Some(side) || self.at(tile).is_none());
                if healing_tile {
                    positioning += (self.rules.units[unit.kind].hp - unit.hp).min(2) as f64 * 2.0;
                }
                if self.terrain_is(tile, "village") && self.owners[tile] != Some(side) {
                    positioning += 3.0;
                }
                if policy == "mixed" {
                    if category == 2 {
                        let healing = self.rules.units[unit.kind].healing;
                        let mut value = 0.0;
                        for ally in &allies {
                            if ally.id == unit.id || self.board.distance(tile, ally.tile) != 1 {
                                continue;
                            }
                            let other = allies
                                .iter()
                                .filter(|o| {
                                    o.id != unit.id
                                        && o.id != ally.id
                                        && self.board.distance(o.tile, ally.tile) == 1
                                })
                                .map(|o| self.rules.units[o.kind].healing)
                                .max()
                                .unwrap_or(0);
                            value += (healing - other)
                                .max(0)
                                .min(self.rules.units[ally.kind].hp - ally.hp)
                                as f64
                                * 3.0;
                            if role(self, ally.kind) == 0 && range_to_foe(ally.tile) <= 4 {
                                value += if range_to_foe(tile) > range_to_foe(ally.tile) {
                                    3.0
                                } else {
                                    0.5
                                };
                            }
                        }
                        positioning += value
                            * if movable_screens
                                && unit.hp as f64 > self.rules.units[unit.kind].hp as f64 / 2.0
                            {
                                0.25
                            } else {
                                1.0
                            };
                    }
                    if category != 0 && !screens.is_empty() && range_to_foe(tile) <= 4 {
                        let front = screens.iter().map(|u| range_to_foe(u.tile)).min().unwrap();
                        positioning += if range_to_foe(tile) > front {
                            2.0
                        } else {
                            -3.0
                        };
                    }
                    if category == 0 {
                        for protected in allies
                            .iter()
                            .filter(|u| self.essential(u) || role(self, u.kind) != 0)
                        {
                            if protected.id != unit.id
                                && self.board.distance(tile, protected.tile) == 1
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
                let mut choices = vec![(None, None)];
                for target in &foes {
                    for weapon in 0..self.rules.units[unit.kind].attacks.len() {
                        if self.can_attack(&unit, tile, target, weapon)
                            && self.diplomacy_allows(&unit, target, weapon, tile, restrained)
                        {
                            choices.push((Some(target), Some(weapon)));
                        }
                    }
                }
                for (target, weapon) in choices {
                    if target.is_none() && tile == unit.tile {
                        continue;
                    }
                    let mut combat = 0.0;
                    let mut retaliation = 0;
                    let mut eliminated = None;
                    let mut finishes_battle = false;
                    if let (Some(target), Some(weapon)) = (target, weapon) {
                        let (damage, response, _) = self.preview(&unit, target, weapon, tile);
                        retaliation = response;
                        let killed = damage >= target.hp;
                        eliminated = killed.then_some(target.id);
                        finishes_battle = killed
                            && self.campaign.is_none()
                            && self.rules.units[target.kind].leader;
                        combat = damage.min(target.hp) as f64 * 2.0
                            + if killed {
                                12.0 + self.rules.units[target.kind].cost as f64 * 0.3
                            } else {
                                0.0
                            }
                            - retaliation as f64 * 1.8;
                        if self.essential(target) {
                            combat += if killed { 18.0 } else { 4.0 };
                        }
                        if killed
                            && self.rules.units[unit.kind]
                                .experience_needed
                                .is_some_and(|needed| {
                                    unit.experience.unwrap_or(0)
                                        + 6 * self.rules.units[target.kind].level
                                        >= needed
                                })
                        {
                            combat += 3.0;
                        }
                        if retaliation >= unit.hp {
                            if important {
                                continue;
                            }
                            combat -= 18.0;
                        }
                    }
                    let incoming = threats.at(&unit, tile, eliminated);
                    let lethal = incoming >= unit.hp - retaliation;
                    let finishing =
                        self.finishes_objective(&unit, tile, &objectives) && retaliation < unit.hp;
                    if self.rules.units[unit.kind].leader
                        && !finishing
                        && !finishes_battle
                        && rebuilding.is_empty()
                        && !self.leader_may_advance(&unit, tile, recruiting, required)
                    {
                        continue;
                    }
                    if important
                        && lethal
                        && incoming > 0
                        && !finishing
                        && !finishes_battle
                        && (minimum_risk < unit.hp
                            || incoming + retaliation > minimum_risk
                            || incoming >= before_risk)
                    {
                        continue;
                    }
                    if important && !finishing && !finishes_battle {
                        combat *= if unit.role.as_deref() == Some("courier") {
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
                        + jitter(
                            seed,
                            &format!(
                                "{}:u{}:{}:{}",
                                self.round,
                                unit.id,
                                self.board.ids[tile],
                                target.map_or("move".into(), |t| format!("u{}", t.id))
                            ),
                        );
                    let withdrawing = unit.side == Side::Enemy
                        && target.is_none()
                        && unit.hp < self.rules.units[unit.kind].hp
                        && range_to_foe(tile) > range_to_foe(unit.tile)
                        && (incoming < before_risk
                            || healing_tile
                            || unit.retreating == Some(true));
                    unit_plans.push(Plan {
                        unit: unit.id,
                        tile,
                        target: target.map(|t| t.id),
                        weapon,
                        score,
                        advance: None,
                        withdrawing,
                    });
                }
            }
            for mut plan in unit_plans {
                plan.score -= wait_score;
                if plan.score > 0.1 && best.as_ref().is_none_or(|b| plan.score > b.score) {
                    best = Some(plan);
                }
            }
        }
        best
    }
    pub fn play_policy(
        &mut self,
        policy: &str,
        seed: u32,
        limit: usize,
        goals: Option<&[Objective]>,
        restrained: bool,
    ) {
        if self.winner.is_some() {
            return;
        }
        let side = self.turn;
        for unit in &mut self.units {
            if unit.side == side
                && unit.retreating == Some(true)
                && unit.hp == self.rules.units[unit.kind].hp
            {
                unit.retreating = None;
            }
        }
        self.policy_recruit(policy, seed, limit, goals, restrained);
        if policy == "classic" {
            self.play_classic(goals, restrained);
            return;
        }
        assert!(policy == "rush" || policy == "mixed");
        let mut routes = HashMap::new();
        let mut commitments = HashMap::new();
        let limit = self.units.iter().filter(|u| u.side == side).count() * 3;
        for _ in 0..limit {
            if self.winner.is_some() {
                break;
            }
            let Some(plan) = self.best_plan(
                policy,
                seed,
                goals,
                restrained,
                &mut routes,
                &mut commitments,
            ) else {
                break;
            };
            if let Some(kind) = plan.advance {
                self.advance(plan.unit, kind);
            } else {
                if self.unit(plan.unit).unwrap().tile != plan.tile {
                    self.move_unit(plan.unit, plan.tile);
                    let index = self.index(plan.unit).unwrap();
                    self.units[index].retreating = plan.withdrawing.then_some(true);
                }
                if let (Some(target), Some(weapon)) = (plan.target, plan.weapon) {
                    if self.winner.is_none() {
                        let index = self.index(plan.unit).unwrap();
                        self.units[index].retreating = None;
                        self.attack(plan.unit, target, weapon);
                    }
                }
            }
        }
    }
}
