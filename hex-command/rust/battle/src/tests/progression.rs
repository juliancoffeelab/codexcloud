//! Native advancement, XP and campaign carryover regressions.
use super::*;
use crate::logic::event_queries::EventFacts;

#[test]
fn named_hold_requires_its_actor_and_resets_without_losing_ownership() {
    let mut bundle = crate::catalogue_fixture::deterministic();
    let tile = Hex::new(5, 4);
    let event:ScriptEvent=serde_json::from_value(serde_json::json!({"id":"hold","remembers":true,"priority":0,"when":{"held":{"unitTypes":[],"tile":{"hex":tile},"character":"mirror-captain","turns":3,"when":{"own":{"tile":{"hex":tile},"faction":"player"}}}},"objective":{"optional":false,"faction":"player","label":"Defend","visibleWhen":{"always":true},"intent":{"hold":{"unitTypes":[],"tile":{"hex":tile},"character":"mirror-captain","turns":3}}},"actions":[{"victory":"Done"}]})).unwrap();
    crate::catalogue_fixture::campaign_events(
        &mut bundle,
        "mirror-night",
        0,
        serde_json::to_value(vec![event.clone()]).unwrap(),
    );
    let rules = Catalogue::from_json_bundle(&bundle).unwrap();
    let mut game =
        LiveBattle::start_campaign(&rules, "mirror-night", 0, vec![], 0, vec![], vec![], vec![]);
    let wreck = game.site(&tile);
    game.owners[wreck] = Some("player".into());
    game.units.retain(|u| u.phase == TurnPhase::Player);
    game.roster_changed();
    let actor = game
        .units
        .iter()
        .find(|u| game.name(u.character) == Some("mirror-captain"))
        .unwrap()
        .id;
    let evaluate = |g: &mut LiveBattle<'_>, round| {
        g.round = round;
        g.evaluate_campaign(true, None);
        let current = EventFacts::new(&g).objective_progress(&event).current;
        if current < 3 {
            let session = crate::BattleSession::from_state(g.catalogue, g.state());
            let objectives = session.campaign_view().planning_objectives();
            let objective = objectives.iter().find(|o| o.id == "hold").unwrap();
            assert_eq!(objective.progress_current, current);
            assert_eq!(objective.progress_total, 3);
            assert_eq!(objective.hold_turns_remaining, Some(3 - current));
        }
        current
    };
    assert_eq!(evaluate(&mut game, 6), 0);
    assert_eq!(evaluate(&mut game, 7), 0);
    game.units.iter_mut().find(|u| u.id == actor).unwrap().tile = wreck;
    game.roster_changed();
    assert_eq!(evaluate(&mut game, 8), 1);
    let away = (0..game.terrain.len())
        .find(|&t| t != wreck && game.at(t).is_none())
        .unwrap();
    game.units.iter_mut().find(|u| u.id == actor).unwrap().tile = away;
    game.roster_changed();
    game.evaluate_campaign(false, None);
    assert_eq!(EventFacts::new(&game).objective_progress(&event).current, 0);
    game.units.iter_mut().find(|u| u.id == actor).unwrap().tile = wreck;
    game.roster_changed();
    for (round, current) in [(9, 1), (10, 2), (11, 3)] {
        assert_eq!(evaluate(&mut game, round), current);
    }
    assert_eq!(game.winner, Some(BattleResult::PlayerVictory));
}
use std::collections::HashSet;

#[test]
fn authored_progression_ladders_are_acyclic_and_preserve_race_and_increase_level() {
    fn visit(catalogue: &Catalogue, unit: &UnitDefinition, path: &mut HashSet<String>) {
        assert!(path.insert(unit.id().to_owned()), "cyclic advancement");
        for next in unit.promotion_ids() {
            let advanced = catalogue.unit(next);
            assert_eq!(advanced.race(), unit.race());
            assert!(advanced.level() > unit.level());
            visit(catalogue, advanced, path);
        }
        path.remove(unit.id());
    }
    for unit in rules().units() {
        visit(rules(), unit, &mut HashSet::new());
    }
}

#[test]
fn xp_requires_real_damage_and_promotion_preserves_identity_and_wounds() {
    let mut game = flat();
    let id = add(&mut game, "swordsman", "player", Hex::new(3, 3));
    let target = add(&mut game, "archer", "enemy", Hex::new(4, 3));
    let index = game.index(id).unwrap();
    game.units[index].hp -= 4;
    let target_index = game.index(target).unwrap();
    game.units[target_index].hp = 3;
    hit(&mut game, id, target, "sword");
    assert_eq!(game.unit(id).unwrap().experience, Some(6));
    assert!(game.promotion_options(game.unit(id).unwrap()).is_empty());
    let needed = Some(game.definitions.unit(game.kind("swordsman")).max_xp);
    let index = game.index(id).unwrap();
    game.units[index].experience = needed;
    game.units[index].moved = false;
    game.units[index].attack_spent = false;
    let before = game.state();
    game.execute(Command::Advance {
        unit: UnitId(id),
        kind: "soldier".into(),
    })
    .unwrap();
    let promoted = game.unit(id).unwrap();
    assert_eq!(promoted.id, id);
    assert_eq!(promoted.hp, game.definitions.unit(promoted.kind).max_hp - 4);
    assert_eq!(promoted.experience, Some(0));
    assert!(promoted.moved && promoted.attack_spent);
    assert_eq!(before.units[0].r#type, "swordsman");
    assert_eq!(
        game.execute(Command::Advance {
            unit: UnitId(id),
            kind: "soldier".into()
        })
        .unwrap_err(),
        CommandError::IllegalAdvance
    );
}

#[test]
fn promoted_leader_veterans_and_named_companions_carry_forward_and_retry_rewinds_xp() {
    let mut game = chapter("lantern-road", 0);
    let leader = game
        .units
        .iter()
        .find(|u| u.phase == TurnPhase::Player && u.role == Some(Role::Leader))
        .unwrap()
        .id;
    let troop = add(&mut game, "scout", "player", Hex::new(1, 1));
    for (id, kind) in [(leader, "soldier"), (troop, "ranger")] {
        let index = game.index(id).unwrap();
        game.units[index].experience = Some(game.definitions.unit(game.units[index].kind).max_xp);
        game.execute(Command::Advance {
            unit: UnitId(id),
            kind: kind.into(),
        })
        .unwrap();
    }
    let pilot = game
        .units
        .iter()
        .position(|u| game.name(u.character) == Some("ivo"))
        .unwrap();
    game.units[pilot].experience = Some(7);
    game.winner = Some(BattleResult::PlayerVictory);
    game.execute(Command::RecallMission {
        units: vec![UnitId(troop)],
    })
    .unwrap();
    assert!(
        game.units.iter().any(
            |u| u.role == Some(Role::Leader) && game.definitions.unit(u.kind).id() == "soldier"
        )
    );
    assert!(
        game.units
            .iter()
            .any(|u| game.definitions.unit(u.kind).id() == "ranger" && u.experience == Some(0))
    );
    assert_eq!(
        game.units
            .iter()
            .filter(|u| game.name(u.character) == Some("ivo"))
            .count(),
        1
    );
    let pilot = game
        .units
        .iter()
        .position(|u| game.name(u.character) == Some("ivo"))
        .unwrap();
    assert_eq!(game.units[pilot].experience, Some(7));
    game.units[pilot].experience = Some(10);
    game.execute(Command::RetryMission).unwrap();
    assert_eq!(
        game.units
            .iter()
            .find(|u| game.name(u.character) == Some("ivo"))
            .unwrap()
            .experience,
        Some(7)
    );
    let restored = LiveBattle::try_from_state(rules(), game.state()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.state()).unwrap(),
        serde_json::to_value(game.state()).unwrap()
    );
}

#[test]
fn planning_deadline_reports_only_unconditional_defeat_clocks() {
    let mut bundle = crate::catalogue_fixture::deterministic();
    let events: Vec<ScriptEvent> = serde_json::from_value(serde_json::json!([
        {"id":"conditional","remembers":true,"priority":0,"when":{"all":[{"turnAtLeast":4},{"flag":"unknown"}]},"actions":[{"defeat":"Conditional"}]},
        {"id":"later","remembers":true,"priority":0,"when":{"turnAtLeast":31},"actions":[{"defeat":"Too late"}]},
        {"id":"earlier","remembers":true,"priority":0,"when":{"turn":20},"actions":[{"defeat":"Too late"}]},
        {"id":"reinforcement","remembers":true,"priority":0,"when":{"turn":3},"actions":[{"log":"Not a deadline"}]}
    ])).unwrap();
    crate::catalogue_fixture::campaign_events(
        &mut bundle,
        "mirror-night",
        0,
        serde_json::to_value(events).unwrap(),
    );
    let rules = Catalogue::from_json_bundle(&bundle).unwrap();
    let game =
        LiveBattle::start_campaign(&rules, "mirror-night", 0, vec![], 0, vec![], vec![], vec![]);
    assert_eq!(
        crate::BattleSession::from_state(game.catalogue, game.state())
            .campaign_view()
            .deadline_round(),
        Some(20)
    );
}

#[test]
fn objective_progress_and_disclosure_use_the_assigned_faction() {
    let mut bundle = crate::catalogue_fixture::deterministic();
    let tile = Hex::new(2, 1);
    let events: Vec<ScriptEvent> = serde_json::from_value(serde_json::json!([
        {"id":"enemy-control","remembers":true,"priority":0,"when":{"always":false}, "objective":{
            "optional":false,"faction":"toll-skeletons", "label":"Control", "visibleWhen":{"always":true},
            "intent":{"capture":[{"hex":tile}]}}, "actions":[]},
        {"id":"hidden-enemy-duty","remembers":true,"priority":0,"when":{"always":false}, "objective":{
            "optional":false,"faction":"toll-skeletons", "label":"Secret", "visibleWhen":{"stand":{"unitTypes":[],"through":false,"tile":{"hex":tile},"side":"observer","role":"leader"}},
            "intent":{"goTo":{"through":false,"unitTypes":[],"tile":{"hex":tile},"side":"observer","role":"leader"}}}, "actions":[]}
    ])).unwrap();
    crate::catalogue_fixture::campaign_events(
        &mut bundle,
        "lantern-road",
        1,
        serde_json::to_value(&events).unwrap(),
    );
    let rules = Catalogue::from_json_bundle(&bundle).unwrap();
    let mut game =
        LiveBattle::start_campaign(&rules, "lantern-road", 1, vec![], 0, vec![], vec![], vec![]);
    let site = game.site(&tile);
    game.owners[site] = Some("toll-skeletons".into());
    assert_eq!(
        EventFacts::new(&game)
            .objective_progress(&events[0])
            .current,
        1
    );
    assert!(
        campaign_session(&game, "player")
            .campaign_view()
            .planning_objectives()
            .is_empty()
    );
    assert_eq!(
        campaign_session(&game, "toll-skeletons")
            .campaign_view()
            .planning_objectives()
            .len(),
        1
    );
    assert!(
        campaign_session(&game, "player")
            .campaign_view()
            .requirements_disclosed()
    );
    assert!(
        !campaign_session(&game, "toll-skeletons")
            .campaign_view()
            .requirements_disclosed()
    );
    let revision = game.story.planning_revision;
    let leader = game
        .units
        .iter()
        .position(|u| game.unit_faction(u) == "toll-skeletons" && u.is_leader())
        .unwrap();
    game.units[leader].tile = site;
    game.roster_changed();
    game.evaluate_campaign(false, None);
    assert!(game.story.planning_revision > revision);
    assert_eq!(
        campaign_session(&game, "toll-skeletons")
            .campaign_view()
            .planning_objectives()
            .len(),
        2
    );
    let restored = LiveBattle::try_from_state(&rules, game.state()).unwrap();
    assert_eq!(
        campaign_session(&restored, "toll-skeletons")
            .campaign_view()
            .planning_objectives()[0]
            .progress_current,
        1
    );
    assert!(
        campaign_session(&restored, "player")
            .campaign_view()
            .planning_objectives()
            .is_empty()
    );
}

// Exercise campaign queries through the same observer grant as their consumer.
fn campaign_session<'a>(game: &LiveBattle<'a>, faction: &str) -> crate::BattleSession<'a> {
    let mut state = game.state();
    state.turn = game.faction(faction).unwrap().phase;
    let mut session = crate::BattleSession::from_state(game.catalogue, state);
    session.set_active_faction(Some(faction.into()));
    session
}
