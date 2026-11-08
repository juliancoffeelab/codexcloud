use hex_command_core::data::Hex;
use hex_command_core::{
    ai::Policy,
    data::{LandscapeConfig, Rules, Side},
    simulation::{ArenaConfig, Armies, run_arena},
};

#[test]
fn authored_arena_uses_its_board_keeps_and_capture_targets() {
    let mut rules = Rules::load().unwrap();
    let source = rules.maps[&("mirror".into(), 1)].clone();
    rules
        .maps
        .remove(&(source.map_id.clone(), source.map_revision));
    rules
        .arena_maps
        .insert((source.map_id.clone(), source.map_revision), source.clone());
    let targets = vec![
        source
            .tiles
            .iter()
            .find(|t| t.terrain == "village")
            .unwrap()
            .hex(),
    ];
    rules.arena.landscapes.push(LandscapeConfig {
        id: "authored-fixture".into(),
        hold_turns: 4,
        map: Some(source.map_id.clone()),
        territory: Some(targets.clone()),
    });
    let army = rules.rosters[0].id.clone();
    let result = run_arena(
        &rules,
        ArenaConfig {
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
            first: Side::Player,
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
            .find(|t| t.terrain == "keep" && t.owner.as_deref() == Some(unit.side.faction_id()))
            .unwrap();
        assert_eq!(unit.tile, keep.hex());
    }
    assert_eq!(result.phases[0].objectives[0].tiles, targets);
}

#[cfg(feature = "wasm")]
#[test]
fn arena_only_scenarios_are_loaded_without_playable_metadata() {
    let mut bundle = hex_command_core::data::catalog::embedded_bundle();
    let mut manifest: serde_json::Value = serde_json::from_str(&bundle["manifest.json"]).unwrap();
    let scenarios = manifest["scenarios"].as_array_mut().unwrap();
    let path = scenarios
        .iter()
        .find(|p| p.as_str().unwrap().contains("/mirror-v1/"))
        .unwrap()
        .clone();
    scenarios.retain(|p| !p.as_str().unwrap().contains("/mirror-"));
    manifest["arenaScenarios"]
        .as_array_mut()
        .unwrap()
        .push(path);
    bundle.insert("manifest.json".into(), manifest.to_string());
    let rules = Rules::from_json_bundle(&bundle).unwrap();
    assert!(rules.arena_maps.keys().any(|(id, _)| id == "mirror"));
    assert!(!rules.maps.keys().any(|(id, _)| id == "mirror"));
    assert!(!rules.map_definitions.iter().any(|map| map.id == "mirror"));
}

#[cfg(feature = "wasm")]
#[test]
fn invalid_authored_layout_and_capture_references_are_rejected() {
    for (map, targets, message) in [
        ("missing-layout", vec![Hex::new(0, 0)], "unknown arena map"),
        (
            "mirror",
            vec![Hex::new(999, 999)],
            "outside its authored map",
        ),
    ] {
        let mut bundle = hex_command_core::data::catalog::embedded_bundle();
        let mut arena: serde_json::Value =
            serde_json::from_str(&bundle["simulation/arena.json"]).unwrap();
        arena["landscapes"][0]["map"] = serde_json::json!(map);
        arena["landscapes"][0]["territory"] = serde_json::json!(targets);
        bundle.insert("simulation/arena.json".into(), arena.to_string());
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "simulation/arena.json");
        assert!(error.message.contains(message), "{}", error.message);
    }
}
