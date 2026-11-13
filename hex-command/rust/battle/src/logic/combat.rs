//! Weapon legality, probabilistic forecasts and seeded combat resolution.

use super::battle::BattlePosition;
use super::exchange::{self, ExchangeForecast, Strikes};
use crate::data::*;
use std::rc::Rc;
struct ResolvedStrike {
    raw: i32,
    damage: i32,
    #[cfg(not(target_arch = "wasm32"))]
    mitigation: DamageMitigationRecord,
}
/// Borrowed combat inputs resolved once for a batch of geometric origin queries.
/// Neither actor owns or clones any unit, inventory, weapon or string data.
pub struct CombatMatchup<'a> {
    actors: [CombatInputs<'a>; 2],
    target_terrain: usize,
    reply_allowed: bool,
}
struct CombatInputs<'a> {
    hp: i32,
    maximum_hp: i32,
    modifiers: [i32; 3],
    environment: &'a [[[i32; 3]; 2]],
    weapons: [&'a [WeaponRules]; 2],
}
impl CombatInputs<'_> {
    #[inline(always)]
    fn weapon(&self, index: usize) -> &WeaponRules {
        if index < self.weapons[0].len() {
            &self.weapons[0][index]
        } else {
            &self.weapons[1][index - self.weapons[0].len()]
        }
    }
    #[inline(always)]
    fn modifier(&self, attack: &WeaponRules, terrain: usize) -> i32 {
        self.modifiers[attack.damage_type.index()]
            + self.environment[terrain][attack.category.index()][attack.damage_type.index()]
    }
    #[inline(always)]
    fn strikes(&self, target: &Self, weapon: usize, terrain: usize, reflection: i32) -> Strikes {
        let attack = self.weapon(weapon);
        Strikes {
            damage: attack.damage,
            spread: attack.spread,
            count: attack.count,
            modifier: target.modifier(attack, terrain),
            drain: attack.drain,
            maximum_hp: self.maximum_hp,
            reflection,
        }
    }
}
impl CombatMatchup<'_> {
    /// `origin` must come from attack_origin_runs for these actors and weapon.
    #[inline]
    pub fn at_origin(
        &self,
        weapon: usize,
        origin: &super::geometry::AttackOriginRun,
        reflection: i32,
    ) -> Rc<ExchangeForecast> {
        let [unit, target] = &self.actors;
        let selected = unit.weapon(weapon);
        let attack = unit.strikes(target, weapon, self.target_terrain, reflection);
        let mut response = None;
        let mut response_mean = 0.0;
        if self.reply_allowed {
            for (slot, reply) in target.weapons[0]
                .iter()
                .chain(target.weapons[1].iter())
                .enumerate()
            {
                if reply.category != selected.category || u16::from(reply.range) < origin.distance {
                    continue;
                }
                if response.is_none() {
                    response = Some(slot);
                    continue;
                }
                if response_mean == 0.0 {
                    let previous = target.weapon(response.unwrap());
                    response_mean = super::damage::distribution(
                        previous,
                        unit.modifier(previous, origin.terrain),
                    )
                    .mean;
                }
                let mean =
                    super::damage::distribution(reply, unit.modifier(reply, origin.terrain)).mean;
                if mean.total_cmp(&response_mean).is_gt() {
                    response = Some(slot);
                    response_mean = mean;
                }
            }
        }
        let reply = response.map_or(
            Strikes {
                count: 0,
                reflection: 0,
                ..attack
            },
            |weapon| target.strikes(unit, weapon, origin.terrain, 0),
        );
        exchange::forecast([unit.hp, target.hp], [attack, reply])
    }
}

impl<'catalog> BattlePosition<'catalog> {
    #[inline]
    pub(crate) fn combat_matchup(&self, unit: &Unit, target: &Unit) -> CombatMatchup<'_> {
        let inputs = |unit: &Unit| {
            let stats = &self.mechanics.units[unit.kind];
            let modifiers = [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
                .map(|kind| stats.damage_modifiers.get(kind) + self.equipment_modifier(unit, kind));
            CombatInputs {
                hp: unit.hp,
                maximum_hp: stats.hp,
                modifiers,
                environment: &self.mechanics.environment
                    [stats.environment..stats.environment + self.rules.terrain_names.len()],
                weapons: [
                    self.mechanics.attacks(unit.kind),
                    if unit.inventory.0 == 0 {
                        &[]
                    } else {
                        self.runtime.inventories.attacks(unit.inventory)
                    },
                ],
            }
        };
        CombatMatchup {
            actors: [inputs(unit), inputs(target)],
            target_terrain: self.runtime.terrain[target.tile],
            reply_allowed: self.hostile(target, unit),
        }
    }
    #[inline]
    pub(crate) fn weapons<'a>(
        &'a self,
        unit: &'a Unit,
    ) -> std::iter::Chain<std::slice::Iter<'a, WeaponRules>, std::slice::Iter<'a, WeaponRules>>
    {
        self.mechanics
            .attacks(unit.kind)
            .iter()
            .chain(if unit.inventory.0 == 0 {
                [].iter()
            } else {
                self.runtime.inventories.attacks(unit.inventory).iter()
            })
    }
    #[inline]
    pub(crate) fn weapon<'a>(&'a self, unit: &'a Unit, index: usize) -> &'a WeaponRules {
        let base = self.mechanics.attacks(unit.kind);
        if index < base.len() {
            &base[index]
        } else {
            &self.runtime.inventories.attacks(unit.inventory)[index - base.len()]
        }
    }

    pub(crate) fn can_attack(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        weapon: usize,
    ) -> bool {
        // Reject mechanically impossible attacks before resolving visibility.
        // Inspection asks about every unit pair; resolving the entire map's fog
        // for friendly or out-of-range targets turns a move into a UI stall.
        if !self.can_attack_at(unit, from, target, target.tile, weapon) {
            return false;
        }
        self.unit_visible_to(self.unit_faction(unit), target, Some((unit.id, from)))
    }
    #[inline]
    fn can_attack_at(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        target_tile: usize,
        weapon: usize,
    ) -> bool {
        if !self.hostile(unit, target) {
            return false;
        }
        let attack = self.weapon(unit, weapon);
        self.in_attack_range(from, target_tile, attack.category, attack.range)
    }
    pub(crate) fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        (0..self.weapons(unit).count())
            .filter(|&weapon| self.can_attack(unit, from, target, weapon))
            .collect()
    }
    pub(crate) fn damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> i32 {
        self.damage_at(attacker, target, weapon, target.tile)
    }
    #[inline]
    pub(crate) fn damage_distribution(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> std::rc::Rc<super::damage::DamageDistribution> {
        let attack = self.weapon(attacker, weapon);
        let rule = &self.mechanics.units[target.kind];
        super::damage::distribution(
            attack,
            rule.damage_modifiers.get(attack.damage_type)
                + self.equipment_modifier(target, attack.damage_type)
                + self
                    .mechanics
                    .environment(target.kind, self.runtime.terrain[tile])[attack.category.index()]
                    [attack.damage_type.index()],
        )
    }
    fn damage_at(&self, attacker: &Unit, target: &Unit, weapon: usize, tile: usize) -> i32 {
        self.damage_distribution(attacker, target, weapon, tile)
            .mean
            .round() as i32
    }
    #[inline]
    fn strike_rules(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
        reflection: i32,
    ) -> Strikes {
        let attack = self.weapon(attacker, weapon);
        let defender = &self.mechanics.units[target.kind];
        Strikes {
            damage: attack.damage,
            spread: attack.spread,
            count: attack.count,
            modifier: defender.damage_modifiers.get(attack.damage_type)
                + self.equipment_modifier(target, attack.damage_type)
                + self
                    .mechanics
                    .environment(target.kind, self.runtime.terrain[tile])[attack.category.index()]
                    [attack.damage_type.index()],
            drain: attack.drain,
            maximum_hp: self.mechanics.units[attacker.kind].hp,
            reflection,
        }
    }
    /// `from` supplies the hypothetical attacker position without cloning its unit.
    #[inline]
    pub(crate) fn exchange(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Rc<ExchangeForecast> {
        let count = self.weapon(unit, weapon).count;
        self.exchange_with_thorns(
            unit,
            target,
            weapon,
            from,
            self.thorn_damage(unit, target, weapon, from) / i32::from(count),
        )
    }
    #[inline]
    pub(crate) fn exchange_with_thorns(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
        reflection: i32,
    ) -> Rc<ExchangeForecast> {
        let attack = self.strike_rules(unit, target, weapon, target.tile, reflection);
        let reply = self.response(unit, target, weapon, from).map_or(
            Strikes {
                count: 0,
                reflection: 0,
                ..attack
            },
            |w| self.strike_rules(target, unit, w, from, 0),
        );
        exchange::forecast([unit.hp, target.hp], [attack, reply])
    }
    #[inline]
    pub(crate) fn response(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Option<usize> {
        let category = self.weapon(unit, weapon).category;
        self.weapons(target)
            .enumerate()
            .filter(|(w, attack)| {
                attack.category == category
                    && self.can_attack_at(target, target.tile, unit, from, *w)
            })
            .max_by(|(a, _), (b, _)| {
                self.damage_distribution(target, unit, *a, from)
                    .mean
                    .total_cmp(&self.damage_distribution(target, unit, *b, from).mean)
                    .then(b.cmp(a))
            })
            .map(|(w, _)| w)
    }
    pub(crate) fn maximum_retaliation(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> i32 {
        self.exchange(unit, target, weapon, from).maximum[1]
    }
    pub(crate) fn death_chance(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> f64 {
        self.exchange(unit, target, weapon, from).death[0]
    }
    fn roll_strike(
        &mut self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> ResolvedStrike {
        let attack = self.weapon(attacker, weapon).clone();
        let armour = self.rules.units[target.kind]
            .damage_modifiers
            .get(attack.damage_type)
            + self.equipment_modifier(target, attack.damage_type);
        let environment = self.rules.units[target.kind].environment[self.runtime.terrain[tile]]
            [attack.category.index()][attack.damage_type.index()];
        // Full-period state, bijective output mixing and rejection sampling.
        let width = (2 * attack.spread + 1) as u32;
        let threshold = width.wrapping_neg() % width;
        let roll = loop {
            self.runtime.combat_rng = self
                .combat_rng
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            let mut sample = self.runtime.combat_rng;
            sample = (sample ^ (sample >> 16)).wrapping_mul(0x21f0aaad);
            sample = (sample ^ (sample >> 15)).wrapping_mul(0x735a2d97);
            sample ^= sample >> 15;
            if sample >= threshold {
                break sample % width;
            }
        };
        let raw = attack.damage - attack.spread + roll as i32;
        let damage = (raw + armour + environment).max(0);
        ResolvedStrike {
            raw,
            damage,
            #[cfg(not(target_arch = "wasm32"))]
            mitigation: {
                let after_armour = (raw + armour).max(0);
                let hp = target.hp.max(0);
                DamageMitigationRecord {
                    available_hp: hp,
                    effective_damage: Some(damage.min(hp)),
                    raw_damage: raw,
                    after_armour_damage: after_armour,
                    after_terrain_damage: damage,
                    effective_armour_prevented: raw.min(hp) - after_armour.min(hp),
                    effective_terrain_prevented: after_armour.min(hp) - damage.min(hp),
                }
            },
        }
    }

    /// Fixed reflection per incoming melee strike. It is separate
    /// from the defender's weapon, survives a killing hit and never drains.
    pub(crate) fn thorn_damage(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> i32 {
        let attack = self.weapon(attacker, weapon);
        if attack.category != Category::Melee {
            return 0;
        }
        let aura = self
            .units
            .iter()
            .filter(|ally| {
                ally.id != target.id
                    && self.allied(ally, &target)
                    && self.runtime.board.distance(
                        if ally.id == attacker.id {
                            from
                        } else {
                            ally.tile
                        },
                        target.tile,
                    ) == 1
            })
            .map(|ally| self.mechanics.units[ally.kind].thorn_aura)
            .max()
            .unwrap_or(0);
        (self.mechanics.units[target.kind].thorns + aura) * i32::from(attack.count)
    }
    pub(crate) fn preview(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> (i32, i32, Option<usize>) {
        let result = self.exchange(unit, target, weapon, from);
        let response = (result.strikes[1] > 0.0)
            .then(|| self.response(unit, target, weapon, from))
            .flatten();
        (
            result.damage[0].round() as i32,
            result.damage[1].round() as i32,
            response,
        )
    }

    pub(crate) fn attack(&mut self, id: u32, target_id: u32, weapon: usize) {
        let a = self.index(id).unwrap();
        let b = self.index(target_id).unwrap();
        let attacker = self.runtime.units[a].clone();
        let target = self.runtime.units[b].clone();
        assert!(
            attacker.phase == self.runtime.turn
                && !attacker.attack_spent
                && self.runtime.winner.is_none()
                && self.can_attack(&attacker, attacker.tile, &target, weapon)
        );
        let attack = self.weapon(&attacker, weapon).clone();
        let response = self.response(&attacker, &target, weapon, attacker.tile);
        let per_thorn =
            self.thorn_damage(&attacker, &target, weapon, attacker.tile) / i32::from(attack.count);
        let strikes = [
            self.strike_rules(&attacker, &target, weapon, target.tile, per_thorn),
            response.map_or(
                Strikes {
                    count: 0,
                    ..self.strike_rules(&attacker, &target, weapon, target.tile, 0)
                },
                |w| self.strike_rules(&target, &attacker, w, attacker.tile, 0),
            ),
        ];
        let mut hp = [attacker.hp, target.hp];
        #[cfg(feature = "wasm")]
        let mut playback = Vec::new();
        #[cfg(not(target_arch = "wasm32"))]
        let mut draws: [Vec<i32>; 2] = Default::default();
        #[cfg(not(target_arch = "wasm32"))]
        let mut hits: [Vec<i32>; 2] = Default::default();
        #[cfg(not(target_arch = "wasm32"))]
        let mut mitigation: [Option<DamageMitigationRecord>; 2] = [None, None];
        #[cfg(not(target_arch = "wasm32"))]
        let mut effective_thorns = 0;
        #[cfg(target_arch = "wasm32")]
        let mut totals = [0i32; 2];
        #[cfg(target_arch = "wasm32")]
        let mut counts = [0usize; 2];
        for index in 0..strikes[0].count.max(strikes[1].count) {
            for side in 0..2 {
                if hp.contains(&0) {
                    break;
                }
                if index >= strikes[side].count {
                    continue;
                }
                let (mut actor, mut defender, selected) = if side == 0 {
                    (attacker.clone(), target.clone(), weapon)
                } else {
                    (target.clone(), attacker.clone(), response.unwrap())
                };
                actor.hp = hp[side];
                defender.hp = hp[1 - side];
                self.reveal_weapon_user(actor.id, self.unit_vision_group(&defender));
                let rolled = self.roll_strike(&actor, &defender, selected, defender.tile);
                let (raw, damage) = (rolled.raw, rolled.damage);
                #[cfg(target_arch = "wasm32")]
                {
                    totals[side] += damage;
                    counts[side] += 1;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    draws[side].push(raw);
                    hits[side].push(damage);
                }
                let effect = strikes[side].apply(&mut hp, side, damage);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    effective_thorns += effect.reflected;
                }
                #[cfg(feature = "wasm")]
                {
                    let attack = self.weapon(&actor, selected);
                    let rule = &self.rules.units[defender.kind];
                    playback.push(crate::model::CombatStrike {
                        weapon: self.source_weapon(&attack).name.clone(),
                        weapon_id: self.source_weapon(&attack).id.clone(),
                        unit_id: format!("u{}", actor.id),
                        roll_min: attack.damage - attack.spread,
                        roll_max: attack.damage + attack.spread,
                        raw,
                        modifier: rule.damage_modifiers.get(attack.damage_type)
                            + self.equipment_modifier(&defender, attack.damage_type),
                        terrain: rule.environment[self.runtime.terrain[defender.tile]]
                            [attack.category.index()][attack.damage_type.index()],
                        damage,
                        effective: effect.effective,
                        drain: attack.drain,
                        healed: effect.healed,
                        thorns: strikes[side].reflection,
                        reflected: effect.reflected,
                        health: hp,
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let record = rolled.mitigation;
                    if let Some(total) = &mut mitigation[side] {
                        total.raw_damage += record.raw_damage;
                        total.after_armour_damage += record.after_armour_damage;
                        total.after_terrain_damage += record.after_terrain_damage;
                        total.effective_armour_prevented += record.effective_armour_prevented;
                        total.effective_terrain_prevented += record.effective_terrain_prevented;
                        total.effective_damage = Some(
                            total.effective_damage.unwrap_or(0)
                                + record.effective_damage.unwrap_or(0),
                        );
                    } else {
                        mitigation[side] = Some(record);
                    }
                }
            }
            if hp.contains(&0) {
                break;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let [raw_rolls, response_raw_rolls] = draws;
        #[cfg(not(target_arch = "wasm32"))]
        let [rolls, response_rolls] = hits;
        #[cfg(not(target_arch = "wasm32"))]
        let [attack_mitigation, response_mitigation] = mitigation;
        #[cfg(target_arch = "wasm32")]
        let damage = totals[0];
        #[cfg(not(target_arch = "wasm32"))]
        let damage = rolls.iter().sum::<i32>();
        #[cfg(target_arch = "wasm32")]
        let thorns = per_thorn * counts[0] as i32;
        #[cfg(not(target_arch = "wasm32"))]
        let thorns = per_thorn * rolls.len() as i32;
        #[cfg(target_arch = "wasm32")]
        let retaliation = totals[1] + thorns;
        #[cfg(not(target_arch = "wasm32"))]
        let retaliation = response_rolls.iter().sum::<i32>() + thorns;
        #[cfg(not(target_arch = "wasm32"))]
        let mediated = false;
        #[cfg(not(target_arch = "wasm32"))]
        let mitigation_record = self
            .runtime
            .recordings
            .as_ref()
            .map(|_| CombatMitigationRecord {
                attack: attack_mitigation.unwrap(),
                retaliation: response_mitigation,
            });
        // Capture sources before movement, deaths or promotions can change them.
        #[cfg(not(target_arch = "wasm32"))]
        let thorn_record = self.runtime.recordings.as_ref().map(|_| {
            let own = if attack.category == Category::Melee {
                self.mechanics.units[target.kind].thorns * rolls.len() as i32
            } else {
                0
            };
            let damage = thorns;
            let aura_damage = damage - own;
            let aura_sources = if aura_damage > 0 {
                let strength = aura_damage / rolls.len() as i32;
                self.runtime
                    .units
                    .iter()
                    .filter(|ally| {
                        ally.id != target.id
                            && self.allied(ally, &target)
                            && self.runtime.board.distance(ally.tile, target.tile) == 1
                            && self.mechanics.units[ally.kind].thorn_aura == strength
                    })
                    .map(|ally| format!("u{}", ally.id))
                    .collect()
            } else {
                vec![]
            };
            ThornRecord {
                damage,
                aura_damage,
                effective_damage: effective_thorns,
                aura_sources,
            }
        });
        let before = [self.public_unit(&attacker), self.public_unit(&target)];
        #[cfg(feature = "wasm")]
        {
            #[cfg(target_arch = "wasm32")]
            let animation_units = before;
            #[cfg(not(target_arch = "wasm32"))]
            let animation_units = before.clone();
            self.runtime.combat_animation = Some(crate::model::CombatAnimation {
                units: animation_units,
                strikes: playback,
            });
        }
        self.runtime.last_combat = Some(LastCombat {
            attacker_id: format!("u{id}"),
            attack_id: self.source_weapon(&attack).id.clone(),
            sound: self.source_weapon(&attack).sound.clone(),
        });
        self.runtime.combat_present = true;
        self.runtime.units[a].moved = true;
        self.runtime.units[a].attack_spent = true;
        self.runtime.units[a].hp = hp[0];
        self.runtime.units[b].hp = hp[1];
        for (own, opponent, dealt) in [(a, b, damage), (b, a, retaliation)] {
            let limit = self.rules.units[self.runtime.units[own].kind]
                .experience_needed
                .unwrap_or(0);
            if self.runtime.units[own].hp > 0 && dealt > 0 && limit > 0 {
                let reward = if self.runtime.units[opponent].hp <= 0 {
                    6 * self.rules.units[self.runtime.units[opponent].kind].level
                } else {
                    1
                };
                self.runtime.units[own].experience = Some(
                    limit.min(
                        self.runtime.units[own]
                            .experience
                            .unwrap_or(0)
                            .saturating_add(reward),
                    ),
                );
            }
        }
        let message = if retaliation > 0 {
            format!(" and took {retaliation} in retaliation")
        } else {
            String::new()
        };
        self.record(format!(
            "{} {} used {} for {damage} damage{message}.",
            if self.unit_faction(&attacker) == "player" {
                "Your"
            } else {
                "Enemy"
            },
            self.rules.units[attacker.kind].name.to_lowercase(),
            self.source_weapon(&attack).name.to_lowercase()
        ));
        let defeated: Vec<Unit> = self
            .units
            .iter()
            .filter(|unit| unit.hp <= 0)
            .cloned()
            .collect();
        self.runtime.mechanical_deaths = defeated.clone();
        for unit in &defeated {
            for item in self.runtime.inventories.item_ids(unit.inventory) {
                self.runtime.ground_items.push(GroundItemState {
                    tile: unit.tile,
                    item: *item,
                });
            }
            self.record(format!(
                "{} {} fell.",
                if self.unit_faction(unit) == "player" {
                    "Your"
                } else {
                    "Enemy"
                },
                self.rules.units[unit.kind].name.to_lowercase()
            ));
        }
        self.runtime.units.retain(|unit| unit.hp > 0);
        self.roster_changed();
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Attack {
            unit: format!("u{id}"),
            target: format!("u{target_id}"),
            weapon: self.source_weapon(&attack).id.clone(),
        });
        #[cfg(not(target_arch = "wasm32"))]
        let recorded_weapon = self
            .runtime
            .recordings
            .as_ref()
            .map(|_| self.source_weapon(&attack).id.clone());
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(records) = &mut self.runtime.recordings {
            records.losses.extend(before.into_iter().filter(|unit| {
                !self
                    .runtime
                    .units
                    .iter()
                    .any(|alive| format!("u{}", alive.id) == unit.id)
            }));
            records.combats.push(CombatRecord {
                unit: format!("u{id}"),
                target: format!("u{target_id}"),
                weapon: recorded_weapon.unwrap(),
                from: self.runtime.board.ids[attacker.tile].clone(),
                target_tile: self.runtime.board.ids[target.tile].clone(),
                damage,
                retaliation,
                count: rolls.len() as u8,
                damage_per_hit: damage / rolls.len() as i32,
                retaliation_count: response_rolls.len() as u8,
                retaliation_per_hit: if response_rolls.is_empty() {
                    0
                } else {
                    response_rolls.iter().sum::<i32>() / response_rolls.len() as i32
                },
                raw_rolls,
                rolls,
                response_raw_rolls,
                response_rolls,
                mediation_at_risk: mediated,
                thorns: thorn_record,
                mitigation: mitigation_record,
            });
        }
    }
}

impl super::Game<'_> {
    pub(crate) fn mediation_at_risk(&self, target: &Unit) -> bool {
        self.story.mission.as_ref().is_some_and(|m| {
            m.scripted_events.iter().any(|e| {
                e.objective
                    .as_ref()
                    .and_then(|o| o.preserve_lives.as_ref())
                    .is_some_and(|s| {
                        self.story
                            .scripts
                            .as_deref()
                            .is_some_and(|state| state.active.contains(&e.id))
                            && self.objective_unit_matches(
                                &e.objective.as_ref().unwrap().faction,
                                s,
                                target,
                            )
                    })
            })
        })
    }
}
