//! Stable movement discovery and cached reverse route distance fields.

use super::battle::Game;
use super::geometry::TileSet;
use crate::data::*;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::{cell::RefCell, cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
struct ReachScratch {
    costs: Vec<u16>,
    order: Vec<usize>,
    buckets: Vec<Vec<usize>>,
}
thread_local! {
    // Each worker reuses its own storage; discovery order is unchanged.
    static REACH_SCRATCH: RefCell<ReachScratch> = RefCell::new(ReachScratch::default());
    static EMPTY_REACH: Rc<Vec<(usize, u16)>> = Rc::new(vec![]);
}
pub(super) type ReachCache = Rc<RefCell<FxHashMap<(usize, usize, TileSet), Rc<Vec<(usize, u16)>>>>>;
pub(super) type ThreatReachCache = Rc<RefCell<FxHashMap<(usize, usize, u128), u128>>>;
pub(super) type MovementMasks = Rc<RefCell<FxHashMap<usize, Rc<Vec<u128>>>>>;
impl<'catalog> Game<'catalog> {
    /// Threat forecasts need membership, not movement discovery order. Advance
    /// all origins at each movement cost together on maps fitting one mask.
    pub(crate) fn threat_reach(&self, unit: &Unit, occupancy: Option<&[Option<Side>]>) -> TileSet {
        let mut occupied = TileSet::new(self.terrain.len());
        if let Some(occupancy) = occupancy {
            for (tile, side) in occupancy.iter().enumerate() {
                if side.is_some() {
                    occupied.insert(tile);
                }
            }
        } else {
            for unit in &self.units {
                occupied.insert(unit.tile);
            }
        }
        let mut positions = TileSet::new(self.terrain.len());
        positions.insert(unit.tile);
        if unit.moved || unit.attacked || self.winner.is_some() {
            return positions;
        }
        let limit = self.rules.units[unit.kind].r#move;
        let TileSet::Small(blocked) = occupied else {
            for &(tile, _) in self.reach_filtered(unit.kind, unit.tile, occupied).iter() {
                positions.insert(tile);
            }
            return positions;
        };
        if limit >= 64
            || self.rules.units[unit.kind]
                .costs
                .iter()
                .any(|cost| *cost == Some(0))
        {
            for &(tile, _) in self
                .reach_filtered(unit.kind, unit.tile, TileSet::Small(blocked))
                .iter()
            {
                positions.insert(tile);
            }
            return positions;
        }
        let movement_class = self.rules.movement_classes[unit.kind];
        let key = (movement_class, unit.tile, blocked);
        if let Some(&bits) = self.threat_reach_cache.borrow().get(&key) {
            return TileSet::Small(bits);
        }
        let cached = self.movement_masks.borrow().get(&movement_class).cloned();
        let costs = cached.unwrap_or_else(|| {
            let mut masks = vec![0u128; limit + 1];
            for tile in 0..self.terrain.len() {
                let cost = self.cost(unit.kind, tile) as usize;
                if cost <= limit {
                    masks[cost] |= 1u128 << tile;
                }
            }
            let masks = Rc::new(masks);
            self.movement_masks
                .borrow_mut()
                .insert(movement_class, masks.clone());
            masks
        });
        let mut pending = [0u128; 64];
        pending[0] = 1u128 << unit.tile;
        let mut reached = 0;
        for spent in 0..=limit {
            let mut frontier = pending[spent] & !reached;
            reached |= frontier;
            let mut neighbors = 0;
            while frontier != 0 {
                let tile = frontier.trailing_zeros() as usize;
                neighbors |= self.board.adjacent_bits[tile];
                frontier &= frontier - 1;
            }
            neighbors &= !blocked & !reached;
            for cost in 1..=limit - spent {
                pending[spent + cost] |= neighbors & costs[cost];
            }
        }
        let mut cache = self.threat_reach_cache.borrow_mut();
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(key, reached);
        TileSet::Small(reached)
    }
    pub(crate) fn cost(&self, kind: usize, tile: usize) -> u16 {
        self.rules.units[kind].costs[self.terrain[tile]].unwrap_or(u16::MAX)
    }
    #[cfg(test)]
    pub(crate) fn reachable_with_occupancy(
        &self,
        unit: &Unit,
        occupancy: &[Option<Side>],
    ) -> Rc<Vec<(usize, u16)>> {
        if unit.moved || unit.attacked || self.winner.is_some() {
            return EMPTY_REACH.with(Clone::clone);
        }
        let mut occupied = TileSet::new(self.terrain.len());
        for (tile, side) in occupancy.iter().enumerate() {
            if side.is_some() {
                occupied.insert(tile);
            }
        }
        self.reach_filtered(unit.kind, unit.tile, occupied)
    }
    pub(crate) fn reachable(&self, unit: &Unit) -> Rc<Vec<(usize, u16)>> {
        if unit.moved || unit.attacked || self.winner.is_some() {
            return EMPTY_REACH.with(Clone::clone);
        }
        let mut occupied = TileSet::new(self.terrain.len());
        for unit in &self.units {
            occupied.insert(unit.tile);
        }
        self.reach_filtered(unit.kind, unit.tile, occupied)
    }
    fn reach_filtered(
        &self,
        kind: usize,
        start: usize,
        occupied: TileSet,
    ) -> Rc<Vec<(usize, u16)>> {
        let key = (self.rules.movement_classes[kind], start, occupied);
        if let Some(reachable) = self.reach_cache.borrow().get(&key) {
            return reachable.clone();
        }
        let reachable = Rc::new(REACH_SCRATCH.with_borrow_mut(|scratch| {
            let limit = self.rules.units[kind].r#move;
            scratch.costs.resize(self.terrain.len(), u16::MAX);
            scratch.costs.fill(u16::MAX);
            scratch.order.clear();
            if scratch.buckets.len() <= limit {
                scratch.buckets.resize_with(limit + 1, Vec::new);
            }
            for bucket in &mut scratch.buckets {
                bucket.clear();
            }
            scratch.costs[start] = 0;
            scratch.buckets[0].push(start);
            for spent in 0..=limit {
                for cursor in 0..scratch.buckets[spent].len() {
                    let current = scratch.buckets[spent][cursor];
                    if scratch.costs[current] as usize != spent {
                        continue;
                    }
                    for &next in &self.board.adjacent[current] {
                        if key.2.contains(next) {
                            continue;
                        }
                        let entry = self.cost(kind, next);
                        if entry == u16::MAX {
                            continue;
                        }
                        let cost = spent as u16 + entry;
                        if cost as usize > limit || cost >= scratch.costs[next] {
                            continue;
                        }
                        if scratch.costs[next] == u16::MAX {
                            scratch.order.push(next);
                        }
                        scratch.costs[next] = cost;
                        scratch.buckets[cost as usize].push(next);
                    }
                }
            }
            scratch
                .order
                .iter()
                .map(|&tile| (tile, scratch.costs[tile]))
                .collect::<Vec<_>>()
        }));
        let mut cache = self.reach_cache.borrow_mut();
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(key, reachable.clone());
        reachable
    }
    pub(crate) fn routes(&self, kind: usize, goals: &[usize]) -> Rc<Vec<u16>> {
        let kind = self.rules.route_classes[kind];
        let key = (kind, goals.to_vec(), self.terrain_revision);
        if let Some(route) = self.route_cache.borrow().get(&key) {
            return route.clone();
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
