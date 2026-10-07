//! Native movement, occupancy, geometry and authored-map invariants.
use super::*;
use std::collections::HashSet;

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
fn scout_and_woodland_routes_spend_the_whole_allowance_and_stop_at_blockers() {
    for (kind, terrain) in [
        ("scout", "hill"),
        ("wolf", "forest"),
        ("boar", "grove"),
        ("wildkeeper", "forest"),
        ("thorncaller", "grove"),
    ] {
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
}

#[test]
fn merfolk_land_water_and_bridge_routes_have_different_reach() {
    for (kind, terrain, steps) in [
        ("reefguard", "plains", 2),
        ("reefguard", "water", 4),
        ("reefguard", "bridge", 4),
        ("harpooner", "plains", 2),
        ("harpooner", "water", 5),
    ] {
        let mut game = flat();
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, game.rules.terrains["forest"]);
        }
        for col in 0..=5 {
            let tile = game.site(&format!("{col},0"));
            game.set_terrain(tile, game.rules.terrains[terrain]);
        }
        let id = add(&mut game, kind, Side::Player, "0,0");
        let goal = game.site(&format!("{steps},0"));
        let reachable = game.reachable(game.unit(id).unwrap());
        assert!(reachable.contains(&(goal, steps * if terrain == "plains" { 2 } else { 1 })));
        if steps < 5 {
            assert!(
                !reachable
                    .iter()
                    .any(|&(tile, _)| tile == game.site(&format!("{},0", steps + 1)))
            );
        }
        game.move_unit(id, goal);
        assert_eq!(game.unit(id).unwrap().tile, goal);
    }
}

#[test]
fn ford_exhausts_heavy_infantry_while_bridge_and_deep_water_differ() {
    let mut game = Game::new(rules(), "river");
    game.units[1].tile = game.site("4,7");
    game.roster_changed();
    let id = game.units[1].id;
    let ford = game.site("5,7");
    let routes = game.reachable(game.unit(id).unwrap());
    assert!(routes.contains(&(ford, 3)));
    assert!(!routes.iter().any(|&(tile, _)| tile == game.site("6,7")));
    game.move_unit(id, ford);
    game.units[1].tile = game.site("6,3");
    game.units[1].moved = false;
    game.roster_changed();
    let routes = game.reachable(game.unit(id).unwrap());
    assert!(routes.iter().any(|&(tile, _)| tile == game.site("7,3")));
    assert!(!routes.iter().any(|&(tile, _)| tile == game.site("8,2")));
}

#[test]
fn adjacency_matches_hex_distance_including_edges() {
    for (map, revision) in [("mirror", 1), ("river", 2)] {
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

fn connected(game: &Game<'_>, allowed: &HashSet<usize>, start: usize) -> HashSet<usize> {
    let mut reached = HashSet::from([start]);
    let mut pending = vec![start];
    while let Some(tile) = pending.pop() {
        for &next in &game.board.adjacent[tile] {
            if allowed.contains(&next) && reached.insert(next) {
                pending.push(next);
            }
        }
    }
    reached
}

#[test]
fn skirmish_land_is_connected_and_river_crossings_each_join_both_banks() {
    for map in ["mirror", "river"] {
        let game = Game::new(rules(), map);
        let land: HashSet<_> = (0..game.terrain.len())
            .filter(|&t| !game.terrain_is(t, Terrain::Water))
            .collect();
        let keeps: Vec<_> = land
            .iter()
            .copied()
            .filter(|&t| game.terrain_is(t, Terrain::Keep))
            .collect();
        assert_eq!(connected(&game, &land, keeps[0]), land);
        assert!(game.units.iter().all(|u| land.contains(&u.tile)));
        if map == "river" {
            let crossings: Vec<_> = land
                .iter()
                .copied()
                .filter(|&t| {
                    game.terrain_is(t, Terrain::Bridge) || game.terrain_is(t, Terrain::Ford)
                })
                .collect();
            let banks: HashSet<_> = land
                .difference(&crossings.iter().copied().collect())
                .copied()
                .collect();
            let a = connected(&game, &banks, keeps[0]);
            let b = connected(&game, &banks, keeps[1]);
            assert!(a.is_disjoint(&b));
            assert_eq!(a.len() + b.len(), banks.len());
            for crossing in crossings {
                let mut joined = banks.clone();
                joined.insert(crossing);
                assert_eq!(connected(&game, &joined, keeps[0]), joined);
            }
        }
    }
}

#[test]
fn mirrored_village_approaches_and_rough_ground_respect_row_parity() {
    let game = Game::new(rules(), "mirror");
    for tile in 0..game.terrain.len() {
        if !matches!(
            game.rules.terrain_types[game.terrain[tile]],
            Terrain::Forest | Terrain::Hill | Terrain::Village
        ) {
            continue;
        }
        let (col, row) = game.board.positions[tile];
        let mirrored = game.site(&format!(
            "{},{}",
            (if row % 2 == 1 { 10 } else { 11 }) - col,
            row
        ));
        assert_eq!(game.terrain[tile], game.terrain[mirrored]);
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

#[test]
fn opening_river_village_routes_do_not_require_forest_detours() {
    let game = Game::new(rules(), "river");
    for (id, destination) in [(2, "4,1"), (3, "4,1"), (3, "2,7")] {
        assert!(
            game.reachable(game.unit(id).unwrap())
                .iter()
                .any(|&(tile, _)| tile == game.site(destination))
        );
    }
}
