//! Weapon legality, probabilistic forecasts and seeded combat resolution.

use super::battle::BattlePosition;
use super::exchange::{self, ExchangeForecast, Strikes};
use crate::data::*;
use std::rc::Rc;
impl<'catalog> BattlePosition<'catalog> {
    #[inline]
    pub(crate) fn with_attacks<R>(
        &self,
        unit: &Unit,
        query: impl FnOnce(&[AttackHandle<'catalog>]) -> R,
    ) -> R {
        let owned = self.definitions.owned(unit, &self.runtime.inventories);
        query(&owned.attacks)
    }
    #[inline]
    pub(crate) fn with_attack<R>(
        &self,
        unit: &Unit,
        weapon: usize,
        query: impl FnOnce(AttackHandle<'_>) -> R,
    ) -> R {
        self.with_attacks(unit, |attacks| query(attacks[weapon]))
    }

    pub(crate) fn weapon_key(&self, unit: &Unit, weapon: usize) -> WeaponKey {
        self.with_attack(unit, weapon, |attack| {
            self.runtime.weapon_ids.borrow_mut().weapon_key(attack.id())
        })
    }
    pub(crate) fn weapon_index(&self, unit: &Unit, weapon: WeaponKey) -> Option<usize> {
        let id = self.runtime.weapon_ids.borrow().weapon_id(weapon)?;
        self.with_attacks(unit, |attacks| {
            attacks.iter().position(|attack| attack.id() == id)
        })
    }
    pub(crate) fn attack_outcomes_at(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> AttackOutcomes {
        let projected = [(*attacker, attacker.tile), (*target, tile)];
        self.with_domain_capabilities_at(
            attacker,
            attacker.tile,
            &self.units,
            &projected,
            super::effect_index::EffectKind::Stats,
            |attack_caps| {
                self.with_domain_capabilities_at(
                    target,
                    tile,
                    &self.units,
                    &projected,
                    super::effect_index::EffectKind::Stats,
                    |defense_caps| {
                        attack_caps.attack_outcomes(
                            attack_caps
                                .attacks()
                                .nth(weapon)
                                .expect("attack index is valid"),
                            defense_caps,
                            self.terrain_definition(tile),
                        )
                    },
                )
            },
        )
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
        self.perception(None)
            .with_unit(unit, from)
            .sees_unit(self.unit_vision_group(unit), target)
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
        self.with_attack(unit, weapon, |attack| {
            self.in_attack_range(from, target_tile, attack.category(), attack.range())
        })
    }
    pub(crate) fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        (0..self.with_attacks(unit, |attacks| attacks.len()))
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
        super::damage::distribution(self.attack_outcomes_at(attacker, target, weapon, tile))
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
        let outcomes = self.attack_outcomes_at(attacker, target, weapon, tile);
        Strikes {
            outcomes,
            count: outcomes.strike_count(),
            maximum_hp: self.definitions.unit(attacker.kind).max_hp,
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
        let mut attacker = *unit;
        attacker.tile = from;
        let outcomes = self.attack_outcomes_at(&attacker, target, weapon, target.tile);
        self.exchange_prepared(
            unit,
            target,
            weapon,
            from,
            outcomes,
            outcomes.reflection_per_strike(),
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
        let mut attacker = *unit;
        attacker.tile = from;
        let outcomes = self.attack_outcomes_at(&attacker, target, weapon, target.tile);
        self.exchange_prepared(unit, target, weapon, from, outcomes, reflection)
    }
    #[inline]
    fn exchange_prepared(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
        outcomes: AttackOutcomes,
        reflection: i32,
    ) -> Rc<ExchangeForecast> {
        let attack = Strikes {
            outcomes,
            count: outcomes.strike_count(),
            maximum_hp: self.definitions.unit(unit.kind).max_hp,
            reflection,
        };
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
        let category = self.with_attack(unit, weapon, |attack| attack.category());
        self.with_attacks(target, |attacks| {
            attacks
                .iter()
                .enumerate()
                .filter(|(w, attack)| {
                    attack.category() == category
                        && self.can_attack_at(target, target.tile, unit, from, *w)
                })
                .max_by(|(a, _), (b, _)| {
                    self.damage_distribution(target, unit, *a, from)
                        .mean
                        .total_cmp(&self.damage_distribution(target, unit, *b, from).mean)
                        .then(b.cmp(a))
                })
                .map(|(w, _)| w)
        })
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
    /// Fixed reflection per incoming melee strike. It is separate
    /// from the defender's weapon, survives a killing hit and never drains.
    pub(crate) fn thorn_damage(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> i32 {
        let mut projected = *attacker;
        projected.tile = from;
        self.with_attack(attacker, weapon, |attack| {
            self.with_selected_domain_effects_at(
                target,
                target.tile,
                &self.units,
                &[(projected, from)],
                super::effect_index::EffectKind::Thorns,
                true,
                |_, _, _| true,
                |caps| caps.reflected_damage(attack),
            )
        })
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
        let (attack_id, attack_name, attack_sound, attack_category) =
            self.with_attack(&attacker, weapon, |attack| {
                (
                    attack.id().to_owned(),
                    attack.name().to_owned(),
                    attack.sound().to_owned(),
                    attack.category(),
                )
            });
        let response = self.response(&attacker, &target, weapon, attacker.tile);
        let per_thorn = self.thorn_damage(&attacker, &target, weapon, attacker.tile)
            / i32::from(
                self.attack_outcomes_at(&attacker, &target, weapon, target.tile)
                    .strike_count(),
            );
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
                let damage = rolled.damage();
                #[cfg(not(target_arch = "wasm32"))]
                let raw = rolled.raw();
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
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let record = rolled.mitigation();
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
                #[cfg(feature = "wasm")]
                playback.push(rolled.into_playback(effect, hp));
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
        #[cfg(all(feature = "simulation", not(target_arch = "wasm32")))]
        let thorn_record = self.runtime.recordings.as_ref().map(|_| {
            let own = self.with_selected_capabilities_at(
                &target,
                target.tile,
                &self.units,
                &[],
                |source, _, _| source.is_some_and(|unit| unit.id == target.id),
                |defender| {
                    self.with_capabilities_at(&attacker, attacker.tile, &self.units, &[], |caps| {
                        caps.attack_outcomes(
                            caps.attacks().nth(weapon).expect("attack index is valid"),
                            defender,
                            self.terrain_definition(target.tile),
                        )
                        .reflection_per_strike()
                    })
                },
            ) * rolls.len() as i32;
            let damage = thorns;
            let aura_damage = damage - own;
            let support = self.support_diagnostic(
                &target,
                |_, _| true,
                |caps| hex_command_catalog::diagnostics::thorns(caps, attack_category),
            );
            let aura_sources = support.source_names(Some(target.id));
            ThornRecord {
                damage,
                aura_damage,
                effective_damage: effective_thorns,
                aura_sources,
            }
        });
        #[cfg(all(not(feature = "simulation"), not(target_arch = "wasm32")))]
        let thorn_record = None;
        let before = [self.public_unit(&attacker), self.public_unit(&target)];
        #[cfg(feature = "wasm")]
        {
            #[cfg(target_arch = "wasm32")]
            let animation_units = before;
            #[cfg(not(target_arch = "wasm32"))]
            let animation_units = before.clone();
            self.runtime.combat_animation = Some(crate::model::CombatAnimation {
                maximum_hp: [
                    self.definitions.unit(attacker.kind).max_hp,
                    self.definitions.unit(target.kind).max_hp,
                ],
                weapons: [attacker, target].map(|u| {
                    (0..self.with_attacks(&u, |attacks| attacks.len()))
                        .map(|index| self.attack_card(&u, index))
                        .collect()
                }),
                units: animation_units,
                strikes: playback,
            });
        }
        self.runtime.last_combat = Some(LastCombat {
            attacker_id: format!("u{id}"),
            attack_id: attack_id.clone(),
            sound: attack_sound.clone(),
        });
        self.runtime.combat_present = true;
        self.runtime.units[a].moved = true;
        self.runtime.units[a].attack_spent = true;
        self.runtime.units[a].hp = hp[0];
        self.runtime.units[b].hp = hp[1];
        for (own, opponent, dealt) in [(a, b, damage), (b, a, retaliation)] {
            let limit = self.definitions.unit(self.runtime.units[own].kind).max_xp;
            if self.runtime.units[own].hp > 0 && dealt > 0 && limit > 0 {
                let reward = if self.runtime.units[opponent].hp <= 0 {
                    6 * self
                        .definitions
                        .unit(self.runtime.units[opponent].kind)
                        .level()
                } else {
                    1
                };
                self.runtime.units[own].experience = Some(
                    limit.min(
                        self.runtime.units[own]
                            .experience
                            .expect("unit with XP limit has current XP")
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
            self.definitions.unit(attacker.kind).name().to_lowercase(),
            attack_name.to_lowercase()
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
                self.definitions.unit(unit.kind).name().to_lowercase()
            ));
        }
        self.runtime.units.retain(|unit| unit.hp > 0);
        self.roster_changed();
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Attack {
            unit: format!("u{id}"),
            target: format!("u{target_id}"),
            weapon: attack_id.clone(),
        });
        #[cfg(not(target_arch = "wasm32"))]
        let recorded_weapon = self.runtime.recordings.as_ref().map(|_| attack_id.clone());
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
