//! Deterministic weapon legality, damage, retaliation and combat resolution.

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
    fn damage_at(&self, attacker: &Unit, target: &Unit, weapon: usize, target_tile: usize) -> i32 {
        self.damage_with_bonus(attacker, target, weapon, target_tile, 0)
    }
    fn damage_with_bonus(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        target_tile: usize,
        bonus: i32,
    ) -> i32 {
        let attack = &self.rules.units[attacker.kind].attacks[weapon];
        let rule = &self.rules.units[target.kind];
        (attack.damage
            + bonus
            + rule.damage_modifiers.get(attack.damage_type)
            + rule.environment[self.terrain[target_tile]][attack.category.index()]
                [attack.damage_type.index()])
        .max(0)
            * i32::from(attack.count)
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
        let damage = self.damage(unit, target, weapon);
        let thorns = self.thorn_damage(unit, target, weapon, from);
        if target.hp <= damage {
            return (damage, thorns, None);
        }
        let category = self.rules.units[unit.kind].attacks[weapon].category;
        let mut best = None;
        let mut response = 0;
        for attack in 0..self.rules.units[target.kind].attacks.len() {
            if !self.can_attack_at(target, target.tile, unit, from, attack) {
                continue;
            }
            if self.rules.units[target.kind].attacks[attack].category != category {
                continue;
            }
            let dealt = self.damage_with_bonus(target, unit, attack, from, 0);
            if best.is_none() || dealt > response {
                best = Some(attack);
                response = dealt;
            }
        }
        (damage, response + thorns, best)
    }
    pub(crate) fn best_attack(&self, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
        let mut best = None;
        let mut score = f64::NEG_INFINITY;
        for attack in self.attack_options(unit, target, from) {
            let (damage, retaliation, _) = self.preview(unit, target, attack, from);
            let healed = self.drain_healing(unit, target, attack, damage);
            let value =
                damage as f64 + healed as f64 + if damage >= target.hp { 18.0 } else { 0.0 }
                    - retaliation as f64 * 0.7
                    - if retaliation >= unit.hp + healed {
                        40.0
                    } else {
                        0.0
                    };
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
        let (damage, retaliation, response) =
            self.preview(&attacker, &target, weapon, attacker.tile);
        let mediated = self.mediation_at_risk(&target);
        let attack = self.rules.units[attacker.kind].attacks[weapon].clone();
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
                retaliation_count: response
                    .map_or(0, |w| self.rules.units[target.kind].attacks[w].count),
                retaliation_per_hit: response.map_or(0, |w| {
                    retaliation / i32::from(self.rules.units[target.kind].attacks[w].count)
                }),
                mediation_at_risk: mediated,
            });
        }
    }
}
