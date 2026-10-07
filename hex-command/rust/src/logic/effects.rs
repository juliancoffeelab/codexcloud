//! Feedback events derived at snapshot boundaries without simulation hot-path work.

use crate::data::*;
use std::collections::HashMap;
/// Build feedback at a host boundary; compact native simulations avoid this work.
pub(crate) fn complete_effects(
    before: &State,
    after: &State,
    command: &Command,
    effects: &mut Effects,
) {
    let mut events = Vec::new();
    if let Command::Attack {
        unit,
        target,
        weapon,
    } = command
    {
        events.push(GameEvent::CombatResolved {
            unit: *unit,
            target: *target,
            weapon: weapon.clone(),
            damage: effects.damage.unwrap_or(0),
            retaliation: effects.retaliation.unwrap_or(0),
        });
    }
    let previous: HashMap<&str, &PublicUnit> =
        before.units.iter().map(|u| (u.id.as_str(), u)).collect();
    let current: HashMap<&str, &PublicUnit> =
        after.units.iter().map(|u| (u.id.as_str(), u)).collect();
    for unit in &before.units {
        if !current.contains_key(unit.id.as_str()) {
            events.push(GameEvent::UnitDied { unit: unit.clone() });
        }
    }
    for unit in &after.units {
        let Some(old) = previous.get(unit.id.as_str()) else {
            events.push(GameEvent::UnitAdded { unit: unit.clone() });
            continue;
        };
        let Some(id) = unit
            .id
            .strip_prefix('u')
            .and_then(|id| id.parse().ok())
            .map(UnitId)
        else {
            continue;
        };
        if old.tile != unit.tile {
            events.push(GameEvent::UnitMoved {
                unit: id,
                from: TileId(old.tile.clone()),
                to: TileId(unit.tile.clone()),
            });
        }
        if old.r#type != unit.r#type {
            events.push(GameEvent::UnitAdvanced {
                unit: id,
                previous: UnitTypeId(old.r#type.clone()),
                unit_type: UnitTypeId(unit.r#type.clone()),
                hp: unit.hp,
            });
        } else if old.hp != unit.hp {
            events.push(if unit.hp > old.hp {
                GameEvent::UnitHealed {
                    unit: id,
                    amount: unit.hp - old.hp,
                    hp: unit.hp,
                }
            } else {
                GameEvent::UnitDamaged {
                    unit: id,
                    amount: old.hp - unit.hp,
                    hp: unit.hp,
                }
            });
        }
        if old.experience != unit.experience {
            events.push(GameEvent::ExperienceChanged {
                unit: id,
                previous: old.experience,
                experience: unit.experience,
            });
        }
        if (old.retreating, old.rebuilding, old.attacked)
            != (unit.retreating, unit.rebuilding, unit.attacked)
        {
            events.push(GameEvent::IntentChanged {
                unit: id,
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
                attacked: unit.attacked,
            });
        }
    }
    let previous_tiles: HashMap<&str, &Tile> =
        before.tiles.iter().map(|t| (t.id.as_str(), t)).collect();
    for tile in &after.tiles {
        if let Some(old) = previous_tiles.get(tile.id.as_str()) {
            if old.owner != tile.owner {
                events.push(GameEvent::TileOwnerChanged {
                    tile: TileId(tile.id.clone()),
                    previous: old.owner,
                    owner: tile.owner,
                });
            }
        }
    }
    for (side, previous, gold) in [
        (Side::Player, before.gold.player, after.gold.player),
        (Side::Enemy, before.gold.enemy, after.gold.enemy),
    ] {
        if previous != gold {
            events.push(GameEvent::GoldChanged {
                side,
                previous,
                gold,
            });
        }
    }
    if before.turn != after.turn || before.round != after.round {
        events.push(GameEvent::TurnBegan {
            side: after.turn,
            round: after.round,
        });
    }
    if let Some(campaign) = &after.campaign {
        let old = before
            .campaign
            .as_ref()
            .filter(|old| old.id == campaign.id && old.mission == campaign.mission);
        if old.is_none() {
            events.push(GameEvent::MissionStarted {
                campaign: CampaignId(campaign.id.clone()),
                mission: campaign.mission,
            });
        }
        let prefix = old.map_or(0, |old| {
            old.dialogue
                .iter()
                .zip(&campaign.dialogue)
                .take_while(|(a, b)| {
                    a.speaker == b.speaker && a.portrait == b.portrait && a.text == b.text
                })
                .count()
        });
        if prefix < campaign.dialogue.len() {
            events.push(GameEvent::DialogueQueued {
                lines: campaign.dialogue[prefix..].to_vec(),
            });
        }
        if old.is_some_and(|old| old.dialogue_index != campaign.dialogue_index) {
            events.push(GameEvent::DialogueAdvanced {
                index: campaign.dialogue_index,
            });
        }
    }
    if before.winner != after.winner {
        if let Some(winner) = after.winner {
            events.push(GameEvent::Victory { winner });
        }
    }
    effects.events = events;
}
