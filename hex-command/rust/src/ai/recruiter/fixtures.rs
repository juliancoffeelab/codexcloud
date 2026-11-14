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
    let next_id = units.iter().map(|u| u.id + 1).max().unwrap_or(0);
    let public = units.iter().map(|u| game.public_unit(u)).collect();
    edit(game, |state| {
        state.next_id = state.next_id.max(next_id);
        state.units = public;
    });
}

pub(in crate::ai) fn add(game: &mut Game<'_>, kind: usize, faction_id: &str, tile: usize) -> u32 {
    let id = game.state().next_id;
    let hp = game.catalogue().units[kind].hp;
    let phase = game.faction(faction_id).expect("fixture faction").phase;
    let faction = game.faction_identity(faction_id);
    let vision_group = game.vision_group_identity(faction_id);
    units(game, |units| {
        units.push(Unit {
            field_available: true,
            perception: PerceptionId(0),
            vision_group,
            id,
            kind,
            phase,
            tile,
            hp,
            moved: false,
            attack_spent: false,
            experience: Some(0),
            role: None,
            retreating: None,
            rebuilding: None,
            faction,
            ai_group: None,
            character: None,
            inventory: InventoryId::default(),
        })
    });
    edit(game, |state| state.next_id = id + 1);
    id
}

pub(in crate::ai) fn terrain(game: &mut Game<'_>, tile: usize, kind: usize) {
    let name = game.catalogue().terrain_names[kind].clone();
    edit(game, |state| state.tiles[tile].terrain = name);
}
