//! Group knowledge and physical forecasts share one detection contract.
use super::*;
use crate::catalogue_fixture as fixture;
use crate::{BattleSession, BattleView};
use serde_json::json;
use std::rc::Rc;

fn corridor(radius: u16, remember: bool) -> State {
    let mut state = LiveBattle::new(rules(), "mirror").state();
    state.turn = TurnPhase::Ai;
    for id in ["alpha", "beta"] {
        state.factions.push(serde_json::from_value(serde_json::json!({
            "id":id,"phase":"ai","visionGroup":id,"controller":{"kind":"ai","policy":null},"palette":"enemy","recruits":null,"transfer":null,"relations":{"player":"hostile"}
        })).unwrap());
        state.vision_groups.push(VisionGroupState {
            id: id.into(),
            sight: SightContribution {
                unlimited: true,
                sight_modifiers: SightModifiers {
                    lit: 0,
                    unlit: 0,
                    darkness: 0,
                },
                penetrates: vec![],
            },
            explored: vec![],
        });
        state.gold.insert(id.into(), 0);
    }
    let ids: Vec<_> = state.factions.iter().map(|f| f.id.clone()).collect();
    for f in &mut state.factions {
        for other in &ids {
            if !f.relations.contains_key(other) {
                let relation = if &f.id == other {
                    Relation::Allied
                } else if f.id == "player" {
                    Relation::Hostile
                } else {
                    Relation::Neutral
                };
                f.relations.insert(other.clone(), relation);
            }
        }
    }
    let template = state.tiles[0].clone();
    state.tiles = (0..7)
        .map(|col| Tile {
            id: format!("{col},0"),
            col,
            row: 0,
            location: None,
            owner: None,
            terrain: "plains".into(),
            connections: None,
            walls: None,
            dead_ends: None,
            perception: TilePerception {
                remember,
                concealment: vec![ConcealmentRule {
                    kind: ConcealmentKind::Fog,
                    reveal_within: radius,
                    requires: ConcealmentRequirements {
                        terrains: None,
                        illumination: None,
                    },
                }],
                illumination: Some(Illumination::Lit),
            },
            ..template.clone()
        })
        .collect();
    state.cols = 7;
    state.rows = 1;
    let unit = state.units[0].clone();
    state.units = [("alpha", 0), ("beta", 6)]
        .into_iter()
        .enumerate()
        .map(|(i, (faction, col))| PublicUnit {
            id: format!("u{}", i + 1),
            faction: faction.into(),
            tile: Hex::new(col, 0),
            role: None,
            r#type: "archer".into(),
            hp: rules().unit("archer").max_hp,
            ..unit.clone()
        })
        .collect();
    state.next_id = 3;
    state
}

#[test]
fn group_history_and_known_positions_are_independent_of_turn_phase() {
    let mut game = LiveBattle::from_state(rules(), corridor(1, true));
    assert!(game.tile_visible_to("alpha", 0));
    assert!(!game.tile_visible_to("beta", 0));
    let quiet = game.fork();
    game.remember_conditions();
    assert!(Rc::ptr_eq(&game.vision_groups, &quiet.vision_groups));
    assert!(Rc::ptr_eq(&game.fog_memory, &quiet.fog_memory));
    game.units[0].tile = 3;
    game.roster_changed();
    game.remember_conditions();
    assert!(game.tile_visible_to("alpha", 0));
    assert!(!game.tile_visible_to("beta", 3));
    let mut session = BattleSession::from_state(rules(), game.state());
    session.set_active_faction(Some("alpha".into()));
    let view = BattleView::new(&session);
    assert_eq!(view.observer_faction(), "alpha");
    assert_eq!(view.units().len(), 1);
    assert!(!view.occupied_tiles().contains(6));
    let projected = session.phase_forecasts().next_phase_without(None);
    assert_eq!(projected.observer_faction(), "alpha");
    assert_eq!(projected.units().len(), 1);
    game.assign_vision_group("beta", "alpha");
    assert!(game.tile_visible_to("alpha", 6));
    assert!(game.tile_visible_to("beta", 0));
    let restored = LiveBattle::from_state(
        rules(),
        serde_json::from_value(serde_json::to_value(game.state()).unwrap()).unwrap(),
    );
    assert_eq!(restored.state().vision_groups, game.state().vision_groups);
}

#[test]
fn concealment_and_attack_availability_do_not_replace_combat_reveals() {
    let mut state = corridor(6, false);
    state.units[0].tile = Hex::new(0, 0);
    state.units[1].tile = Hex::new(3, 0);
    state
        .factions
        .iter_mut()
        .find(|f| f.id == "alpha")
        .unwrap()
        .relations
        .insert("beta".into(), Relation::Hostile);
    let mut bundle = fixture::deterministic();
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["attacks"][0]["range"] = json!(3)
    });
    crate::effect_fixture::concealment(
        &mut bundle,
        "archer",
        vec![ConcealmentRule {
            kind: ConcealmentKind::ForestCamouflage,
            reveal_within: 1,
            requires: ConcealmentRequirements {
                terrains: None,
                illumination: None,
            },
        }],
    );
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let mut game = LiveBattle::from_state(&local, state);
    let alpha = game.units[0];
    let beta = game.units[1];
    assert!(!game.unit_visible_to("alpha", &beta, None));
    assert!(!game.unit_visible_to("beta", &alpha, None));
    game.turn = TurnPhase::Ai;
    game.execute_runtime(crate::RuntimeCommand::SetUnitIntent {
        unit: UnitId(alpha.id),
        retreating: Some(true),
        rebuilding: None,
        attack_spent: true,
        record_retreat: false,
    })
    .unwrap();
    assert!(!game.unit_visible_to("beta", &alpha, None));
    game.units[0].attack_spent = false;
    game.reveal_weapon_user(beta.id, alpha.vision_group);
    let weapon = game.attack_options(&game.units[0], &game.units[1], 0)[0];
    game.attack(alpha.id, beta.id, weapon);
    if let Some(alpha) = game.unit(alpha.id) {
        assert!(game.unit_visible_to("beta", alpha, None));
    }
    game.begin_turn(TurnPhase::Ai);
    if let Some(alpha) = game.unit(alpha.id) {
        assert!(!game.unit_visible_to("beta", alpha, None));
    }
}

#[test]
fn sight_modifiers_follow_target_light_for_sight_and_threats() {
    let mut bundle = fixture::deterministic();
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["unlimited"] = json!(false)
    });
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = serde_json::to_value(SightModifiers {
            lit: 7,
            unlit: 4,
            darkness: 4,
        })
        .unwrap()
    });
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["attacks"][0]["range"] = json!(6)
    });
    let mut state = corridor(0, false);
    for group in &mut state.vision_groups {
        group.sight.unlimited = false;
    }
    state.lighting.time = TimeOfDay::Night;
    for tile in &mut state.tiles {
        tile.perception = TilePerception {
            illumination: Some(Illumination::Unlit),
            remember: false,
            concealment: vec![],
        };
        tile.light = None;
    }
    let light: std::sync::Arc<TileLight> = std::sync::Arc::new(
        serde_json::from_value(serde_json::json!({
            "emission":{"radius":0},"radius":64.0,"power":1.0
        }))
        .unwrap(),
    );
    state.tiles[6].light = Some(light.clone());
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let lit_target = LiveBattle::from_state(&local, state.clone());
    assert!(lit_target.tile_in_sight_to("alpha", 6));
    assert!(lit_target.unit_visible_to("alpha", &lit_target.units[1], None));
    assert!(lit_target.tile_in_sight_to("alpha", 4));
    assert!(!lit_target.tile_in_sight_to("alpha", 5));

    // Lighting the observer does not restore sight of an unlit distant target.
    state.tiles[6].light = None;
    state.tiles[0].light = Some(light.clone());
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let lit_observer = LiveBattle::from_state(&local, state.clone());
    assert!(!lit_observer.tile_in_sight_to("alpha", 6));
    assert!(!lit_observer.unit_visible_to("alpha", &lit_observer.units[1], None));
    let mut origins = crate::logic::geometry::TileSet::new(7);
    origins.insert(0);
    assert!(!lit_observer.can_threaten_from(
        &lit_observer.units[0],
        &lit_observer.units[1],
        0,
        &origins,
    ));

    // A positive modifier can extend sight beyond the base cap.
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = serde_json::to_value(SightModifiers {
            lit: 6,
            unlit: 3,
            darkness: 3,
        })
        .unwrap()
    });
    state.tiles[6].light = Some(light);
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let boosted = LiveBattle::from_state(&local, state.clone());
    assert!(boosted.tile_in_sight_to("alpha", 6));
    assert!(!boosted.tile_in_sight_to("alpha", 4));

    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = serde_json::to_value(SightModifiers {
            lit: -20,
            unlit: -20,
            darkness: -20,
        })
        .unwrap()
    });
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let clamped = LiveBattle::from_state(&local, state.clone());
    assert!(clamped.tile_in_sight_to("alpha", 0));
    assert!(!clamped.tile_in_sight_to("alpha", 1));
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["unlimited"] = json!(true)
    });
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let unlimited = LiveBattle::from_state(&local, state);
    assert!(unlimited.tile_in_sight_to("alpha", 6));

    let mut state = unlimited.state();
    for tile in &mut state.tiles {
        tile.light = None;
    }

    // Daylight illuminates outdoor targets, while underground cells are unlit.
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["unlimited"] = json!(false)
    });
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = serde_json::to_value(SightModifiers {
            lit: 7,
            unlit: 4,
            darkness: 7,
        })
        .unwrap()
    });
    state.lighting.time = TimeOfDay::Day;
    for tile in &mut state.tiles {
        tile.perception.illumination = if tile.hex() == Hex::new(5, 0) {
            Some(Illumination::Unlit)
        } else {
            Some(Illumination::Lit)
        };
    }
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let day = LiveBattle::from_state(&local, state);
    assert!(day.tile_in_sight_to("alpha", 6));
    assert!(!day.tile_in_sight_to("alpha", 5));
}

#[test]
fn authored_darkness_is_distinct_from_night_and_darkcloaks_hide_only_in_darkness() {
    let mut bundle = fixture::deterministic();
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["unlimited"] = json!(false)
    });
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = serde_json::to_value(SightModifiers {
            lit: 7,
            unlit: 7,
            darkness: 4,
        })
        .unwrap()
    });
    let mut state = corridor(0, false);
    for group in &mut state.vision_groups {
        group.sight.unlimited = false;
    }
    state.lighting.time = TimeOfDay::Night;
    for tile in &mut state.tiles {
        tile.perception = TilePerception {
            illumination: Some(Illumination::Unlit),
            remember: false,
            concealment: vec![],
        };
        tile.light = None;
    }
    state.tiles[5].perception.illumination = Some(Illumination::Darkness);
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let game = LiveBattle::from_state(&local, state.clone());
    assert_eq!(game.perception(None).illumination(6), Illumination::Unlit);
    assert_eq!(
        game.perception(None).illumination(5),
        Illumination::Darkness
    );
    assert!(game.tile_in_sight_to("alpha", 6));
    assert!(!game.tile_in_sight_to("alpha", 5));
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = json!({"lit":7,"unlit":7,"darkness":7})
    });
    state.units[1].perception_effects.push(
        serde_json::from_value(serde_json::json!({
            "id":"darkcloak", "penetrates":[],"light":null,"concealment":[{"kind":"darkness","revealWithin":1,
                "requires":{"terrains":null,"illumination":["darkness"]}}]
        }))
        .unwrap(),
    );
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let night = LiveBattle::from_state(&local, state.clone());
    assert!(night.unit_visible_to("alpha", &night.units[1], None));
    state.tiles[6].perception.illumination = Some(Illumination::Darkness);
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let dark = LiveBattle::from_state(&local, state.clone());
    assert!(dark.tile_in_sight_to("alpha", 6));
    assert!(!dark.unit_visible_to("alpha", &dark.units[1], None));
    state.tiles[6].light = Some(std::sync::Arc::new(
        serde_json::from_value(serde_json::json!({
            "emission":{"radius":0},"radius":64.0,"power":1.0
        }))
        .unwrap(),
    ));
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let lit = LiveBattle::from_state(&local, state);
    assert_eq!(lit.perception(None).illumination(6), Illumination::Lit);
    assert!(lit.unit_visible_to("alpha", &lit.units[1], None));
}

#[test]
fn enemy_sight_estimates_do_not_read_private_exploration() {
    let mut state = corridor(0, true);
    state
        .vision_groups
        .iter_mut()
        .find(|g| g.id == "alpha")
        .unwrap()
        .sight
        .penetrates = vec![ConcealmentKind::Fog];
    state
        .factions
        .iter_mut()
        .find(|f| f.id == "alpha")
        .unwrap()
        .relations
        .insert("beta".into(), Relation::Hostile);
    let first = BattleSession::from_state(rules(), state.clone());
    state
        .vision_groups
        .iter_mut()
        .find(|g| g.id == "beta")
        .unwrap()
        .explored
        .push(Hex::new(3, 0));
    let second = BattleSession::from_state(rules(), state);
    let mut first = first;
    let mut second = second;
    first.set_active_faction(Some("alpha".into()));
    second.set_active_faction(Some("alpha".into()));
    assert!(
        !first
            .state()
            .vision_groups
            .iter()
            .find(|g| g.id == "beta")
            .unwrap()
            .explored
            .contains(&Hex::new(3, 0))
    );
    assert!(
        second
            .state()
            .vision_groups
            .iter()
            .find(|g| g.id == "beta")
            .unwrap()
            .explored
            .contains(&Hex::new(3, 0))
    );
    let before: Vec<_> = (0..7)
        .map(|tile| first.tile_insight_of_enemy(tile))
        .collect();
    let after: Vec<_> = (0..7)
        .map(|tile| second.tile_insight_of_enemy(tile))
        .collect();
    assert_eq!(before, after);
    assert!(before[3]); // conservative estimate: a remembered fog tile is not safe cover
}

#[test]
fn sight_contributions_stack_from_weather_equipment_and_scoped_sources() {
    let mut bundle = fixture::deterministic();
    fixture::unit(&mut bundle, "archer", |unit| {
        unit["sight"]["sightModifiers"] = json!({"lit":2,"unlit":2,"darkness":2});
        unit["sight"]["unlimited"] = json!(false);
    });
    fixture::effects(&mut bundle, "archer", |effects| {
        *effects = vec![
            serde_json::from_value(json!({
                "scope":{"radius":1,"includeSelf":false,"targets":"allies"},
                "stacking":{"group":"sight","combine":"add"},
                "effect":{"kind":"sight","unlimited":false,
                    "sightModifiers":{"lit":1,"unlit":4,"darkness":4},"penetrates":["darkness"]}
            }))
            .unwrap(),
        ]
    });
    fixture::item(
        &mut bundle,
        "test-sight",
        json!({
            "id":"test-sight","name":"Sight","description":"Numeric sight contribution",
            "sight":{"unlimited":false,"sightModifiers":{"lit":2,"unlit":3,"darkness":3},"penetrates":[]},
            "abilities":[],"attacks":[],"armor":null
        }),
    );
    let mut state = corridor(0, false);
    for group in &mut state.vision_groups {
        group.sight.unlimited = false;
    }
    state.map_modifiers.sight.sight_modifiers = SightModifiers {
        lit: 1,
        unlit: 1,
        darkness: 1,
    };
    let mut ally = state.units[0].clone();
    ally.id = "u3".into();
    ally.tile = Hex::new(1, 0);
    state.units.push(ally);
    state.next_id = 4;
    let local = Catalogue::from_json_bundle(&bundle).unwrap();
    let mut game = LiveBattle::from_state(&local, state);
    let sight =
        |game: &LiveBattle<'_>, light| game.perception(None).profile(game.unit(1).unwrap(), light);
    assert_eq!(sight(&game, Illumination::Lit).range, Some(4));
    assert_eq!(sight(&game, Illumination::Darkness).range, Some(7));
    assert!(
        sight(&game, Illumination::Lit)
            .penetrates
            .contains(&ConcealmentKind::Darkness)
    );
    game.give_item(1, "test-sight".into());
    assert_eq!(sight(&game, Illumination::Lit).range, Some(6));
    assert_eq!(sight(&game, Illumination::Darkness).range, Some(10));
    game.drop_item(1, 0);
    assert_eq!(sight(&game, Illumination::Lit).range, Some(4));
    let projected = game
        .perception(Some((1, 4)))
        .profile(game.unit(1).unwrap(), Illumination::Lit);
    assert_eq!(projected.range, Some(3));
    assert!(!projected.penetrates.contains(&ConcealmentKind::Darkness));
}
