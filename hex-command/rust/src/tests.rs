use crate::{engine::Game, model::*, policies::Random};
use std::sync::Arc;
use std::sync::OnceLock;
fn rules() -> Arc<Rules> {
    static RULES: OnceLock<Arc<Rules>> = OnceLock::new();
    RULES.get_or_init(Rules::load).clone()
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
    assert_eq!(game.state(), before);
    assert_eq!(fork.round, 2);
    game.terrain[0] = game.rules.terrains["plains"];
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
        .find(|g| g.kind == "hold")
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
    game.terrain[goal] = game.rules.terrains["plains"];
    assert_eq!(game.routes(kind, &[goal])[start], 1);
    game.terrain[goal] = game.rules.terrains["water"];
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
