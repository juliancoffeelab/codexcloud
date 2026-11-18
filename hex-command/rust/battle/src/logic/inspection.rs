//! Immutable browser query publication; no work is added to simulation turns.
use super::battle::LiveBattle;
use crate::data::presentation::ConditionContext;
use crate::logic::conditions::LiveContext;
use crate::logic::event_conditions::{ConditionTree, ScriptTree};
use crate::logic::event_queries::{EventFacts, reference_site};
use crate::{data::*, model::*};

fn campaign_objectives(game: &LiveBattle<'_>) -> Option<CampaignObjectives> {
    game.story.campaign.as_ref()?;
    let mission = game.story.mission.as_ref()?;
    let leader_alive = EventFacts::new(game).campaign_leader_survives("player");
    let survival = |label: String, alive: bool| ObjectiveStatus {
        active_tiles: vec![],
        visible: true,
        available: true,
        progress: Progress {
            label,
            current: u32::from(game.runtime.winner.is_some() && alive),
            total: 1,
            done: game.runtime.winner.is_some() && alive,
            tiles: vec![],
        },
        failed: !alive,
    };
    let mut protected = vec![survival("Keep your leader alive.".into(), leader_alive)];
    for name in &mission.required_companions {
        let alive = EventFacts::new(game).campaign_companion_survives("player", name);
        protected.push(survival(
            format!("Keep {} alive.", character_name(game, name).unwrap_or(name)),
            alive,
        ));
    }
    let status = |optional| {
        mission
            .scripted_events
            .iter()
            .filter(|e| {
                e.objective
                    .as_ref()
                    .is_some_and(|o| o.faction == "player" && o.optional == optional)
            })
            .map(|e| scripted_objective_status(game, e))
            .collect()
    };
    Some(CampaignObjectives {
        goals: status(false),
        optional: status(true),
        survival: protected,
    })
}

pub(crate) fn inspect_board(game: &super::BattlePosition<'_>) -> BoardInspection {
    BoardInspection {
        neighbors: game
            .board
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| {
                (
                    id.to_string(),
                    game.runtime.board.adjacent[index]
                        .iter()
                        .map(|&next| game.runtime.board.ids[next].clone())
                        .collect(),
                )
            })
            .collect(),
    }
}

pub(crate) fn inspect(game: &LiveBattle<'_>, revision: u32) -> BattleInspection {
    let units = game
        .units
        .iter()
        .map(|unit| {
            let mut attacks = Vec::new();
            for target in &game.runtime.units {
                for weapon in game.attack_options(unit, target, unit.tile) {
                    let attack = game.attack_card(unit, weapon);
                    let (damage, retaliation, response) =
                        game.preview(unit, target, weapon, unit.tile);
                    let forecast = game.exchange(unit, target, weapon, unit.tile);
                    let health = |side| HpForecast {
                        minimum: forecast.hp_minimum[side],
                        maximum: forecast.hp_maximum[side],
                        expected: forecast.hp_expected[side],
                    };
                    let retaliation_detail =
                        response.map(|index| game.damage_detail(target, unit, index));
                    let response = response.map(|index| game.attack_card(target, index));
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
                        recommended: false,
                        target: format!("u{}", target.id),
                        damage,
                        retaliation,
                        damage_detail: game.damage_detail(unit, target, weapon),
                        attack,
                        response,
                        retaliation_detail,
                    });
                }
            }
            UnitActions {
                details: game.unit_details(unit),
                promotion_details: game
                    .promotion_options(unit)
                    .into_iter()
                    .map(|kind| game.unit_details(&game.promotion(unit, kind)))
                    .collect(),
                name: game
                    .name(unit.character)
                    .and_then(|id| character_name(game, id))
                    .map(str::to_owned),
                faction: game.unit_faction_view(unit),
                unit: format!("u{}", unit.id),
                reachable: game
                    .observed_reachable(unit)
                    .iter()
                    .copied()
                    .map(|(tile, cost)| ReachableTile {
                        tile: game.runtime.board.ids[tile],
                        cost,
                    })
                    .collect(),
                attacks,
                promotions: game
                    .promotion_options(unit)
                    .into_iter()
                    .map(|kind| game.definitions.unit(kind).id().to_owned())
                    .collect(),
                recruitment: game
                    .recruitment_options_for(unit)
                    .into_iter()
                    .map(|kind| {
                        (
                            game.definitions.unit(kind).id().to_owned(),
                            game.placement_tiles(unit, kind, true)
                                .into_iter()
                                .map(|tile| game.runtime.board.ids[tile].clone())
                                .collect(),
                        )
                    })
                    .collect(),
            }
        })
        .collect();
    BattleInspection {
        tiles: (0..game.terrain.len())
            .map(|tile| game.tile_details(tile))
            .collect(),
        recruit_details: game
            .gold
            .keys()
            .map(|id| {
                let details = game
                    .units
                    .iter()
                    .find(|u| game.unit_faction(u) == id && u.role == Some(Role::Leader))
                    .map(|leader| {
                        game.recruitment_options_for_faction(id)
                            .into_iter()
                            .map(|kind| {
                                game.unit_details(&game.prospective(
                                    kind,
                                    leader.faction,
                                    leader.tile,
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (id.clone(), details)
            })
            .collect(),
        story: LiveContext::new(&game, None, None).resolved_story(),
        speakers: game
            .story
            .campaign
            .as_ref()
            .map(|state| {
                state
                    .dialogue
                    .iter()
                    .map(|line| {
                        line.character
                            .as_deref()
                            .and_then(|id| character_name(game, id))
                            .unwrap_or(&line.speaker)
                            .to_owned()
                    })
                    .collect()
            })
            .unwrap_or_default(),
        objectives: campaign_objectives(game),
        revision,
        units,
        recruitment: game
            .gold
            .keys()
            .map(|id| {
                (
                    id.clone(),
                    game.recruitment_options_for_faction(id)
                        .into_iter()
                        .map(|kind| game.definitions.unit(kind).id().to_owned())
                        .collect(),
                )
            })
            .collect(),
        recruitment_prices: game
            .gold
            .keys()
            .map(|id| {
                (
                    id.clone(),
                    game.recruitment_options_for_faction(id)
                        .into_iter()
                        .map(|kind| {
                            (
                                game.definitions.unit(kind).id().to_owned(),
                                game.recruit_cost_for_faction(id, kind),
                            )
                        })
                        .collect(),
                )
            })
            .collect(),
        income: Gold(std::sync::Arc::new(
            game.runtime
                .gold
                .keys()
                .map(|id| (id.clone(), game.faction_income(id)))
                .collect(),
        )),
    }
}

impl super::BattlePosition<'_> {
    pub(crate) fn attack_card(&self, unit: &Unit, weapon: usize) -> AttackCard {
        self.with_capabilities_at(unit, unit.tile, &self.units, &[], |caps| {
            let attack = caps.attacks().nth(weapon).expect("attack index is valid");
            let values = caps.attack_description(attack);
            AttackCard {
                id: attack.id().into(),
                name: attack.name().into(),
                sound: attack.sound().into(),
                category: attack.category(),
                damage_type: attack.damage_type(),
                range: attack.range(),
                damage: values.damage,
                spread: values.spread,
                count: values.count,
                drain: values.drain,
            }
        })
    }
    pub(crate) fn tile_details(&self, tile: usize) -> TileDetails {
        let terrain = self.terrain_definition(tile);
        TileDetails {
            tile: self.board.ids[tile],
            income: terrain.income(),
            blocks_shots: terrain.blocks_shots(),
            capturable: terrain.capturable(),
        }
    }
    pub(crate) fn unit_details(&self, unit: &Unit) -> UnitDetails {
        let definition = self.definitions.unit(unit.kind);
        let wiki = self.catalogue.wiki().unit(definition.id());
        let (damage_modifiers, effects, attacks) =
            self.with_capabilities_at(unit, unit.tile, &self.units, &[], |caps| {
                let armour = |kind| {
                    caps.protection(self.terrain_definition(unit.tile), Category::Melee, kind)
                        .armour
                };
                let effects = caps
                    .effects()
                    .map(|effect| ScopedEffect {
                        scope: effect.scope().clone(),
                        stacking: effect.stacking().clone(),
                        effect: effect.value().clone(),
                    })
                    .collect();
                let attacks = (0..caps.attacks().count())
                    .map(|index| self.attack_card(unit, index))
                    .collect();
                (
                    DamageModifiers {
                        blade: armour(DamageType::Blade),
                        pierce: armour(DamageType::Pierce),
                        impact: armour(DamageType::Impact),
                    },
                    effects,
                    attacks,
                )
            });
        // Non-stat emissions cannot alter terrain protection or path cost.
        // Terrain healing explicitly excludes every unit emitter below.
        let terrain_providers: &[Unit] =
            if self.effect_sources.movement || self.effect_sources.damage {
                &self.units
            } else {
                &[]
            };
        let spatial_stats = self.effect_sources.movement || self.effect_sources.damage;
        let mut terrain_mechanics = std::collections::HashMap::new();
        let terrain = (0..self.terrain.len())
            .map(|tile| {
                let definition = self.terrain_definition(tile);
                let cached = (!spatial_stats)
                    .then(|| terrain_mechanics.get(&definition.id()).copied())
                    .flatten();
                let (cost, defense) = cached.unwrap_or_else(|| {
                    let value =
                        self.with_capabilities_at(unit, tile, terrain_providers, &[], |caps| {
                            let defense = [Category::Melee, Category::Ranged].map(|category| {
                                [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
                                    .map(|kind| caps.protection(definition, category, kind).terrain)
                            });
                            (caps.step_cost(definition), defense)
                        });
                    if !spatial_stats {
                        terrain_mechanics.insert(definition.id(), value);
                    }
                    value
                });
                let healing = self.with_selected_capabilities_at(
                    unit,
                    tile,
                    &[],
                    &[],
                    |source, _, _| source.is_none(),
                    |caps| caps.healing(),
                );
                UnitTileDetails {
                    tile: self.board.ids[tile],
                    terrain: definition.id(),
                    name: definition.name().into(),
                    cost,
                    defense,
                    healing,
                    income: definition.income(),
                }
            })
            .collect();
        UnitDetails {
            id: definition.id().into(),
            name: definition.name().into(),
            description: definition.description().into(),
            ability: wiki.ability_label,
            level: definition.level(),
            hp: definition.max_hp,
            r#move: self.movement_limit(unit),
            cost: self.recruit_cost(unit, unit.kind),
            experience_needed: (definition.max_xp != 0).then_some(definition.max_xp),
            attacks,
            damage_modifiers,
            effects,
            terrain,
        }
    }
    fn damage_detail(&self, attacker: &Unit, defender: &Unit, weapon: usize) -> DamageDetail {
        let outcomes = self.attack_outcomes_at(attacker, defender, weapon, defender.tile);
        let (low, high) = outcomes.bounds();
        let protection = outcomes.protection();
        let distribution = super::damage::distribution(outcomes);
        let base = i32::try_from((i64::from(low.raw) + i64::from(high.raw)) / 2)
            .expect("attack centre fits i32");
        DamageDetail {
            roll_min: low.raw,
            roll_max: high.raw,
            minimum: distribution.min,
            maximum: distribution.max,
            expected: distribution.mean,
            probabilities: distribution.probabilities.clone(),
            count: outcomes.strike_count(),
            total: distribution.mean.round() as i32,
            base,
            damage_type: self.with_attack(attacker, weapon, |attack| attack.damage_type()),
            modifier: protection.armour,
            modified: base + protection.armour,
            protection: (-protection.terrain).max(0),
            exposure: protection.terrain.max(0),
            terrain: self.terrain_definition(defender.tile).id().as_str().into(),
            damage: (base + protection.armour + protection.terrain).max(0),
        }
    }
}

fn character_name<'a>(game: &'a LiveBattle<'_>, id: &str) -> Option<&'a str> {
    game.story
        .campaign_definition
        .as_ref()
        .and_then(|campaign| campaign.characters().iter().find(|c| c.id == id))
        .or_else(|| {
            game.catalogue
                .map(&game.map_id, game.map_revision)
                .characters()
                .iter()
                .find(|c| c.id == id)
        })
        .map(|c| c.name.as_str())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn event_goal_progress(game: &LiveBattle<'_>) -> Vec<Progress> {
    game.story.mission.as_ref().map_or_else(Vec::new, |m| {
        m.scripted_events
            .iter()
            .filter(|e| {
                e.objective
                    .as_ref()
                    .is_some_and(|o| o.faction == "player" && !o.optional)
            })
            .map(|e| EventFacts::new(game).objective_progress(e))
            .collect()
    })
}

fn scripted_objective_status(game: &LiveBattle<'_>, event: &ScriptEvent) -> ObjectiveStatus {
    let objective = event.objective.as_ref().unwrap();
    let state = game.story.scripts.as_deref().unwrap();
    let facts = EventFacts::new(game);
    let failed = state.failed.iter().any(|id| id == event.objective_id());
    let progress = facts.objective_progress(event);
    let done = progress.done;
    let available = facts.objective_available(event);
    let visible = state.revealed.iter().any(|id| id == event.objective_id());
    // Markers are authored UI data, never inferred from AI movement or completion regions.
    let markers = objective
        .player_marker
        .as_ref()
        .map_or_else(Vec::new, |marker| {
            let destinations: Vec<Hex> = if let Some(select) = &marker.units {
                game.runtime
                    .units
                    .iter()
                    .filter(|unit| {
                        EventFacts::for_faction(game, &objective.faction)
                            .script_unit_matches(select, unit)
                    })
                    .filter(|unit| {
                        marker.visible_in_fog
                            || game.unit_visible_to(&objective.faction, unit, None)
                    })
                    .map(|unit| game.position.runtime.board.ids[unit.tile].clone())
                    .collect()
            } else {
                marker
                    .tiles
                    .iter()
                    .map(|name| {
                        game.position.runtime.board.ids[reference_site(&game.board, name)].clone()
                    })
                    .collect()
            };
            destinations
                .into_iter()
                .filter(|tile| {
                    marker.visible_in_fog
                        || super::conditions::LiveContext::for_faction(
                            game,
                            &objective.faction,
                            None,
                        )
                        .visible_at(
                            game.position.runtime.board.positions[game.site(tile)],
                            &objective.faction,
                        )
                })
                .collect()
        });
    ObjectiveStatus {
        active_tiles: if !done
            && !failed
            && visible
            && available
            && game.position.runtime.winner.is_none()
        {
            markers
        } else {
            vec![]
        },
        visible,
        available,
        failed,
        progress,
    }
}
