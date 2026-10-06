use crate::{
    engine::Game,
    model::*,
    objectives::{TileOrder, individual},
    threats::Threats,
};
use std::collections::HashSet;
pub struct ClassicRules {
    pub unit: Unit,
    pub goals: Vec<Objective>,
    reserved: HashSet<usize>,
    lanes: HashSet<usize>,
    important: bool,
    threats: Option<Threats>,
    minimum: i32,
    holding: bool,
    restrained: bool,
}
impl ClassicRules {
    pub fn new(g: &Game, u: &Unit, goals: &[Objective], restrained: bool) -> Self {
        let important = g.essential(u);
        let mut threats = important.then(|| Threats::new(g, u.side));
        let minimum = if let Some(t) = &mut threats {
            std::iter::once(u.tile)
                .chain(g.reachable(u).into_iter().map(|(t, _)| t))
                .map(|tile| t.at(u, tile, None))
                .min()
                .unwrap_or(0)
        } else {
            0
        };
        Self {
            unit: u.clone(),
            goals: goals.to_vec(),
            reserved: g.reserved(u, goals),
            lanes: g.withdrawal_lanes(u.side, restrained),
            important,
            threats,
            minimum,
            holding: goals.iter().any(|o| {
                o.actor.as_ref() == Some(&format!("u{}", u.id))
                    && o.tiles.contains(&g.board.ids[u.tile])
            }),
            restrained,
        }
    }
    fn risk(&mut self, tile: usize, eliminated: Option<u32>) -> i32 {
        self.threats
            .as_mut()
            .map_or(0, |t| t.at(&self.unit, tile, eliminated))
    }
    fn penalty(&mut self, g: &Game, tile: usize) -> f64 {
        let finishing = g.finishes_objective(&self.unit, tile, &self.goals);
        (if self.reserved.contains(&tile) {
            160.0
        } else {
            0.0
        }) + (if self.lanes.contains(&tile) {
            100.0
        } else {
            0.0
        }) + (if self.holding && tile != self.unit.tile {
            100.0
        } else {
            0.0
        }) + if finishing {
            -1000.0
        } else {
            self.risk(tile, None) as f64 * 12.0
        }
    }
    fn movement(&mut self, g: &Game, tile: usize) -> bool {
        !self.important || g.finishes_objective(&self.unit, tile, &self.goals) || {
            let risk = self.risk(tile, None);
            risk < self.unit.hp || risk <= self.minimum
        }
    }
    fn attack(&mut self, g: &Game, u: &Unit, target: &Unit, weapon: usize, from: usize) -> bool {
        if !g.diplomacy_allows(&self.unit, target, weapon, from, self.restrained) {
            return false;
        }
        if !self.important {
            return true;
        }
        let (damage, retaliation, _) = g.preview(&self.unit, target, weapon, from);
        if retaliation >= self.unit.hp {
            return false;
        }
        if damage >= target.hp && g.campaign.is_none() && g.rules.units[target.kind].leader {
            return true;
        }
        let incoming = self.risk(from, (damage >= target.hp).then_some(target.id));
        incoming + retaliation < self.unit.hp
            && (self.unit.role.as_deref() != Some("courier") || incoming + retaliation == 0)
            && u.side == self.unit.side
    }
}
impl Game {
    pub fn policy_objectives(&mut self, goals: Option<&[Objective]>) -> Vec<Objective> {
        let mut goals = goals.map_or_else(|| self.objectives(self.turn), |g| g.to_vec());
        let leaders: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side == self.turn && self.rules.units[u.kind].leader)
            .map(|u| u.id)
            .collect();
        for id in leaders {
            let required = goals
                .iter()
                .any(|g| g.kind == "escape" && g.actor.as_ref() == Some(&format!("u{id}")));
            let keeps = self.leader_rebuild(id, required);
            if !keeps.is_empty() {
                let mut o = crate::objectives::objective(
                    &format!("rebuild:u{id}"),
                    "capture",
                    keeps.iter().map(|&t| self.board.ids[t].clone()).collect(),
                    80.0,
                );
                o.actor = Some(format!("u{id}"));
                goals.push(o);
            }
        }
        goals
    }
    pub fn enemy_danger(&self, u: &Unit, tile: usize) -> i32 {
        let mut target = u.clone();
        target.tile = tile;
        let mut first = 0;
        let mut second = 0;
        for foe in self.units.iter().filter(|o| o.side != u.side) {
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
        let (damage, retaliation, _) = self.preview(u, target, w, from);
        (if self.rules.units[target.kind].leader {
            20.0
        } else if target.role.as_deref() == Some("courier") {
            12.0
        } else {
            0.0
        }) + if target.hp <= damage { 12.0 } else { 0.0 }
            + (self.rules.units[target.kind].hp - target.hp) as f64 * 0.3
            + damage as f64
            - retaliation as f64 * 0.7
            - if retaliation >= u.hp { 30.0 } else { 0.0 }
    }
    fn try_attack(&mut self, id: u32, desperate: bool, rules: &mut ClassicRules) -> bool {
        let Some(u) = self.unit(id).cloned() else {
            return false;
        };
        let order = TileOrder::new(self, &u, false);
        let mut best: Option<(Unit, usize, f64)> = None;
        for target in self.units.clone().iter().filter(|t| t.side != u.side) {
            for weapon in self.attack_options(&u, target, u.tile) {
                if !rules.attack(self, &u, target, weapon, u.tile)
                    || (!desperate && self.preview(&u, target, weapon, u.tile).1 >= u.hp)
                {
                    continue;
                }
                let score = self.attack_score(&u, target, u.tile, Some(weapon));
                if best.as_ref().is_none_or(|(b, _, s)| {
                    score > *s || (score == *s && order.compare(self, target.tile, b.tile).is_lt())
                }) {
                    best = Some((target.clone(), weapon, score));
                }
            }
        }
        if let Some((target, weapon, _)) = best {
            self.attack(id, target.id, weapon);
            true
        } else {
            false
        }
    }
    fn retreat(&mut self, id: u32, rules: &mut ClassicRules) -> u8 {
        let u = self.unit(id).unwrap().clone();
        let enemies: Vec<_> = self
            .units
            .iter()
            .filter(|o| o.side != u.side)
            .cloned()
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
            self.units
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .retreating = None;
            return 0;
        }
        let trapped = !retaliations.is_empty();
        let villages: Vec<_> = (0..self.terrain.len())
            .filter(|&t| {
                self.terrain_is(t, "village")
                    && self.owners[t] == Some(u.side)
                    && self.at(t).is_none_or(|o| o.id == id)
                    && self.enemy_danger(&u, t) == 0
            })
            .collect();
        let routes = (!villages.is_empty()).then(|| self.routes(u.kind, &villages));
        let recovering = u.retreating == Some(true)
            && u.hp < self.rules.units[u.kind].hp
            && routes.as_ref().is_some_and(|r| r[u.tile] != u16::MAX);
        if !trapped && !recovering {
            self.units
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .retreating = None;
            return 0;
        }
        if recovering && villages.contains(&u.tile) {
            self.units
                .iter_mut()
                .find(|unit| unit.id == id)
                .unwrap()
                .attacked = true;
            return 1;
        }
        let mut destinations: Vec<_> = std::iter::once(u.tile)
            .chain(self.reachable(&u).into_iter().map(|(t, _)| t))
            .filter(|&t| rules.movement(self, t))
            .collect();
        let mut scores = vec![f64::NEG_INFINITY; self.terrain.len()];
        for &t in &destinations {
            let separation = enemies
                .iter()
                .map(|e| self.board.distance(t, e.tile))
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
        let order = TileOrder::new(self, &u, true);
        destinations.sort_by(|&a, &b| {
            scores[b]
                .total_cmp(&scores[a])
                .then_with(|| order.compare(self, a, b))
        });
        if let Some(&t) = destinations.first() {
            if t != u.tile && scores[u.tile] != f64::NEG_INFINITY && scores[t] > scores[u.tile] {
                self.move_unit(id, t);
                let index = self.index(id).unwrap();
                self.units[index].retreating = Some(true);
                self.units[index].attacked = true;
                self.record(format!(
                    "{} is retreating.",
                    u.name
                        .unwrap_or_else(|| self.rules.units[u.kind].name.clone())
                ));
                return 1;
            }
        }
        self.units
            .iter_mut()
            .find(|unit| unit.id == id)
            .unwrap()
            .retreating = None;
        if trapped { 2 } else { 0 }
    }
    fn movement_goals(&self, u: &Unit) -> Vec<usize> {
        let villages: Vec<_> = (0..self.terrain.len())
            .filter(|&t| self.terrain_is(t, "village") && self.owners[t] != Some(u.side))
            .collect();
        let mut goals = if self.rules.units[u.kind].family == "scout" && !villages.is_empty() {
            villages
        } else {
            self.units
                .iter()
                .filter(|o| o.side != u.side)
                .map(|o| o.tile)
                .collect()
        };
        if let Some(m) = &self.mission {
            if u.side == Side::Enemy && self.rules.units[u.kind].leader {
                if let Some(post) = &m.enemy_leader_post {
                    return vec![self.site(post)];
                }
            }
            let territory = self.territory();
            let contested: Vec<_> = territory
                .iter()
                .copied()
                .filter(|&t| self.owners[t] != Some(u.side))
                .collect();
            if !contested.is_empty() {
                goals = contested;
            } else if m.defend_objectives && !territory.is_empty() {
                goals = territory;
            }
        }
        goals
    }
    pub fn classic_act(
        &mut self,
        id: u32,
        movement: Option<Vec<usize>>,
        required: bool,
        mut rules: ClassicRules,
    ) {
        if self.winner.is_some() {
            return;
        }
        let Some(mut u) = self.unit(id).cloned() else {
            return;
        };
        if u.side != self.turn || u.attacked {
            return;
        }
        if let Some(&promotion) = self.promotion_options(&u).first() {
            self.advance(id, promotion);
            return;
        }
        let rebuild = self.leader_rebuild(id, required);
        u = self.unit(id).unwrap().clone();
        let retreat = self.retreat(id, &mut rules);
        if retreat == 1 {
            return;
        }
        if rebuild.is_empty()
            && rules.penalty(self, u.tile) < 100.0
            && self.try_attack(id, retreat == 2, &mut rules)
        {
            return;
        }
        let goals = if !rebuild.is_empty() {
            rebuild.clone()
        } else {
            movement.unwrap_or_else(|| self.movement_goals(&u))
        };
        if goals.is_empty() {
            return;
        }
        let routes = (self.map_id == "river" || self.campaign.is_some() || !rebuild.is_empty())
            .then(|| self.routes(u.kind, &goals));
        let recruiting = self
            .recruitment_options(u.side)
            .iter()
            .any(|&k| !self.placement_tiles(&u, k, true).is_empty());
        let mut dest: Vec<_> = std::iter::once(u.tile)
            .chain(self.reachable(&u).into_iter().map(|(t, _)| t))
            .filter(|&t| rules.movement(self, t))
            .filter(|&t| {
                !self.rules.units[u.kind].leader
                    || !rebuild.is_empty()
                    || self.leader_may_advance(&u, t, recruiting, required)
                    || (self.campaign.is_none()
                        && self.units.iter().any(|target| {
                            target.side != u.side
                                && self.rules.units[target.kind].leader
                                && self
                                    .attack_options(&u, target, t)
                                    .iter()
                                    .any(|&w| self.preview(&u, target, w, t).0 >= target.hp)
                        }))
            })
            .collect();
        if u.side == Side::Enemy
            && self.mission.as_ref().is_some_and(|m| {
                m.defend_objectives
                    || (m.enemy_leader_post.is_some() && self.rules.units[u.kind].leader)
            })
        {
            if let Some(r) = &routes {
                let limit = dest.iter().map(|&t| r[t]).min().unwrap_or(u16::MAX).max(1);
                dest.retain(|&t| r[t] <= limit);
            }
        }
        let mut scores = vec![f64::NEG_INFINITY; self.terrain.len()];
        for &t in &dest {
            let distance = routes.as_ref().map_or_else(
                || {
                    goals
                        .iter()
                        .map(|&g| self.board.distance(t, g))
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
            for target in &self.units {
                if self
                    .attack_options(&u, target, t)
                    .into_iter()
                    .any(|w| rules.attack(self, &u, target, w, t))
                {
                    opportunity =
                        opportunity.max(5.0 + self.attack_score(&u, target, t, None) / 5.0);
                }
            }
            let capture = if self.terrain_is(t, "village") && self.owners[t] != Some(u.side) {
                if self.rules.units[u.kind].family == "scout" {
                    8.0
                } else {
                    5.0
                }
            } else {
                0.0
            };
            let heal =
                if u.hp <= 4 && self.terrain_is(t, "village") && self.owners[t] == Some(u.side) {
                    9.0
                } else {
                    0.0
                };
            scores[t] = -distance
                + opportunity
                + capture
                + heal
                + if self.terrain_is(t, "hill") {
                    0.25
                } else {
                    0.0
                }
                - self.enemy_danger(&u, t) as f64 * 0.5
                + if rebuild.contains(&t) { 40.0 } else { 0.0 }
                - rules.penalty(self, t);
        }
        let order = TileOrder::new(self, &u, true);
        dest.sort_by(|&a, &b| {
            scores[b]
                .total_cmp(&scores[a])
                .then_with(|| order.compare(self, a, b))
        });
        if let Some(&t) = dest.first() {
            if t != u.tile {
                self.move_unit(id, t);
            }
            if self.winner.is_none() {
                self.try_attack(id, false, &mut rules);
            }
        }
    }
    pub fn play_classic(&mut self, goals: Option<&[Objective]>, restrained: bool) {
        let empty = vec![];
        let objectives =
            if goals.is_some() || (self.turn == Side::Player && self.campaign.is_some()) {
                self.policy_objectives(goals)
            } else {
                self.policy_objectives(Some(&empty))
            };
        let mut ids: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side == self.turn)
            .map(|u| u.id)
            .collect();
        let order = |u: &Unit| {
            let r = &self.rules.units[u.kind];
            match r.id.as_str() {
                "scout" | "wolf" => 0,
                "archer" | "thorncaller" | "bonearcher" | "adept" => 1,
                "soldier" | "skeleton" | "warden" | "boar" => 2,
                "commander" | "wildkeeper" | "necromancer" => 3,
                _ => match r.family.as_str() {
                    "scout" | "wolf" => 0,
                    "archer" | "thorncaller" | "bonearcher" | "adept" => 1,
                    "soldier" | "skeleton" | "warden" | "boar" => 2,
                    "commander" | "wildkeeper" | "necromancer" => 3,
                    _ => {
                        if r.leader {
                            3
                        } else if r.range > 1 {
                            1
                        } else {
                            2
                        }
                    }
                },
            }
        };
        ids.sort_by_key(|&id| order(self.unit(id).unwrap()));
        ids.sort_by_key(|&id| {
            let u = self.unit(id).unwrap();
            !self.reserved(u, &objectives).contains(&u.tile)
        });
        let individual = individual(objectives.clone());
        for id in ids {
            if self.winner.is_some() {
                break;
            }
            let Some(u) = self.unit(id).cloned() else {
                continue;
            };
            let tile_order = TileOrder::new(self, &u, true);
            let nearest = |g: &Objective| {
                g.tiles
                    .iter()
                    .map(|t| self.site(t))
                    .min_by(|&a, &b| {
                        self.board
                            .distance(u.tile, a)
                            .cmp(&self.board.distance(u.tile, b))
                            .then_with(|| tile_order.compare(self, a, b))
                    })
                    .unwrap()
            };
            let mut eligible: Vec<_> = individual
                .iter()
                .filter(|g| {
                    self.can_pursue(&u, g)
                        && (g.kind != "capture"
                            || g.tiles
                                .iter()
                                .any(|t| self.owners[self.site(t)] != Some(u.side)))
                })
                .collect();
            eligible.sort_by(|a, b| {
                let score = |g: &Objective| {
                    g.tiles
                        .iter()
                        .map(|t| self.board.distance(u.tile, self.site(t)))
                        .min()
                        .unwrap() as f64
                        / g.weight
                };
                score(a)
                    .total_cmp(&score(b))
                    .then_with(|| tile_order.compare(self, nearest(a), nearest(b)))
            });
            let best = eligible.first().copied();
            let reserved = self.reserved(&u, &objectives);
            let movement = if reserved.contains(&u.tile) {
                Some(
                    self.board.adjacent[u.tile]
                        .iter()
                        .copied()
                        .filter(|t| !reserved.contains(t))
                        .collect(),
                )
            } else {
                best.map(|g| g.tiles.iter().map(|t| self.site(t)).collect())
            };
            let required = best.is_some_and(|g| g.kind == "escape");
            let rules = ClassicRules::new(self, &u, &objectives, restrained);
            self.classic_act(id, movement, required, rules);
        }
    }
}
