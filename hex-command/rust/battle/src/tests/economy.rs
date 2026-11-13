//! Recruitment prices, ownership, income and healing behavior.
use super::*;

#[test]
fn recruitment_requires_gold_legal_destination_and_owned_keep() {
    let kind = "scout";
    let mut game = Game::new(rules(), "mirror");
    let leader = game
        .units
        .iter()
        .find(|unit| unit.phase == TurnPhase::Player && unit.is_leader())
        .unwrap()
        .id;
    let recruit = game.kind(kind);
    let price = game.rules.units[recruit].cost;
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
    game.set_terrain(village, game.rules.terrains["village"]);
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
    let grove = game.rules.terrains["grove"];
    game.set_terrain(village, grove);
    game.owners[village] = Some("player".into());
    let hp = game.rules.units[game.unit(id).unwrap().kind].hp;
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
    add(&mut game, "tidesage", "player", Hex::new(4, 3));
    let enemy = add(&mut game, "reefguard", "enemy", Hex::new(4, 4));
    for id in [ally, enemy] {
        let index = game.index(id).unwrap();
        game.units[index].hp = 1;
    }
    game.turn = TurnPhase::Ai;
    game.end_turn();
    let strongest = game.rules.units[game.kind("tidesage")].healing;
    assert_eq!(game.unit(ally).unwrap().hp, 1 + strongest);
    assert_eq!(game.unit(enemy).unwrap().hp, 1);
    let maximum = game.rules.units[game.kind("reefguard")].hp;
    let index = game.index(ally).unwrap();
    game.units[index].hp = maximum - 1;
    game.end_turn();
    assert_eq!(game.unit(enemy).unwrap().hp, 1);
    game.end_turn();
    assert_eq!(game.unit(ally).unwrap().hp, maximum);
}

#[test]
fn healing_records_credit_tied_strongest_sources_after_terrain_and_hp_caps() {
    let mut game = flat();
    let ally = add(&mut game, "reefguard", "player", Hex::new(3, 3));
    let first = add(&mut game, "tidesage", "player", Hex::new(2, 3));
    let second = add(&mut game, "tidesage", "player", Hex::new(4, 3));
    add(&mut game, "wavemender", "player", Hex::new(3, 4));
    let tile = game.site(&Hex::new(3, 3));
    game.terrain[tile] = game
        .rules
        .terrain_types
        .iter()
        .position(|terrain| *terrain == Terrain::Village)
        .unwrap();
    game.owners[tile] = Some("player".into());
    let index = game.index(ally).unwrap();
    game.units[index].hp = game.rules.units[game.units[index].kind].hp - 5;
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
