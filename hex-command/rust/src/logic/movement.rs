//! Stable movement discovery and cached reverse route distance fields.

use super::battle::Game;
use crate::data::*;
use std::rc::Rc;
use std::{cmp::Reverse, collections::BinaryHeap};
impl<'catalog> Game<'catalog> {
    pub(crate) fn cost(&self, kind: usize, tile: usize) -> u16 {
        self.rules.units[kind].costs[self.terrain[tile]].unwrap_or(u16::MAX)
    }
    pub(crate) fn reachable_with_occupancy(
        &self,
        unit: &Unit,
        occupancy: &[Option<Side>],
    ) -> Vec<(usize, u16)> {
        if unit.moved || unit.attacked || self.winner.is_some() {
            return vec![];
        }
        self.reach_filtered(unit.kind, unit.tile, |tile| occupancy[tile].is_some())
    }
    pub(crate) fn reachable(&self, unit: &Unit) -> Vec<(usize, u16)> {
        if unit.moved || unit.attacked || self.winner.is_some() {
            return vec![];
        }
        self.reach_filtered(unit.kind, unit.tile, |tile| self.occupancy[tile].is_some())
    }
    fn reach_filtered(
        &self,
        kind: usize,
        start: usize,
        occupied: impl Fn(usize) -> bool,
    ) -> Vec<(usize, u16)> {
        let limit = self.rules.units[kind].r#move;
        let mut costs = vec![u16::MAX; self.terrain.len()];
        let mut order = Vec::new();
        let mut buckets = vec![Vec::new(); limit + 1];
        costs[start] = 0;
        buckets[0].push(start);
        for spent in 0..=limit {
            for cursor in 0..buckets[spent].len() {
                let current = buckets[spent][cursor];
                if costs[current] as usize != spent {
                    continue;
                }
                for &next in &self.board.adjacent[current] {
                    if occupied(next) {
                        continue;
                    }
                    let entry = self.cost(kind, next);
                    if entry == u16::MAX {
                        continue;
                    }
                    let cost = spent as u16 + entry;
                    if cost as usize > limit || cost >= costs[next] {
                        continue;
                    }
                    if costs[next] == u16::MAX {
                        order.push(next);
                    }
                    costs[next] = cost;
                    buckets[cost as usize].push(next);
                }
            }
        }
        order.into_iter().map(|tile| (tile, costs[tile])).collect()
    }
    pub(crate) fn routes(&self, kind: usize, goals: &[usize]) -> Rc<Vec<u16>> {
        let key = (kind, goals.to_vec(), self.terrain_revision);
        if self.route_cache.borrow().contains_key(&key) {
            return self.route_cache.borrow()[&key].clone();
        }
        let mut costs = vec![u16::MAX; self.terrain.len()];
        // Reverse movement costs run from each goal. Tile index breaks ties
        // deterministically; only the resulting distance field is observable.
        let mut pending = BinaryHeap::new();
        for &goal in goals {
            if costs[goal] == 0 {
                continue;
            }
            costs[goal] = 0;
            pending.push(Reverse((0, goal)));
        }
        while let Some(Reverse((spent, current))) = pending.pop() {
            if spent != costs[current] {
                continue;
            }
            let entry = self.cost(kind, current);
            if entry == u16::MAX {
                continue;
            }
            for &next in &self.board.adjacent[current] {
                if self.cost(kind, next) == u16::MAX {
                    continue;
                }
                let cost = spent.saturating_add(entry);
                if cost >= costs[next] {
                    continue;
                }
                costs[next] = cost;
                pending.push(Reverse((cost, next)));
            }
        }
        let mut cache = self.route_cache.borrow_mut();
        let mut order = self.route_order.borrow_mut();
        if cache.len() >= 128 {
            if let Some(oldest) = order.pop_front() {
                cache.remove(&oldest);
            }
        }
        order.push_back(key.clone());
        cache.insert(key.clone(), Rc::new(costs));
        drop(cache);
        self.route_cache.borrow()[&key].clone()
    }
    /// Turn-local route fields respect blockers instead of scoring impossible steps.
    /// With yielding enabled, friendly occupants add a cost and enemies remain blocked.
    pub(crate) fn occupied_routes(&self, unit: &Unit, goals: &[usize], yielding: bool) -> Vec<u16> {
        let entry = |tile: usize| {
            let terrain = self.cost(unit.kind, tile);
            if terrain == u16::MAX {
                return terrain;
            }
            match self.at(tile) {
                Some(other) if other.id != unit.id => {
                    if yielding && other.side == unit.side {
                        terrain.saturating_add(2)
                    } else {
                        u16::MAX
                    }
                }
                _ => terrain,
            }
        };
        let mut costs = vec![u16::MAX; self.terrain.len()];
        let mut pending = BinaryHeap::new();
        for &goal in goals {
            if entry(goal) != u16::MAX {
                costs[goal] = 0;
                pending.push(Reverse((0, goal)));
            }
        }
        while let Some(Reverse((spent, current))) = pending.pop() {
            if spent != costs[current] {
                continue;
            }
            for &next in &self.board.adjacent[current] {
                if entry(next) == u16::MAX {
                    continue;
                }
                let cost = spent.saturating_add(entry(current));
                if cost < costs[next] {
                    costs[next] = cost;
                    pending.push(Reverse((cost, next)));
                }
            }
        }
        costs
    }
    pub(crate) fn move_unit(&mut self, id: u32, tile: usize) {
        let index = self.index(id).unwrap();
        let actor = self.units[index].clone();
        assert!(
            actor.side == self.turn
                && self.winner.is_none()
                && actor.tile != tile
                && self.reachable(&actor).iter().any(|&(site, _)| site == tile)
        );
        self.occupancy[actor.tile] = None;
        self.occupancy[tile] = Some(index);
        self.units[index].tile = tile;
        self.units[index].moved = true;
        if [
            Terrain::Village,
            Terrain::Grove,
            Terrain::Fort,
            Terrain::Ruins,
        ]
        .iter()
        .any(|name| self.terrain_is(tile, *name))
            && self.owners[tile] != Some(actor.side)
        {
            self.owners[tile] = Some(actor.side);
            let unit_name = self.rules.units[actor.kind].name.to_lowercase();
            let capture = if self.terrain_is(tile, Terrain::Village) {
                "captured a village".to_owned()
            } else {
                format!(
                    "claimed {}",
                    self.rules.terrain_labels[self.terrain[tile]].to_lowercase()
                )
            };
            self.record(format!(
                "{} {unit_name} {capture}.",
                if actor.side == Side::Player {
                    "Your"
                } else {
                    "Enemy"
                }
            ));
        }
        self.evaluate_campaign(
            false,
            if actor.side == Side::Player {
                Some(tile)
            } else {
                None
            },
        );
        self.action(ActionRecord::Move {
            unit: format!("u{id}"),
            tile: self.board.ids[tile].clone(),
        });
    }
}
