use hex_command_core::data;
#[path = "../testing/catalogue.rs"]
mod catalogue_fixture;
#[path = "../testing/effects.rs"]
mod effect_fixture;
use hex_command_core::{
    ai::Policy,
    data::{ArenaForces, ArenaScenario, ArenaVariant, Catalogue, TurnPhase},
    simulation::{ArenaConfig, Armies, run_arena},
};

#[test]
fn authored_arena_uses_its_board_keeps_and_capture_targets() {
    let mut bundle = catalogue_fixture::bundle();
    let source = {
        let rules = catalogue_fixture::frozen(&bundle);
        hex_command_core::logic::BattleSession::new(rules, "mirror").state()
    };
    let targets = vec![
        source
            .tiles
            .iter()
            .find(|t| t.terrain == "village")
            .unwrap()
            .hex(),
    ];
    let scenario = ArenaScenario {
        id: "authored-fixture".into(),
        hold_turns: 4,
        variants: vec![ArenaVariant {
            map: source.map_id.clone(),
            map_revision: 1,
            forces: ArenaForces::Selected { budget: 36 },
        }],
        territory: targets.clone(),
    };
    catalogue_fixture::edit(&mut bundle, "simulation/arena.json", |arena| {
        arena["scenarios"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(scenario).unwrap())
    });
    let rules = catalogue_fixture::frozen(&bundle);
    let army = rules.meta().simulation.rosters[0].id.clone();
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
    let mut bundle = catalogue_fixture::bundle();
    let path = "game/scenario/mirror-v1/scenario.json";
    let mut descriptor: serde_json::Value = serde_json::from_str(&bundle[path]).unwrap();
    descriptor["tags"] = serde_json::json!(["simulation"]);
    bundle.insert(path.into(), descriptor.to_string());
    let rules = Catalogue::from_json_bundle(&bundle).unwrap();
    assert!(rules.maps().any(|map| map.id() == "mirror"));
    assert!(!rules.map("mirror", 1).tags().contains(&"human".to_owned()));
}

#[test]
fn authored_combat_preserves_starting_forces_and_faction_gold() {
    let mut bundle = catalogue_fixture::bundle();
    let source = {
        let rules = catalogue_fixture::frozen(&bundle);
        hex_command_core::logic::BattleSession::new(rules, "mirror").state()
    };
    let point = source.tiles[0].hex();
    let scenario = ArenaScenario {
        id: "authored-forces".into(),
        hold_turns: 4,
        variants: vec![ArenaVariant {
            map: source.map_id.clone(),
            map_revision: 1,
            forces: ArenaForces::Scenario,
        }],
        territory: vec![point],
    };
    catalogue_fixture::edit(&mut bundle, "simulation/arena.json", |arena| {
        arena["scenarios"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(scenario).unwrap())
    });
    let rules = catalogue_fixture::frozen(&bundle);
    let army = rules.meta().simulation.rosters[0].id.clone();
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
