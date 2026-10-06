use super::roles::role;
use super::{Policy, TacticalRole};
use crate::{
    ai::objectives::{TileOrder, individual, objective},
    ai::threats::Threats,
    data::*,
    logic::Game,
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
pub(super) fn jitter(seed: u32, key: &str) -> f64 {
    let mut h = seed;
    for c in key.encode_utf16() {
        h = (h ^ c as u32).wrapping_mul(16777619);
    }
    h as f64 / 4294967296.0 * 0.05
}
#[derive(Clone)]
pub(super) struct Plan {
    pub(super) unit: u32,
    pub(super) tile: usize,
    pub(super) score: f64,
    pub(super) target: Option<u32>,
    pub(super) weapon: Option<usize>,
    pub(super) advance: Option<usize>,
    pub(super) withdrawing: bool,
}
impl Game<'_> {
    pub(super) fn best_plan(
        &self,
        policy: Policy,
        seed: u32,
        goals: Option<&[Objective]>,
        restrained: bool,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut HashMap<u32, String>,
        intents: &mut Vec<(u32, Option<bool>)>,
    ) -> Option<Plan> {
        let side = self.turn;
        let objectives = individual(self.policy_objectives(goals));
        let rebuilds: HashMap<_, _> = self
            .units
            .iter()
            .filter(|u| u.side == side)
            .map(|u| {
                (
                    u.id,
                    objectives.iter().any(|g| {
                        g.kind == ObjectiveKind::Escape
                            && g.actor.as_ref() == Some(&format!("u{}", u.id))
                    }),
                )
            })
            .map(|(id, required)| (id, self.leader_rebuild_plan(id, required)))
            .collect();
        for (&id, (_, intent)) in &rebuilds {
            if self.unit(id).unwrap().rebuilding != *intent {
                intents.push((id, *intent));
            }
        }
        // Commit bookkeeping in roster order, as policy_objectives and the
        // original planning loop visited leaders and units in that order.
        intents.sort_by_key(|(id, _)| self.index(*id).unwrap());
        let allies: Vec<_> = self.units.iter().filter(|u| u.side == side).collect();
        if allies.iter().all(|u| u.attacked) {
            return None;
        }
        let foes: Vec<_> = self.units.iter().filter(|u| u.side != side).collect();
        let lanes = self.withdrawal_lanes(side, restrained);
        let mut threats = Threats::new(self, side);
        let screens: Vec<_> = allies
            .iter()
            .filter(|u| {
                role(self, u.kind) == TacticalRole::Frontline && !self.rules.units[u.kind].leader
            })
            .collect();
        let movable_screens = screens.iter().any(|u| !u.moved && !u.attacked);
        let board = &self.board;
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
                g.kind == ObjectiveKind::Escape
                    && g.actor.as_ref() == Some(&format!("u{}", original_unit.id))
            });
            let (rebuilding, intent) = &rebuilds[&original_unit.id];
            let mut planned_unit = (*original_unit).clone();
            planned_unit.rebuilding = *intent;
            let unit = &planned_unit;
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
                    ObjectiveKind::Capture,
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
                route: std::rc::Rc<Vec<u16>>,
                before: u16,
                assigned: bool,
                priority: f64,
            }
            let mut duties = vec![];
            for original in goals {
                if original.kind == ObjectiveKind::Capture
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
            } else if category == TacticalRole::Frontline {
                if policy == Policy::Rush { 0.35 } else { 0.55 }
            } else {
                0.9
            };

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
                let healing_tile = (self.terrain_is(tile, Terrain::Village)
                    || self.terrain_is(tile, Terrain::Grove))
                    && (self.owners[tile] == Some(side) || self.at(tile).is_none());
                if healing_tile {
                    positioning += (self.rules.units[unit.kind].hp - unit.hp).min(2) as f64 * 2.0;
                }
                if self.terrain_is(tile, Terrain::Village) && self.owners[tile] != Some(side) {
                    positioning += 3.0;
                }
                if policy == Policy::Mixed {
                    if category == TacticalRole::Healer {
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
                                && unit.hp as f64 > self.rules.units[unit.kind].hp as f64 / 2.0
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
                        for protected in allies.iter().filter(|u| {
                            self.essential(u) || role(self, u.kind) != TacticalRole::Frontline
                        }) {
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
                let choices =
                    std::iter::once((None, None)).chain(foes.iter().flat_map(|&target| {
                        (0..self.rules.units[unit.kind].attacks.len())
                            .filter_map(move |weapon| Some((Some(target), Some(weapon))))
                    }));
                for (target, weapon) in choices {
                    if let (Some(target), Some(weapon)) = (target, weapon) {
                        if !self.can_attack(unit, tile, target, weapon)
                            || !self.diplomacy_allows(unit, target, weapon, tile, restrained)
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
                    if plan.score > 0.1 && best.as_ref().is_none_or(|b| plan.score > b.score) {
                        best = Some(plan);
                    }
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::{Random, jitter};
    #[test]
    fn random_stream_and_utf16_jitter_remain_stable() {
        let mut random = Random(1);
        for expected in [2693262067u32, 11749833, 2265367787] {
            assert_eq!(random.next(), expected as f64 / 4294967296.0);
        }
        assert_eq!(jitter(19, "😀"), 3961270494u32 as f64 / 4294967296.0 * 0.05);
    }
}
