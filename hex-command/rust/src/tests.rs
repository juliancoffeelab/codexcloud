use crate::{ai::Random, data::*, logic::Game};
use std::sync::OnceLock;
fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| Rules::embedded().expect("authored catalog"))
}
#[test]
fn authored_movement_and_damage_are_preserved() {
    let rules = rules();
    let human = &rules.units[rules.types["swordsman"]];
    let merfolk = &rules.units[rules.types["reefguard"]];
    assert_eq!(human.costs[rules.terrains["water"]], None);
    assert_eq!(merfolk.costs[rules.terrains["water"]], Some(1));
    assert!(merfolk.environment[rules.terrains["forest"]][0][0] > 0);
}
#[test]
fn promotion_preserves_wounds_and_consumes_actions() {
    let mut game = Game::new(rules(), "mirror");
    let kind = game.kind("swordsman");
    let tile = game.site("4,4");
    game.units.clear();
    game.roster_changed();
    let id = game.add(kind, Side::Player, tile);
    let rule = game.rules.units[kind].clone();
    game.units[0].experience = rule.experience_needed;
    game.units[0].hp = rule.hp - 3;
    let promotion = game.promotion_options(&game.units[0])[0];
    game.advance(id, promotion);
    let unit = game.unit(id).unwrap();
    assert_eq!(unit.hp, game.rules.units[promotion].hp - 3);
    assert_eq!(unit.experience, Some(0));
    assert!(unit.moved && unit.attacked);
}
#[test]
fn cloned_forecasts_do_not_mutate_source_or_record_commands() {
    let mut game = Game::start_campaign(
        rules(),
        "borrowed-tide",
        0,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    );
    let before = game.state();
    let mut fork = game.fork();
    fork.end_turn();
    fork.end_turn();
    assert_eq!(
        serde_json::to_value(game.state()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert_eq!(fork.round, 2);
    game.set_terrain(0, game.rules.terrains["plains"]);
    assert_ne!(game.terrain, fork.terrain);
}
#[test]
fn tide_starts_rescue_before_counting_defended_turns() {
    let mut game = Game::start_campaign(
        rules(),
        "borrowed-tide",
        1,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    );
    let wreck = game.site("5,4");
    game.owners[wreck] = Some(Side::Player);
    game.evaluate_campaign(false, Some(wreck));
    assert!(game.campaign.as_ref().unwrap().marked("event:await-tide"));
    let goal = game
        .mission
        .as_ref()
        .unwrap()
        .goals
        .iter()
        .find(|g| g.kind() == GoalKind::Hold)
        .unwrap()
        .clone();
    while game.round < 6 {
        game.end_turn();
        game.end_turn();
    }
    assert!(game.campaign.as_ref().unwrap().marked("event:rescue"));
    assert_eq!(game.goal_progress(&goal).current, 0);
    for _ in 0..3 {
        game.end_turn();
        game.end_turn();
    }
    assert_eq!(game.goal_progress(&goal).current, 3);
    assert_eq!(game.winner, Some(Side::Player));
}
#[test]
fn routing_cache_tracks_terrain_changes() {
    let mut game = Game::new(rules(), "mirror");
    let start = game.site("4,4");
    let goal = game.site("5,4");
    let kind = game.kind("swordsman");
    game.set_terrain(goal, game.rules.terrains["plains"]);
    assert_eq!(game.routes(kind, &[goal])[start], 1);
    game.set_terrain(goal, game.rules.terrains["water"]);
    assert_eq!(game.routes(kind, &[goal])[start], u16::MAX);
}
#[test]
fn seeded_random_matches_js_wrapping() {
    let mut random = Random(17);
    let expected = [0.6771502960473299, 0.19265692122280598, 0.5313839064911008];
    for value in expected {
        assert_eq!(random.next(), value);
    }
}
#[test]
fn invalid_commands_are_atomic_and_records_remain_owned() {
    let mut game = Game::new(rules(), "mirror");
    game.recordings = Some(Records::default());
    let before = serde_json::to_value(game.state()).unwrap();
    let err = game
        .execute(Command::Move {
            unit: UnitId(u32::MAX),
            tile: TileId("4,4".into()),
        })
        .unwrap_err();
    assert_eq!(err, CommandError::UnknownUnit(UnitId(u32::MAX)));
    assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
    game.action(ActionRecord::Move {
        unit: "u1".into(),
        tile: "4,4".into(),
    });
    let fork = game.fork();
    assert!(fork.recordings.is_none());
    assert_eq!(game.recordings.as_ref().unwrap().actions.len(), 1);
}
#[test]
fn snapshots_preserve_absent_and_null_combat_fields() {
    let game = Game::new(rules(), "mirror");
    let mut state = game.state();
    state.last_combat = None;
    let absent = serde_json::to_value(&state).unwrap();
    assert!(absent.get("lastCombat").is_none());
    state.last_combat = Some(None);
    let null = serde_json::to_value(&state).unwrap();
    assert_eq!(null["lastCombat"], serde_json::Value::Null);
    let restored: State = serde_json::from_value(null).unwrap();
    assert!(matches!(restored.last_combat, Some(None)));
}
#[test]
fn restore_rejects_colliding_roster_before_indexing() {
    let mut state = Game::new(rules(), "mirror").state();
    state.units[1].tile = state.units[0].tile.clone();
    assert!(Game::try_from_state(rules(), state).is_err());
}
#[test]
fn public_battle_applies_typed_moves_with_feedback() {
    use crate::logic::Battle;
    let mut battle = Battle::new(rules(), MapId("mirror".into()), None).unwrap();
    let unit = battle
        .snapshot()
        .units
        .iter()
        .find(|u| u.side == Side::Player)
        .unwrap();
    let id = UnitId(unit.id[1..].parse().unwrap());
    let destination = battle.reachable(rules(), id).unwrap().remove(0).tile;
    let effects = battle
        .apply(
            rules(),
            Command::Move {
                unit: id,
                tile: destination.clone(),
            },
        )
        .unwrap();
    assert!(effects.events.iter().any(
        |event| matches!(event,GameEvent::UnitMoved{unit,to,..} if *unit==id && *to==destination)
    ));
    assert_eq!(
        battle
            .snapshot()
            .units
            .iter()
            .find(|u| u.id == format!("u{}", id.0))
            .unwrap()
            .tile,
        destination.0
    );
}
#[test]
fn catalogs_are_shareable_and_cached_routes_reuse_the_distance_field() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Catalog>();
    let game = Game::new(rules(), "mirror");
    let kind = game.kind("swordsman");
    let goal = game.site("5,4");
    assert!(std::rc::Rc::ptr_eq(
        &game.routes(kind, &[goal]),
        &game.routes(kind, &[goal])
    ));
}
#[test]
fn tagged_goals_require_the_fields_their_rules_use() {
    assert!(
        serde_json::from_str::<Goal>(r#"{"kind":"hold","label":"Rescue","tile":"5,4","turns":3}"#)
            .is_err()
    );
    let goal: Goal =
        serde_json::from_str(r#"{"kind":"defeat","label":"Defeat the Bellwarden"}"#).unwrap();
    assert!(matches!(goal, Goal::Defeat { .. }));
}
