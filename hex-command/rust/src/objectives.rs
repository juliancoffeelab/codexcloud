use crate::{engine::Game, model::*};
use std::{cmp::Ordering, collections::HashSet};
pub fn objective(id: &str, kind: &str, tiles: Vec<String>, weight: f64) -> Objective {
    Objective {
        id: id.into(),
        kind: kind.into(),
        tiles,
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}
impl Game {
    pub fn essential(&self, u: &Unit) -> bool {
        self.rules.units[u.kind].leader
            || u.role.as_deref() == Some("courier")
            || (u.side == Side::Player
                && self.mission.as_ref().is_some_and(|m| {
                    u.name
                        .as_ref()
                        .is_some_and(|n| m.required_companions.contains(n))
                }))
    }
    pub fn can_pursue(&self, u: &Unit, g: &Objective) -> bool {
        (!["escort", "escape", "vigil"].contains(&g.kind.as_str()) || g.actor.is_some())
            && g.actor.as_ref().is_none_or(|a| *a == format!("u{}", u.id))
            && g.family
                .as_ref()
                .is_none_or(|f| self.rules.units[u.kind].family == *f)
    }
    pub fn reserved(&self, u: &Unit, goals: &[Objective]) -> HashSet<usize> {
        goals
            .iter()
            .filter(|g| {
                (g.actor.is_some()
                    || g.family.is_some()
                    || ["escort", "escape", "vigil"].contains(&g.kind.as_str()))
                    && !self.can_pursue(u, g)
            })
            .flat_map(|g| g.tiles.iter().map(|t| self.site(t)))
            .collect()
    }
    pub fn objectives(&self, side: Side) -> Vec<Objective> {
        let mut goals = vec![];
        if let Some(mission) = &self.mission {
            let state = self.campaign.as_ref().unwrap();
            if side == Side::Player {
                for (index, g) in mission.goals.iter().enumerate() {
                    if self.goal_progress(g).done
                        && ["seal", "event", "defeat"].contains(&g.kind.as_str())
                    {
                        continue;
                    }
                    let mut tiles = self.active_goal_tiles(g);
                    let mut kind = "capture";
                    let mut actor = None;
                    if ["escort", "escape", "vigil"].contains(&g.kind.as_str()) {
                        kind = &g.kind;
                        actor = self
                            .units
                            .iter()
                            .find(|u| {
                                u.side == side
                                    && if g.kind == "vigil" {
                                        u.name == g.companion
                                    } else {
                                        u.role.as_deref()
                                            == Some(if g.kind == "escort" {
                                                "courier"
                                            } else {
                                                "leader"
                                            })
                                    }
                            })
                            .map(|u| format!("u{}", u.id));
                    } else if g.kind == "seal" {
                        kind = "seal";
                    } else if g.kind == "event" {
                        kind = "visit";
                    } else if g.kind == "defeat" || (g.kind == "refuge" && state.escalated()) {
                        kind = "defeat";
                        tiles.extend(
                            self.units
                                .iter()
                                .filter(|u| {
                                    u.side != side
                                        && (g.kind == "refuge" || self.rules.units[u.kind].leader)
                                })
                                .map(|u| self.board.ids[u.tile].clone()),
                        );
                    }
                    if !tiles.is_empty() {
                        let mut result = objective(&format!("required:{index}"), kind, tiles, 24.0);
                        result.actor = actor;
                        result.family = if g.kind == "seal" {
                            g.unit_type.clone()
                        } else {
                            None
                        };
                        result.hold = Some(g.kind == "hold" || g.kind == "refuge");
                        goals.push(result);
                    }
                }
                for e in &mission.events {
                    if e.trigger.kind == "visit"
                        && !state.marked(&format!("event:{}", e.id))
                        && !mission
                            .goals
                            .iter()
                            .any(|g| g.kind == "event" && g.event.as_ref() == Some(&e.id))
                    {
                        goals.push(objective(
                            &format!("optional:{}", e.id),
                            "visit",
                            vec![e.trigger.tile.clone().unwrap()],
                            4.0,
                        ));
                    }
                }
            } else {
                if let (Some(post), Some(leader)) = (
                    &mission.enemy_leader_post,
                    self.units
                        .iter()
                        .find(|u| u.side == side && self.rules.units[u.kind].leader),
                ) {
                    let mut o = objective("authored-post", "capture", vec![post.clone()], 80.0);
                    o.actor = Some(format!("u{}", leader.id));
                    goals.push(o);
                }
                for (i, g) in mission.goals.iter().enumerate() {
                    if ["control", "hold", "refuge", "vigil"].contains(&g.kind.as_str()) {
                        let mut o = objective(
                            &format!("deny:{i}"),
                            "capture",
                            self.active_goal_tiles(g),
                            24.0,
                        );
                        o.hold = Some(true);
                        goals.push(o);
                    }
                }
            }
        }
        for tile in 0..self.terrain.len() {
            if self.terrain_is(tile, "village") && self.owners[tile] != Some(side) {
                goals.push(objective(
                    &format!("village:{}", self.board.ids[tile]),
                    "capture",
                    vec![self.board.ids[tile].clone()],
                    8.0,
                ));
            }
        }
        if !goals.iter().any(|g| g.kind == "defeat") {
            goals.push(objective(
                "opposing-leader",
                "defeat",
                self.units
                    .iter()
                    .filter(|u| u.side != side && self.rules.units[u.kind].leader)
                    .map(|u| self.board.ids[u.tile].clone())
                    .collect(),
                12.0,
            ));
        }
        goals.retain(|g| !g.tiles.is_empty());
        goals
    }
    pub fn support_objective(&self, u: &Unit, g: &Objective) -> Objective {
        let mut result = g.clone();
        if self.can_pursue(u, g) {
            return result;
        }
        let reserved = self.reserved(u, std::slice::from_ref(g));
        let mut seen = HashSet::new();
        result.tiles = g
            .tiles
            .iter()
            .flat_map(|t| self.board.adjacent[self.site(t)].iter().copied())
            .filter(|&t| {
                seen.insert(t) && !reserved.contains(&t) && self.cost(u.kind, t) != u16::MAX
            })
            .map(|t| self.board.ids[t].clone())
            .collect();
        result.weight *= 0.35;
        result
    }
    pub fn finishes_objective(&self, u: &Unit, tile: usize, goals: &[Objective]) -> bool {
        self.campaign.is_some()
            && u.side == Side::Player
            && goals.iter().any(|g| {
                g.actor.as_ref() == Some(&format!("u{}", u.id))
                    && ["escort", "escape"].contains(&g.kind.as_str())
                    && g.tiles.contains(&self.board.ids[tile])
            })
            && self.mission.as_ref().unwrap().goals.iter().all(|g| {
                self.goal_progress(g).done
                    || ((g.kind == "escort" && u.role.as_deref() == Some("courier")
                        || g.kind == "escape" && u.role.as_deref() == Some("leader"))
                        && g.tile.as_ref() == Some(&self.board.ids[tile]))
            })
    }
    pub fn authored_destinations(&self, u: &Unit, dest: Vec<usize>) -> Vec<usize> {
        if u.side != Side::Enemy || self.mission.is_none() {
            return dest;
        }
        let mission = self.mission.as_ref().unwrap();
        let post = if self.rules.units[u.kind].leader {
            mission.enemy_leader_post.as_ref()
        } else {
            None
        };
        let goals = if let Some(post) = post {
            vec![self.site(post)]
        } else if mission.defend_objectives {
            self.territory()
        } else {
            vec![]
        };
        if goals.is_empty() {
            return dest;
        }
        let routes = self.routes(u.kind, &goals);
        let limit = dest
            .iter()
            .map(|&t| routes[t])
            .min()
            .unwrap_or(u16::MAX)
            .max(1);
        dest.into_iter().filter(|&t| routes[t] <= limit).collect()
    }
    pub fn territory(&self) -> Vec<usize> {
        self.mission.as_ref().map_or(vec![], |m| {
            m.goals
                .iter()
                .flat_map(|g| {
                    if g.kind == "control" {
                        g.tiles.clone()
                    } else if g.kind == "hold"
                        || (g.kind == "refuge" && !self.campaign.as_ref().unwrap().escalated())
                    {
                        vec![g.tile.clone().unwrap()]
                    } else {
                        vec![]
                    }
                })
                .map(|t| self.site(&t))
                .collect()
        })
    }
    pub fn protects_withdrawal(&self, side: Side, restrained: bool) -> bool {
        restrained
            && side == Side::Player
            && self.mission.as_ref().is_some_and(|m| m.refuge.is_some())
            && !self.campaign.as_ref().unwrap().escalated()
    }
    pub fn diplomacy_allows(
        &self,
        u: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
        restrained: bool,
    ) -> bool {
        !self.protects_withdrawal(u.side, restrained)
            || (target.retreating != Some(true)
                && (!self.mediation_at_risk(target)
                    || self.preview(u, target, weapon, from).0 < target.hp))
    }
    pub fn withdrawal_lanes(&self, side: Side, restrained: bool) -> HashSet<usize> {
        let mut lanes = HashSet::new();
        if !self.protects_withdrawal(side, restrained) {
            return lanes;
        }
        for u in self
            .units
            .iter()
            .filter(|u| u.side != side && u.retreating == Some(true))
        {
            let mut homes: Vec<_> = (0..self.terrain.len())
                .filter(|&t| {
                    self.owners[t] == Some(u.side)
                        && (self.terrain_is(t, "keep") || self.terrain_is(t, "village"))
                })
                .collect();
            if homes.is_empty() {
                homes = (0..self.terrain.len())
                    .filter(|&t| self.board.positions[t].0 == self.cols - 1)
                    .collect();
            }
            let routes = self.routes(u.kind, &homes);
            for &t in &self.board.adjacent[u.tile] {
                if routes[t] < routes[u.tile] && self.cost(u.kind, t) != u16::MAX {
                    lanes.insert(t);
                }
            }
        }
        lanes
    }
    pub fn leader_rebuild(&mut self, id: u32, required: bool) -> Vec<usize> {
        let Some(u) = self.unit(id).cloned() else {
            return vec![];
        };
        if !self.rules.units[u.kind].leader || required {
            return vec![];
        }
        let keeps: Vec<_> = (0..self.terrain.len())
            .filter(|&t| self.terrain_is(t, "keep") && self.owners[t] == Some(u.side))
            .collect();
        let options = self.recruitment_options(u.side);
        let index = self.index(id).unwrap();
        if keeps.is_empty() || options.is_empty() {
            self.units[index].rebuilding = None;
            return vec![];
        }
        let count = self.ordinary(&u).len();
        let rebuilding = if count >= 5 {
            false
        } else if count <= 2 {
            true
        } else {
            u.rebuilding.unwrap_or(true)
        };
        self.units[index].rebuilding = Some(rebuilding);
        if rebuilding
            && (self.income(u.side) > 0
                || options
                    .iter()
                    .any(|&k| self.rules.units[k].cost <= self.gold[u.side.index()]))
        {
            keeps
        } else {
            vec![]
        }
    }
    pub fn ordinary(&self, u: &Unit) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|o| {
                o.side == u.side
                    && !self.rules.units[o.kind].leader
                    && o.role.as_deref() != Some("courier")
                    && self.rules.units[o.kind].healing == 0
            })
            .collect()
    }
    pub fn proximity(&self, side: Side, tile: usize) -> u16 {
        self.units
            .iter()
            .filter(|o| o.side != side)
            .map(|o| self.board.distance(tile, o.tile))
            .min()
            .unwrap_or(20)
    }
    pub fn leader_may_advance(
        &self,
        u: &Unit,
        tile: usize,
        recruiting: bool,
        required: bool,
    ) -> bool {
        if tile == u.tile || required || !self.units.iter().any(|o| o.side != u.side) {
            return true;
        }
        let before = self.proximity(u.side, u.tile);
        let after = self.proximity(u.side, tile);
        if u.rebuilding == Some(true) && recruiting && before > 2 {
            return false;
        }
        if after >= before {
            return true;
        }
        let escorts: Vec<_> = self
            .ordinary(u)
            .into_iter()
            .filter(|o| {
                o.hp as f64 >= self.rules.units[o.kind].hp as f64 / 2.0
                    && self.board.distance(tile, o.tile) <= 2
            })
            .collect();
        escorts.len() >= 2
            && escorts
                .iter()
                .any(|o| self.proximity(u.side, o.tile) <= after)
    }
}
pub fn individual(goals: Vec<Objective>) -> Vec<Objective> {
    goals
        .into_iter()
        .flat_map(|g| {
            if g.kind == "capture" {
                g.tiles
                    .iter()
                    .map(|t| {
                        let mut o = g.clone();
                        o.id = format!("{}:{t}", g.id);
                        o.tiles = vec![t.clone()];
                        o
                    })
                    .collect()
            } else {
                vec![g]
            }
        })
        .collect()
}
pub struct TileOrder {
    home: (f64, f64),
    direction: (f64, f64),
    villages: bool,
}
impl TileOrder {
    pub fn new(g: &Game, u: &Unit, villages: bool) -> Self {
        let center = |tiles: Vec<usize>| {
            let n = tiles.len() as f64;
            let sum = tiles.into_iter().fold((0.0, 0.0), |s, t| {
                let (c, r) = g.board.positions[t];
                (s.0 + (c - r.div_euclid(2)) as f64, s.1 + r as f64)
            });
            (sum.0 / n, sum.1 / n)
        };
        let home_keeps: Vec<_> = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, "keep") && g.owners[t] == Some(u.side))
            .collect();
        let home = center(if home_keeps.is_empty() {
            vec![u.tile]
        } else {
            home_keeps
        });
        let opposing: Vec<_> = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, "keep") && g.owners[t].is_some_and(|s| s != u.side))
            .collect();
        let enemies: Vec<_> = g
            .units
            .iter()
            .filter(|o| o.side != u.side)
            .map(|o| o.tile)
            .collect();
        let target = center(if !opposing.is_empty() {
            opposing
        } else if !enemies.is_empty() {
            enemies
        } else {
            (0..g.terrain.len()).collect()
        });
        let mut direction = (target.0 - home.0, target.1 - home.1);
        if direction == (0.0, 0.0) {
            let (c, r) = g.board.positions[u.tile];
            direction = ((c - r.div_euclid(2)) as f64 - home.0, r as f64 - home.1);
        }
        if direction == (0.0, 0.0) {
            direction.0 = if u.side == Side::Player { 1.0 } else { -1.0 };
        }
        Self {
            home,
            direction,
            villages,
        }
    }
    pub fn compare(&self, g: &Game, first: usize, second: usize) -> Ordering {
        let axial = |t: usize| {
            let (c, r) = g.board.positions[t];
            ((c - r.div_euclid(2)) as f64, r as f64)
        };
        let a = axial(first);
        let b = axial(second);
        let av = self.villages && g.terrain_is(first, "village");
        let bv = self.villages && g.terrain_is(second, "village");
        let home_distance = |p: (f64, f64)| {
            ((p.0 - self.home.0).abs()
                + (p.1 - self.home.1).abs()
                + (p.0 + p.1 - self.home.0 - self.home.1).abs())
                / 2.0
        };
        let values = [
            (bv as i32 - av as i32) as f64,
            if av && bv {
                home_distance(a) - home_distance(b)
            } else {
                0.0
            },
            (b.0 - a.0) * self.direction.0 + (b.1 - a.1) * self.direction.1,
            (a.0 - b.0) * self.direction.1 - (a.1 - b.1) * self.direction.0,
        ];
        values
            .into_iter()
            .find(|v| *v != 0.0)
            .unwrap_or(0.0)
            .partial_cmp(&0.0)
            .unwrap_or(Ordering::Equal)
    }
}
