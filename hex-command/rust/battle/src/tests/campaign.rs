//! Objective timing, legal authored missions and native checkpoint validation.
use super::*;

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
fn story_companions_are_not_eligible_for_explicit_recall() {
    let mut game = chapter("lantern-road", 0);
    let dain = game
        .units
        .iter()
        .find(|u| game.name(u.character) == Some("ivo"))
        .unwrap()
        .id;
    game.winner = Some(BattleResult::PlayerVictory);
    assert_eq!(
        game.execute(Command::RecallMission {
            units: vec![UnitId(dain)]
        })
        .unwrap_err(),
        CommandError::NoNextMission
    );
    let ordinary = add(&mut game, "swordsman", "player", Hex::new(1, 1));
    game.execute(Command::RecallMission {
        units: vec![UnitId(ordinary)],
    })
    .unwrap();
    assert_eq!(
        game.units
            .iter()
            .filter(|u| game.name(u.character) == Some("ivo"))
            .count(),
        1
    );
}

#[test]
fn native_restore_rejects_invalid_positions_hit_points_and_wrong_campaign_map() {
    let state = Game::new(rules(), "developer").state();
    let mut outside = state.clone();
    outside.units[0].tile = Hex::new(-1, -1);
    assert!(Game::try_from_state(rules(), outside).is_err());
    let mut hp = state.clone();
    hp.units[0].hp = 0;
    assert!(Game::try_from_state(rules(), hp).is_err());
    let mut attached = state.clone();
    attached.campaign = chapter("mirror-night", 0).state().campaign.map(Into::into);
    assert!(Game::try_from_state(rules(), attached).is_err());
}
