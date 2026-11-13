use hex_command_core::data::Hex;
use hex_command_core::{
    ai::Policy,
    data::{ArenaForces, ArenaScenario, ArenaVariant, Rules, TurnPhase},
    simulation::{ArenaConfig, Armies, run_arena},
};

#[test]
fn authored_arena_uses_its_board_keeps_and_capture_targets() {
    let mut rules = Rules::load().unwrap();
    let source = rules.maps[&("mirror".into(), 1)].clone();
    let targets = vec![
        source
            .tiles
            .iter()
            .find(|t| t.terrain == "village")
            .unwrap()
            .hex(),
    ];
    rules.arena.scenarios.push(ArenaScenario {
        id: "authored-fixture".into(),
        hold_turns: 4,
        variants: vec![ArenaVariant {
            map: source.map_id.clone(),
            forces: ArenaForces::Selected { budget: 36 },
        }],
        territory: targets.clone(),
    });
    let army = rules.rosters[0].id.clone();
    let result = run_arena(
        &rules,
        ArenaConfig {
            objective: Default::default(),
            seed: 19,
            armies: Armies {
                player: army.clone(),
                enemy: army,
            },
            landscape: "authored-fixture".into(),
            player: Policy::Balanced,
            enemy: Policy::Balanced,
            player_blocks: None,
            enemy_blocks: None,
            first: TurnPhase::Player,
            max_rounds: 1,
            recruit_limit: Some(0),
        },
    )
    .unwrap();
    assert_eq!(result.initial.cols, source.cols);
    assert_eq!(result.initial.rows, source.rows);
    assert_eq!(
        serde_json::to_value(&result.initial.tiles).unwrap(),
        serde_json::to_value(&source.tiles).unwrap()
    );
    assert_eq!(result.initial.units.len(), 2);
    for unit in &result.initial.units {
        let keep = source
            .tiles
            .iter()
            .find(|t| t.terrain == "keep" && t.owner.as_deref() == Some(unit.faction.as_str()))
            .unwrap();
        assert_eq!(unit.tile, keep.hex());
    }
    assert_eq!(result.phases[0].objectives[0].tiles, targets);
}

#[cfg(feature = "wasm")]
#[test]
fn non_human_scenarios_use_the_shared_map_registry() {
    let mut bundle = hex_command_core::data::catalog::embedded_bundle();
    let path = "game/scenario/mirror-v1/scenario.json";
    let mut descriptor: serde_json::Value = serde_json::from_str(&bundle[path]).unwrap();
    descriptor["human"] = serde_json::json!(false);
    bundle.insert(path.into(), descriptor.to_string());
    let rules = Rules::from_json_bundle(&bundle).unwrap();
    assert!(rules.maps.keys().any(|(id, _)| id == "mirror"));
    assert!(
        !rules
            .map_definitions
            .iter()
            .find(|map| map.id == "mirror")
            .unwrap()
            .human
    );
}

#[cfg(feature = "wasm")]
#[test]
fn invalid_authored_layout_and_capture_references_are_rejected() {
    for (map, targets, message) in [
        ("missing-layout", vec![Hex::new(0, 0)], "unknown arena map"),
        ("mirror", vec![Hex::new(999, 999)], "outside its map"),
    ] {
        let mut bundle = hex_command_core::data::catalog::embedded_bundle();
        let mut arena: serde_json::Value =
            serde_json::from_str(&bundle["simulation/arena.json"]).unwrap();
        arena["scenarios"][0]["variants"][0]["map"] = serde_json::json!(map);
        arena["scenarios"][0]["territory"] = serde_json::json!(targets);
        bundle.insert("simulation/arena.json".into(), arena.to_string());
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "simulation/arena.json");
        assert!(error.message.contains(message), "{}", error.message);
    }
}

#[test]
fn authored_combat_preserves_starting_forces_and_faction_gold() {
    let mut rules = Rules::load().unwrap();
    let source = rules.maps[&("mirror".into(), 1)].clone();
    let point = source.tiles[0].hex();
    rules.arena.scenarios.push(ArenaScenario {
        id: "authored-forces".into(),
        hold_turns: 4,
        variants: vec![ArenaVariant {
            map: source.map_id.clone(),
            forces: ArenaForces::Scenario,
        }],
        territory: vec![point],
    });
    let army = rules.rosters[0].id.clone();
    let result = run_arena(
        &rules,
        ArenaConfig {
            objective: Default::default(),
            seed: 19,
            armies: Armies {
                player: army.clone(),
                enemy: army,
            },
            landscape: "authored-forces".into(),
            player: Policy::Balanced,
            enemy: Policy::Classic,
            player_blocks: None,
            enemy_blocks: None,
            first: TurnPhase::Ai,
            max_rounds: 1,
            recruit_limit: Some(0),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&result.initial.units).unwrap(),
        serde_json::to_value(&source.units).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&result.initial.gold).unwrap(),
        serde_json::to_value(&source.gold).unwrap()
    );
    assert!(result.initial.campaign.is_none());
    assert_eq!(result.initial.turn, TurnPhase::Ai);
}
