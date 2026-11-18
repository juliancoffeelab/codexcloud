//! Build owned runtime state from immutable authored map definitions.
use crate::data::*;
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn initial_state(catalogue: &Catalogue, map: &MapDefinition) -> State {
    let tiles = map
        .tiles()
        .iter()
        .map(|source| {
            let mut perception = TilePerception {
                illumination: None,
                remember: false,
                concealment: Vec::new(),
            };
            let mut light = None;
            let mut effects = Vec::new();
            for id in &source.environment_effects {
                for effect in catalogue.environment_effect(id).effects() {
                    match effect {
                        EnvironmentEffect::AmbientLight { level } => assert!(
                            perception.illumination.replace(*level).is_none(),
                            "tile has at most one ambient-light contribution"
                        ),
                        EnvironmentEffect::Concealment { rule } => {
                            perception.concealment.push(rule.clone())
                        }
                        EnvironmentEffect::RememberTerrain { remember } => {
                            perception.remember |= *remember
                        }
                        EnvironmentEffect::Light { light: emission } => assert!(
                            light.replace(Arc::new(emission.clone())).is_none(),
                            "tile has at most one light fixture"
                        ),
                        EnvironmentEffect::Unit { effect } => effects.push(effect.clone()),
                    }
                }
            }
            Tile {
                id: Hex::new(source.col, source.row).to_string(),
                location: source.id.clone(),
                col: source.col,
                row: source.row,
                terrain: source.terrain.as_str().to_owned(),
                owner: source.owner.clone(),
                connections: source.connections.clone(),
                dead_ends: source.dead_ends.clone(),
                walls: source.walls.clone(),
                floor: source.floor.clone(),
                perception,
                light,
                effects,
            }
        })
        .collect();
    let roster = map.roster();
    let factions = roster
        .factions
        .iter()
        .map(|army| {
            let source = &army.faction;
            Faction {
                id: source.id.clone(),
                vision_group: source.vision_group.clone(),
                phase: source.phase,
                controller: source.controller.clone(),
                palette: source.palette.clone(),
                name: source.name.clone(),
                recruits: source.recruits.clone(),
                relations: source.relations.clone(),
                transfer: source.transfer.clone(),
            }
        })
        .collect();
    let units: Vec<_> = roster
        .factions
        .iter()
        .flat_map(|army| {
            army.units.iter().map(|source| {
                let hp = match &source.hp {
                    InitialHealth::Extent(FullHealth::Full) => {
                        catalogue.unit(&source.unit_type).max_hp
                    }
                    InitialHealth::Wounded(hp) => *hp,
                };
                PublicUnit {
                    id: source.id.clone(),
                    r#type: source.unit_type.clone(),
                    faction: army.faction.id.clone(),
                    tile: source.tile,
                    hp,
                    moved: source.moved,
                    attack_spent: source.attack_spent,
                    experience: Some(source.experience),
                    character: source.character.clone(),
                    role: source.role,
                    retreating: source.retreating,
                    rebuilding: source.rebuilding,
                    ai_group: source.ai_group.clone(),
                    items: source.items.clone(),
                    perception_effects: source.perception_effects.clone(),
                    revealed_to: Vec::new(),
                }
            })
        })
        .collect();
    let mut next_id = 1u32;
    for unit in &units {
        let id = unit
            .id
            .strip_prefix('u')
            .expect("authored unit ID has u prefix")
            .parse::<u32>()
            .expect("authored unit ID has numeric suffix");
        next_id = next_id.max(id.checked_add(1).expect("unit ID space available"));
    }
    let gold: BTreeMap<_, _> = roster
        .factions
        .iter()
        .map(|army| (army.faction.id.clone(), army.gold))
        .collect();
    let (cols, rows) = map.dimensions();
    State {
        map_modifiers: map.modifiers().clone(),
        lighting: map.lighting().clone(),
        vision_groups: roster
            .vision_groups
            .iter()
            .map(|g| VisionGroupState {
                id: g.id.clone(),
                sight: g.sight.clone(),
                explored: g.explored.clone(),
            })
            .collect(),
        factions,
        ground_items: Vec::new(),
        armies: None,
        condition_memory: Vec::new(),
        map_id: map.id().to_owned(),
        map_revision: map.revision(),
        cols,
        rows,
        round: roster.round,
        turn: roster.turn,
        gold: Gold(Arc::new(gold)),
        winner: None,
        next_id,
        combat_rng: roster.combat_rng,
        log: roster.log.clone(),
        tiles,
        units,
        campaign: None,
        last_combat: Some(None),
    }
}
