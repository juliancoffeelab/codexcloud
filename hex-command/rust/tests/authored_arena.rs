use hex_command_core::{
    ai::Policy,
    data::{LandscapeConfig, Rules, Side},
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
            .id
            .clone(),
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
            .find(|t| t.terrain == "keep" && t.owner == Some(unit.side))
            .unwrap();
        assert_eq!(unit.tile, keep.id);
    }
    assert_eq!(result.phases[0].objectives[0].tiles, targets);
}

#[cfg(feature = "wasm")]
#[test]
fn invalid_authored_layout_and_capture_references_are_rejected() {
    for (map, targets, message) in [
        ("missing-layout", vec!["0,0"], "unknown arena map"),
        ("mirror", vec!["999,999"], "outside its authored map"),
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
