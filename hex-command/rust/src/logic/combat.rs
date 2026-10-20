//! Weapon legality, probabilistic forecasts and seeded combat resolution.

use super::battle::Game;
use crate::data::*;
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
    pub(crate) fn kill_chance(&self, attacker: &Unit, target: &Unit, weapon: usize) -> f64 {
        self.damage_distribution(attacker, target, weapon, target.tile)
            .chance(target.hp)
    }
    pub(crate) fn guaranteed_kill(&self, attacker: &Unit, target: &Unit, weapon: usize) -> bool {
        self.damage_distribution(attacker, target, weapon, target.tile)
            .min
            >= target.hp
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
        self.thorn_damage(unit, target, weapon, from)
            + if self.guaranteed_kill(unit, target, weapon) {
                0
            } else {
                self.response(unit, target, weapon, from)
                    .map_or(0, |w| self.damage_distribution(target, unit, w, from).max)
            }
    }
    /// Exact death probability includes killing blows suppressing the reply,
    /// integer drain healing before retaliation, and fixed reflected damage.
    pub(crate) fn death_chance(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> f64 {
        let incoming = self
            .response(unit, target, weapon, from)
            .map(|w| self.damage_distribution(target, unit, w, from));
        let thorns = self.thorn_damage(unit, target, weapon, from);
        self.damage_distribution(unit, target, weapon, target.tile)
            .outcomes()
            .map(|(damage, p)| {
                let hp = unit.hp + self.drain_healing(unit, target, weapon, damage) - thorns;
                p * if hp <= 0 {
                    1.0
                } else if damage >= target.hp {
                    0.0
                } else {
                    incoming.as_ref().map_or(0.0, |d| d.chance(hp))
                }
            })
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }
    fn roll_weapon(
        &mut self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> (Vec<i32>, Vec<i32>, DamageMitigationRecord) {
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let armour = self.rules.units[target.kind]
            .damage_modifiers
            .get(attack.damage_type);
        let environment = self.rules.units[target.kind].environment[self.terrain[tile]]
            [attack.category.index()][attack.damage_type.index()];
        let mut raw = Vec::with_capacity(attack.count as usize);
        let mut hits = Vec::with_capacity(attack.count as usize);
        let mut after_armour = 0;
        for _ in 0..attack.count {
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
            let value = attack.damage - attack.spread + roll as i32;
            raw.push(value);
            after_armour += (value + armour).max(0);
            hits.push((value + armour + environment).max(0));
        }
        let raw_damage = raw.iter().sum();
        let after_terrain_damage = hits.iter().sum();
        let hp = target.hp.max(0);
        (
            raw,
            hits,
            DamageMitigationRecord {
                available_hp: hp,
                raw_damage,
                after_armour_damage: after_armour,
                after_terrain_damage,
                effective_armour_prevented: raw_damage.min(hp) - after_armour.min(hp),
                effective_terrain_prevented: after_armour.min(hp) - after_terrain_damage.min(hp),
            },
        )
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
    pub(crate) fn drain_healing(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        damage: i32,
    ) -> i32 {
        (damage.max(0).min(target.hp.max(0))
            * i32::from(self.rules.units[unit.kind].attacks[weapon].drain)
            / 100)
            .min((self.rules.units[unit.kind].hp - unit.hp).max(0))
    }
    pub(crate) fn preview(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> (i32, i32, Option<usize>) {
        let distribution = self.damage_distribution(unit, target, weapon, target.tile);
        let damage = distribution.mean.round() as i32;
        let thorns = self.thorn_damage(unit, target, weapon, from);
        let response = if distribution.min >= target.hp {
            None
        } else {
            self.response(unit, target, weapon, from)
        };
        let retaliation = response.map_or(0.0, |w| {
            self.damage_distribution(target, unit, w, from).mean
                * (1.0 - distribution.chance(target.hp))
        });
        (damage, retaliation.round() as i32 + thorns, response)
    }
    pub(crate) fn best_attack(&self, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
        let mut best = None;
        let mut score = f64::NEG_INFINITY;
        for attack in self.attack_options(unit, target, from) {
            let (damage, retaliation, _) = self.preview(unit, target, attack, from);
            let healed = self.drain_healing(unit, target, attack, damage);
            let value = self
                .damage_distribution(unit, target, attack, target.tile)
                .effective(target.hp)
                + healed as f64
                + self.kill_chance(unit, target, attack) * 18.0
                - retaliation as f64 * 0.7
                - self.death_chance(unit, target, attack, from) * 40.0;
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
        let (raw_rolls, rolls, attack_mitigation) =
            self.roll_weapon(&attacker, &target, weapon, target.tile);
        let damage = rolls.iter().sum::<i32>();
        let response = if damage >= target.hp {
            None
        } else {
            self.response(&attacker, &target, weapon, attacker.tile)
        };
        let healed = self.drain_healing(&attacker, &target, weapon, damage);
        let mut responding_to = attacker.clone();
        responding_to.hp += healed;
        let (response_raw_rolls, response_rolls, response_mitigation) = if let Some(w) = response {
            let (raw, hits, mitigation) =
                self.roll_weapon(&target, &responding_to, w, attacker.tile);
            (raw, hits, Some(mitigation))
        } else {
            (vec![], vec![], None)
        };
        let retaliation = response_rolls.iter().sum::<i32>()
            + self.thorn_damage(&attacker, &target, weapon, attacker.tile);
        let mediated = self.mediation_at_risk(&target);
        let mitigation_record = self.recordings.as_ref().map(|_| CombatMitigationRecord {
            attack: attack_mitigation,
            retaliation: response_mitigation,
        });
        // Capture sources before movement, deaths or promotions can change them.
        let thorn_record = self.recordings.as_ref().map(|_| {
            let own = if attack.category == Category::Melee {
                self.rules.units[target.kind].thorns * i32::from(attack.count)
            } else {
                0
            };
            let damage = self.thorn_damage(&attacker, &target, weapon, attacker.tile);
            let aura_damage = damage - own;
            let aura_sources = if aura_damage > 0 {
                let strength = aura_damage / i32::from(attack.count);
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
            let available = attacker.hp + healed;
            ThornRecord {
                damage,
                aura_damage,
                effective_damage: retaliation.min(available)
                    - (retaliation - damage).min(available),
                aura_sources,
            }
        });
        let before = [self.public_unit(&attacker), self.public_unit(&target)];
        self.last_combat = Some(LastCombat {
            attacker_id: format!("u{id}"),
            attack_id: attack.id.clone(),
            sound: attack.sound.clone(),
        });
        self.combat_present = true;
        self.units[a].moved = true;
        self.units[a].attacked = true;
        self.units[b].hp -= damage;
        let healed = self.drain_healing(&attacker, &target, weapon, damage);
        self.units[a].hp += healed;
        let thorns = self.thorn_damage(&attacker, &target, weapon, attacker.tile);
        self.units[a].hp -= retaliation;
        if self.units[b].hp > 0 {
            if let Some(response) = response {
                let mut wounded = target.clone();
                wounded.hp = self.units[b].hp;
                let mut responding_to = attacker.clone();
                responding_to.hp += healed;
                self.units[b].hp +=
                    self.drain_healing(&wounded, &responding_to, response, retaliation - thorns);
            }
        }
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
        let message = if retaliation > 0 {
            format!(" and took {retaliation} in retaliation")
        } else {
            String::new()
        };
        self.record(format!(
            "{} {} used {} for {damage} damage{message}.",
            if attacker.side == Side::Player {
                "Your"
            } else {
                "Enemy"
            },
            self.rules.units[attacker.kind].name.to_lowercase(),
            attack.name.to_lowercase()
        ));
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
                                        && incoming.name.as_ref() == Some(name)
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
            self.record(format!(
                "{} {} fell.",
                if unit.side == Side::Player {
                    "Your"
                } else {
                    "Enemy"
                },
                self.rules.units[unit.kind].name.to_lowercase()
            ));
        }
        self.units.retain(|unit| unit.hp > 0);
        self.roster_changed();
        self.refresh_mission();
        self.evaluate_campaign(false, None);
        self.action(ActionRecord::Attack {
            unit: format!("u{id}"),
            target: format!("u{target_id}"),
            weapon: attack.id.clone(),
        });
        if let Some(records) = &mut self.recordings {
            records.losses.extend(before.into_iter().filter(|unit| {
                !self
                    .units
                    .iter()
                    .any(|alive| format!("u{}", alive.id) == unit.id)
            }));
            records.combats.push(CombatRecord {
                unit: format!("u{id}"),
                target: format!("u{target_id}"),
                weapon: attack.id,
                from: self.board.ids[attacker.tile].clone(),
                target_tile: self.board.ids[target.tile].clone(),
                damage,
                retaliation,
                count: attack.count,
                damage_per_hit: damage / i32::from(attack.count),
                raw_rolls,
                rolls,
                response_raw_rolls,
                response_rolls,
                retaliation_count: response
                    .map_or(0, |w| self.rules.units[target.kind].attacks[w].count),
                retaliation_per_hit: response.map_or(0, |w| {
                    retaliation / i32::from(self.rules.units[target.kind].attacks[w].count)
                }),
                mediation_at_risk: mediated,
                thorns: thorn_record,
                mitigation: mitigation_record,
            });
        }
    }
}
