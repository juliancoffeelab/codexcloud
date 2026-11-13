//! Native advancement, XP and campaign carryover regressions.
use super::*;

#[test]
fn named_hold_requires_its_actor_and_resets_without_losing_ownership() {
    let mut rules = Rules::load().unwrap();
    let tile = Hex::new(5, 4);
    let event:ScriptEvent=serde_json::from_value(serde_json::json!({"id":"hold","when":{"held":{"tile":{"hex":tile},"character":"edda","turns":3,"when":{"own":{"tile":{"hex":tile},"faction":"player"}}}},"objective":{"label":"Defend","visibleWhen":{"always":true},"intent":{"hold":{"tile":{"hex":tile},"character":"edda","turns":3}}},"actions":[{"victory":"Done"}]})).unwrap();
    rules.campaigns.get_mut("borrowed-tide").unwrap().missions[1].scripted_events =
        vec![event.clone()];
    let mut game = Game::start_campaign(
        &rules,
        "borrowed-tide",
        1,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    );
    let wreck = game.site(&tile);
    game.owners[wreck] = Some("player".into());
    game.units.retain(|u| u.side == TurnPhase::Player);
    game.roster_changed();
    let actor = game
        .units
        .iter()
        .find(|u| game.name(u.character) == Some("edda"))
        .unwrap()
        .id;
    let evaluate = |g: &mut Game<'_>, round| {
        g.round = round;
        g.evaluate_campaign(true, None);
        let current = g.scripted_objective_status(&event).progress.current;
        if current < 3 {
            let objectives = g.planning_objectives();
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
    assert_eq!(game.scripted_objective_status(&event).progress.current, 0);
    game.units.iter_mut().find(|u| u.id == actor).unwrap().tile = wreck;
    game.roster_changed();
    for (round, current) in [(9, 1), (10, 2), (11, 3)] {
        assert_eq!(evaluate(&mut game, round), current);
    }
    assert_eq!(game.winner, Some(BattleResult::PlayerVictory));
}
#[test]
fn mission_cache_keeps_authored_content_while_story_resolves_from_state() {
    let mut game =
        Game::start_campaign(rules(), "ashen-oath", 1, vec![], 0, vec![], vec![], vec![]);
    let original = game.story.mission.clone().unwrap();
    let before = game.resolved_story().unwrap().briefing;
    game.story
        .campaign
        .as_mut()
        .unwrap()
        .set_flag("peace-lost", true, 1);
    game.refresh_mission();
    assert!(std::rc::Rc::ptr_eq(
        &original,
        game.story.mission.as_ref().unwrap()
    ));
    assert_ne!(before, game.resolved_story().unwrap().briefing);
    let restored = Game::from_state(rules(), game.state());
    assert_eq!(
        game.resolved_story().unwrap().briefing,
        restored.resolved_story().unwrap().briefing
    );
}
use std::collections::HashSet;

#[test]
fn authored_progression_ladders_are_acyclic_and_preserve_race_and_increase_level() {
    fn visit(rules: &Rules, kind: usize, path: &mut HashSet<usize>) {
        assert!(path.insert(kind), "cyclic advancement");
        let unit = &rules.units[kind];
        for next in &unit.advances_to {
            let advanced = rules.types[next];
            let rule = &rules.units[advanced];
            assert_eq!(rule.race, unit.race);
            assert!(rule.level > unit.level);
            visit(rules, advanced, path);
        }
        path.remove(&kind);
    }
    for kind in 0..rules().units.len() {
        visit(rules(), kind, &mut HashSet::new());
    }
}

#[test]
fn xp_requires_real_damage_and_promotion_preserves_identity_and_wounds() {
    let mut game = flat();
    let id = add(&mut game, "swordsman", TurnPhase::Player, Hex::new(3, 3));
    let target = add(&mut game, "archer", TurnPhase::Ai, Hex::new(4, 3));
    let index = game.index(id).unwrap();
    game.units[index].hp -= 4;
    let target_index = game.index(target).unwrap();
    game.units[target_index].hp = 3;
    hit(&mut game, id, target, "sword");
    assert_eq!(game.unit(id).unwrap().experience, Some(6));
    assert!(game.promotion_options(game.unit(id).unwrap()).is_empty());
    let needed = game.rules.units[game.kind("swordsman")].experience_needed;
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
    assert_eq!(promoted.hp, game.rules.units[promoted.kind].hp - 4);
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
    let mut game = chapter("borrowed-tide", 0);
    let leader = game
        .units
        .iter()
        .find(|u| u.side == TurnPhase::Player && u.role == Some(Role::Leader))
        .unwrap()
        .id;
    let troop = game
        .units
        .iter()
        .find(|u| u.side == TurnPhase::Player && game.rules.units[u.kind].id == "reefguard")
        .unwrap()
        .id;
    for (id, kind) in [(leader, "tideseer"), (troop, "tridentguard")] {
        let index = game.index(id).unwrap();
        game.units[index].experience = game.rules.units[game.units[index].kind].experience_needed;
        game.execute(Command::Advance {
            unit: UnitId(id),
            kind: kind.into(),
        })
        .unwrap();
    }
    let pilot = game
        .units
        .iter()
        .position(|u| game.name(u.character) == Some("edda"))
        .unwrap();
    game.units[pilot].experience = Some(7);
    game.winner = Some(BattleResult::PlayerVictory);
    game.execute(Command::RecallMission {
        units: vec![UnitId(troop)],
    })
    .unwrap();
    assert!(
        game.units
            .iter()
            .any(|u| u.role == Some(Role::Leader) && game.rules.units[u.kind].id == "tideseer")
    );
    assert!(
        game.units
            .iter()
            .any(|u| game.rules.units[u.kind].id == "tridentguard" && u.experience == Some(0))
    );
    assert_eq!(
        game.units
            .iter()
            .filter(|u| game.name(u.character) == Some("edda"))
            .count(),
        1
    );
    let pilot = game
        .units
        .iter()
        .position(|u| game.name(u.character) == Some("edda"))
        .unwrap();
    assert_eq!(game.units[pilot].experience, Some(7));
    game.units[pilot].experience = Some(10);
    game.execute(Command::RetryMission).unwrap();
    assert_eq!(
        game.units
            .iter()
            .find(|u| game.name(u.character) == Some("edda"))
            .unwrap()
            .experience,
        Some(7)
    );
    let restored = Game::try_from_state(rules(), game.state()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.state()).unwrap(),
        serde_json::to_value(game.state()).unwrap()
    );
}

#[test]
fn planning_deadline_reports_only_unconditional_defeat_clocks() {
    let mut rules = Rules::load().unwrap();
    let events = serde_json::from_value(serde_json::json!([
        {"id":"conditional","when":{"all":[{"turnAtLeast":4},{"flag":"unknown"}]},"actions":[{"defeat":"Conditional"}]},
        {"id":"later","when":{"turnAtLeast":31},"actions":[{"defeat":"Too late"}]},
        {"id":"earlier","when":{"turn":20},"actions":[{"defeat":"Too late"}]},
        {"id":"reinforcement","when":{"turn":3},"actions":[{"log":"Not a deadline"}]}
    ])).unwrap();
    rules.campaigns.get_mut("borrowed-tide").unwrap().missions[1].scripted_events = events;
    let game = Game::start_campaign(
        &rules,
        "borrowed-tide",
        1,
        vec![],
        0,
        vec![],
        vec![],
        vec![],
    );
    assert_eq!(game.planning_deadline_round(), Some(20));
}
