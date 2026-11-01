//! Stable movement discovery and cached reverse route distance fields.

use super::battle::Game;
use super::geometry::TileSet;
use crate::data::*;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
type RouteEntries = smallvec::SmallVec<[u16; 256]>;
#[cfg(not(target_arch = "wasm32"))]
type RouteEntries = smallvec::SmallVec<[u16; 128]>;
#[derive(Default)]
struct ReachScratch {
    costs: Vec<u16>,
    order: Vec<usize>,
    buckets: Vec<Vec<usize>>,
}
thread_local! {
    // Each worker reuses its own storage; discovery order is unchanged.
    static REACH_SCRATCH: RefCell<ReachScratch> = RefCell::new(ReachScratch::default());
    static ROUTE_SCRATCH: RefCell<RouteScratch> = RefCell::new(RouteScratch::default());
    static EMPTY_REACH: Rc<Vec<(usize, u16)>> = Rc::new(vec![]);
}

#[derive(Default)]
struct RouteScratch {
    heads: Vec<usize>,
    next: Vec<usize>,
    previous: Vec<usize>,
}
impl RouteScratch {
    fn prepare(&mut self, tiles: usize, maximum: usize) {
        let buckets = tiles.saturating_mul(maximum).min(u16::MAX as usize - 1) + 1;
        if self.heads.len() < buckets {
            self.heads.resize(buckets, usize::MAX);
        }
        if self.next.len() < tiles {
            self.next.resize(tiles, usize::MAX);
            self.previous.resize(tiles, usize::MAX);
        }
    }
    fn insert(&mut self, tile: usize, cost: usize) {
        let head = self.heads[cost];
        self.next[tile] = head;
        self.previous[tile] = usize::MAX;
        if head != usize::MAX {
            self.previous[head] = tile;
        }
        self.heads[cost] = tile;
    }
    fn remove(&mut self, tile: usize, cost: usize) {
        let previous = self.previous[tile];
        let next = self.next[tile];
        if previous == usize::MAX {
            self.heads[cost] = next;
        } else {
            self.next[previous] = next;
        }
        if next != usize::MAX {
            self.previous[next] = previous;
        }
    }
}
// Distances do not depend on discovery order. One queued entry per tile allows
// constant-time decrease-key and supports every board size without a heap.
fn distances(
    costs: &mut Vec<u16>,
    goals: &[usize],
    entry: &[u16],
    adjacent: &[Vec<usize>],
    blocked_goals: bool,
) {
    costs.clear();
    costs.resize(entry.len(), u16::MAX);
    let maximum = entry
        .iter()
        .copied()
        .filter(|&v| v != u16::MAX)
        .max()
        .unwrap_or(0) as usize;
    ROUTE_SCRATCH.with_borrow_mut(|scratch| {
        scratch.prepare(entry.len(), maximum);
        let mut queued = 0;
        for &goal in goals {
            if (entry[goal] == u16::MAX && !blocked_goals) || costs[goal] == 0 {
                continue;
            }
            costs[goal] = 0;
            scratch.insert(goal, 0);
            queued += 1;
        }
        let mut spent = 0;
        while queued > 0 {
            let current = scratch.heads[spent];
            if current == usize::MAX {
                spent += 1;
                continue;
            }
            scratch.remove(current, spent);
            queued -= 1;
            if entry[current] == u16::MAX {
                continue;
            }
            let cost = (spent as u16).saturating_add(entry[current]);
            for &next in &adjacent[current] {
                let old = costs[next];
                if entry[next] == u16::MAX || cost >= old {
                    continue;
                }
                if old != u16::MAX {
                    scratch.remove(next, old as usize);
                } else {
                    queued += 1;
                }
                costs[next] = cost;
                scratch.insert(next, cost as usize);
            }
        }
    });
}

pub(super) fn prepare(tiles: usize, movement: usize, maximum_cost: usize) {
    EMPTY_REACH.with(|_| {});
    ROUTE_SCRATCH.with_borrow_mut(|scratch| scratch.prepare(tiles, maximum_cost.saturating_add(2)));
    REACH_SCRATCH.with_borrow_mut(|scratch| {
        scratch.costs.resize(tiles, u16::MAX);
        scratch.order.reserve(tiles);
        scratch.buckets.resize_with(movement + 1, Vec::new);
        for bucket in &mut scratch.buckets {
            // Strict improvements enqueue a tile at most once at each cost.
            bucket.reserve(tiles);
        }
    });
}
#[derive(Debug, Default)]
pub(super) struct RouteAtlas {
    slots: Vec<usize>,
    fields: Vec<Option<Rc<Vec<u16>>>>,
    components: Vec<Vec<u16>>,
    bounded: bool,
    entries: Vec<Rc<[u16]>>,
}
impl RouteAtlas {
    pub(super) fn new(rules: &Rules, terrain: &[usize], board: &super::battle::Board) -> Self {
        let tiles = terrain.len();
        let mut unique = Vec::new();
        let slots = rules
            .route_classes
            .iter()
            .map(|&class| {
                if let Some(slot) = unique.iter().position(|&c| c == class) {
                    slot
                } else {
                    unique.push(class);
                    unique.len() - 1
                }
            })
            .collect();
        let mut components = Vec::with_capacity(unique.len());
        let mut pending = Vec::with_capacity(tiles);
        let mut bounded = true;
        for &kind in &unique {
            let mut labels = vec![u16::MAX; tiles];
            let costs = &rules.units[kind].costs;
            let maximum = terrain.iter().filter_map(|&t| costs[t]).max().unwrap_or(0) as usize;
            bounded &= tiles.saturating_mul(maximum) < u16::MAX as usize;
            let mut label = 0;
            for start in 0..tiles {
                if labels[start] != u16::MAX || costs[terrain[start]].is_none() {
                    continue;
                }
                labels[start] = label;
                pending.clear();
                pending.push(start);
                while let Some(tile) = pending.pop() {
                    for &next in &board.adjacent[tile] {
                        if labels[next] == u16::MAX && costs[terrain[next]].is_some() {
                            labels[next] = label;
                            pending.push(next);
                        }
                    }
                }
                label += 1;
            }
            components.push(labels);
        }
        let entries = unique
            .iter()
            .map(|&kind| {
                terrain
                    .iter()
                    .map(|&tile| rules.units[kind].costs[tile].unwrap_or(u16::MAX))
                    .collect()
            })
            .collect();
        Self {
            entries,
            slots,
            fields: if tiles <= 128 {
                vec![None; unique.len() * tiles]
            } else {
                Vec::new()
            },
            components,
            bounded,
        }
    }
}
pub(super) type OccupiedRouteCache =
    Rc<RefCell<FxHashMap<(usize, Option<u16>, u128, u128, u128), Rc<Vec<u16>>>>>;
pub(super) type ReachCache = Rc<RefCell<FxHashMap<(usize, usize, TileSet), Rc<Vec<(usize, u16)>>>>>;
pub(super) type ThreatReachCache = Rc<RefCell<FxHashMap<(usize, usize, u128), u128>>>;
pub(super) type MovementMasks = Rc<RefCell<FxHashMap<usize, Rc<Vec<u128>>>>>;
impl<'catalog> Game<'catalog> {
    #[cfg(target_arch = "wasm32")]
    pub(super) fn prepare_masks(&self) {
        if self.terrain.len() > 256 {
            return;
        }
        let mut geometry: smallvec::SmallVec<[_; 8]> = smallvec::SmallVec::new();
        for unit in self.rules.units.iter() {
            for attack in unit.attacks.iter() {
                let key = (attack.category, attack.range);
                if !geometry.contains(&key) {
                    geometry.push(key);
                }
            }
        }
        self.attack_geometry.borrow_mut().reserve(geometry.len());
        self.attack_masks.borrow_mut().reserve(geometry.len());
        for (category, range) in geometry {
            if self.terrain.len() <= 128 {
                self.attack_masks(category, range);
            } else {
                self.attack_origins(category, range);
            }
        }
        if self.terrain.len() > 128 {
            return;
        }
        self.movement_masks
            .borrow_mut()
            .reserve(self.rules.units.len());
        for kind in 0..self.rules.units.len() {
            if self.rules.movement_classes[kind] != kind {
                continue;
            }
            let limit = self.rules.units[kind].r#move;
            if limit >= 64 {
                continue;
            }
            let mut masks = vec![0u128; limit + 1];
            for tile in 0..self.terrain.len() {
                let cost = self.cost(kind, tile) as usize;
                if cost <= limit {
                    masks[cost] |= 1u128 << tile;
                }
            }
            self.movement_masks
                .borrow_mut()
                .insert(kind, Rc::new(masks));
        }
    }
    /// Threat forecasts need membership, not movement discovery order. Advance
    /// all origins at each movement cost together on maps fitting one mask.
    pub(crate) fn threat_reach(&self, unit: &Unit, occupancy: Option<&[Option<Side>]>) -> TileSet {
        let occupied = if let Some(occupancy) = occupancy {
            let mut occupied = TileSet::new(self.terrain.len());
            for (tile, side) in occupancy.iter().enumerate() {
                if side.is_some() {
                    occupied.insert(tile);
                }
            }
            occupied
        } else {
            self.occupied_tiles.clone()
        };
        self.threat_reach_blocked(unit, occupied)
    }
    pub(crate) fn threat_reach_blocked(&self, unit: &Unit, occupied: TileSet) -> TileSet {
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
        let mut relevant = TileSet::Small(blocked);
        let row = &self.board.distances
            [unit.tile * self.terrain.len()..(unit.tile + 1) * self.terrain.len()];
        relevant.restrict_radius(row, limit);
        let TileSet::Small(blocked) = relevant else {
            unreachable!()
        };
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
        if cache.len()
            >= if cfg!(target_arch = "wasm32") {
                512
            } else {
                4096
            }
        {
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
        self.reach_filtered(unit.kind, unit.tile, self.occupied_tiles.clone())
    }
    fn reach_filtered(
        &self,
        kind: usize,
        start: usize,
        mut occupied: TileSet,
    ) -> Rc<Vec<(usize, u16)>> {
        let movement = self.rules.units[kind].r#move;
        if !self.rules.units[kind]
            .costs
            .iter()
            .any(|cost| *cost == Some(0))
        {
            let row =
                &self.board.distances[start * self.terrain.len()..(start + 1) * self.terrain.len()];
            occupied.restrict_radius(row, movement);
        }
        let key = (self.rules.movement_classes[kind], start, occupied);
        if let Some(reachable) = self.reach_cache.borrow().get(&key) {
            return reachable.clone();
        }
        let atlas = self.route_atlas.borrow();
        let entries = &atlas.entries[atlas.slots[kind]];
        let reachable = self.reach_buffers.borrow_mut().fill(|buffer| {
            REACH_SCRATCH.with_borrow_mut(|scratch| {
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
                            let entry = entries[next];
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
                buffer.extend(
                    scratch
                        .order
                        .iter()
                        .map(|&tile| (tile, scratch.costs[tile])),
                );
            })
        });
        let mut cache = self.reach_cache.borrow_mut();
        if cache.len()
            >= if cfg!(target_arch = "wasm32") {
                256
            } else {
                1024
            }
        {
            cache.clear();
        }
        cache.insert(key, reachable.clone());
        reachable
    }
    /// Static reachability is a component lookup; no distance field is needed.
    pub(crate) fn route_connected(&self, kind: usize, from: usize, to: usize) -> bool {
        let atlas = self.route_atlas.borrow();
        if !atlas.bounded {
            drop(atlas);
            return self.routes(kind, &[to])[from] != u16::MAX;
        }
        let labels = &atlas.components[atlas.slots[kind]];
        from == to || (labels[from] != u16::MAX && labels[from] == labels[to])
    }
    pub(crate) fn route_region(&self, kind: usize, tile: usize) -> usize {
        let atlas = self.route_atlas.borrow();
        let label = atlas.components[atlas.slots[kind]][tile];
        if atlas.bounded && label != u16::MAX {
            usize::from(label)
        } else {
            self.terrain.len() + tile
        }
    }
    fn single_route(&self, kind: usize, goal: usize) -> Rc<Vec<u16>> {
        let slot = self.route_atlas.borrow().slots[kind] * self.terrain.len() + goal;
        if let Some(field) = &self.route_atlas.borrow().fields[slot] {
            return field.clone();
        }
        let atlas = self.route_atlas.borrow();
        let entries = &atlas.entries[atlas.slots[kind]];
        let field = self
            .route_buffers
            .borrow_mut()
            .fill(|costs| distances(costs, &[goal], &entries, &self.board.adjacent, true));
        drop(atlas);
        self.route_atlas.borrow_mut().fields[slot] = Some(field.clone());
        field
    }
    pub(crate) fn routes(&self, kind: usize, goals: &[usize]) -> Rc<Vec<u16>> {
        let kind = self.rules.route_classes[kind];
        let compact = self.terrain.len() <= 128;
        if compact && goals.len() == 1 {
            return self.single_route(kind, goals[0]);
        }
        let mut sites = TileSet::new(self.terrain.len());
        for &goal in goals {
            sites.insert(goal);
        }
        let key = (kind, sites, self.terrain_revision);
        if let Some(route) = self.route_cache.borrow().get(&key) {
            return route.clone();
        }
        let costs = if compact {
            let fields: smallvec::SmallVec<[_; 32]> = goals
                .iter()
                .map(|&goal| self.single_route(kind, goal))
                .collect();
            self.route_buffers.borrow_mut().fill(|costs| {
                costs.resize(self.terrain.len(), u16::MAX);
                for field in &fields {
                    for (cost, &candidate) in costs.iter_mut().zip(field.iter()) {
                        *cost = (*cost).min(candidate);
                    }
                }
            })
        } else {
            let atlas = self.route_atlas.borrow();
            let entries = &atlas.entries[atlas.slots[kind]];
            self.route_buffers
                .borrow_mut()
                .fill(|costs| distances(costs, goals, &entries, &self.board.adjacent, true))
        };
        let mut cache = self.route_cache.borrow_mut();
        let mut order = self.route_order.borrow_mut();
        if cache.len() >= 128 {
            if let Some(oldest) = order.pop_front() {
                cache.remove(&oldest);
            }
        }
        order.push_back(key.clone());
        cache.insert(key.clone(), costs);
        drop(cache);
        self.route_cache.borrow()[&key].clone()
    }
    /// Turn-local route fields respect blockers instead of scoring impossible steps.
    /// With yielding enabled, friendly occupants add a cost and enemies remain blocked.
    pub(crate) fn occupied_routes(
        &self,
        unit: &Unit,
        goals: &[usize],
        yielding: bool,
    ) -> Rc<Vec<u16>> {
        self.occupied_routes_with_penalty(unit, goals, yielding.then_some(2))
    }
    /// Campaign corridor requests can ask allies to yield without preferring
    /// a longer detour that postpones an otherwise reachable actor arrival.
    pub(crate) fn occupied_routes_with_penalty(
        &self,
        unit: &Unit,
        goals: &[usize],
        friendly_penalty: Option<u16>,
    ) -> Rc<Vec<u16>> {
        let compact = self.terrain.len() <= 128;
        let mut friends = 0u128;
        let mut blocked = 0u128;
        let mut sites = 0u128;
        if compact {
            for other in &self.units {
                if other.id == unit.id {
                    continue;
                }
                if other.side == unit.side {
                    friends |= 1u128 << other.tile;
                } else {
                    blocked |= 1u128 << other.tile;
                }
            }
            for &goal in goals {
                sites |= 1u128 << goal;
            }
        }
        let key = (
            self.rules.route_classes[unit.kind],
            friendly_penalty,
            sites,
            friends,
            blocked,
        );
        if compact {
            if let Some(route) = self.occupied_route_cache.borrow().get(&key) {
                return route.clone();
            }
        }
        let entries: RouteEntries = (0..self.terrain.len())
            .map(|tile| {
                let terrain = self.cost(unit.kind, tile);
                if terrain == u16::MAX {
                    return terrain;
                }
                let side = if compact {
                    let bit = 1u128 << tile;
                    if blocked & bit != 0 {
                        Some(false)
                    } else if friends & bit != 0 {
                        Some(true)
                    } else {
                        None
                    }
                } else {
                    self.at(tile)
                        .filter(|other| other.id != unit.id)
                        .map(|other| other.side == unit.side)
                };
                match side {
                    Some(true) => {
                        friendly_penalty.map_or(u16::MAX, |penalty| terrain.saturating_add(penalty))
                    }
                    Some(false) => u16::MAX,
                    None => terrain,
                }
            })
            .collect();
        let result = self
            .route_buffers
            .borrow_mut()
            .fill(|costs| distances(costs, goals, &entries, &self.board.adjacent, false));
        if compact {
            let mut cache = self.occupied_route_cache.borrow_mut();
            if cache.len() >= 256 {
                cache.clear();
            }
            cache.insert(key, result.clone());
        }
        result
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
        self.occupied_tiles.remove(actor.tile);
        self.occupied_tiles.insert(tile);
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
            if self.presentation {
                if self.terrain_is(tile, Terrain::Village) {
                    game_log!(
                        self,
                        format!(
                            "{} {} captured a village.",
                            if actor.side == Side::Player {
                                "Your"
                            } else {
                                "Enemy"
                            },
                            self.rules.unit_log_names[actor.kind]
                        )
                    );
                } else {
                    game_log!(
                        self,
                        format!(
                            "{} {} claimed {}.",
                            if actor.side == Side::Player {
                                "Your"
                            } else {
                                "Enemy"
                            },
                            self.rules.unit_log_names[actor.kind],
                            self.rules.terrain_log_labels[self.terrain[tile]]
                        )
                    );
                }
            }
        }
        self.evaluate_campaign(
            false,
            if actor.side == Side::Player {
                Some(tile)
            } else {
                None
            },
        );
        #[cfg(not(target_arch = "wasm32"))]
        game_action!(
            self,
            ActionRecord::Move {
                unit: format!("u{id}"),
                tile: self.board.ids[tile].clone(),
            }
        );
    }
}

#[cfg(test)]
mod occupancy_tests {
    use super::*;
    #[test]
    fn occupancy_mask_follows_moves_roster_changes_and_independent_forks() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let forecast = game.fork();
        let (actor, destination) = game
            .units
            .iter()
            .filter(|u| u.side == game.turn)
            .find_map(|u| game.reachable(u).first().map(|&(tile, _)| (u.id, tile)))
            .unwrap();
        game.move_unit(actor, destination);
        let free = (0..game.terrain.len())
            .find(|&tile| game.at(tile).is_none())
            .unwrap();
        game.add(game.units[0].kind, game.turn, free);
        game.units.remove(0);
        game.roster_changed();
        for board in [&game, &forecast] {
            for tile in 0..board.terrain.len() {
                assert_eq!(
                    board.occupied_tiles.contains(tile),
                    board.at(tile).is_some()
                );
            }
        }
        assert_ne!(game.occupied_tiles, forecast.occupied_tiles);
    }
}
