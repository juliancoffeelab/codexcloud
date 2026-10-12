//! Native advancement, XP and campaign carryover regressions.
use super::*;

#[test]
fn internal_completion_queries_match_published_progress_at_goal_transitions() {
    for (campaign, index) in [
        ("lantern-road", 0),
        ("green-accord", 1),
        ("ashen-oath", 1),
        ("borrowed-tide", 1),
    ] {
        let mut game =
            Game::start_campaign(rules(), campaign, index, vec![], 0, vec![], vec![], vec![]);
        let goals = game.mission.as_ref().unwrap().goals.clone();
        for goal in &goals {
            assert_eq!(game.goal_done(goal), game.goal_progress(goal).done);
        }
        game.units.retain(|unit| unit.side == Side::Player);
        game.roster_changed();
        game.owners.fill(Some(Side::Player));
        game.round = 100;
        for side in [Side::Enemy, Side::Player] {
            game.turn = side;
            for held in [0, 1, 100] {
                let state = game.campaign.as_mut().unwrap();
                for goal in &goals {
                    for tile in goal.tiles() {
                        state.marks.push(format!("sealed:{tile}"));
                    }
                    if let Goal::Event { event, .. } = goal {
                        state.marks.push(format!("event:{event}"));
                    }
                    if let Goal::Hold {
                        start_event: Some(event),
                        ..
                    } = goal
                    {
                        state.marks.push(format!("event:{event}"));
                    }
                }
                if let Some(holds) = &mut state.holds {
                    for counter in holds.values_mut() {
                        counter.held_turns = held;
                    }
                }
                if let Some(vigil) = &mut state.vigil {
                    vigil.held_turns = held;
                }
                if let Some(refuge) = &mut state.refuge {
                    refuge.held_turns = held;
                }
                for goal in &goals {
                    assert_eq!(game.goal_done(goal), game.goal_progress(goal).done);
                }
            }
        }
    }
}

#[test]
fn mission_cache_reuses_unmodified_content_and_refreshes_story_patches() {
    let mut game =
        Game::start_campaign(rules(), "ashen-oath", 1, vec![], 0, vec![], vec![], vec![]);
    let original = game.mission.clone().unwrap();
    game.campaign
        .as_mut()
        .unwrap()
        .marks
        .push("unrelated".into());
    game.refresh_mission();
    assert!(std::rc::Rc::ptr_eq(
        &original,
        game.mission.as_ref().unwrap()
    ));
    game.campaign
        .as_mut()
        .unwrap()
        .marks
        .push("dain-fell".into());
    game.refresh_mission();
    assert!(!std::rc::Rc::ptr_eq(
        &original,
        game.mission.as_ref().unwrap()
    ));
    let cached = serde_json::to_value(game.mission.as_deref()).unwrap();
    let restored = Game::from_state(rules(), game.state());
    assert_eq!(
        cached,
        serde_json::to_value(restored.mission.as_deref()).unwrap()
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
        let family = rules().types[rules().units[kind].family.as_str()];
        assert!(!rules().units[family].advances_to.is_empty());
        visit(rules(), kind, &mut HashSet::new());
    }
}

#[test]
fn xp_requires_real_damage_and_promotion_preserves_identity_and_wounds() {
    let mut game = flat();
    let id = add(&mut game, "swordsman", Side::Player, "3,3");
    let target = add(&mut game, "archer", Side::Enemy, "4,3");
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
    game.units[index].attacked = false;
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
    assert!(promoted.moved && promoted.attacked);
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
        .find(|u| u.side == Side::Player && u.role == Some(Role::Leader))
        .unwrap()
        .id;
    let troop = game
        .units
        .iter()
        .find(|u| u.side == Side::Player && game.rules.units[u.kind].id == "reefguard")
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
        .position(|u| u.name.as_deref() == Some("Edda"))
        .unwrap();
    game.units[pilot].experience = Some(7);
    game.winner = Some(Side::Player);
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
            .filter(|u| u.name.as_deref() == Some("Edda"))
            .count(),
        1
    );
    let pilot = game
        .units
        .iter()
        .position(|u| u.name.as_deref() == Some("Edda"))
        .unwrap();
    assert_eq!(game.units[pilot].experience, Some(7));
    game.units[pilot].experience = Some(10);
    game.execute(Command::RetryMission).unwrap();
    assert_eq!(
        game.units
            .iter()
            .find(|u| u.name.as_deref() == Some("Edda"))
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
