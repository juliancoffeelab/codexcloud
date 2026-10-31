//! Weapon legality, probabilistic forecasts and seeded combat resolution.

use super::battle::Game;
use super::exchange::{self, ExchangeForecast, Strikes};
use crate::data::*;
use std::rc::Rc;
struct ResolvedStrike {
    raw: i32,
    damage: i32,
    #[cfg(not(target_arch = "wasm32"))]
    mitigation: DamageMitigationRecord,
}
impl<'catalog> Game<'catalog> {
    pub(crate) fn can_attack(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        weapon: usize,
    ) -> bool {
        self.can_attack_at(unit, from, target, target.tile, weapon)
    }
    fn can_attack_at(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        target_tile: usize,
        weapon: usize,
    ) -> bool {
        if unit.side == target.side {
            return false;
        }
        let attack = &self.rules.units[unit.kind].attacks[weapon];
        self.in_attack_range(from, target_tile, attack.category, attack.range)
    }
    pub(crate) fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        (0..self.rules.units[unit.kind].attacks.len())
            .filter(|&weapon| self.can_attack(unit, from, target, weapon))
            .collect()
    }
    pub(crate) fn damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> i32 {
        self.damage_at(attacker, target, weapon, target.tile)
    }
    pub(crate) fn damage_distribution(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> std::rc::Rc<super::damage::DamageDistribution> {
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let rule = &self.rules.units[target.kind];
        super::damage::distribution(
            attack,
            rule.damage_modifiers.get(attack.damage_type)
                + rule.environment[self.terrain[tile]][attack.category.index()]
                    [attack.damage_type.index()],
        )
    }
    fn damage_at(&self, attacker: &Unit, target: &Unit, weapon: usize, tile: usize) -> i32 {
        self.damage_distribution(attacker, target, weapon, tile)
            .mean
            .round() as i32
    }
    fn strike_rules(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
        reflection: i32,
    ) -> Strikes {
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let defender = &self.rules.units[target.kind];
        Strikes {
            damage: attack.damage,
            spread: attack.spread,
            count: attack.count,
            modifier: defender.damage_modifiers.get(attack.damage_type)
                + defender.environment[self.terrain[tile]][attack.category.index()]
                    [attack.damage_type.index()],
            drain: attack.drain,
            maximum_hp: self.rules.units[attacker.kind].hp,
            reflection,
        }
    }
    pub(crate) fn exchange(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Rc<ExchangeForecast> {
        let count = self.rules.units[unit.kind].attacks[weapon].count;
        self.exchange_with_thorns(
            unit,
            target,
            weapon,
            from,
            self.thorn_damage(unit, target, weapon, from) / i32::from(count),
        )
    }
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
    pub(crate) fn response(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Option<usize> {
        let category = self.rules.units[unit.kind].attacks[weapon].category;
        self.rules.units[target.kind]
            .attacks
            .iter()
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
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let armour = self.rules.units[target.kind]
            .damage_modifiers
            .get(attack.damage_type);
        let environment = self.rules.units[target.kind].environment[self.terrain[tile]]
            [attack.category.index()][attack.damage_type.index()];
        // Full-period state, bijective output mixing and rejection sampling.
        let width = (2 * attack.spread + 1) as u32;
        let threshold = width.wrapping_neg() % width;
        let roll = loop {
            self.combat_rng = self
                .combat_rng
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            let mut sample = self.combat_rng;
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
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        if attack.category != Category::Melee {
            return 0;
        }
        let aura = self
            .units
            .iter()
            .filter(|ally| {
                ally.id != target.id
                    && ally.side == target.side
                    && self.board.distance(
                        if ally.id == attacker.id {
                            from
                        } else {
                            ally.tile
                        },
                        target.tile,
                    ) == 1
            })
            .map(|ally| self.rules.units[ally.kind].thorn_aura)
            .max()
            .unwrap_or(0);
        (self.rules.units[target.kind].thorns + aura) * i32::from(attack.count)
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
    pub(crate) fn best_attack(&self, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
        let mut best = None;
        let mut score = f64::NEG_INFINITY;
        for attack in self.attack_options(unit, target, from) {
            let result = self.exchange(unit, target, attack, from);
            let value = result.effective[0] + result.healing[0] + result.death[1] * 18.0
                - result.damage[1] * 0.7
                - result.death[0] * 40.0;
            if value > score {
                score = value;
                best = Some(attack);
            }
        }
        best
    }

    pub(crate) fn mediation_at_risk(&self, target: &Unit) -> bool {
        self.campaign
            .as_ref()
            .is_some_and(|state| !state.escalated())
            && self
                .mission
                .as_ref()
                .is_some_and(|mission| mission.refuge.is_some())
            && target.side == Side::Enemy
            && (target.retreating == Some(true) || target.role == Some(Role::Leader))
    }
    pub(crate) fn attack(&mut self, id: u32, target_id: u32, weapon: usize) {
        let a = self.index(id).unwrap();
        let b = self.index(target_id).unwrap();
        let attacker = self.units[a].clone();
        let target = self.units[b].clone();
        assert!(
            attacker.side == self.turn
                && !attacker.attacked
                && self.winner.is_none()
                && self.can_attack(&attacker, attacker.tile, &target, weapon)
        );
        let attack = self.rules.units[attacker.kind].attacks[weapon].clone();
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
        let mut totals = [0i32; 2];
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
                let rolled = self.roll_strike(&actor, &defender, selected, defender.tile);
                let (raw, damage) = (rolled.raw, rolled.damage);
                totals[side] += damage;
                counts[side] += 1;
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
                if self.presentation {
                    let attack = &self.rules.units[actor.kind].attacks[selected];
                    let rule = &self.rules.units[defender.kind];
                    playback.push(crate::protocol::CombatStrike {
                        side: actor.side,
                        weapon: attack.name.to_string(),
                        roll_min: attack.damage - attack.spread,
                        roll_max: attack.damage + attack.spread,
                        raw,
                        modifier: rule.damage_modifiers.get(attack.damage_type),
                        terrain: rule.environment[self.terrain[defender.tile]]
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
        let damage = totals[0];
        let thorns = per_thorn * counts[0] as i32;
        let retaliation = totals[1] + thorns;
        #[cfg(not(target_arch = "wasm32"))]
        let mediated = self.mediation_at_risk(&target);
        #[cfg(not(target_arch = "wasm32"))]
        let mitigation_record = self.recordings.as_ref().map(|_| CombatMitigationRecord {
            attack: attack_mitigation.unwrap(),
            retaliation: response_mitigation,
        });
        // Capture sources before movement, deaths or promotions can change them.
        #[cfg(not(target_arch = "wasm32"))]
        let thorn_record = self.recordings.as_ref().map(|_| {
            let own = if attack.category == Category::Melee {
                self.rules.units[target.kind].thorns * rolls.len() as i32
            } else {
                0
            };
            let damage = thorns;
            let aura_damage = damage - own;
            let aura_sources = if aura_damage > 0 {
                let strength = aura_damage / rolls.len() as i32;
                self.units
                    .iter()
                    .filter(|ally| {
                        ally.id != target.id
                            && ally.side == target.side
                            && self.board.distance(ally.tile, target.tile) == 1
                            && self.rules.units[ally.kind].thorn_aura == strength
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
        let need_snapshot = false;
        #[cfg(feature = "wasm")]
        let need_snapshot = need_snapshot || self.presentation;
        #[cfg(not(target_arch = "wasm32"))]
        let need_snapshot = need_snapshot || self.recordings.is_some();
        let before =
            need_snapshot.then(|| [self.public_unit(&attacker), self.public_unit(&target)]);
        #[cfg(feature = "wasm")]
        if self.presentation {
            #[cfg(target_arch = "wasm32")]
            let animation_units = before.expect("animation snapshot");
            #[cfg(not(target_arch = "wasm32"))]
            let animation_units = before.as_ref().expect("animation snapshot").clone();
            self.combat_animation = Some(crate::protocol::CombatAnimation {
                units: animation_units,
                strikes: playback,
            });
        }
        if self.presentation {
            self.last_combat = Some(LastCombat {
                attacker_id: format!("u{id}"),
                attack_id: attack.id.to_string(),
                sound: attack.sound.to_string(),
            });
            self.combat_present = true;
        }
        self.units[a].moved = true;
        self.units[a].attacked = true;
        self.units[a].hp = hp[0];
        self.units[b].hp = hp[1];
        for (own, opponent, dealt) in [(a, b, damage), (b, a, retaliation)] {
            let limit = self.rules.units[self.units[own].kind]
                .experience_needed
                .unwrap_or(0);
            if self.units[own].hp > 0 && dealt > 0 && limit > 0 {
                let reward = if self.units[opponent].hp <= 0 {
                    6 * self.rules.units[self.units[opponent].kind].level
                } else {
                    1
                };
                self.units[own].experience = Some(
                    limit.min(
                        self.units[own]
                            .experience
                            .unwrap_or(0)
                            .saturating_add(reward),
                    ),
                );
            }
        }
        if self.presentation {
            let message = if retaliation > 0 {
                format!(" and took {retaliation} in retaliation")
            } else {
                String::new()
            };
            game_log!(
                self,
                format!(
                    "{} {} used {} for {damage} damage{message}.",
                    if attacker.side == Side::Player {
                        "Your"
                    } else {
                        "Enemy"
                    },
                    self.rules.unit_log_names[attacker.kind],
                    self.rules.weapon_log_names[attacker.kind][weapon]
                )
            );
        }
        let defeated: Vec<Unit> = self
            .units
            .iter()
            .filter(|unit| unit.hp <= 0)
            .cloned()
            .collect();
        for unit in &defeated {
            if let Some(campaign) = &mut self.campaign {
                if unit.side == Side::Player {
                    if let Some(name) = &unit.name {
                        if self.rules.campaigns[&campaign.id]
                            .missions
                            .iter()
                            .any(|chapter| {
                                chapter.map.units.iter().any(|incoming| {
                                    incoming.side == Side::Player
                                        && incoming.name.as_deref() == Some(name.as_ref())
                                })
                            })
                        {
                            let mark = format!("fallen:{name}");
                            if !campaign.marked(&mark) {
                                campaign.marks.push(mark);
                            }
                        }
                    }
                }
                if campaign.id == "green-accord"
                    && unit.name.as_deref() == Some("Captain Dain")
                    && !campaign.marked("dain-fell")
                {
                    campaign.marks.push("dain-fell".into());
                }
            }
            if self.mediation_at_risk(unit) {
                self.escalate_refuge();
            }
            if self.rules.units[unit.kind].leader && self.campaign.is_none() {
                self.winner = Some(unit.side.other());
            }
            game_log!(
                self,
                format!(
                    "{} {} fell.",
                    if unit.side == Side::Player {
                        "Your"
                    } else {
                        "Enemy"
                    },
                    self.rules.unit_log_names[unit.kind]
                )
            );
        }
        self.units.retain(|unit| unit.hp > 0);
        self.roster_changed();
        self.refresh_mission();
        self.evaluate_campaign(false, None);
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Attack {
            unit: format!("u{id}"),
            target: format!("u{target_id}"),
            weapon: attack.id.to_string(),
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(records) = &mut self.recordings {
            records.losses.extend(
                before
                    .expect("recording snapshot")
                    .into_iter()
                    .filter(|unit| {
                        !self
                            .units
                            .iter()
                            .any(|alive| format!("u{}", alive.id) == unit.id)
                    }),
            );
            records.combats.push(CombatRecord {
                unit: format!("u{id}"),
                target: format!("u{target_id}"),
                weapon: attack.id.to_string(),
                from: self.board.ids[attacker.tile].clone(),
                target_tile: self.board.ids[target.tile].clone(),
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
