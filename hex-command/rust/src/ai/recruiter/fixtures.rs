//! Test fixtures use the public checkpoint boundary, never mutable engine storage.
use crate::{data::*, logic::Game};

pub(in crate::ai) fn edit(game: &mut Game<'_>, change: impl FnOnce(&mut State)) {
    let pools = game.pools().cloned();
    let first = game.arena_first();
    let mut state = game.state();
    let terrain: Vec<_> = state
        .tiles
        .iter()
        .map(|tile| tile.terrain.clone())
        .collect();
    change(&mut state);
    for (tile, old) in state.tiles.iter_mut().zip(terrain) {
        if tile.terrain != old {
            tile.connections = None;
            tile.dead_ends = None;
            tile.walls = None;
        }
    }
    *game = Game::try_from_state(game.catalogue(), state).unwrap();
    game.configure_arena(pools, first);
}

pub(in crate::ai) fn units(game: &mut Game<'_>, change: impl FnOnce(&mut Vec<Unit>)) {
    let mut units = game.units().to_vec();
    change(&mut units);
    let rules = game.catalogue();
    edit(game, |state| {
        state.next_id = state
            .next_id
            .max(units.iter().map(|u| u.id + 1).max().unwrap_or(0));
        state.units = units
            .into_iter()
            .map(|u| PublicUnit {
                faction: u.faction,
                items: u.items,
                ai_group: u.ai_group,
                id: format!("u{}", u.id),
                r#type: rules.units[u.kind].id.clone(),
                side: u.side,
                tile: state.tiles[u.tile].hex(),
                hp: u.hp,
                moved: u.moved,
                attacked: u.attacked,
                experience: u.experience,
                character: u.character,
                role: u.role,
                retreating: u.retreating,
                rebuilding: u.rebuilding,
            })
            .collect();
    });
}

pub(in crate::ai) fn add(game: &mut Game<'_>, kind: usize, side: Side, tile: usize) -> u32 {
    let id = game.state().next_id;
    let hp = game.catalogue().units[kind].hp;
    units(game, |units| {
        units.push(Unit {
            faction: None,
            items: vec![],
            ai_group: None,
            id,
            kind,
            side,
            tile,
            hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            character: None,
            role: None,
            retreating: None,
            rebuilding: None,
        })
    });
    edit(game, |state| state.next_id = id + 1);
    id
}

pub(in crate::ai) fn terrain(game: &mut Game<'_>, tile: usize, kind: usize) {
    let name = game.catalogue().terrain_names[kind].clone();
    edit(game, |state| state.tiles[tile].terrain = name);
}
