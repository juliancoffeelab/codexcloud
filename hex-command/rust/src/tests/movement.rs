//! Native movement, occupancy, geometry and authored-map invariants.
use super::*;
use std::collections::HashSet;

#[test]
fn threat_flood_matches_ordered_movement_with_blockers_and_terrain_changes() {
    use crate::logic::geometry::TileSet;
    for size in [61, 128, 135] {
        let mut state = flat().state();
        state.tiles = (0..size)
            .map(|tile| Tile {
                fog: None,
                light: None,
                id: format!("{},{}", tile % 9, tile / 9),
                col: (tile % 9) as i32,
                row: (tile / 9) as i32,
                terrain: "plains".into(),
                owner: None,
                connections: None,
                walls: None,
                dead_ends: None,
                floor: None,
            })
            .collect();
        let mut game = Game::from_state(rules(), state);
        for (index, kind) in ["swordsman", "reefguard", "scout"].into_iter().enumerate() {
            let tile = game.board.ids[index * 3].clone();
            add(&mut game, kind, Side::Player, &tile);
        }
        let blocker = game.add(game.kind("wolf"), Side::Enemy, 11);
        for terrain in ["forest", "water", "plains"] {
            for tile in (0..size).step_by(5) {
                game.set_terrain(tile, rules().terrains[terrain]);
            }
            let mut occupancy = vec![None; size];
            for unit in &game.units {
                occupancy[unit.tile] = Some(unit.side);
            }
            for unit in &game.units {
                for override_occupancy in [None, Some(&occupancy[..])] {
                    let ordered = if let Some(occupancy) = override_occupancy {
                        game.reachable_with_occupancy(unit, occupancy)
                    } else {
                        game.reachable(unit)
                    };
                    let mut expected = TileSet::new(size);
                    expected.insert(unit.tile);
                    for &(tile, _) in ordered.iter() {
                        expected.insert(tile);
                    }
                    assert_eq!(game.threat_reach(unit, override_occupancy), expected);
                }
            }
            occupancy[game.unit(blocker).unwrap().tile] = None;
            let unit = &game.units[0];
            let mut expected = TileSet::new(size);
            expected.insert(unit.tile);
            for &(tile, _) in game.reachable_with_occupancy(unit, &occupancy).iter() {
                expected.insert(tile);
            }
            assert_eq!(game.threat_reach(unit, Some(&occupancy)), expected);
        }
        let unit = game.units[0].clone();
        let cached = game.reachable(&unit);
        assert!(std::rc::Rc::ptr_eq(&cached, &game.fork().reachable(&unit)));
        let mut forecast = game.fork();
        forecast.set_terrain(1, rules().terrains["water"]);
        assert!(std::rc::Rc::ptr_eq(&cached, &game.reachable(&unit)));
        game.units[0].moved = true;
        let mut expected = TileSet::new(size);
        expected.insert(unit.tile);
        assert_eq!(game.threat_reach(&game.units[0], None), expected);
    }
}

#[test]
fn moves_obey_turns_actions_occupancy_and_passability() {
    let mut game = flat();
    let actor = add(&mut game, "swordsman", Side::Player, "2,3");
    add(&mut game, "scout", Side::Enemy, "3,3");
    let water = game.site("2,2");
    game.set_terrain(water, game.rules.terrains["water"]);
    assert!(
        !game
            .reachable(game.unit(actor).unwrap())
            .iter()
            .any(|&(tile, _)| tile == water || tile == game.site("3,3"))
    );
    let movement = Command::Move {
        unit: UnitId(actor),
        tile: "1,3".into(),
    };
    game.execute(movement.clone()).unwrap();
    assert!(game.reachable(game.unit(actor).unwrap()).is_empty());
    assert_eq!(
        game.execute(movement.clone()).unwrap_err(),
        CommandError::IllegalMove
    );
    game.end_turn();
    assert_eq!(game.execute(movement).unwrap_err(), CommandError::WrongTurn);
    game.end_turn();
    assert!(!game.unit(actor).unwrap().moved);
}

#[test]
fn routes_spend_the_whole_allowance_and_stop_at_blockers() {
    let kind = "scout";
    let terrain = "hill";
    let mut game = flat();
    let water = game.rules.terrains["water"];
    for tile in 0..game.terrain.len() {
        game.set_terrain(tile, water);
    }
    let start = game.site("0,0");
    game.set_terrain(start, game.rules.terrains["plains"]);
    for col in 1..=5 {
        let tile = game.site(&format!("{col},0"));
        game.set_terrain(tile, game.rules.terrains[terrain]);
    }
    let id = add(&mut game, kind, Side::Player, "0,0");
    let allowance = game.rules.units[game.kind(kind)].r#move;
    let goal = game.site(&format!("{allowance},0"));
    assert!(
        game.reachable(game.unit(id).unwrap())
            .contains(&(goal, allowance as u16)),
        "{kind}"
    );
    game.execute(Command::Move {
        unit: UnitId(id),
        tile: game.board.ids[goal].as_str().into(),
    })
    .unwrap();
    assert_eq!(game.unit(id).unwrap().tile, goal);
    assert!(game.reachable(game.unit(id).unwrap()).is_empty());
    let index = game.index(id).unwrap();
    game.units[index].tile = start;
    game.units[index].moved = false;
    game.units[index].attacked = true;
    game.roster_changed();
    assert!(game.reachable(game.unit(id).unwrap()).is_empty());
    game.units[index].attacked = false;
    add(&mut game, "wolf", Side::Enemy, "1,0");
    assert!(game.reachable(game.unit(id).unwrap()).is_empty());
}

#[test]
fn adjacency_matches_hex_distance_including_edges() {
    for (map, revision) in [("mirror", 1), ("river-valley", 2)] {
        let game = Game::from_state(rules(), rules().maps[&(map.into(), revision)].clone());
        for first in 0..game.board.ids.len() {
            let expected: Vec<_> = (0..game.board.ids.len())
                .filter(|&second| game.board.distance(first, second) == 1)
                .collect();
            assert_eq!(game.board.adjacent[first], expected);
        }
        let restored = Game::try_from_state(rules(), game.state()).unwrap();
        assert_eq!(restored.board.adjacent, game.board.adjacent);
    }
}

#[test]
fn artwork_choices_survive_restore_and_forks_without_changing_movement() {
    let original = Game::new(rules(), "mirror");
    let mut state = original.state();
    let tile = state
        .tiles
        .iter_mut()
        .find(|tile| tile.terrain == "plains")
        .unwrap();
    let id = tile.id.clone();
    tile.terrain = "road".into();
    tile.connections = Some(vec![HexDirection::Se, HexDirection::Ne]);
    tile.dead_ends = Some(vec![HexDirection::W]);
    tile.floor = Some("green-forest".into());
    let state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    let decorated = Game::try_from_state(rules(), state).unwrap();
    for unit in &original.units {
        assert_eq!(original.reachable(unit), decorated.reachable(unit));
    }
    for game in [&decorated, &decorated.fork()] {
        assert_eq!(
            game.state()
                .tiles
                .iter()
                .find(|tile| tile.id == id)
                .unwrap()
                .floor
                .as_deref(),
            Some("green-forest")
        );
        assert_eq!(
            game.state()
                .tiles
                .iter()
                .find(|tile| tile.id == id)
                .unwrap()
                .dead_ends,
            Some(vec![HexDirection::W])
        );
        assert_eq!(
            game.state()
                .tiles
                .iter()
                .find(|tile| tile.id == id)
                .unwrap()
                .connections,
            Some(vec![HexDirection::Se, HexDirection::Ne])
        );
    }
    let mut invalid = decorated.state();
    let tile = invalid.tiles.iter_mut().find(|tile| tile.id == id).unwrap();
    tile.connections = Some(vec![HexDirection::E, HexDirection::E]);
    assert!(Game::try_from_state(rules(), invalid).is_err());

    for ends in [
        vec![HexDirection::Se],
        vec![HexDirection::W, HexDirection::W],
        vec![HexDirection::Nw],
    ] {
        let mut state = decorated.state();
        state
            .tiles
            .iter_mut()
            .find(|tile| tile.id == id)
            .unwrap()
            .dead_ends = Some(ends);
        assert!(Game::try_from_state(rules(), state).is_err());
    }

    for (terrain, exits, valid) in [
        (
            "village",
            vec![HexDirection::E, HexDirection::W, HexDirection::Se],
            true,
        ),
        (
            "village",
            vec![
                HexDirection::E,
                HexDirection::W,
                HexDirection::Se,
                HexDirection::Nw,
            ],
            false,
        ),
        ("keep", vec![HexDirection::W], false),
        ("keep", vec![HexDirection::E, HexDirection::W], false),
    ] {
        let mut state = decorated.state();
        let tile = state.tiles.iter_mut().find(|tile| tile.id == id).unwrap();
        tile.terrain = terrain.into();
        tile.connections = Some(exits.clone());
        tile.dead_ends = None;
        let restored = Game::try_from_state(rules(), state);
        assert_eq!(restored.is_ok(), valid, "{terrain} {exits:?}");
        if let Ok(game) = restored {
            assert_eq!(
                game.fork()
                    .state()
                    .tiles
                    .iter()
                    .find(|tile| tile.id == id)
                    .unwrap()
                    .connections,
                Some(exits)
            );
        }
    }
}

#[test]
fn developer_showcase_has_each_unit_for_both_sides_and_every_terrain() {
    let game = Game::new(rules(), "developer");
    for side in [Side::Player, Side::Enemy] {
        let mut kinds: Vec<_> = game
            .units
            .iter()
            .filter(|u| u.side == side)
            .map(|u| u.kind)
            .collect();
        kinds.sort_unstable();
        assert_eq!(kinds, (0..rules().units.len()).collect::<Vec<_>>());
    }
    let terrains: HashSet<_> = game.terrain.iter().copied().collect();
    assert_eq!(terrains, (0..rules().terrain_names.len()).collect());
}
