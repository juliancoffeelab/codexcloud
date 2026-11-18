//! Stable movement discovery and cached reverse route distance fields.

use super::battle::BattlePosition;
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) enum RouteGoals {
    Tile(usize),
    Tiles(Vec<usize>),
}
pub(super) type RouteKey = (usize, RouteGoals, u64, bool);
pub(super) type ReachCache = Rc<RefCell<FxHashMap<(usize, usize, TileSet), Rc<Vec<(usize, u16)>>>>>;
pub(super) type ThreatReachCache = Rc<RefCell<FxHashMap<(usize, usize, u128), u128>>>;
pub(super) type MovementMasks = Rc<RefCell<FxHashMap<usize, Rc<Vec<u128>>>>>;
// A None entry retains weighted-distance semantics when u16 saturation could
// distinguish a connected tile from a finite route.
pub(super) type RegionCache = Rc<RefCell<FxHashMap<usize, Option<Rc<Vec<u16>>>>>>;

/// Borrowed final movement costs for a single immutable battle query batch.
/// Spatial movement rules retain the general per-tile calculation.
pub struct MovementField<'a, 'catalog> {
    game: &'a BattlePosition<'catalog>,
    unit: Unit,
    costs: Option<Rc<Vec<u16>>>,
}
impl<'a, 'catalog> MovementField<'a, 'catalog> {
    pub(crate) fn new(game: &'a BattlePosition<'catalog>, unit: &Unit) -> Self {
        Self {
            game,
            unit: *unit,
            costs: game.movement_costs(unit),
        }
    }
    #[inline]
    pub fn cost_at(&self, tile: usize) -> u16 {
        match &self.costs {
            Some(costs) => costs[tile],
            None => self.game.step_cost(&self.unit, tile),
        }
    }
    pub fn costs(&self) -> impl ExactSizeIterator<Item = u16> + '_ {
        (0..self.game.terrain.len()).map(|tile| self.cost_at(tile))
    }
}

/// Terrain reachability without occupancy or a movement budget. Small distance
/// bounds use connected components; large bounds retain weighted route fields.
#[derive(Clone)]
pub struct Reachability {
    field: Rc<Vec<u16>>,
    component: Option<u16>,
    start: usize,
}
impl Reachability {
    /// Equal keys identify equal reachable sets within one immutable terrain
    /// layout and movement class. Saturating route fields keep distinct sources.
    #[inline]
    pub fn source_key(&self) -> usize {
        match self.component {
            Some(component) if component != u16::MAX => usize::from(component),
            _ => self.field.len() + self.start,
        }
    }
    #[inline]
    pub fn contains(&self, tile: usize) -> bool {
        if let Some(component) = self.component {
            tile == self.start || (component != u16::MAX && self.field[tile] == component)
        } else {
            self.field[tile] != u16::MAX
        }
    }
}

pub(super) struct ValidatedMove {
    id: u32,
    destination: usize,
    path: Option<Vec<usize>>,
}

impl<'catalog> BattlePosition<'catalog> {
    /// Borrow the same final evaluated cost vector for a whole geometric batch.
    pub(crate) fn movement_costs(&self, unit: &Unit) -> Option<Rc<Vec<u16>>> {
        let cache = self.effect_sources.movement_cache.as_ref()?;
        let identity = (
            unit.kind,
            unit.faction,
            self.runtime.inventories.identity(unit.inventory),
        );
        if let Some(costs) = cache.costs.borrow().get(&identity) {
            return Some(Rc::clone(costs));
        }
        // Without spatial movement effects, the same composed context answers
        // every terrain query. Other effect kinds do not contribute to step_cost.
        // Own effects of hypothetical kinds/equipment are still selected normally.
        let costs = self.with_capabilities_at(unit, unit.tile, &[], &[], |caps| {
            // A layout repeats terrain handles. Resolve each distinct handle
            // once rather than recomposing the same step cost for every hex.
            let mut terrain_costs = vec![None; self.catalogue.terrains().count()];
            self.terrain
                .iter()
                .map(|&terrain| {
                    *terrain_costs[terrain].get_or_insert_with(|| {
                        match caps.step_cost(self.definitions.terrain(terrain)) {
                            Some(cost) => cost,
                            None => u16::MAX,
                        }
                    })
                })
                .collect::<Vec<_>>()
        });
        let costs = Rc::new(costs);
        if cache.costs.borrow().len() >= 512 {
            cache.costs.borrow_mut().clear();
        }
        cache.costs.borrow_mut().insert(identity, Rc::clone(&costs));
        Some(costs)
    }
    #[inline]
    pub(crate) fn step_cost(&self, unit: &Unit, tile: usize) -> u16 {
        if let Some(costs) = self.movement_costs(unit) {
            return costs[tile];
        }
        let providers: &[Unit] = if self.effect_sources.movement {
            &self.units
        } else {
            &[]
        };
        self.with_capabilities_at(unit, tile, providers, &[], |caps| {
            match caps.step_cost(self.terrain_definition(tile)) {
                Some(cost) => cost,
                None => u16::MAX,
            }
        })
    }
    /// Bound bucket storage by the final Catalogue fact and u16 path representation.
    pub(crate) fn movement_limit(&self, unit: &Unit) -> usize {
        let key = (
            unit.kind,
            unit.faction,
            self.runtime.inventories.identity(unit.inventory),
        );
        if let Some(cache) = &self.effect_sources.movement_cache {
            if let Some(limit) = cache.limits.borrow().get(&key) {
                return *limit;
            }
        }
        let limit = self.with_capabilities_at(unit, unit.tile, &self.units, &[], |caps| {
            caps.path_cost_limit().min(usize::from(u16::MAX))
        });
        if let Some(cache) = &self.effect_sources.movement_cache {
            if cache.limits.borrow().len() >= 4096 {
                cache.limits.borrow_mut().clear();
            }
            cache.limits.borrow_mut().insert(key, limit);
        }
        limit
    }
    /// Classes index equal evaluated movement behavior, never raw Catalogue rows.
    pub(crate) fn route_class(&self, unit: &Unit) -> usize {
        self.intern_movement_class(unit, 0)
    }
    fn movement_class(&self, unit: &Unit) -> usize {
        self.intern_movement_class(unit, self.movement_limit(unit))
    }
    fn intern_movement_class(&self, unit: &Unit, limit: usize) -> usize {
        let identity = (
            unit.kind,
            unit.faction,
            self.runtime.inventories.identity(unit.inventory),
            limit,
        );
        if let Some(cache) = &self.effect_sources.movement_cache {
            if let Some(class) = cache.classes.borrow().get(&identity) {
                return *class;
            }
        }
        let batch_costs = self.movement_costs(unit);
        let cost_at = |tile: usize| match &batch_costs {
            Some(costs) => costs[tile],
            None => self.step_cost(unit, tile),
        };
        type ClassKey = (Vec<u16>, usize);
        thread_local! {
            static CLASSES: RefCell<(FxHashMap<ClassKey, usize>, usize)> = RefCell::new((FxHashMap::default(), 0));
        }
        let key = (
            (0..self.terrain.len()).map(|tile| cost_at(tile)).collect(),
            limit,
        );
        let class = CLASSES.with_borrow_mut(|(classes, next)| {
            if let Some(id) = classes.get(&key) {
                return *id;
            }
            if classes.len() >= 16384 {
                classes.clear();
            }
            let id = *next;
            *next = next.checked_add(1).expect("movement class identity space");
            classes.insert(key, id);
            id
        });
        if let Some(cache) = &self.effect_sources.movement_cache {
            if cache.classes.borrow().len() >= 4096 {
                cache.classes.borrow_mut().clear();
            }
            cache.classes.borrow_mut().insert(identity, class);
        }
        class
    }
    pub(crate) fn reachability(&self, unit: &Unit, start: usize) -> Reachability {
        if self.effect_sources.movement {
            return Reachability {
                field: self.routes_from(unit, start),
                component: None,
                start,
            };
        }
        let kind = self.route_class(unit);
        let cached = self.runtime.movement_regions.borrow().get(&kind).cloned();
        let regions = cached.unwrap_or_else(|| {
            let batch_costs = self.movement_costs(unit);
            let cost_at = |tile: usize| match &batch_costs {
                Some(costs) => costs[tile],
                None => self.step_cost(unit, tile),
            };

            let maximum = (0..self.terrain.len())
                .map(|tile| cost_at(tile))
                .filter(|&cost| cost != u16::MAX)
                .max()
                .unwrap_or(0);
            let tiles = self.runtime.terrain.len();
            let bounded = tiles < usize::from(u16::MAX)
                && tiles
                    .checked_mul(usize::from(maximum))
                    .is_some_and(|bound| bound < usize::from(u16::MAX));
            let regions = bounded.then(|| {
                let mut regions = vec![u16::MAX; tiles];
                let mut pending = Vec::new();
                let mut component = 0;
                for start in 0..tiles {
                    if regions[start] != u16::MAX || cost_at(start) == u16::MAX {
                        continue;
                    }
                    regions[start] = component;
                    pending.push(start);
                    while let Some(tile) = pending.pop() {
                        for &next in &self.runtime.board.adjacent[tile] {
                            if regions[next] == u16::MAX && cost_at(next) != u16::MAX {
                                regions[next] = component;
                                pending.push(next);
                            }
                        }
                    }
                    component += 1;
                }
                Rc::new(regions)
            });
            self.runtime
                .movement_regions
                .borrow_mut()
                .insert(kind, regions.clone());
            regions
        });
        if let Some(field) = regions {
            Reachability {
                component: Some(field[start]),
                field,
                start,
            }
        } else {
            Reachability {
                field: self.routes_from(unit, start),
                component: None,
                start,
            }
        }
    }

    /// Threat forecasts need membership, not movement discovery order. Advance
    /// all origins at each movement cost together on maps fitting one mask.
    pub(crate) fn threat_reach(&self, unit: &Unit, occupancy: Option<&[Option<u32>]>) -> TileSet {
        let occupied = if let Some(occupancy) = occupancy {
            let mut occupied = TileSet::new(self.runtime.terrain.len());
            for (tile, occupant) in occupancy.iter().enumerate() {
                if occupant.is_some() {
                    occupied.insert(tile);
                }
            }
            occupied
        } else {
            self.runtime.occupied_tiles.clone()
        };
        self.threat_reach_blocked(unit, occupied)
    }
    pub(crate) fn threat_reach_blocked(&self, unit: &Unit, occupied: TileSet) -> TileSet {
        let mut positions = TileSet::new(self.runtime.terrain.len());
        positions.insert(unit.tile);
        if unit.moved || unit.attack_spent || self.runtime.winner.is_some() {
            return positions;
        }
        if self.effect_sources.movement {
            for &(tile, _) in self.effect_reachable(unit, &occupied).iter() {
                positions.insert(tile);
            }
            return positions;
        }
        let batch_costs = self.movement_costs(unit);
        let cost_at = |tile: usize| match &batch_costs {
            Some(costs) => costs[tile],
            None => self.step_cost(unit, tile),
        };
        let limit = self.movement_limit(unit);
        let TileSet::Small(blocked) = occupied else {
            for &(tile, _) in self.reach_filtered(unit, unit.tile, occupied).iter() {
                positions.insert(tile);
            }
            return positions;
        };
        if limit >= 64 || (0..self.terrain.len()).any(|tile| cost_at(tile) == 0) {
            for &(tile, _) in self
                .reach_filtered(unit, unit.tile, TileSet::Small(blocked))
                .iter()
            {
                positions.insert(tile);
            }
            return positions;
        }
        let movement_class = self.movement_class(unit);
        let key = (movement_class, unit.tile, blocked);
        if let Some(&bits) = self.runtime.threat_reach_cache.borrow().get(&key) {
            return TileSet::Small(bits);
        }
        let cached = self
            .runtime
            .movement_masks
            .borrow()
            .get(&movement_class)
            .cloned();
        let costs = cached.unwrap_or_else(|| {
            let mut masks = vec![0u128; limit + 1];
            for tile in 0..self.runtime.terrain.len() {
                let cost = cost_at(tile) as usize;
                if cost <= limit {
                    masks[cost] |= 1u128 << tile;
                }
            }
            let masks = Rc::new(masks);
            self.runtime
                .movement_masks
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
                neighbors |= self.runtime.board.adjacent_bits[tile];
                frontier &= frontier - 1;
            }
            neighbors &= !blocked & !reached;
            for cost in 1..=limit - spent {
                pending[spent + cost] |= neighbors & costs[cost];
            }
        }
        let mut cache = self.runtime.threat_reach_cache.borrow_mut();
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(key, reached);
        TileSet::Small(reached)
    }
    #[inline]
    fn effect_distances(
        &self,
        unit: &Unit,
        goals: &[usize],
        reverse: bool,
        occupied: Option<&TileSet>,
        limit: u16,
    ) -> Vec<u16> {
        let mut costs = vec![u16::MAX; self.terrain.len()];
        let mut queue = BinaryHeap::new();
        for &goal in goals {
            costs[goal] = 0;
            queue.push(Reverse((0u16, goal)));
        }
        while let Some(Reverse((spent, current))) = queue.pop() {
            if costs[current] != spent {
                continue;
            }
            for &next in &self.board.adjacent[current] {
                if occupied.is_some_and(|mask| mask.contains(next)) {
                    continue;
                }
                let entry = self.step_cost(unit, if reverse { current } else { next });
                if let Some(total) = spent
                    .checked_add(entry)
                    .filter(|value| *value <= limit && *value < costs[next])
                {
                    costs[next] = total;
                    queue.push(Reverse((total, next)));
                }
            }
        }
        costs
    }
    fn effect_reachable(&self, unit: &Unit, occupied: &TileSet) -> Rc<Vec<(usize, u16)>> {
        let costs = self.effect_distances(
            unit,
            &[unit.tile],
            false,
            Some(occupied),
            self.movement_limit(unit) as u16,
        );
        Rc::new(
            costs
                .into_iter()
                .enumerate()
                .filter(|(tile, cost)| *tile != unit.tile && *cost != u16::MAX)
                .collect(),
        )
    }
    pub(crate) fn routes(&self, unit: &Unit, goals: &[usize]) -> Rc<Vec<u16>> {
        if self.effect_sources.movement {
            Rc::new(self.effect_distances(unit, goals, true, None, u16::MAX))
        } else {
            self.route_distances(unit, goals, false)
        }
    }
    pub(crate) fn routes_from(&self, unit: &Unit, start: usize) -> Rc<Vec<u16>> {
        if self.effect_sources.movement {
            Rc::new(self.effect_distances(unit, &[start], false, None, u16::MAX))
        } else {
            self.route_distances(unit, &[start], true)
        }
    }
    #[cfg(test)]
    pub(crate) fn reachable_with_occupancy(
        &self,
        unit: &Unit,
        occupancy: &[Option<u32>],
    ) -> Rc<Vec<(usize, u16)>> {
        if unit.moved || unit.attack_spent || self.runtime.winner.is_some() {
            return EMPTY_REACH.with(Clone::clone);
        }
        let mut occupied = TileSet::new(self.runtime.terrain.len());
        for (tile, occupant) in occupancy.iter().enumerate() {
            if occupant.is_some() {
                occupied.insert(tile);
            }
        }
        if self.effect_sources.movement {
            self.effect_reachable(unit, &occupied)
        } else {
            self.reach_filtered(unit, unit.tile, occupied)
        }
    }
    pub(crate) fn reachable(&self, unit: &Unit) -> Rc<Vec<(usize, u16)>> {
        if unit.moved || unit.attack_spent || self.runtime.winner.is_some() {
            return EMPTY_REACH.with(Clone::clone);
        }
        if self.effect_sources.movement {
            self.effect_reachable(unit, &self.runtime.occupied_tiles)
        } else {
            self.reach_filtered(unit, unit.tile, self.runtime.occupied_tiles.clone())
        }
    }
    fn reach_filtered(
        &self,
        unit: &Unit,
        start: usize,
        occupied: TileSet,
    ) -> Rc<Vec<(usize, u16)>> {
        let key = (self.movement_class(unit), start, occupied);
        if let Some(reachable) = self.runtime.reach_cache.borrow().get(&key) {
            return reachable.clone();
        }
        let batch_costs = self.movement_costs(unit);
        let cost_at = |tile: usize| match &batch_costs {
            Some(costs) => costs[tile],
            None => self.step_cost(unit, tile),
        };
        let reachable = Rc::new(REACH_SCRATCH.with_borrow_mut(|scratch| {
            let limit = self.movement_limit(unit);
            scratch.costs.resize(self.runtime.terrain.len(), u16::MAX);
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
                    for &next in &self.runtime.board.adjacent[current] {
                        if key.2.contains(next) {
                            continue;
                        }
                        let entry = cost_at(next);
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
        let mut cache = self.runtime.reach_cache.borrow_mut();
        if cache.len() >= 1024 {
            cache.clear();
        }
        cache.insert(key, reachable.clone());
        reachable
    }
    fn route_distances(&self, unit: &Unit, goals: &[usize], forward: bool) -> Rc<Vec<u16>> {
        let kind = self.route_class(unit);
        let query = match goals {
            [tile] => RouteGoals::Tile(*tile),
            tiles => RouteGoals::Tiles(tiles.to_vec()),
        };
        let key = (kind, query, self.runtime.terrain_revision, forward);
        if let Some(route) = self.runtime.route_cache.borrow().get(&key) {
            return route.clone();
        }
        let batch_costs = self.movement_costs(unit);
        let cost_at = |tile: usize| match &batch_costs {
            Some(costs) => costs[tile],
            None => self.step_cost(unit, tile),
        };
        let mut costs = vec![u16::MAX; self.runtime.terrain.len()];
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
            let entry = cost_at(current);
            if entry == u16::MAX {
                continue;
            }
            for &next in &self.runtime.board.adjacent[current] {
                let next_entry = cost_at(next);
                if next_entry == u16::MAX {
                    continue;
                }
                let cost = spent.saturating_add(if forward { next_entry } else { entry });
                if cost >= costs[next] {
                    continue;
                }
                costs[next] = cost;
                pending.push(Reverse((cost, next)));
            }
        }
        let mut cache = self.runtime.route_cache.borrow_mut();
        let mut order = self.runtime.route_order.borrow_mut();
        if cache.len() >= 128 {
            if let Some(oldest) = order.pop_front() {
                cache.remove(&oldest);
            }
        }
        order.push_back(key.clone());
        cache.insert(key.clone(), Rc::new(costs));
        drop(cache);
        self.runtime.route_cache.borrow()[&key].clone()
    }
    /// Turn-local route fields respect blockers instead of scoring impossible steps.
    /// With yielding enabled, friendly occupants add a cost and enemies remain blocked.
    pub(crate) fn occupied_routes(&self, unit: &Unit, goals: &[usize], yielding: bool) -> Vec<u16> {
        self.occupied_routes_with_penalty(unit, goals, yielding.then_some(2))
    }
    /// Campaign corridor requests can ask allies to yield without preferring
    /// a longer detour that postpones an otherwise reachable actor arrival.
    pub(crate) fn occupied_routes_with_penalty(
        &self,
        unit: &Unit,
        goals: &[usize],
        friendly_penalty: Option<u16>,
    ) -> Vec<u16> {
        let batch_costs = self.movement_costs(unit);
        let cost_at = |tile: usize| match &batch_costs {
            Some(costs) => costs[tile],
            None => self.step_cost(unit, tile),
        };
        let entry = |tile: usize| {
            let terrain = cost_at(tile);
            if terrain == u16::MAX {
                return terrain;
            }
            match self.at(tile) {
                Some(other) if other.id != unit.id => {
                    if let Some(penalty) = friendly_penalty.filter(|_| self.allied(other, unit)) {
                        terrain.saturating_add(penalty)
                    } else {
                        u16::MAX
                    }
                }
                _ => terrain,
            }
        };
        let mut costs = vec![u16::MAX; self.runtime.terrain.len()];
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
            for &next in &self.runtime.board.adjacent[current] {
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
    /// Human previews and live proposal admission use the acting group's
    /// observation, never the occupancy of concealed units.
    pub(crate) fn observed_reachable(&self, unit: &Unit) -> Rc<Vec<(usize, u16)>> {
        if unit.moved || unit.attack_spent || self.winner.is_some() {
            return EMPTY_REACH.with(Clone::clone);
        }
        self.reach_filtered(unit, unit.tile, self.observed_occupancy(unit))
    }
    fn observed_occupancy(&self, unit: &Unit) -> TileSet {
        if !self.perception_sensitive() {
            return self.occupied_tiles.clone();
        }
        let mut occupied = TileSet::new(self.terrain.len());
        let perception = self.perception(None);
        for other in &self.units {
            if perception.sees_unit(unit.vision_group, other) {
                occupied.insert(other.tile);
            }
        }
        occupied
    }
    /// Admission creates a different type from a request. The route is selected
    /// from known occupancy; only execution examines hidden blockers on it.
    pub(super) fn validate_observed_move(
        &self,
        id: u32,
        tile: usize,
    ) -> Result<ValidatedMove, CommandError> {
        let actor = self.unit(id).ok_or(CommandError::UnknownUnit(UnitId(id)))?;
        if !self.may_act(actor) {
            return Err(CommandError::WrongTurn);
        }
        let reachable = self.observed_reachable(actor);
        if actor.tile == tile || !reachable.iter().any(|&(t, _)| t == tile) {
            return Err(CommandError::IllegalMove);
        }
        let hidden = self.observed_occupancy(actor) != self.occupied_tiles;
        let path = (hidden || self.tracks_movement_path())
            .then(|| self.movement_path(actor, tile, &reachable));
        Ok(ValidatedMove {
            id,
            destination: tile,
            path,
        })
    }
    fn tracks_movement_path(&self) -> bool {
        self.tracks_path || self.tile_perception.iter().any(|p| p.remember) || {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.recordings.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        }
    }
    fn movement_path(
        &self,
        actor: &Unit,
        destination: usize,
        reachable: &[(usize, u16)],
    ) -> Vec<usize> {
        let mut costs = vec![u16::MAX; self.terrain.len()];
        costs[actor.tile] = 0;
        for &(site, cost) in reachable {
            costs[site] = cost;
        }
        // A visited predecessor search also handles zero-cost terrain without
        // cycling. It uses exactly the admitted observation's distance field.
        let mut next = vec![usize::MAX; self.terrain.len()];
        let mut queue = std::collections::VecDeque::from([destination]);
        next[destination] = destination;
        while let Some(cursor) = queue.pop_front() {
            if cursor == actor.tile {
                break;
            }
            for &previous in &self.board.adjacent[cursor] {
                if next[previous] == usize::MAX
                    && costs[previous] != u16::MAX
                    && costs[previous].checked_add(self.step_cost(actor, cursor))
                        == Some(costs[cursor])
                {
                    next[previous] = cursor;
                    queue.push_back(previous);
                }
            }
        }
        let mut path = vec![actor.tile];
        while *path.last().unwrap() != destination {
            let tile = next[*path.last().unwrap()];
            assert_ne!(tile, usize::MAX, "admitted movement has a route");
            path.push(tile);
        }
        path
    }
    pub(super) fn execute_observed_move(&mut self, mut movement: ValidatedMove) -> bool {
        let actor = *self.unit(movement.id).expect("admitted actor");
        let requested = movement.destination;
        let mut contact = None;
        if let Some(path) = &mut movement.path {
            if let Some((step, blocker)) = path
                .iter()
                .enumerate()
                .skip(1)
                .find_map(|(step, &tile)| self.at(tile).map(|unit| (step, unit.id)))
            {
                contact = Some(blocker);
                path.truncate(step);
                movement.destination = *path.last().unwrap();
            }
        }
        if let Some(blocker) = contact {
            self.reveal_contact(blocker, actor.vision_group);
        }
        self.apply_movement(
            movement.id,
            movement.destination,
            movement.path,
            contact.map(|_| requested),
        );
        contact.is_some()
    }
    pub(crate) fn move_unit(&mut self, id: u32, tile: usize) {
        let actor = *self.unit(id).unwrap();
        assert!(self.may_act(&actor) && actor.tile != tile && self.winner.is_none());
        let reachable = self.reachable(&actor);
        assert!(reachable.iter().any(|&(site, _)| site == tile));
        let path = self
            .tracks_movement_path()
            .then(|| self.movement_path(&actor, tile, &reachable));
        self.apply_movement(id, tile, path, None);
    }
    fn apply_movement(
        &mut self,
        id: u32,
        tile: usize,
        path: Option<Vec<usize>>,
        requested: Option<usize>,
    ) {
        let index = self.index(id).unwrap();
        let actor = self.units[index];
        self.runtime.traversal = path.map(|path| (id, path));
        self.runtime.occupancy[actor.tile] = None;
        self.runtime.occupancy[tile] = Some(index);
        self.runtime.occupied_tiles.remove(actor.tile);
        self.runtime.occupied_tiles.insert(tile);
        self.runtime.units[index].tile = tile;
        self.runtime.units[index].moved = true;
        if self.terrain_definition(tile).capturable()
            && self.runtime.owners[tile].as_deref().is_none_or(|owner| {
                self.faction_relation(self.unit_faction(&actor), owner) == Relation::Hostile
            })
        {
            self.runtime.owners[tile] = Some(self.unit_faction(&actor).to_owned());
            let unit_name = self.definitions.unit(actor.kind).name().to_lowercase();
            let capture = if self.terrain[tile] == self.definitions.terrain_name("village") {
                "captured a village".to_owned()
            } else {
                format!(
                    "claimed {}",
                    self.terrain_definition(tile).name().to_lowercase()
                )
            };
            self.record(format!(
                "{} {unit_name} {capture}.",
                if self.unit_faction(&actor) == "player" {
                    "Your"
                } else {
                    "Enemy"
                }
            ));
        }
        self.collect_items(id);
        #[cfg(not(target_arch = "wasm32"))]
        let path = self
            .traversal
            .as_ref()
            .map(|(_, path)| {
                path.iter()
                    .copied()
                    .map(|site| self.runtime.board.ids[site].clone())
                    .collect()
            })
            .unwrap_or_default();
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Move {
            requested: requested.map(|site| self.board.ids[site]),
            path,
            unit: format!("u{id}"),
            tile: self.runtime.board.ids[tile].clone(),
        });
    }
}

#[cfg(test)]
mod occupancy_tests {
    use super::*;
    #[test]
    fn occupancy_mask_follows_moves_roster_changes_and_independent_forks() {
        let rules = Catalogue::load().unwrap();
        let mut game = crate::logic::LiveBattle::new(&rules, "mirror");
        let forecast = game.fork();
        let (actor, destination) = game
            .units
            .iter()
            .filter(|u| u.phase == game.turn)
            .find_map(|u| game.reachable(u).first().map(|&(tile, _)| (u.id, tile)))
            .unwrap();
        game.move_unit(actor, destination);
        let free = (0..game.terrain.len())
            .find(|&tile| game.at(tile).is_none())
            .unwrap();
        game.add(game.units[0].kind, "player", free);
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
