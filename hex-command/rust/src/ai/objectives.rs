use crate::{data::*, logic::Game};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::cmp::Ordering;
/// Compare canonical actor identities without formatting in candidate loops.
pub(super) fn actor_matches(actor: &Option<String>, id: u32) -> bool {
    actor.as_deref().is_some_and(|actor| {
        actor.strip_prefix('u').is_some_and(|number| {
            !number.is_empty()
                && (number.len() == 1 || !number.starts_with('0'))
                && number.bytes().all(|byte| byte.is_ascii_digit())
                && number.parse::<u32>() == Ok(id)
        })
    })
}
#[cfg(test)]
mod actor_tests {
    use super::actor_matches;
    #[test]
    fn only_canonical_unit_keys_match() {
        for id in [0, 12, u32::MAX] {
            assert!(actor_matches(&Some(format!("u{id}")), id));
            assert!(!actor_matches(&Some(format!("u0{id}")), id));
        }
        for key in ["", "u", "u+12", "u１２", "u4294967296", "12"] {
            assert!(!actor_matches(&Some(key.into()), 12));
        }
        assert!(!actor_matches(&None, 0));
    }
}
pub fn objective(id: &str, kind: ObjectiveKind, tiles: Vec<String>, weight: f64) -> Objective {
    Objective {
        id: id.into(),
        kind,
        tiles,
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}
impl Game<'_> {
    pub fn essential(&self, u: &Unit) -> bool {
        self.rules.units[u.kind].leader
            || u.role == Some(Role::Courier)
            || (u.side == Side::Player
                && self.mission.as_ref().is_some_and(|m| {
                    u.name
                        .as_ref()
                        .is_some_and(|n| m.required_companions.contains(n))
                }))
    }
    pub fn can_pursue(&self, u: &Unit, g: &Objective) -> bool {
        (!matches!(
            g.kind,
            ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil
        ) || g.actor.is_some())
            && (g.actor.is_none() || actor_matches(&g.actor, u.id))
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
                    || matches!(
                        g.kind,
                        ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil
                    ))
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
                    if self.goal_done(g)
                        && matches!(
                            g,
                            Goal::Seal { .. } | Goal::Event { .. } | Goal::Defeat { .. }
                        )
                    {
                        continue;
                    }
                    let mut tiles = self.active_goal_tiles(g);
                    let mut kind = ObjectiveKind::Capture;
                    let mut actor = None;
                    if matches!(
                        g,
                        Goal::Escort { .. } | Goal::Escape { .. } | Goal::Vigil { .. }
                    ) {
                        kind = match g {
                            Goal::Escort { .. } => ObjectiveKind::Escort,
                            Goal::Escape { .. } => ObjectiveKind::Escape,
                            Goal::Vigil { .. } => ObjectiveKind::Vigil,
                            _ => unreachable!(),
                        };
                        actor = self
                            .units
                            .iter()
                            .find(|u| {
                                u.side == side
                                    && if matches!(g, Goal::Vigil { .. }) {
                                        u.name.as_ref() == g.companion()
                                    } else {
                                        u.role
                                            == Some(if matches!(g, Goal::Escort { .. }) {
                                                Role::Courier
                                            } else {
                                                Role::Leader
                                            })
                                    }
                            })
                            .map(|u| format!("u{}", u.id));
                    } else if matches!(g, Goal::Seal { .. }) {
                        kind = ObjectiveKind::Seal;
                    } else if matches!(g, Goal::Event { .. }) {
                        kind = ObjectiveKind::Visit;
                    } else if matches!(g, Goal::Defeat { .. })
                        || (matches!(g, Goal::Refuge { .. }) && state.escalated())
                    {
                        kind = ObjectiveKind::Defeat;
                        tiles.extend(
                            self.units
                                .iter()
                                .filter(|u| {
                                    u.side != side
                                        && (matches!(g, Goal::Refuge { .. })
                                            || self.rules.units[u.kind].leader)
                                })
                                .map(|u| self.board.ids[u.tile].clone()),
                        );
                    }
                    if !tiles.is_empty() {
                        let mut result = objective(&format!("required:{index}"), kind, tiles, 24.0);
                        result.actor = actor;
                        result.family = if matches!(g, Goal::Seal { .. }) {
                            g.unit_type().map(|k| self.rules.units[self.kind(k)].family)
                        } else {
                            None
                        };
                        result.hold = Some(
                            matches!(g, Goal::Hold { .. }) || matches!(g, Goal::Refuge { .. }),
                        );
                        goals.push(result);
                    }
                }
                for e in &mission.events {
                    if matches!(e.trigger, Trigger::Visit { .. })
                        && !state.marked(&format!("event:{}", e.id))
                        && !mission
                            .goals
                            .iter()
                            .any(|g| matches!(g, Goal::Event { .. }) && g.event() == Some(&e.id))
                    {
                        goals.push(objective(
                            &format!("optional:{}", e.id),
                            ObjectiveKind::Visit,
                            vec![match &e.trigger {
                                Trigger::Visit { tile } => tile.clone(),
                                _ => unreachable!(),
                            }],
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
                    let mut o = objective(
                        "authored-post",
                        ObjectiveKind::Capture,
                        vec![post.clone()],
                        80.0,
                    );
                    o.actor = Some(format!("u{}", leader.id));
                    goals.push(o);
                }
                for (i, g) in mission.goals.iter().enumerate() {
                    if matches!(
                        g,
                        Goal::Control { .. }
                            | Goal::Hold { .. }
                            | Goal::Refuge { .. }
                            | Goal::Vigil { .. }
                    ) {
                        let mut o = objective(
                            &format!("deny:{i}"),
                            ObjectiveKind::Capture,
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
            if self.terrain_is(tile, Terrain::Village) && self.owners[tile] != Some(side) {
                goals.push(objective(
                    &format!("village:{}", self.board.ids[tile]),
                    ObjectiveKind::Capture,
                    vec![self.board.ids[tile].clone()],
                    8.0,
                ));
            }
        }
        if !goals.iter().any(|g| g.kind == ObjectiveKind::Defeat) {
            goals.push(objective(
                "opposing-leader",
                ObjectiveKind::Defeat,
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
        let mut seen = HashSet::default();
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
    /// Identify friendly blockers on a required actor's next movement corridor.
    /// Escorts clear the path rather than camping beside its destination forever.
    pub(super) fn escort_lanes(&self, goals: &[Objective]) -> HashMap<u32, HashSet<usize>> {
        let mut lanes: HashMap<u32, HashSet<usize>> = HashMap::default();
        for goal in goals
            .iter()
            .filter(|g| matches!(g.kind, ObjectiveKind::Escort | ObjectiveKind::Escape))
        {
            let Some(actor) = self.units.iter().find(|u| actor_matches(&goal.actor, u.id)) else {
                continue;
            };
            let targets: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
            if targets.contains(&actor.tile) {
                continue;
            }
            let route = self.occupied_routes(actor, &targets, true);
            if route[actor.tile] == u16::MAX {
                continue;
            }
            let mut path = vec![actor.tile];
            let mut current = actor.tile;
            while !targets.contains(&current) {
                let next = self.board.adjacent[current]
                    .iter()
                    .copied()
                    .filter(|&t| route[t] < route[current])
                    .min_by_key(|&t| {
                        let occupant = self
                            .at(t)
                            .is_some_and(|u| u.id != actor.id && u.side == actor.side);
                        route[t]
                            .saturating_add(self.cost(actor.kind, t))
                            .saturating_add(if occupant { 2 } else { 0 })
                    });
                let Some(next) = next else {
                    break;
                };
                path.push(next);
                current = next;
            }
            let mut spent = 0;
            for &tile in path.iter().skip(1) {
                spent += self.cost(actor.kind, tile) as usize;
                if spent > self.rules.units[actor.kind].r#move {
                    break;
                }
                if let Some(blocker) = self
                    .at(tile)
                    .filter(|u| u.side == actor.side && u.id != actor.id)
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
    pub fn finishes_objective(&self, u: &Unit, tile: usize, goals: &[Objective]) -> bool {
        self.campaign.is_some()
            && u.side == Side::Player
            && goals.iter().any(|g| {
                actor_matches(&g.actor, u.id)
                    && matches!(g.kind, ObjectiveKind::Escort | ObjectiveKind::Escape)
                    && g.tiles.contains(&self.board.ids[tile])
            })
            && self.mission.as_ref().unwrap().goals.iter().all(|g| {
                self.goal_done(g)
                    || ((matches!(g, Goal::Escort { .. }) && u.role == Some(Role::Courier)
                        || matches!(g, Goal::Escape { .. }) && u.role == Some(Role::Leader))
                        && g.tile() == Some(&self.board.ids[tile]))
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
                    if matches!(g, Goal::Control { .. }) {
                        g.tiles().to_vec()
                    } else if matches!(g, Goal::Hold { .. })
                        || (matches!(g, Goal::Refuge { .. })
                            && !self.campaign.as_ref().unwrap().escalated())
                    {
                        vec![g.tile().unwrap().clone()]
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
        let mut lanes = HashSet::default();
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
                        && (self.terrain_is(t, Terrain::Keep)
                            || self.terrain_is(t, Terrain::Village))
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
    pub(super) fn leader_rebuild_plan(
        &self,
        id: u32,
        required: bool,
    ) -> (Vec<usize>, Option<bool>) {
        let Some(u) = self.unit(id) else {
            return (vec![], None);
        };
        if !self.rules.units[u.kind].leader || required {
            return (vec![], u.rebuilding);
        }
        let keeps: Vec<_> = (0..self.terrain.len())
            .filter(|&t| self.terrain_is(t, Terrain::Keep) && self.owners[t] == Some(u.side))
            .collect();
        let options = self.recruitment_options(u.side);
        if keeps.is_empty() || options.is_empty() {
            return (vec![], None);
        }
        let count = self.ordinary(&u).count();
        let rebuilding = if count >= 5 {
            false
        } else if count <= 2 {
            true
        } else {
            u.rebuilding.unwrap_or(true)
        };
        if rebuilding
            && (self.income(u.side) > 0
                || options
                    .iter()
                    .any(|&k| self.rules.units[k].cost <= self.gold[u.side.index()]))
        {
            (keeps, Some(rebuilding))
        } else {
            (vec![], Some(rebuilding))
        }
    }
    pub(super) fn leader_rebuild(&mut self, id: u32, required: bool) -> Vec<usize> {
        let (keeps, intent) = self.leader_rebuild_plan(id, required);
        if let Some(index) = self.index(id) {
            self.units[index].rebuilding = intent;
        }
        keeps
    }
    pub(super) fn ordinary(&self, u: &Unit) -> impl Iterator<Item = &Unit> {
        let side = u.side;
        self.units.iter().filter(move |o| {
            o.side == side
                && !self.rules.units[o.kind].leader
                && o.role != Some(Role::Courier)
                && self.rules.units[o.kind].healing == 0
        })
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
        let escorts = self.ordinary(u).filter(|o| {
            o.hp as f64 >= self.rules.units[o.kind].hp as f64 / 2.0
                && self.board.distance(tile, o.tile) <= 2
        });
        let (count, at_front) = escorts.fold((0, false), |(count, front), unit| {
            (
                count + 1,
                front || self.proximity(u.side, unit.tile) <= after,
            )
        });
        count >= 2 && at_front
    }
}
pub fn individual(goals: Vec<Objective>) -> Vec<Objective> {
    goals
        .into_iter()
        .flat_map(|g| {
            if g.kind == ObjectiveKind::Capture {
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
pub(crate) struct TileOrder {
    home: (f64, f64),
    direction: (f64, f64),
    villages: bool,
}
impl TileOrder {
    pub(crate) fn new(g: &Game, u: &Unit, villages: bool) -> Self {
        fn center(g: &Game, tiles: impl Iterator<Item = usize>) -> (f64, f64) {
            let (sum, count) = tiles.fold(((0.0, 0.0), 0usize), |(s, n), t| {
                let (c, r) = g.board.positions[t];
                ((s.0 + (c - r.div_euclid(2)) as f64, s.1 + r as f64), n + 1)
            });
            (sum.0 / count as f64, sum.1 / count as f64)
        }
        let mut home_keeps = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, Terrain::Keep) && g.owners[t] == Some(u.side))
            .peekable();
        let home = if home_keeps.peek().is_some() {
            center(g, home_keeps)
        } else {
            center(g, std::iter::once(u.tile))
        };
        let mut opposing = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, Terrain::Keep) && g.owners[t].is_some_and(|s| s != u.side))
            .peekable();
        let mut enemies = g
            .units
            .iter()
            .filter(|o| o.side != u.side)
            .map(|o| o.tile)
            .peekable();
        let target = if opposing.peek().is_some() {
            center(g, opposing)
        } else if enemies.peek().is_some() {
            center(g, enemies)
        } else {
            center(g, 0..g.terrain.len())
        };
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
    pub(crate) fn compare(&self, g: &Game, first: usize, second: usize) -> Ordering {
        let axial = |t: usize| {
            let (c, r) = g.board.positions[t];
            ((c - r.div_euclid(2)) as f64, r as f64)
        };
        let a = axial(first);
        let b = axial(second);
        let av = self.villages && g.terrain_is(first, Terrain::Village);
        let bv = self.villages && g.terrain_is(second, Terrain::Village);
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
