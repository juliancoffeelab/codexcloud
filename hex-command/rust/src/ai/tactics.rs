use super::objectives::actor_matches;
use super::roles::role;
use super::{Policy, TacticalRole};
use crate::{
    ai::objectives::{TileOrder, individual, objective},
    ai::threats::Threats,
    data::*,
    logic::Game,
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
pub(super) fn jitter(seed: u32, key: &str) -> f64 {
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
/// Hash exactly the historical UTF-16 key without allocating formatted strings.
fn action_jitter(seed: u32, round: u32, unit: u32, tile: &str, target: Option<u32>) -> f64 {
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
        prepared_goals: Option<&[Objective]>,
        restrained: bool,
        routes: &mut HashMap<(usize, Vec<usize>), std::rc::Rc<Vec<u16>>>,
        commitments: &mut HashMap<u32, String>,
        intents: &mut Vec<(u32, Option<bool>)>,
    ) -> Option<Plan> {
        let side = self.turn;
        let objectives: Cow<'_, [Objective]> = if let Some(prepared) = prepared_goals {
            let rebuilding = self.rebuild_objectives(prepared);
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
            Cow::Owned(individual(self.policy_objectives(goals)))
        };
        let escort_lanes = self.escort_lanes(&objectives);
        let rebuilds: HashMap<_, _> = self
            .units
            .iter()
            .filter(|u| u.side == side && self.rules.units[u.kind].leader)
            .map(|u| {
                (
                    u.id,
                    objectives
                        .iter()
                        .any(|g| g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, u.id)),
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
            .flat_map(|u| &self.rules.units[u.kind].attacks)
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
        let protected: Vec<_> = if policy == Policy::Mixed {
            allies
                .iter()
                .filter(|u| self.essential(u) || role(self, u.kind) != TacticalRole::Frontline)
                .collect()
        } else {
            Vec::new()
        };
        let mut best: Option<Plan> = None;
        let mut best_order = (usize::MAX, usize::MAX, usize::MAX);
        // Score units with the strongest immediate opportunities first. This
        // only changes evaluation order: roster/tile/weapon ordinals still
        // decide all equal scores and bookkeeping remains unit-local.
        let mut roster: Vec<_> = allies
            .iter()
            .enumerate()
            .filter(|(_, u)| !u.attacked)
            .map(|(index, &unit)| {
                let before_risk = threats.at(unit, unit.tile, None);
                let important = self.essential(unit);
                let category = role(self, unit.kind);
                let danger = if important {
                    9.0
                } else if category == TacticalRole::Frontline {
                    if policy == Policy::Rush { 0.55 } else { 0.75 }
                } else {
                    1.1
                };
                let movement = if unit.moved {
                    0
                } else {
                    self.rules.units[unit.kind].r#move
                };
                let offense = foes
                    .iter()
                    .flat_map(|&target| {
                        self.rules.units[unit.kind]
                            .attacks
                            .iter()
                            .enumerate()
                            .filter_map(move |(weapon, attack)| {
                                if board.distance(unit.tile, target.tile) as usize
                                    > movement + attack.range as usize
                                {
                                    return None;
                                }
                                let damage = self.damage(unit, target, weapon);
                                let killed = damage >= target.hp;
                                let value = damage.min(target.hp) as f64 * 2.0
                                    + if killed {
                                        12.0 + self.rules.units[target.kind].cost as f64 * 0.3
                                    } else {
                                        0.0
                                    }
                                    + if self.essential(target) {
                                        if killed { 18.0 } else { 4.0 }
                                    } else {
                                        0.0
                                    };
                                Some(
                                    if killed
                                        && self.campaign.is_none()
                                        && self.rules.units[target.kind].leader
                                    {
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
                                    },
                                )
                            })
                    })
                    .fold(0.0f64, f64::max);
                (
                    index,
                    unit,
                    before_risk,
                    offense + before_risk as f64 * danger,
                )
            })
            .collect();
        roster.sort_unstable_by(|a, b| b.3.total_cmp(&a.3).then(a.0.cmp(&b.0)));
        for (unit_order, original_unit, before_risk, _) in roster {
            let category = role(self, original_unit.kind);
            let important = self.essential(original_unit);
            if let Some(&promotion) = self.promotion_options(original_unit).first() {
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
            let reachable = self.reachable(original_unit);
            let mut destinations = Vec::with_capacity(reachable.len() + 1);
            destinations.push(original_unit.tile);
            destinations.extend(reachable.iter().copied().map(|(t, _)| t));
            destinations = self.authored_destinations(original_unit, destinations);
            let reserved = self.reserved(original_unit, &objectives);
            let required = objectives.iter().any(|g| {
                g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, original_unit.id)
            });
            let unchanged = (Vec::new(), original_unit.rebuilding);
            let (rebuilding, intent) = rebuilds.get(&original_unit.id).unwrap_or(&unchanged);
            let mut planned_unit = (*original_unit).clone();
            planned_unit.rebuilding = *intent;
            let unit = &planned_unit;
            // Only a lethal essential-actor candidate needs the full minimum.
            let mut minimum_risk = None;
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
                    && original.actor.is_some()
                    && matches!(original.kind, ObjectiveKind::Escort | ObjectiveKind::Escape)
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
                    assigned: !assisting && (original.actor.is_some() || original.family.is_some()),
                    goal,
                    route,
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

            let other_healing: Vec<_> =
                if policy == Policy::Mixed && category == TacticalRole::Healer {
                    allies
                        .iter()
                        .map(|ally| {
                            allies
                                .iter()
                                .filter(|o| {
                                    o.id != unit.id
                                        && o.id != ally.id
                                        && self.board.distance(o.tile, ally.tile) == 1
                                })
                                .map(|o| self.rules.units[o.kind].healing)
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
                        actor_matches(&g.actor, unit.id)
                            && g.tiles.contains(&self.board.ids[unit.tile])
                    })
                {
                    positioning -= 24.0;
                }
                let healing_tile = (self.terrain_is(tile, Terrain::Village)
                    || self.terrain_is(tile, Terrain::Grove))
                    && (self.owners[tile] == Some(side) || self.at(tile).is_none());
                if healing_tile {
                    positioning += (self.rules.units[unit.kind].hp - unit.hp).min(4) as f64 * 2.0;
                }
                if self.terrain_is(tile, Terrain::Village) && self.owners[tile] != Some(side) {
                    positioning += 3.0;
                }
                if policy == Policy::Mixed {
                    if category == TacticalRole::Healer {
                        let healing = self.rules.units[unit.kind].healing;
                        let mut value = 0.0;
                        for (ally_index, ally) in allies.iter().enumerate() {
                            if ally.id == unit.id || self.board.distance(tile, ally.tile) != 1 {
                                continue;
                            }
                            let other = other_healing[ally_index];
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
                        for protected in &protected {
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
                let objective_finish = self.finishes_objective(&unit, tile, &objectives);
                let choices = std::iter::once((None, None)).chain(
                    nearby_foes[nearby_offsets[tile]..nearby_offsets[tile + 1]]
                        .iter()
                        .flat_map(|&target| {
                            (0..self.rules.units[unit.kind].attacks.len())
                                .filter_map(move |weapon| Some((Some(target), Some(weapon))))
                        }),
                );
                for (choice_order, (target, weapon)) in choices.enumerate() {
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
                    if self.rules.units[unit.kind].leader
                        && !finishing
                        && !finishes_battle
                        && rebuilding.is_empty()
                        && !self.leader_may_advance(&unit, tile, recruiting, required)
                    {
                        continue;
                    }
                    let incoming = threats.at(&unit, tile, eliminated);
                    let lethal = incoming >= unit.hp - retaliation;
                    if important && lethal && incoming > 0 && !finishing && !finishes_battle && {
                        let minimum_risk = *minimum_risk.get_or_insert_with(|| {
                            threats.minimum(&unit, destinations.iter().copied())
                        });
                        minimum_risk < unit.hp
                            || incoming + retaliation > minimum_risk
                            || incoming >= before_risk
                    } {
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
                        + action_jitter(
                            seed,
                            self.round,
                            unit.id,
                            &self.board.ids[tile],
                            target.map(|target| target.id),
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
        best
    }
}

#[cfg(test)]
mod tests {
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
    fn random_stream_and_utf16_jitter_remain_stable() {
        let mut random = Random(1);
        for expected in [2693262067u32, 11749833, 2265367787] {
            assert_eq!(random.next(), expected as f64 / 4294967296.0);
        }
        assert_eq!(jitter(19, "😀"), 3961270494u32 as f64 / 4294967296.0 * 0.05);
    }
}
