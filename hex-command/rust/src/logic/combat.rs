//! Weapon legality, probabilistic forecasts and seeded combat resolution.

use super::battle::Game;
use super::exchange::{self, ExchangeForecast, Strikes};
use crate::data::*;
use std::rc::Rc;
impl<'catalog> Game<'catalog> {
    pub(crate) fn weapons<'a>(&'a self, unit: &'a Unit) -> impl Iterator<Item = &'a Weapon> {
        self.rules.units[unit.kind]
            .attacks
            .iter()
            .chain(unit.items.iter().flat_map(|item| item.attacks.iter()))
    }
    pub(crate) fn weapon<'a>(&'a self, unit: &'a Unit, index: usize) -> &'a Weapon {
        self.weapons(unit)
            .nth(index)
            .expect("validated weapon index")
    }

    pub(crate) fn can_attack(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        weapon: usize,
    ) -> bool {
        if unit.side == Side::Player
            && self.fog[target.tile].is_some()
            && self
                .presentation()
                .fog
                .contains(&self.board.ids[target.tile])
        {
            return false;
        }
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
    pub(crate) fn damage_distribution(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> std::rc::Rc<super::damage::DamageDistribution> {
        let attack = self.weapon(attacker, weapon);
        let rule = &self.rules.units[target.kind];
        super::damage::distribution(
            attack,
            rule.damage_modifiers.get(attack.damage_type)
                + self.equipment_modifier(target, attack.damage_type)
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
        let attack = self.weapon(attacker, weapon);
        let defender = &self.rules.units[target.kind];
        Strikes {
            damage: attack.damage,
            spread: attack.spread,
            count: attack.count,
            modifier: defender.damage_modifiers.get(attack.damage_type)
                + self.equipment_modifier(target, attack.damage_type)
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
        let count = self.weapon(unit, weapon).count;
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
    ) -> (i32, i32, DamageMitigationRecord) {
        let attack = self.weapon(attacker, weapon).clone();
        let armour = self.rules.units[target.kind]
            .damage_modifiers
            .get(attack.damage_type)
            + self.equipment_modifier(target, attack.damage_type);
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
        let after_armour = (raw + armour).max(0);
        let damage = (raw + armour + environment).max(0);
        let hp = target.hp.max(0);
        (
            raw,
            damage,
            DamageMitigationRecord {
                available_hp: hp,
                effective_damage: Some(damage.min(hp)),
                raw_damage: raw,
                after_armour_damage: after_armour,
                after_terrain_damage: damage,
                effective_armour_prevented: raw.min(hp) - after_armour.min(hp),
                effective_terrain_prevented: after_armour.min(hp) - damage.min(hp),
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
        let mut draws: [Vec<i32>; 2] = Default::default();
        let mut hits: [Vec<i32>; 2] = Default::default();
        let mut mitigation: [Option<DamageMitigationRecord>; 2] = [None, None];
        let mut effective_thorns = 0;
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
                let (raw, damage, record) =
                    self.roll_strike(&actor, &defender, selected, defender.tile);
                draws[side].push(raw);
                hits[side].push(damage);
                let effect = strikes[side].apply(&mut hp, side, damage);
                effective_thorns += effect.reflected;
                #[cfg(feature = "wasm")]
                {
                    let attack = self.weapon(&actor, selected);
                    let rule = &self.rules.units[defender.kind];
                    playback.push(crate::protocol::CombatStrike {
                        side: actor.side,
                        weapon: attack.name.clone(),
                        roll_min: attack.damage - attack.spread,
                        roll_max: attack.damage + attack.spread,
                        raw,
                        modifier: rule.damage_modifiers.get(attack.damage_type)
                            + self.equipment_modifier(&defender, attack.damage_type),
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
                if let Some(total) = &mut mitigation[side] {
                    total.raw_damage += record.raw_damage;
                    total.after_armour_damage += record.after_armour_damage;
                    total.after_terrain_damage += record.after_terrain_damage;
                    total.effective_armour_prevented += record.effective_armour_prevented;
                    total.effective_terrain_prevented += record.effective_terrain_prevented;
                    total.effective_damage = Some(
                        total.effective_damage.unwrap_or(0) + record.effective_damage.unwrap_or(0),
                    );
                } else {
                    mitigation[side] = Some(record);
                }
            }
            if hp.contains(&0) {
                break;
            }
        }
        let [raw_rolls, response_raw_rolls] = draws;
        let [rolls, response_rolls] = hits;
        let [attack_mitigation, response_mitigation] = mitigation;
        let damage = rolls.iter().sum::<i32>();
        let thorns = per_thorn * rolls.len() as i32;
        let retaliation = response_rolls.iter().sum::<i32>() + thorns;
        let mediated = self.mediation_at_risk(&target);
        let mitigation_record = self.recordings.as_ref().map(|_| CombatMitigationRecord {
            attack: attack_mitigation.unwrap(),
            retaliation: response_mitigation,
        });
        // Capture sources before movement, deaths or promotions can change them.
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
                            && self.allied(ally, &target)
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
        let before = [self.public_unit(&attacker), self.public_unit(&target)];
        #[cfg(feature = "wasm")]
        {
            self.combat_animation = Some(crate::protocol::CombatAnimation {
                units: before.clone(),
                strikes: playback,
            });
        }
        self.last_combat = Some(LastCombat {
            attacker_id: format!("u{id}"),
            attack_id: attack.id.clone(),
            sound: attack.sound.clone(),
        });
        self.combat_present = true;
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
        self.charge_deaths(&defeated);
        for unit in &defeated {
            for item in &unit.items {
                self.ground_items.push(GroundItem {
                    tile: self.board.ids[unit.tile].clone(),
                    item: item.clone(),
                });
            }
            if let Some(campaign) = &mut self.campaign {
                if unit.side == Side::Player {
                    if let Some(name) = &unit.name {
                        if self.rules.campaigns[&campaign.id]
                            .missions
                            .iter()
                            .any(|chapter| {
                                chapter.companion_names().any(|incoming| incoming == name)
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
            if unit.is_leader() && self.campaign.is_none() {
                let relation = self.faction_relation("player", self.unit_faction(unit));
                let remaining = self.units.iter().any(|other| {
                    other.hp > 0
                        && other.is_leader()
                        && self.faction_relation("player", self.unit_faction(other)) == relation
                });
                if !remaining {
                    self.winner = match relation {
                        Relation::Hostile => Some(Side::Player),
                        Relation::Allied => Some(Side::Enemy),
                        Relation::Neutral => None,
                    };
                }
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
