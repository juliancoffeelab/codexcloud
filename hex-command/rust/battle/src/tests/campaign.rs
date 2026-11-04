//! Objective timing, legal authored missions and native checkpoint validation.
use super::*;

fn advance_to(game: &mut Game<'_>, round: u32) {
    while game.round < round {
        game.end_turn();
        game.end_turn();
    }
}
fn reach_wreck(game: &mut Game<'_>) {
    let index = game
        .units
        .iter()
        .position(|u| u.character.as_deref() == Some("edda"))
        .unwrap();
    game.units[index].tile = game.site("hex-0859-8a12839c45");
    game.roster_changed();
    game.move_unit(game.units[index].id, game.site("hex-0891-b20a50ef8c"));
}

#[test]
fn late_rescue_arrivals_start_immediately_then_require_three_defended_rounds() {
    for arrival in [6, 8] {
        let mut game = chapter("borrowed-tide", 1);
        advance_to(&mut game, arrival);
        assert!(game.campaign.as_ref().unwrap().chosen("tide-arrived"));
        assert!(!game.campaign.as_ref().unwrap().chosen("rescue"));
        reach_wreck(&mut game);
        assert!(game.campaign.as_ref().unwrap().chosen("rescue"));
        assert!(!game.campaign.as_ref().unwrap().chosen("await-tide"));
        advance_to(&mut game, arrival + 2);
        assert_eq!(game.winner, None);
        advance_to(&mut game, arrival + 3);
        assert_eq!(game.winner, Some(Side::Player));
    }
}

#[test]
fn enemy_recapture_resets_defense_and_pre_tide_loss_postpones_rescue() {
    for before_tide in [false, true] {
        let mut game = chapter("borrowed-tide", 1);
        if !before_tide {
            game.round = 6;
        }
        reach_wreck(&mut game);
        game.end_turn();
        game.end_turn();
        if !before_tide {
            assert_eq!(
                game.scripts
                    .as_ref()
                    .unwrap()
                    .holds
                    .iter()
                    .find(|h| h.event == "goal-1")
                    .unwrap()
                    .turns,
                1
            );
        }
        let edda = game
            .units
            .iter()
            .find(|u| u.character.as_deref() == Some("edda"))
            .unwrap()
            .id;
        game.move_unit(edda, game.site("hex-0888-a99d723b63"));
        let raider = add(&mut game, "swordsman", Side::Enemy, "hex-0923-ec307e92cf");
        game.end_turn();
        game.move_unit(raider, game.site("hex-0891-b20a50ef8c"));
        game.end_turn();
        assert_eq!(
            game.scripts
                .as_ref()
                .unwrap()
                .holds
                .iter()
                .find(|h| h.event == "goal-1")
                .map(|clock| clock.turns)
                .unwrap_or(0),
            0
        );
        assert_eq!(game.winner, None);
        let restored = Game::try_from_state(rules(), game.state()).unwrap();
        assert_eq!(
            serde_json::to_value(restored.state()).unwrap(),
            serde_json::to_value(game.state()).unwrap()
        );
        if before_tide {
            advance_to(&mut game, 6);
            assert!(!game.campaign.as_ref().unwrap().chosen("rescue"));
            let index = game.index(raider).unwrap();
            game.units[index].tile = game.site("hex-0923-ec307e92cf");
            game.roster_changed();
            game.move_unit(edda, game.site("hex-0891-b20a50ef8c"));
            assert!(game.campaign.as_ref().unwrap().chosen("rescue"));
        }
    }
}

#[test]
fn authored_missions_have_valid_positions_and_restore_cleanly() {
    for campaign in rules().campaigns.values() {
        for (index, mission) in campaign.missions.iter().enumerate() {
            let game = chapter(&campaign.id, index);
            for wave in &mission.waves {
                for unit in &wave.units {
                    assert!(game.board.indices.contains_key(&unit.tile));
                }
            }
            assert!(Game::try_from_state(rules(), game.state()).is_ok());
        }
    }
}

#[test]
fn any_death_escalates_mediation() {
    let mut game = chapter("ashen-oath", 1);
    let defender = game.at(game.site("hex-0891-b20a50ef8c")).unwrap().id;
    let index = game
        .units
        .iter()
        .position(|u| u.side == Side::Enemy && game.rules.units[u.kind].id == "soldier")
        .unwrap();
    let target = game.units[index].id;
    game.units[index].tile = game.site("hex-0923-ec307e92cf");
    game.units[index].hp = 1;
    game.roster_changed();
    hit(&mut game, defender, target, "shieldbash");
    assert!(game.campaign.as_ref().unwrap().chosen("peace-lost"));
}

#[test]
fn story_companions_are_not_eligible_for_explicit_recall() {
    let mut game = chapter("green-accord", 1);
    let dain = game
        .units
        .iter()
        .find(|u| u.character.as_deref() == Some("dain"))
        .unwrap()
        .id;
    game.winner = Some(Side::Player);
    assert_eq!(
        game.execute(Command::RecallMission {
            units: vec![UnitId(dain)]
        })
        .unwrap_err(),
        CommandError::NoNextMission
    );
    let ordinary = add(&mut game, "warden", Side::Player, "hex-0034-53333de58a");
    game.execute(Command::RecallMission {
        units: vec![UnitId(ordinary)],
    })
    .unwrap();
    assert_eq!(
        game.units
            .iter()
            .filter(|u| u.character.as_deref() == Some("dain"))
            .count(),
        1
    );
}

#[test]
fn native_restore_rejects_invalid_positions_hit_points_and_wrong_campaign_map() {
    let state = Game::new(rules(), "developer").state();
    let mut outside = state.clone();
    outside.units[0].tile = "missing".into();
    assert!(Game::try_from_state(rules(), outside).is_err());
    let mut hp = state.clone();
    hp.units[0].hp = 0;
    assert!(Game::try_from_state(rules(), hp).is_err());
    let mut attached = state.clone();
    attached.campaign = chapter("green-accord", 0).state().campaign.map(Into::into);
    assert!(Game::try_from_state(rules(), attached).is_err());
}
