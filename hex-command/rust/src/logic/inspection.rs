//! Immutable browser query publication; no work is added to simulation turns.
use super::battle::Game;
use crate::{data::*, protocol::*};

impl Game<'_> {
    fn damage_detail(&self, attack: &Weapon, defender: &Unit, tile: usize) -> DamageDetail {
        let rule = &self.rules.units[defender.kind];
        let modifier = rule.damage_modifiers.get(attack.damage_type)
            + self.equipment_modifier(defender, attack.damage_type);
        let environment = rule.environment[self.terrain[tile]][attack.category.index()]
            [attack.damage_type.index()];
        let distribution = super::damage::distribution(attack, modifier + environment);
        DamageDetail {
            roll_min: attack.damage - attack.spread,
            roll_max: attack.damage + attack.spread,
            minimum: distribution.min,
            maximum: distribution.max,
            expected: distribution.mean,
            probabilities: distribution.probabilities.clone(),
            count: attack.count,
            total: distribution.mean.round() as i32,
            base: attack.damage,
            damage_type: attack.damage_type,
            modifier,
            modified: attack.damage + modifier,
            protection: (-environment).max(0),
            exposure: environment.max(0),
            terrain: self.rules.terrain_names[self.terrain[tile]].clone(),
            damage: (attack.damage + modifier + environment).max(0),
        }
    }

    fn event_unavailable(&self, id: &str, remaining: usize) -> bool {
        let state = self.campaign.as_ref().unwrap();
        if state.marked(&format!("event:{id}")) || remaining == 0 {
            return false;
        }
        let Some(event) = self
            .mission
            .as_ref()
            .unwrap()
            .events
            .iter()
            .find(|e| e.id == id)
        else {
            return false;
        };
        event.before_round.is_some_and(|round| self.round >= round)
            || event
                .unless_event
                .as_ref()
                .is_some_and(|other| state.marked(&format!("event:{other}")))
            || event
                .requires_event
                .as_ref()
                .is_some_and(|other| self.event_unavailable(other, remaining - 1))
    }
    fn objective_status(&self, goal: &Goal) -> ObjectiveStatus {
        let progress = self.goal_progress(goal);
        let failed = !progress.done
            && (self.winner.is_some()
                || match goal {
                    Goal::Event { event, .. } => {
                        self.event_unavailable(event, self.mission.as_ref().unwrap().events.len())
                    }
                    _ => false,
                });
        ObjectiveStatus {
            active_tiles: self.active_goal_tiles(goal),
            visible: self.goal_visible(goal),
            progress,
            failed,
        }
    }
    fn campaign_objectives(&self) -> Option<CampaignObjectives> {
        let state = self.campaign.as_ref()?;
        let mission = self.mission.as_ref()?;
        let leader_alive = self
            .units
            .iter()
            .any(|u| u.side == Side::Player && u.role == Some(Role::Leader))
            || state.evacuated.iter().any(|record| {
                record.unit.side == Side::Player && record.unit.role == Some(Role::Leader)
            });
        let survival = |label: String, alive: bool| ObjectiveStatus {
            active_tiles: vec![],
            visible: true,
            progress: Progress {
                label,
                current: u32::from(self.winner.is_some() && alive),
                total: 1,
                done: self.winner.is_some() && alive,
                tiles: vec![],
            },
            failed: !alive,
        };
        let mut protected = vec![survival("Keep your leader alive.".into(), leader_alive)];
        for name in &mission.required_companions {
            let alive = self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.character.as_ref() == Some(name))
                || state
                    .evacuated
                    .iter()
                    .any(|record| record.unit.character.as_ref() == Some(name));
            protected.push(survival(
                format!("Keep {} alive.", self.character_name(name).unwrap_or(name)),
                alive,
            ));
        }
        let mut notices = self.ritual_notices();
        for loss in &mission.lose_conditions {
            if let Condition::Round { round: deadline } = &loss.condition {
                notices.push(format!(
                    "Turn {}/{} — {}",
                    self.round,
                    deadline.saturating_sub(1),
                    loss.label
                ));
            }
            protected.push(survival(
                loss.label.clone(),
                !self.event_condition_met(&loss.condition),
            ));
        }
        Some(CampaignObjectives {
            notices,
            goals: mission
                .goals
                .iter()
                .map(|g| self.objective_status(g))
                .collect(),
            optional: mission
                .optional_goals
                .iter()
                .map(|g| self.objective_status(g))
                .collect(),
            survival: protected,
        })
    }

    pub(crate) fn inspect_board(&self) -> BoardInspection {
        BoardInspection {
            costs: self
                .rules
                .units
                .iter()
                .map(|rule| {
                    (
                        rule.id.clone(),
                        self.rules
                            .terrain_names
                            .iter()
                            .cloned()
                            .zip(rule.costs.iter().copied())
                            .collect(),
                    )
                })
                .collect(),
            neighbors: self
                .board
                .ids
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    (
                        id.clone(),
                        self.board.adjacent[index]
                            .iter()
                            .map(|&next| self.board.ids[next].clone())
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn inspect(&self, revision: u32) -> BattleInspection {
        let units = self
            .units
            .iter()
            .map(|unit| {
                let mut attacks = Vec::new();
                for target in &self.units {
                    let recommended = self.best_attack(unit, target, unit.tile);
                    for weapon in self.attack_options(unit, target, unit.tile) {
                        let attack = self.weapon(unit, weapon).clone();
                        let (damage, retaliation, response) =
                            self.preview(unit, target, weapon, unit.tile);
                        let forecast = self.exchange(unit, target, weapon, unit.tile);
                        let health = |side| HpForecast {
                            minimum: forecast.hp_minimum[side],
                            maximum: forecast.hp_maximum[side],
                            expected: forecast.hp_expected[side],
                        };
                        let response = response.map(|index| self.weapon(target, index).clone());
                        let retaliation_detail = response
                            .as_ref()
                            .map(|r| self.damage_detail(r, unit, unit.tile));
                        attacks.push(AttackDetails {
                            attacker_health: health(0),
                            defender_health: health(1),
                            kill_chance: forecast.death[1],
                            death_chance: forecast.death[0],
                            retaliation_min: forecast.minimum[1],
                            retaliation_max: forecast.maximum[1],
                            healing: forecast.healing[0].round() as i32,
                            retaliation_healing: forecast.healing[1].round() as i32,
                            thorn_damage: forecast.reflection.round() as i32,
                            recommended: recommended == Some(weapon),
                            target: format!("u{}", target.id),
                            damage,
                            retaliation,
                            damage_detail: self.damage_detail(&attack, target, target.tile),
                            attack,
                            response,
                            retaliation_detail,
                        });
                    }
                }
                UnitActions {
                    name: unit
                        .character
                        .as_deref()
                        .and_then(|id| self.character_name(id))
                        .map(str::to_owned),
                    faction: self.unit_faction_view(unit),
                    unit: format!("u{}", unit.id),
                    reachable: self
                        .reachable(unit)
                        .iter()
                        .copied()
                        .map(|(tile, cost)| ReachableTile {
                            tile: TileId(self.board.ids[tile].clone()),
                            cost,
                        })
                        .collect(),
                    attacks,
                    promotions: self
                        .promotion_options(unit)
                        .into_iter()
                        .map(|kind| self.rules.units[kind].id.clone())
                        .collect(),
                    recruitment: self
                        .recruitment_options_for(unit)
                        .into_iter()
                        .map(|kind| {
                            (
                                self.rules.units[kind].id.clone(),
                                self.placement_tiles(unit, kind, true)
                                    .into_iter()
                                    .map(|tile| self.board.ids[tile].clone())
                                    .collect(),
                            )
                        })
                        .collect(),
                }
            })
            .collect();
        BattleInspection {
            speakers: self
                .campaign
                .as_ref()
                .map(|state| {
                    state
                        .dialogue
                        .iter()
                        .map(|line| {
                            line.character
                                .as_deref()
                                .and_then(|id| self.character_name(id))
                                .unwrap_or(&line.speaker)
                                .to_owned()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            objectives: self.campaign_objectives(),
            revision,
            units,
            recruitment: self
                .gold
                .keys()
                .map(|id| {
                    (
                        id.clone(),
                        self.recruitment_options_for_faction(id)
                            .into_iter()
                            .map(|kind| self.rules.units[kind].id.clone())
                            .collect(),
                    )
                })
                .collect(),
            recruitment_prices: self
                .gold
                .keys()
                .map(|id| {
                    (
                        id.clone(),
                        self.recruitment_options_for_faction(id)
                            .into_iter()
                            .map(|kind| {
                                (
                                    self.rules.units[kind].id.clone(),
                                    self.recruit_cost_for_faction(id, kind),
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
            income: Gold(std::sync::Arc::new(
                self.gold
                    .keys()
                    .map(|id| (id.clone(), self.faction_income(id)))
                    .collect(),
            )),
        }
    }
}
