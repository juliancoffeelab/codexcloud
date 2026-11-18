use crate::{data::*, logic::LiveBattle};
use std::sync::OnceLock;
fn rules() -> &'static Catalogue {
    static RULES: OnceLock<Catalogue> = OnceLock::new();
    RULES.get_or_init(|| {
        Catalogue::from_json_bundle(&crate::catalogue_fixture::deterministic()).unwrap()
    })
}
#[test]
fn cloned_forecasts_do_not_mutate_source_or_record_commands() {
    let mut game = LiveBattle::start_campaign(
        rules(),
        "mirror-night",
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
    game.set_terrain(0, game.definitions.terrain_name("plains"));
    assert_ne!(game.terrain, fork.terrain);
}
#[test]
fn routing_cache_tracks_terrain_changes() {
    let mut game = LiveBattle::new(rules(), "mirror");
    let start = game.site(&Hex::new(4, 4));
    let goal = game.site(&Hex::new(5, 4));
    let kind = game.kind("swordsman");
    game.set_terrain(goal, game.definitions.terrain_name("plains"));
    assert_eq!(
        game.routes(
            &game.prospective(kind, game.units.first().expect("leader").faction, goal),
            &[goal]
        )[start],
        1
    );
    game.set_terrain(goal, game.definitions.terrain_name("water"));
    assert_eq!(
        game.routes(
            &game.prospective(kind, game.units.first().expect("leader").faction, goal),
            &[goal]
        )[start],
        u16::MAX
    );
}
#[test]
fn invalid_commands_are_atomic_and_records_remain_owned() {
    let mut game = LiveBattle::new(rules(), "mirror");
    game.recordings = Some(Records::default().into());
    let before = serde_json::to_value(game.state()).unwrap();
    let err = game
        .execute(Command::Move {
            unit: UnitId(u32::MAX),
            tile: TileId(Hex::new(4, 4).into()),
        })
        .unwrap_err();
    assert_eq!(err, CommandError::UnknownUnit(UnitId(u32::MAX)));
    assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
    game.action(ActionRecord::Move {
        requested: None,
        path: vec![],
        unit: "u1".into(),
        tile: Hex::new(4, 4).into(),
    });
    let fork = game.fork();
    assert!(fork.recordings.is_none());
    assert_eq!(game.recordings.as_ref().unwrap().actions.len(), 1);
}
#[test]
fn snapshots_require_explicit_null_combat_fields() {
    let game = LiveBattle::new(rules(), "mirror");
    let mut state = game.state();
    state.last_combat = None;
    let mut absent = serde_json::to_value(&state).unwrap();
    absent.as_object_mut().unwrap().remove("lastCombat");
    assert!(serde_json::from_value::<State>(absent).is_err());
    state.last_combat = Some(None);
    let null = serde_json::to_value(&state).unwrap();
    assert_eq!(null["lastCombat"], serde_json::Value::Null);
    let restored: State = serde_json::from_value(null).unwrap();
    assert!(matches!(restored.last_combat, Some(None)));
}
#[test]
fn restore_rejects_colliding_roster_before_indexing() {
    let mut state = LiveBattle::new(rules(), "mirror").state();
    state.units[1].tile = state.units[0].tile.clone();
    assert!(LiveBattle::try_from_state(rules(), state).is_err());
}
#[test]
fn restore_rejects_experience_outside_progression_limits() {
    let mut state = LiveBattle::new(rules(), "developer").state();
    state.units[0].experience = Some(u32::MAX);
    assert!(LiveBattle::try_from_state(rules(), state).is_err());
    let opening = LiveBattle::start_campaign(
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
            items: vec![],
            r#type: if story { "commander" } else { "swordsman" }.into(),
            experience: u32::MAX,
            character: story.then(|| "nera".into()),
            role: story.then_some(Role::Leader),
        };
        if story {
            campaign.story_veterans = Some(vec![veteran]);
        } else {
            campaign.recall = vec!["swordsman".into()];
            campaign.veterans = Some(vec![veteran]);
        }
        assert!(LiveBattle::try_from_state(rules(), state).is_err());
    }
}
#[test]
fn restore_rejects_invalid_campaign_carryover_before_retry() {
    let game = LiveBattle::start_campaign(
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
        assert!(LiveBattle::try_from_state(rules(), state).is_err());
    }
    let mut state = opening.clone();
    state.campaign.as_mut().unwrap().recall = vec!["commander".into()];
    assert!(LiveBattle::try_from_state(rules(), state).is_err());
    let mut state = opening;
    state.campaign.as_mut().unwrap().fallen_companions = Some(vec!["ivo".into()]);
    assert!(LiveBattle::try_from_state(rules(), state).is_ok());
}
#[test]
fn catalogs_are_shareable_and_cached_routes_reuse_the_distance_field() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Catalogue>();
    let game = LiveBattle::new(rules(), "mirror");
    let kind = game.kind("swordsman");
    let goal = game.site(&Hex::new(5, 4));
    assert!(std::rc::Rc::ptr_eq(
        &game.routes(
            &game.prospective(kind, game.units.first().expect("leader").faction, goal),
            &[goal]
        ),
        &game.routes(
            &game.prospective(kind, game.units.first().expect("leader").faction, goal),
            &[goal]
        )
    ));
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

fn flat() -> LiveBattle<'static> {
    let mut game = LiveBattle::new(rules(), "mirror");
    game.units.clear();
    game.roster_changed();
    for tile in 0..game.terrain.len() {
        game.set_terrain(tile, game.definitions.terrain_name("plains"));
    }
    game.owners.fill(None);
    game
}
fn add(
    game: &mut LiveBattle<'_>,
    kind: &str,
    faction: &str,
    tile: impl std::borrow::Borrow<Hex>,
) -> u32 {
    game.add(game.kind(kind), faction, game.site(tile.borrow()))
}

fn weapon(game: &LiveBattle<'_>, id: u32, weapon: &str) -> usize {
    let actor = game.unit(id).unwrap();
    game.with_capabilities_at(actor, actor.tile, &game.units, &[], |caps| {
        caps.attacks()
            .position(|attack| attack.id() == weapon)
            .unwrap()
    })
}
fn hit(game: &mut LiveBattle<'_>, attacker: u32, target: u32, name: &str) {
    game.execute(Command::Attack {
        unit: UnitId(attacker),
        target: UnitId(target),
        weapon: name.into(),
    })
    .unwrap()
}
fn chapter(id: &str, mission: usize) -> LiveBattle<'static> {
    LiveBattle::start_campaign(rules(), id, mission, vec![], 0, vec![], vec![], vec![])
}

#[path = "tests/visibility.rs"]
mod visibility;

fn healing_from_at(game: &LiveBattle<'_>, provider: &Unit, target: &Unit, origin: usize) -> i32 {
    let destination = if provider.id == target.id {
        origin
    } else {
        target.tile
    };
    game.with_selected_capabilities_at(
        target,
        destination,
        &game.units,
        &[(*provider, origin)],
        |source, _, _| source.is_some_and(|unit| unit.id == provider.id),
        |caps| caps.healing(),
    )
}
