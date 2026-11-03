use crate::ai::orders::{Target as Objective, TargetKind as ObjectiveKind};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};
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
pub fn objective(
    id: &str,
    kind: crate::data::ObjectiveKind,
    tiles: Vec<String>,
    weight: f64,
) -> crate::data::Objective {
    crate::data::Objective {
        id: id.into(),
        kind,
        tiles,
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}
pub fn individual(goals: Vec<Objective>) -> Vec<Objective> {
    goals
        .into_iter()
        .flat_map(|g| {
            if g.kind == ObjectiveKind::Capture && g.tiles.len() > 1 {
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
pub(crate) struct TileOrderContext {
    home: Option<(f64, f64)>,
    target: (f64, f64),
    villages: bool,
}
impl TileOrderContext {
    pub(crate) fn for_unit(&self, g: &BattleView, u: &Unit) -> TileOrder {
        let (c, r) = g.board().positions[u.tile];
        let position = ((c - r.div_euclid(2)) as f64, r as f64);
        let home = self.home.unwrap_or(position);
        let mut direction = (self.target.0 - home.0, self.target.1 - home.1);
        if direction == (0.0, 0.0) {
            direction = (position.0 - home.0, position.1 - home.1);
        }
        if direction == (0.0, 0.0) {
            direction.0 = if u.side == Side::Player { 1.0 } else { -1.0 };
        }
        TileOrder {
            home,
            direction,
            villages: self.villages,
        }
    }
}
impl TileOrder {
    pub(crate) fn context(g: &BattleView, side: Side, villages: bool) -> TileOrderContext {
        fn center(g: &BattleView, tiles: impl Iterator<Item = usize>) -> (f64, f64) {
            let (sum, count) = tiles.fold(((0.0, 0.0), 0usize), |(s, n), t| {
                let (c, r) = g.board().positions[t];
                ((s.0 + (c - r.div_euclid(2)) as f64, s.1 + r as f64), n + 1)
            });
            (sum.0 / count as f64, sum.1 / count as f64)
        }
        let mut home_keeps = (0..g.terrain().len())
            .filter(|&t| g.is_keep(t) && g.owns(t, side))
            .peekable();
        let home = if home_keeps.peek().is_some() {
            Some(center(g, home_keeps))
        } else {
            None
        };
        let mut opposing = (0..g.terrain().len())
            .filter(|&t| g.is_keep(t) && g.owners()[t].is_some() && g.captures(t, side))
            .peekable();
        let mut enemies = g
            .units()
            .iter()
            .filter(|o| g.hostile_to_phase(o, side))
            .map(|o| o.tile)
            .peekable();
        let target = if opposing.peek().is_some() {
            center(g, opposing)
        } else if enemies.peek().is_some() {
            center(g, enemies)
        } else {
            center(g, 0..g.terrain().len())
        };
        TileOrderContext {
            home,
            target,
            villages,
        }
    }
    pub(crate) fn new(g: &BattleView, u: &Unit, villages: bool) -> Self {
        Self::context(g, u.side, villages).for_unit(g, u)
    }
    pub(crate) fn compare(&self, g: &BattleView, first: usize, second: usize) -> Ordering {
        let axial = |t: usize| {
            let (c, r) = g.board().positions[t];
            ((c - r.div_euclid(2)) as f64, r as f64)
        };
        let a = axial(first);
        let b = axial(second);
        let av = self.villages && g.role_is(first, TerrainRole::Village);
        let bv = self.villages && g.role_is(second, TerrainRole::Village);
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
            // Axial axes are 60 degrees apart: use the Cartesian hex metric.
            (b.0 - a.0) * (self.direction.0 + self.direction.1 * 0.5)
                + (b.1 - a.1) * (self.direction.1 + self.direction.0 * 0.5),
            // Keep equal-progress choices in a fixed world order. Rotating this
            // tie-break with the side sends mirrored armies into opposite biomes.
            a.1 - b.1,
            (a.0 + a.1 * 0.5) - (b.0 + b.1 * 0.5),
        ];
        values
            .into_iter()
            .find(|v| *v != 0.0)
            .unwrap_or(0.0)
            .partial_cmp(&0.0)
            .unwrap_or(Ordering::Equal)
    }
}

impl BattleView<'_, '_> {
    pub fn can_pursue(&self, u: &Unit, g: &Objective) -> bool {
        (g.actor.is_none() || actor_matches(&g.actor, u.id))
            && g.family
                .as_ref()
                .is_none_or(|family| self.rules().units[u.kind].family == *family)
    }

    /// Personal duties take precedence over incidental nearby resources.
    pub(super) fn assigned_duty(&self, unit: &Unit, goal: &Objective) -> bool {
        self.can_pursue(unit, goal) && (goal.actor.is_some() || goal.family.is_some())
    }
    pub fn reserved(&self, u: &Unit, goals: &[Objective]) -> HashSet<usize> {
        goals
            .iter()
            .filter(|g| (g.actor.is_some() || g.family.is_some()) && !self.can_pursue(u, g))
            .flat_map(|g| g.tiles.iter().map(|t| self.site(t)))
            .collect()
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
            .flat_map(|t| self.board().adjacent[self.site(t)].iter().copied())
            .filter(|&t| {
                seen.insert(t) && !reserved.contains(&t) && self.cost(u.kind, t) != u16::MAX
            })
            .map(|t| self.board().ids[t].clone())
            .collect();
        result.weight *= 0.35;
        result
    }
    /// Identify friendly blockers on an explicitly assigned actor corridor.
    pub(super) fn actor_lanes(&self, goals: &[Objective]) -> HashMap<u32, HashSet<usize>> {
        let mut lanes: HashMap<u32, HashSet<usize>> = HashMap::default();
        for goal in goals.iter().filter(|g| actor_route(g)) {
            let Some(actor) = self
                .units()
                .iter()
                .filter(|u| {
                    self.belongs_to_phase(u, self.turn())
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
    pub(super) fn ordinary(&self, u: &Unit) -> impl Iterator<Item = &Unit> {
        self.units().iter().filter(move |o| {
            self.allied(u, o)
                && self.available_combatant(o)
                && !o.is_leader()
                && o.role != Some(Role::Courier)
                && self.rules().units[o.kind].healing == 0
        })
    }
    pub fn proximity(&self, side: Side, tile: usize) -> u16 {
        self.units()
            .iter()
            .filter(|o| self.hostile_to_phase(o, side))
            .map(|o| self.board().distance(tile, o.tile))
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
        if tile == u.tile || required || !self.units().iter().any(|o| self.hostile(u, o)) {
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
            o.hp as f64 >= self.rules().units[o.kind].hp as f64 / 2.0
                && self.board().distance(tile, o.tile) <= 2
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
/// Whether an actor must be routed through current occupancy and allies must yield.
pub(super) fn actor_route(goal: &Objective) -> bool {
    (goal.actor.is_some() || goal.family.is_some()) && goal.kind != ObjectiveKind::Defeat
}

#[cfg(test)]
mod target_tests {
    use super::*;
    #[test]
    fn horizontal_spawn_reflections_preserve_tile_preferences() {
        let rules = crate::data::Rules::load().unwrap();
        let game = crate::logic::Game::new(&rules, "mirror");
        let view = BattleView::new(&game);
        let leader = |side| {
            view.units()
                .iter()
                .find(|unit| unit.side == side && unit.is_leader())
                .unwrap()
        };
        let left = leader(Side::Player);
        let right = leader(Side::Enemy);
        let (lc, lr) = view.board().positions[left.tile];
        let (rc, rr) = view.board().positions[right.tile];
        assert_eq!(lr, rr);
        let axis = lc + rc + lr.rem_euclid(2);
        let pairs: Vec<_> = view
            .board()
            .positions
            .iter()
            .enumerate()
            .filter_map(|(tile, &(col, row))| {
                let reflected = (axis - col - row.rem_euclid(2), row);
                view.board()
                    .positions
                    .iter()
                    .position(|p| *p == reflected)
                    .map(|other| (tile, other))
            })
            .collect();
        assert!(pairs.len() > 2);
        let left_order = TileOrder::new(&view, left, false);
        let right_order = TileOrder::new(&view, right, false);
        for &(a, reflected_a) in &pairs {
            for &(b, reflected_b) in &pairs {
                assert_eq!(
                    left_order.compare(&view, a, b),
                    right_order.compare(&view, reflected_a, reflected_b),
                    "tile preferences changed after exchanging horizontal spawns"
                );
            }
        }
    }

    #[test]
    fn repeated_target_normalization_preserves_stable_duty_ids() {
        let target = crate::ai::orders::target(
            "territory",
            ObjectiveKind::Capture,
            vec!["a".into(), "b".into()],
            24.0,
        );
        let once = individual(vec![target]);
        let twice = individual(once.clone());
        assert_eq!(
            once.iter().map(|target| &target.id).collect::<Vec<_>>(),
            twice.iter().map(|target| &target.id).collect::<Vec<_>>()
        );
        assert_eq!(once.len(), 2);
    }
}
