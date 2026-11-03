//! Compact battle sessions borrowing catalog rules and indexing the live roster.

use crate::{data::*, protocol::StateFrame};
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct Board {
    pub(crate) ids: Vec<String>,
    pub(crate) positions: Vec<(i32, i32)>,
    pub(crate) indices: HashMap<String, usize>,
    pub(crate) adjacent: Vec<Vec<usize>>,
    pub(crate) adjacent_bits: Vec<u128>,
    pub(crate) distances: Vec<u16>,
}
impl Board {
    pub(crate) fn new(tiles: &[Tile]) -> Rc<Self> {
        let ids: Vec<String> = tiles.iter().map(|tile| tile.id.clone()).collect();
        let positions: Vec<_> = tiles.iter().map(|tile| (tile.col, tile.row)).collect();
        let indices = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.clone(), index))
            .collect();
        let adjacent: Vec<Vec<usize>> = (0..tiles.len())
            .map(|first| {
                (0..tiles.len())
                    .filter(|&second| hex_distance(positions[first], positions[second]) == 1)
                    .collect()
            })
            .collect();
        let adjacent_bits = if tiles.len() <= 128 {
            adjacent
                .iter()
                .map(|neighbors| {
                    neighbors
                        .iter()
                        .fold(0, |bits, &tile| bits | (1u128 << tile))
                })
                .collect()
        } else {
            vec![]
        };
        let coordinates = &positions;
        let distances = (0..tiles.len())
            .flat_map(|first| {
                (0..tiles.len())
                    .map(move |second| hex_distance(coordinates[first], coordinates[second]))
            })
            .collect();
        Rc::new(Self {
            ids,
            positions,
            indices,
            adjacent,
            adjacent_bits,
            distances,
        })
    }
    pub(crate) fn distance(&self, first: usize, second: usize) -> u16 {
        self.distances[first * self.ids.len() + second]
    }
}
pub(crate) fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
// A generated layout is reused by many policy/race pairings. These caches
// depend on catalog rules, geometry and terrain, never on a game's mutable state.
type TerrainKey = (String, Vec<(String, i32, i32, String)>);
#[derive(Clone)]
struct SharedTerrain {
    board: Rc<Board>,
    routes: Rc<RefCell<HashMap<(usize, Vec<usize>, u64), Rc<Vec<u16>>>>>,
    route_order: Rc<RefCell<VecDeque<(usize, Vec<usize>, u64)>>>,
    attacks: super::geometry::AttackGeometry,
    masks: super::geometry::AttackMasks,
    reach: super::movement::ReachCache,
    threat_reach: super::movement::ThreatReachCache,
    movement: super::movement::MovementMasks,
}
thread_local! {
    static TERRAIN_POOL: RefCell<(HashMap<TerrainKey, SharedTerrain>, VecDeque<TerrainKey>)> = RefCell::new((HashMap::default(), VecDeque::new()));
}
type MissionKey = (String, usize, bool, Vec<String>, bool);
/// Derived data for the current battle revision; refreshed after successful commands.
#[derive(Clone, Debug)]
struct TileGeometry {
    connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    floors: Rc<Vec<Option<String>>>,
    fog: Rc<Vec<Option<std::sync::Arc<ConditionRule>>>>,
}
#[cfg(feature = "wasm")]
impl TileGeometry {
    fn from_tiles(tiles: &[Tile]) -> Self {
        Self {
            connections: Rc::new(tiles.iter().map(|t| t.connections.clone()).collect()),
            walls: Rc::new(tiles.iter().map(|t| t.walls.clone()).collect()),
            dead_ends: Rc::new(tiles.iter().map(|t| t.dead_ends.clone()).collect()),
            floors: Rc::new(tiles.iter().map(|t| t.floor.clone()).collect()),
            fog: Rc::new(tiles.iter().map(|t| t.fog.clone()).collect()),
        }
    }
}
#[derive(Clone, Default)]
pub(crate) struct RuntimeCache {
    #[cfg(feature = "wasm")]
    geometry: Option<TileGeometry>,
    key: Option<MissionKey>,
    mission: Option<Rc<Mission>>,
}
/// Immutable planning facts are shared; a mutation detaches only its branch.
#[derive(Clone, Debug)]
pub(crate) struct Shared<T: Clone>(Rc<T>);
impl<T: Clone> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}
impl<T: Clone> Shared<T> {
    fn into_owned(self) -> T {
        Rc::unwrap_or_clone(self.0)
    }
}
impl<T: Clone> std::ops::Deref for Shared<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Clone> std::ops::DerefMut for Shared<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::make_mut(&mut self.0)
    }
}
impl<T: Clone> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
#[derive(Debug)]
pub(crate) struct Game<'catalog> {
    pub(crate) factions: Rc<Vec<Faction>>,
    /// Turn-local controller scope, never persisted.
    pub(crate) active_faction: Option<String>,
    pub(crate) rules: &'catalog Rules,
    pub(crate) board: Rc<Board>,
    pub(crate) terrain: Vec<usize>,
    pub(crate) owners: Shared<Vec<Option<String>>>,
    pub(crate) connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) fog: Rc<Vec<Option<std::sync::Arc<ConditionRule>>>>,
    pub(crate) condition_memory: Rc<Vec<String>>,
    /// Derived tile-indexed remembered fog; never serialized.
    pub(crate) fog_memory: Vec<u64>,
    pub(super) traversal: Option<(u32, Vec<usize>)>,
    pub(crate) floors: Rc<Vec<Option<String>>>,
    pub(crate) units: Vec<Unit>,
    pub(crate) ground_items: Vec<GroundItem>,
    pub(crate) map_id: String,
    pub(crate) map_revision: u32,
    pub(crate) cols: i32,
    pub(crate) rows: i32,
    pub(crate) round: u32,
    pub(crate) turn: Side,
    pub(crate) gold: Gold,
    pub(crate) winner: Option<Side>,
    pub(crate) next_id: u32,
    pub(crate) combat_rng: u32,
    pub(crate) log: Vec<String>,
    pub(crate) campaign: Option<Shared<CampaignState>>,
    pub(crate) last_combat: Option<LastCombat>,
    #[cfg(feature = "wasm")]
    pub(crate) combat_animation: Option<crate::protocol::CombatAnimation>,
    pub(crate) combat_present: bool,
    pub(crate) armies: Option<[String; 2]>,
    pub(crate) pools: Option<[Vec<usize>; 2]>,
    pub(crate) arena_first: Option<Side>,
    pub(crate) mission: Option<Rc<Mission>>,
    mission_key: Option<MissionKey>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) recordings: Option<Records>,
    pub(super) route_cache: Rc<RefCell<HashMap<(usize, Vec<usize>, u64), Rc<Vec<u16>>>>>,
    pub(super) terrain_revision: u64,
    pub(super) attack_geometry: super::geometry::AttackGeometry,
    pub(super) attack_masks: super::geometry::AttackMasks,
    pub(super) reach_cache: super::movement::ReachCache,
    pub(super) threat_reach_cache: super::movement::ThreatReachCache,
    pub(super) movement_masks: super::movement::MovementMasks,
    pub(super) route_order: Rc<RefCell<VecDeque<(usize, Vec<usize>, u64)>>>,
    pub(super) occupancy: Vec<Option<usize>>,
    pub(crate) occupied_tiles: super::geometry::TileSet,
    pub(super) unit_indices: HashMap<u32, usize>,
}
impl<'catalog> Game<'catalog> {
    pub(crate) fn from_state(rules: &'catalog Rules, state: State) -> Self {
        let key = (
            rules.content_fingerprint.clone(),
            state
                .tiles
                .iter()
                .map(|t| (t.id.clone(), t.col, t.row, t.terrain.clone()))
                .collect(),
        );
        let cached = TERRAIN_POOL.with_borrow(|(pool, _)| pool.get(&key).cloned());
        let board = cached
            .as_ref()
            .map_or_else(|| Board::new(&state.tiles), |c| c.board.clone());
        let mut game = Self::from_state_with_board(rules, state, board);
        if let Some(cached) = cached {
            game.route_cache = cached.routes;
            game.route_order = cached.route_order;
            game.attack_geometry = cached.attacks;
            game.attack_masks = cached.masks;
            game.reach_cache = cached.reach;
            game.threat_reach_cache = cached.threat_reach;
            game.movement_masks = cached.movement;
        } else {
            TERRAIN_POOL.with_borrow_mut(|(pool, order)| {
                if pool.len() >= 64 {
                    if let Some(oldest) = order.pop_front() {
                        pool.remove(&oldest);
                    }
                }
                order.push_back(key.clone());
                pool.insert(
                    key,
                    SharedTerrain {
                        board: game.board.clone(),
                        routes: game.route_cache.clone(),
                        route_order: game.route_order.clone(),
                        attacks: game.attack_geometry.clone(),
                        masks: game.attack_masks.clone(),
                        reach: game.reach_cache.clone(),
                        threat_reach: game.threat_reach_cache.clone(),
                        movement: game.movement_masks.clone(),
                    },
                );
            });
        }
        game
    }
    pub(crate) fn from_state_with_board(
        rules: &'catalog Rules,
        state: State,
        board: Rc<Board>,
    ) -> Self {
        let (state, tiles) = StateFrame::split(state);
        let n = tiles.len();
        let mut terrain = Vec::with_capacity(n);
        let mut owners = Vec::with_capacity(n);
        let mut connections = Vec::with_capacity(n);
        let mut walls = Vec::with_capacity(n);
        let mut dead_ends = Vec::with_capacity(n);
        let mut floors = Vec::with_capacity(n);
        let mut fog = Vec::with_capacity(n);
        for tile in tiles {
            terrain.push(rules.terrains[&tile.terrain]);
            owners.push(tile.owner);
            connections.push(tile.connections);
            walls.push(tile.walls);
            dead_ends.push(tile.dead_ends);
            floors.push(tile.floor);
            fog.push(tile.fog);
        }
        let connections = Rc::new(connections);
        let walls = Rc::new(walls);
        let dead_ends = Rc::new(dead_ends);
        let floors = Rc::new(floors);
        let geometry = TileGeometry {
            connections,
            walls,
            dead_ends,
            floors,
            fog: Rc::new(fog),
        };
        Self::from_parts(
            rules,
            state,
            board,
            RuntimeCache::default(),
            terrain,
            owners,
            geometry,
        )
    }
    #[cfg(feature = "wasm")]
    pub(crate) fn from_borrowed_state(
        rules: &'catalog Rules,
        state: &State,
        board: Rc<Board>,
        mut cache: RuntimeCache,
    ) -> Self {
        let geometry = cache
            .geometry
            .take()
            .unwrap_or_else(|| TileGeometry::from_tiles(&state.tiles));
        let terrain = state
            .tiles
            .iter()
            .map(|t| rules.terrains[&t.terrain])
            .collect();
        let owners = state.tiles.iter().map(|t| t.owner.clone()).collect();
        Self::from_parts(
            rules,
            StateFrame::from(state),
            board,
            cache,
            terrain,
            owners,
            geometry,
        )
    }
    fn from_parts(
        rules: &'catalog Rules,
        state: StateFrame,
        board: Rc<Board>,
        cache: RuntimeCache,
        terrain: Vec<usize>,
        owners: Vec<Option<String>>,
        geometry: TileGeometry,
    ) -> Self {
        let mut fog_memory = if geometry
            .fog
            .iter()
            .any(|rule| rule.as_ref().is_some_and(|r| r.remember))
        {
            vec![0; terrain.len().div_ceil(64)]
        } else {
            Vec::new()
        };
        if !fog_memory.is_empty() {
            for key in &state.condition_memory {
                if let Some(&i) = key
                    .strip_prefix("fog:")
                    .and_then(|id| board.indices.get(id))
                {
                    fog_memory[i / 64] |= 1u64 << (i % 64);
                }
            }
        }
        let units: Vec<Unit> = state
            .units
            .into_iter()
            .map(|unit| Unit {
                faction: unit.faction,
                items: unit.items,
                ai_group: unit.ai_group,
                id: unit.id.strip_prefix('u').unwrap().parse().unwrap(),
                kind: rules.types[&unit.r#type],
                side: unit.side,
                tile: board.indices[&unit.tile],
                hp: unit.hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                name: unit.name,
                role: unit.role,
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
            })
            .collect();
        let combat_present = state.last_combat.is_some();
        let last_combat = state.last_combat.flatten();
        let mut game = Self {
            factions: Rc::new(state.factions),
            ground_items: state.ground_items,
            active_faction: None,
            fog: geometry.fog,
            condition_memory: Rc::new(state.condition_memory),
            fog_memory,
            traversal: None,
            rules,
            board,
            terrain,
            owners: owners.into(),
            connections: geometry.connections,
            walls: geometry.walls,
            dead_ends: geometry.dead_ends,
            floors: geometry.floors,
            units,
            map_id: state.map_id,
            map_revision: state.map_revision,
            cols: state.cols,
            rows: state.rows,
            round: state.round,
            turn: state.turn,
            gold: state.gold,
            winner: state.winner,
            next_id: state.next_id,
            combat_rng: state.combat_rng.unwrap_or(0x6d2b79f5),
            log: state.log,
            campaign: state.campaign.map(Into::into),
            last_combat,
            #[cfg(feature = "wasm")]
            combat_animation: None,
            combat_present,
            armies: state.armies,
            pools: None,
            arena_first: None,
            mission: cache.mission,
            mission_key: cache.key,
            #[cfg(not(target_arch = "wasm32"))]
            recordings: None,
            route_cache: Rc::new(RefCell::new(HashMap::default())),
            terrain_revision: 0,
            attack_geometry: Rc::new(RefCell::new(HashMap::default())),
            attack_masks: super::geometry::AttackMasks::default(),
            reach_cache: super::movement::ReachCache::default(),
            threat_reach_cache: super::movement::ThreatReachCache::default(),
            movement_masks: super::movement::MovementMasks::default(),
            route_order: Rc::new(RefCell::new(VecDeque::new())),
            occupancy: vec![],
            occupied_tiles: super::geometry::TileSet::new(0),
            unit_indices: HashMap::default(),
        };
        game.roster_changed();
        game.refresh_mission();
        game.remember_conditions();
        game
    }
    pub(crate) fn new(rules: &'catalog Rules, map: &str) -> Self {
        let state = rules
            .maps
            .iter()
            .filter(|((id, _), _)| id == map)
            .max_by_key(|((_, revision), _)| revision)
            .unwrap()
            .1
            .clone();
        Self::from_state(rules, state)
    }
    /// Materialize the protocol snapshot without changing battle state.
    pub(crate) fn state(&self) -> State {
        State {
            factions: self.factions.as_ref().clone(),
            ground_items: self.ground_items.clone(),
            condition_memory: self.condition_memory.as_ref().clone(),
            armies: self.armies.clone(),
            map_id: self.map_id.clone(),
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold.clone(),
            winner: self.winner,
            next_id: self.next_id,
            combat_rng: Some(self.combat_rng),
            log: self.log.clone(),
            tiles: self.tiles(),
            units: self.public_units(),
            campaign: self.campaign.as_deref().cloned(),
            last_combat: self.combat_present.then(|| self.last_combat.clone()),
        }
    }
    /// Consume a transient runtime after its inspection and effects are resolved.
    #[cfg(feature = "wasm")]
    pub(crate) fn into_state(mut self, tiles: Vec<Tile>) -> State {
        debug_assert_eq!(tiles.len(), self.board.ids.len());
        let units = std::mem::take(&mut self.units)
            .into_iter()
            .map(|unit| PublicUnit {
                faction: unit.faction,
                items: unit.items,
                ai_group: unit.ai_group,
                id: format!("u{}", unit.id),
                r#type: self.rules.units[unit.kind].id.clone(),
                side: unit.side,
                tile: self.board.ids[unit.tile].clone(),
                hp: unit.hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                name: unit.name,
                role: unit.role,
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
            })
            .collect();
        State {
            factions: Rc::unwrap_or_clone(self.factions),
            ground_items: self.ground_items,
            condition_memory: Rc::unwrap_or_clone(self.condition_memory),
            armies: self.armies,
            map_id: self.map_id,
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold,
            winner: self.winner,
            next_id: self.next_id,
            combat_rng: Some(self.combat_rng),
            log: self.log,
            tiles,
            units,
            campaign: self.campaign.map(Shared::into_owned),
            last_combat: self.combat_present.then_some(self.last_combat),
        }
    }
    /// Patch the authoritative terrain only where current runtime values differ.
    #[cfg(feature = "wasm")]
    pub(crate) fn patch_tiles(&self, tiles: &mut [Tile]) -> Vec<usize> {
        debug_assert_eq!(tiles.len(), self.board.ids.len());
        let mut changed = Vec::new();
        for (i, tile) in tiles.iter_mut().enumerate() {
            let Tile {
                id: _,
                col,
                row,
                terrain,
                owner,
                connections,
                dead_ends,
                walls,
                floor,
                fog,
            } = tile;
            let position = self.board.positions[i];
            let kind = &self.rules.terrain_names[self.terrain[i]];
            if *col != position.0
                || *row != position.1
                || terrain != kind
                || *owner != self.owners[i]
                || *connections != self.connections[i]
                || *dead_ends != self.dead_ends[i]
                || *walls != self.walls[i]
                || *floor != self.floors[i]
                || *fog != self.fog[i]
            {
                *col = position.0;
                *row = position.1;
                terrain.clone_from(kind);
                owner.clone_from(&self.owners[i]);
                connections.clone_from(&self.connections[i]);
                dead_ends.clone_from(&self.dead_ends[i]);
                walls.clone_from(&self.walls[i]);
                floor.clone_from(&self.floors[i]);
                fog.clone_from(&self.fog[i]);
                changed.push(i);
            }
        }
        changed
    }
    pub(crate) fn tiles(&self) -> Vec<Tile> {
        self.board
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| Tile {
                id: id.clone(),
                col: self.board.positions[index].0,
                row: self.board.positions[index].1,
                terrain: self.rules.terrain_names[self.terrain[index]].clone(),
                owner: self.owners[index].clone(),
                connections: self.connections[index].clone(),
                walls: self.walls[index].clone(),
                dead_ends: self.dead_ends[index].clone(),
                floor: self.floors[index].clone(),
                fog: self.fog[index].clone(),
            })
            .collect()
    }
    pub(crate) fn public_unit(&self, unit: &Unit) -> PublicUnit {
        PublicUnit {
            faction: unit.faction.clone(),
            items: unit.items.clone(),
            ai_group: unit.ai_group.clone(),
            id: format!("u{}", unit.id),
            r#type: self.rules.units[unit.kind].id.clone(),
            side: unit.side,
            tile: self.board.ids[unit.tile].clone(),
            hp: unit.hp,
            moved: unit.moved,
            attacked: unit.attacked,
            experience: unit.experience,
            name: unit.name.clone(),
            role: unit.role.clone(),
            retreating: unit.retreating,
            rebuilding: unit.rebuilding,
        }
    }
    pub(crate) fn public_units(&self) -> Vec<PublicUnit> {
        self.units
            .iter()
            .map(|unit| self.public_unit(unit))
            .collect()
    }
    pub(crate) fn unit(&self, id: u32) -> Option<&Unit> {
        self.index(id).map(|index| &self.units[index])
    }
    pub(crate) fn at(&self, tile: usize) -> Option<&Unit> {
        self.occupancy
            .get(tile)
            .copied()
            .flatten()
            .and_then(|index| self.units.get(index))
            .filter(|unit| unit.tile == tile)
    }
    pub(crate) fn index(&self, id: u32) -> Option<usize> {
        self.unit_indices
            .get(&id)
            .copied()
            .filter(|&index| self.units.get(index).is_some_and(|u| u.id == id))
    }
    pub(crate) fn site(&self, id: &str) -> usize {
        self.board.indices[id]
    }
    pub(crate) fn kind(&self, name: &str) -> usize {
        self.rules.types[name]
    }
    pub(crate) fn terrain_role(&self, tile: usize) -> TerrainRole {
        self.rules.terrain_roles[self.terrain[tile]]
    }
    pub(crate) fn role_is(&self, tile: usize, role: TerrainRole) -> bool {
        self.terrain_role(tile) == role
    }
    /// Capability is authored once and exported through the catalog for UI discovery too.
    pub(crate) fn is_keep(&self, tile: usize) -> bool {
        self.role_is(tile, TerrainRole::Keep)
    }
    pub(crate) fn record(&mut self, message: String) {
        self.log.insert(0, message);
        self.log.truncate(8);
    }
    pub(crate) fn add(&mut self, kind: usize, side: Side, tile: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            faction: None,
            items: vec![],
            ai_group: None,
            id,
            kind,
            side,
            tile,
            hp: self.rules.units[kind].hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            name: None,
            role: None,
            retreating: None,
            rebuilding: None,
        });
        self.unit_indices.insert(id, self.units.len() - 1);
        self.occupancy[tile] = Some(self.units.len() - 1);
        self.occupied_tiles.insert(tile);
        id
    }
    /// Refresh dense lookup tables after an internal roster transformation.
    pub(crate) fn roster_changed(&mut self) {
        self.occupancy = vec![None; self.terrain.len()];
        self.occupied_tiles = super::geometry::TileSet::new(self.terrain.len());
        self.unit_indices.clear();
        for (index, unit) in self.units.iter().enumerate() {
            self.occupancy[unit.tile] = Some(index);
            self.occupied_tiles.insert(unit.tile);
            self.unit_indices.insert(unit.id, index);
        }
    }
    /// Change terrain and invalidate only terrain-dependent route results.
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        if self.terrain[tile] != terrain {
            self.terrain[tile] = terrain;
            Rc::make_mut(&mut self.connections)[tile] = None;
            Rc::make_mut(&mut self.walls)[tile] = None;
            Rc::make_mut(&mut self.dead_ends)[tile] = None;
            self.attack_geometry = Rc::new(RefCell::new(HashMap::default()));
            self.attack_masks = super::geometry::AttackMasks::default();
            self.reach_cache = super::movement::ReachCache::default();
            self.threat_reach_cache = super::movement::ThreatReachCache::default();
            self.movement_masks = super::movement::MovementMasks::default();
            self.terrain_revision = self.terrain_revision.wrapping_add(1);
            self.route_cache = Rc::new(RefCell::new(HashMap::default()));
            self.route_order = Rc::new(RefCell::new(VecDeque::new()));
        }
    }

    #[cfg(feature = "wasm")]
    pub(crate) fn runtime_cache(&self) -> RuntimeCache {
        RuntimeCache {
            geometry: Some(TileGeometry {
                connections: self.connections.clone(),
                walls: self.walls.clone(),
                dead_ends: self.dead_ends.clone(),
                floors: self.floors.clone(),
                fog: self.fog.clone(),
            }),
            key: self.mission_key.clone(),
            mission: self.mission.clone(),
        }
    }
    pub(crate) fn refresh_mission(&mut self) {
        let Some(state) = &self.campaign else {
            self.mission = None;
            self.mission_key = None;
            return;
        };
        let without_dain = state.chosen("refuge-war")
            || state.chosen("green-dain-dead")
            || state.marked("dain-fell");
        let mut fallen = state.fallen_companions.clone().unwrap_or_default();
        for mark in &state.marks {
            if let Some(name) = mark.strip_prefix("fallen:") {
                if !fallen.iter().any(|n| n == name) {
                    fallen.push(name.to_owned());
                }
            }
        }
        let key = (
            state.id.clone(),
            state.mission,
            without_dain,
            fallen.clone(),
            state.escalated(),
        );
        if self.mission_key.as_ref() == Some(&key) {
            return;
        }
        self.mission_key = Some(key);
        let mut mission = Rc::new(self.rules.campaigns[&state.id].missions[state.mission].clone());
        if without_dain {
            if let Some(patch) = mission.without_dain.clone() {
                Rc::make_mut(&mut mission).apply(&patch);
            }
        }
        for name in &fallen {
            if let Some(patch) = mission
                .companion_losses
                .as_ref()
                .and_then(|patches| patches.get(name))
                .cloned()
            {
                Rc::make_mut(&mut mission).apply(&patch);
            }
        }
        if mission
            .intro
            .iter()
            .chain(&mission.victory)
            .any(|line| fallen.contains(&line.speaker))
        {
            let changed = Rc::make_mut(&mut mission);
            changed.intro.retain(|line| !fallen.contains(&line.speaker));
            changed
                .victory
                .retain(|line| !fallen.contains(&line.speaker));
        }
        if state.escalated() {
            if let Some(refuge) = mission.refuge.clone() {
                let changed = Rc::make_mut(&mut mission);
                changed.briefing = refuge.war_briefing;
                changed.hints = refuge.war_hints;
            }
        }
        self.mission = Some(mission);
    }
}

impl<'catalog> Game<'catalog> {
    pub(crate) fn planning_fork(&self) -> Self {
        let mut draft = self.fork();
        draft.combat_rng = 0x6d2b79f5;
        draft
    }
    pub(crate) fn fork(&self) -> Self {
        Self {
            factions: self.factions.clone(),
            ground_items: self.ground_items.clone(),
            active_faction: self.active_faction.clone(),
            condition_memory: self.condition_memory.clone(),
            fog_memory: self.fog_memory.clone(),
            traversal: None,
            fog: self.fog.clone(),
            rules: self.rules,
            board: self.board.clone(),
            terrain: self.terrain.clone(),
            owners: self.owners.clone(),
            connections: self.connections.clone(),
            walls: self.walls.clone(),
            dead_ends: self.dead_ends.clone(),
            floors: self.floors.clone(),
            units: self.units.clone(),
            map_id: self.map_id.clone(),
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold.clone(),
            winner: self.winner,
            next_id: self.next_id,
            combat_rng: self.combat_rng,
            log: self.log.clone(),
            campaign: self.campaign.clone(),
            last_combat: self.last_combat.clone(),
            #[cfg(feature = "wasm")]
            combat_animation: None,
            combat_present: self.combat_present,
            armies: self.armies.clone(),
            pools: self.pools.clone(),
            arena_first: self.arena_first,
            mission: self.mission.clone(),
            mission_key: self.mission_key.clone(),
            #[cfg(not(target_arch = "wasm32"))]
            recordings: None,
            route_cache: self.route_cache.clone(),
            terrain_revision: self.terrain_revision,
            attack_geometry: self.attack_geometry.clone(),
            attack_masks: self.attack_masks.clone(),
            reach_cache: self.reach_cache.clone(),
            threat_reach_cache: self.threat_reach_cache.clone(),
            movement_masks: self.movement_masks.clone(),
            route_order: self.route_order.clone(),
            occupancy: self.occupancy.clone(),
            occupied_tiles: self.occupied_tiles.clone(),
            unit_indices: self.unit_indices.clone(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn action(&mut self, value: ActionRecord) {
        if let Some(records) = &mut self.recordings {
            records.actions.push(value);
        }
    }
    pub(crate) fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        if self.winner.is_some() || unit.side != self.turn || unit.moved || unit.attacked {
            return vec![];
        }
        let rule = &self.rules.units[unit.kind];
        if rule
            .experience_needed
            .is_none_or(|needed| unit.experience.unwrap_or(0) < needed)
        {
            return vec![];
        }
        rule.advances_to
            .iter()
            .map(|name| self.kind(name))
            .filter(|&kind| self.cost(kind, unit.tile) != u16::MAX)
            .collect()
    }
    pub(crate) fn advance(&mut self, id: u32, kind: usize) {
        let index = self.index(id).unwrap();
        assert!(self.promotion_options(&self.units[index]).contains(&kind));
        let previous = &self.rules.units[self.units[index].kind];
        let name = self.units[index]
            .name
            .clone()
            .unwrap_or(previous.name.clone());
        let wounds = previous.hp - self.units[index].hp;
        let experience = self.units[index]
            .experience
            .unwrap_or(0)
            .saturating_sub(previous.experience_needed.unwrap_or(0));
        let unit = &mut self.units[index];
        unit.kind = kind;
        unit.experience = Some(experience);
        unit.hp = (self.rules.units[kind].hp - wounds).max(1);
        unit.moved = true;
        unit.attacked = true;
        self.record(format!(
            "{} advanced to {}.",
            name, self.rules.units[kind].name
        ));
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Advance {
            unit: format!("u{id}"),
            unit_type: self.rules.units[kind].id.clone(),
        });
    }

    pub(crate) fn recruitment_options(&self, side: Side) -> Vec<usize> {
        self.recruitment_options_for_faction(self.faction_for_phase(side))
    }
    pub(crate) fn recruitment_options_for_faction(&self, id: &str) -> Vec<usize> {
        if let Some(pool) = self.faction(id).and_then(|f| f.recruits.as_ref()) {
            return pool.iter().map(|entry| self.kind(&entry.unit)).collect();
        }
        match id {
            "player" => self.phase_recruitment_options(Side::Player),
            "enemy" => self.phase_recruitment_options(Side::Enemy),
            _ => vec![],
        }
    }
    fn phase_recruitment_options(&self, side: Side) -> Vec<usize> {
        if let Some(pools) = &self.pools {
            return pools[side.index()].clone();
        }
        if let Some(armies) = &self.armies {
            return self
                .rules
                .rosters
                .iter()
                .find(|roster| roster.id == armies[side.index()])
                .expect("validated skirmish faction")
                .pool
                .iter()
                .map(|name| self.kind(name))
                .collect();
        }
        if let Some(mission) = &self.mission {
            let mut options: Vec<_> = if side == Side::Player {
                &mission.recruits
            } else {
                &mission.enemy_recruits
            }
            .iter()
            .map(|name| self.kind(name))
            .collect();
            if side == Side::Player {
                for event in &mission.events {
                    if self
                        .campaign
                        .as_ref()
                        .is_some_and(|state| state.marked_prefixed("event:", &event.id))
                    {
                        for kind in event.recruits.iter().map(|name| self.kind(name)) {
                            if !options.contains(&kind) {
                                options.push(kind);
                            }
                        }
                    }
                }
            }
            return options;
        }
        [
            "swordsman",
            "spearman",
            "archer",
            "scout",
            "adept",
            "rider",
            "horse-archer",
        ]
        .iter()
        .map(|name| self.kind(name))
        .collect()
    }
    pub(crate) fn recruitment_options_for(&self, leader: &Unit) -> Vec<usize> {
        self.recruitment_options_for_faction(self.unit_faction(leader))
    }
    pub(crate) fn recruit_cost(&self, leader: &Unit, kind: usize) -> i32 {
        self.recruit_cost_for_faction(self.unit_faction(leader), kind)
    }
    pub(crate) fn recruit_cost_for_faction(&self, id: &str, kind: usize) -> i32 {
        self.faction(id)
            .and_then(|f| f.recruits.as_ref())
            .and_then(|pool| {
                pool.iter()
                    .find(|entry| entry.unit == self.rules.units[kind].id)
            })
            .map_or(self.rules.units[kind].cost, |entry| entry.cost)
    }
    pub(crate) fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> Vec<usize> {
        if !leader.is_leader()
            || !self.recruitment_options_for(leader).contains(&kind)
            || self.winner.is_some()
            || !self.may_act(leader)
            || !self.is_keep(leader.tile)
            || self.owners[leader.tile].as_deref() != Some(self.unit_faction(leader))
            || (price && self.gold_for_unit(leader) < self.recruit_cost(leader, kind))
        {
            return vec![];
        }
        self.board.adjacent[leader.tile]
            .iter()
            .copied()
            .filter(|&tile| self.cost(kind, tile) != u16::MAX && self.at(tile).is_none())
            .collect()
    }
    pub(crate) fn recruit(&mut self, id: u32, kind: usize, tile: usize) {
        let leader = self.unit(id).unwrap().clone();
        assert!(self.placement_tiles(&leader, kind, true).contains(&tile));
        let faction = self.unit_faction(&leader).to_owned();
        let cost = self.recruit_cost(&leader, kind);
        self.gold[&faction] -= cost;
        self.add(kind, leader.side, tile);
        let unit = self.units.last_mut().unwrap();
        unit.faction = leader.faction.clone();
        unit.moved = true;
        unit.attacked = true;
        self.record(format!(
            "{} recruited a {}. It acts next turn.",
            if leader.side == Side::Player {
                "You"
            } else {
                "Enemy"
            },
            self.rules.units[kind].name.to_lowercase()
        ));
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Recruit {
            unit: format!("u{id}"),
            unit_type: self.rules.units[kind].id.clone(),
            tile: self.board.ids[tile].clone(),
            cost: Some(cost),
        });
    }
}
