//! Group knowledge and physical forecasts share one detection contract.
use super::*;
use crate::{BattleSession, BattleView};
use std::rc::Rc;

fn corridor(radius: u16, remember: bool) -> State {
    let mut state = Game::new(rules(), "mirror").state();
    state.turn = TurnPhase::Ai;
    for id in ["alpha", "beta"] {
        state.factions.push(serde_json::from_value(serde_json::json!({
            "id":id,"phase":"ai","visionGroup":id,"controller":{"kind":"ai"},"palette":"enemy","relations":{"player":"hostile"}
        })).unwrap());
        state.vision_groups.push(VisionGroupState {
            id: id.into(),
            sight: SightProfile {
                range: SightRange::UNLIMITED,
                penetrates: vec![],
            },
            explored: vec![],
        });
        state.gold.insert(id.into(), 0);
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
                    requires: ConcealmentRequirements::default(),
                }],
                ..Default::default()
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
            side: TurnPhase::Ai,
            tile: Hex::new(col, 0),
            role: None,
            r#type: "archer".into(),
            hp: rules().units[rules().types["archer"]].hp,
            ..unit.clone()
        })
        .collect();
    state.next_id = 3;
    state
}

#[test]
fn group_history_and_known_positions_are_independent_of_turn_phase() {
    let mut game = Game::from_state(rules(), corridor(1, true));
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
    let restored = Game::from_state(
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
    let mut local = rules().clone();
    let archer = local.types["archer"];
    local.units[archer].attacks[0].range = 3;
    local.units[archer].perception.concealment = vec![ConcealmentRule {
        kind: ConcealmentKind::ForestCamouflage,
        reveal_within: 1,
        requires: ConcealmentRequirements::default(),
    }];
    let mut game = Game::from_state(&local, state);
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
