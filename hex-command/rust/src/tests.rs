use crate::{data::*, logic::Game};
use std::sync::OnceLock;
fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| Rules::load().expect("authored catalog"))
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
fn restore_rejects_experience_outside_progression_limits() {
    let mut state = Game::new(rules(), "developer").state();
    state.units[0].experience = Some(u32::MAX);
    assert!(Game::try_from_state(rules(), state).is_err());
    let opening = Game::start_campaign(
        rules(),
        "lantern-road",
        1,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    )
    .state();
    for story in [false, true] {
        let mut state = opening.clone();
        let campaign = state.campaign.as_mut().unwrap();
        let veteran = Veteran {
            r#type: if story { "commander" } else { "swordsman" }.into(),
            experience: u32::MAX,
            name: story.then(|| "Captain Nera".into()),
            role: story.then_some(Role::Leader),
        };
        if story {
            campaign.story_veterans = Some(vec![veteran]);
        } else {
            campaign.recall = vec!["swordsman".into()];
            campaign.veterans = Some(vec![veteran]);
        }
        assert!(Game::try_from_state(rules(), state).is_err());
    }
}
#[test]
fn restore_rejects_invalid_campaign_carryover_before_retry() {
    let game = Game::start_campaign(
        rules(),
        "lantern-road",
        1,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    );
    let opening = game.state();
    for name in ["Captain Nera", "unknown companion"] {
        let mut state = opening.clone();
        state.campaign.as_mut().unwrap().fallen_companions = Some(vec![name.into()]);
        assert!(Game::try_from_state(rules(), state).is_err());
    }
    let mut state = opening.clone();
    state.campaign.as_mut().unwrap().recall = vec!["commander".into()];
    assert!(Game::try_from_state(rules(), state).is_err());
    let mut state = opening;
    state.campaign.as_mut().unwrap().fallen_companions = Some(vec!["Ivo".into()]);
    assert!(Game::try_from_state(rules(), state).is_ok());
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

// Behavioral coverage moved from the web suite. Helpers make small native boards;
// scenarios remain separately named so one failure cannot hide another category.
#[path = "tests/campaign.rs"]
mod campaign;
#[path = "tests/combat.rs"]
mod combat;
#[path = "tests/economy.rs"]
mod economy;
#[path = "tests/movement.rs"]
mod movement;
#[path = "tests/progression.rs"]
mod progression;

fn flat() -> Game<'static> {
    let mut game = Game::new(rules(), "mirror");
    game.units.clear();
    game.roster_changed();
    for tile in 0..game.terrain.len() {
        game.set_terrain(tile, game.rules.terrains["plains"]);
    }
    game.owners.fill(None);
    game
}
fn add(game: &mut Game<'_>, kind: &str, side: Side, tile: &str) -> u32 {
    game.add(game.kind(kind), side, game.site(tile))
}
fn weapon(game: &Game<'_>, id: u32, weapon: &str) -> usize {
    game.rules.units[game.unit(id).unwrap().kind]
        .attacks
        .iter()
        .position(|w| w.id == weapon)
        .unwrap()
}
fn hit(game: &mut Game<'_>, attacker: u32, target: u32, name: &str) -> Effects {
    game.execute(Command::Attack {
        unit: UnitId(attacker),
        target: UnitId(target),
        weapon: name.into(),
    })
    .unwrap()
}
fn chapter(id: &str, mission: usize) -> Game<'static> {
    Game::start_campaign(rules(), id, mission, vec![], 0, vec![], vec![], vec![])
}
