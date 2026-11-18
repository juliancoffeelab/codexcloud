//! Recruitment prices, ownership, income and healing behavior.
use super::*;

#[test]
fn recruitment_requires_gold_legal_destination_and_owned_keep() {
    let kind = "scout";
    let mut game = LiveBattle::new(rules(), "mirror");
    let leader = game
        .units
        .iter()
        .find(|unit| unit.phase == TurnPhase::Player && unit.is_leader())
        .unwrap()
        .id;
    let recruit = game.kind(kind);
    let price = game.recruit_cost(game.unit(leader).unwrap(), recruit);
    game.gold["player"] = price;
    let destination = game.placement_tiles(game.unit(leader).unwrap(), recruit, true)[0];
    let command = Command::Recruit {
        unit: UnitId(leader),
        kind: kind.into(),
        tile: game.board.ids[destination].into(),
    };
    game.gold["player"] = price - 1;
    let before = serde_json::to_value(game.state()).unwrap();
    assert!(
        game.placement_tiles(game.unit(leader).unwrap(), recruit, true)
            .is_empty()
    );
    assert_eq!(
        game.execute(command.clone()).unwrap_err(),
        CommandError::IllegalRecruit
    );
    assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
    game.gold["player"] = price;
    let id = game.next_id;
    game.execute(command.clone()).unwrap();
    assert_eq!(game.gold["player"], 0);
    assert_eq!(game.unit(id).unwrap().kind, recruit);
    assert!(game.unit(id).unwrap().moved && game.unit(id).unwrap().attack_spent);
    assert!(game.execute(command).is_err());
    game.end_turn();
    game.end_turn();
    assert!(!game.unit(id).unwrap().moved && !game.unit(id).unwrap().attack_spent);
    let keep = game.unit(leader).unwrap().tile;
    game.gold["player"] = price;
    game.owners[keep] = Some("enemy".into());
    assert!(
        game.placement_tiles(game.unit(leader).unwrap(), recruit, true)
            .is_empty()
    );
}

#[test]
fn village_capture_changes_income_and_heals_only_the_owners_occupant() {
    let mut game = flat();
    let village = game.site(&Hex::new(3, 3));
    game.set_terrain(village, game.definitions.terrain_name("village"));
    let id = add(&mut game, "scout", "player", Hex::new(2, 3));
    game.move_unit(id, village);
    assert_eq!(game.owners[village], Some("player".into()));
    let index = game.index(id).unwrap();
    game.units[index].hp = 1;
    let gold = game.gold["player"];
    game.end_turn();
    game.end_turn();
    assert_eq!(game.gold["player"], gold + 2);
    assert_eq!(game.unit(id).unwrap().hp, 5);
    game.owners[village] = Some("enemy".into());
    game.end_turn();
    game.end_turn();
    assert_eq!(game.gold["player"], gold + 2);
    assert_eq!(game.unit(id).unwrap().hp, 5);
    let grove = game.definitions.terrain_name("grove");
    game.set_terrain(village, grove);
    game.owners[village] = Some("player".into());
    let hp = game.definitions.unit(game.unit(id).unwrap().kind).max_hp;
    let index = game.index(id).unwrap();
    game.units[index].hp = hp - 1;
    game.end_turn();
    game.end_turn();
    assert_eq!(game.unit(id).unwrap().hp, hp);
}

#[test]
fn strongest_adjacent_friendly_healer_wins_without_stacking_or_overhealing() {
    let mut game = flat();
    let ally = add(&mut game, "reefguard", "player", Hex::new(3, 3));
    add(&mut game, "wavemender", "player", Hex::new(2, 3));
    let healer = add(&mut game, "tidesage", "player", Hex::new(4, 3));
    let enemy = add(&mut game, "reefguard", "enemy", Hex::new(4, 4));
    for id in [ally, enemy] {
        let index = game.index(id).unwrap();
        game.units[index].hp = 1;
    }
    game.turn = TurnPhase::Ai;
    game.end_turn();
    let strongest = healing_from_at(
        &game,
        game.unit(healer).unwrap(),
        game.unit(ally).unwrap(),
        game.unit(healer).unwrap().tile,
    );
    assert_eq!(game.unit(ally).unwrap().hp, 1 + strongest);
    assert_eq!(game.unit(enemy).unwrap().hp, 1);
    let maximum = game.definitions.unit(game.kind("reefguard")).max_hp;
    let index = game.index(ally).unwrap();
    game.units[index].hp = maximum - 1;
    game.end_turn();
    assert_eq!(game.unit(enemy).unwrap().hp, 1);
    game.end_turn();
    assert_eq!(game.unit(ally).unwrap().hp, maximum);
}

#[test]
#[cfg(feature = "simulation")]
fn healing_records_credit_tied_strongest_sources_after_terrain_and_hp_caps() {
    let mut game = flat();
    let ally = add(&mut game, "reefguard", "player", Hex::new(3, 3));
    let first = add(&mut game, "tidesage", "player", Hex::new(2, 3));
    let second = add(&mut game, "tidesage", "player", Hex::new(4, 3));
    add(&mut game, "wavemender", "player", Hex::new(3, 4));
    let tile = game.site(&Hex::new(3, 3));
    game.set_terrain(tile, game.definitions.terrain_kind(Terrain::Village));
    game.owners[tile] = Some("player".into());
    let index = game.index(ally).unwrap();
    game.units[index].hp = game.definitions.unit(game.units[index].kind).max_hp - 5;
    game.recordings = Some(Records::default().into());
    game.begin_turn(TurnPhase::Player);
    let events = &game.recordings.as_ref().unwrap().healing;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].amount, 1);
    assert_eq!(events[0].target, format!("u{ally}"));
    assert_eq!(
        events[0].providers,
        vec![format!("u{first}"), format!("u{second}")]
    );
    game.recordings = Some(Records::default().into());
    game.begin_turn(TurnPhase::Player);
    assert!(game.recordings.as_ref().unwrap().healing.is_empty());
}

#[test]
fn authored_healing_radius_self_and_targets_drive_forecasts_and_turns() {
    for targets in [
        HealingTargets::OwnFaction,
        HealingTargets::Allies,
        HealingTargets::Enemies,
        HealingTargets::Everyone,
    ] {
        for include_self in [false, true] {
            let mut bundle = crate::catalogue_fixture::deterministic();
            crate::effect_fixture::healing(
                &mut bundle,
                "wavemender",
                Some((3, 2, include_self, targets)),
            );
            let local = Catalogue::from_json_bundle(&bundle).unwrap();
            let mut game = LiveBattle::from_state(&local, flat().state());
            let healer = add(&mut game, "wavemender", "player", Hex::new(3, 3));
            let ally = add(&mut game, "reefguard", "player", Hex::new(1, 3));
            let enemy = add(&mut game, "reefguard", "enemy", Hex::new(5, 3));
            let far = add(&mut game, "reefguard", "player", Hex::new(0, 3));
            for id in [healer, ally, enemy, far] {
                let index = game.index(id).unwrap();
                game.units[index].hp = 1;
            }
            let friendly = !matches!(targets, HealingTargets::Enemies);
            let hostile = matches!(targets, HealingTargets::Enemies | HealingTargets::Everyone);
            let provider = game.unit(healer).unwrap();
            assert_eq!(
                healing_from_at(&game, provider, game.unit(ally).unwrap(), provider.tile),
                if friendly { 3 } else { 0 }
            );
            assert_eq!(
                healing_from_at(&game, provider, game.unit(enemy).unwrap(), provider.tile),
                if hostile { 3 } else { 0 }
            );
            assert_eq!(
                healing_from_at(&game, provider, game.unit(far).unwrap(), provider.tile),
                0
            );
            assert_eq!(
                healing_from_at(&game, provider, provider, game.site(&Hex::new(6, 6))),
                if friendly && include_self { 3 } else { 0 }
            );
            game.begin_turn(TurnPhase::Player);
            assert_eq!(game.unit(ally).unwrap().hp, if friendly { 4 } else { 1 });
            assert_eq!(
                game.unit(healer).unwrap().hp,
                if friendly && include_self { 4 } else { 1 }
            );
            assert_eq!(game.unit(far).unwrap().hp, 1);
            game.begin_turn(TurnPhase::Ai);
            assert_eq!(game.unit(enemy).unwrap().hp, if hostile { 4 } else { 1 });
        }
    }
}
