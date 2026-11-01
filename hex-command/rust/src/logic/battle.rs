//! Compact battle sessions borrowing catalog rules and indexing the live roster.

#[cfg(not(target_arch = "wasm32"))]
use super::indices::UnitIndices;
use crate::data::*;
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
type UnitIndices = HashMap<u32, usize>;

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
fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
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
    route_buffers: super::storage::Buffers<u16>,
    reach_buffers: super::storage::Buffers<(usize, u16)>,
    routes: Rc<RefCell<HashMap<(usize, super::geometry::TileSet, u64), Rc<Vec<u16>>>>>,
    route_order: Rc<RefCell<VecDeque<(usize, super::geometry::TileSet, u64)>>>,
    occupied_routes: super::movement::OccupiedRouteCache,
    atlas: Rc<RefCell<super::movement::RouteAtlas>>,
    attacks: super::geometry::AttackGeometry,
    masks: super::geometry::AttackMasks,
    reach: super::movement::ReachCache,
    threat_reach: super::movement::ThreatReachCache,
    movement: super::movement::MovementMasks,
}
thread_local! {
    static TERRAIN_POOL: RefCell<(HashMap<TerrainKey, SharedTerrain>, VecDeque<TerrainKey>)> = RefCell::new((HashMap::default(), VecDeque::new()));
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct MissionKey {
    campaign: usize,
    stage: usize,
    without_dain: bool,
    escalated: bool,
    losses: smallvec::SmallVec<[usize; 4]>,
}
type MissionCache = Rc<RefCell<Vec<(MissionKey, Rc<Mission>)>>>;
/// Borrow native catalogs without ownership overhead; browser battles retain their
/// catalog safely without borrowing the factory that owns their handle table.
#[derive(Clone, Debug)]
pub(crate) enum RuleSource<'catalog> {
    Borrowed(&'catalog Rules),
    #[cfg(all(
        feature = "wasm",
        any(not(target_arch = "wasm32"), feature = "runtime-catalog")
    ))]
    Shared(Rc<Rules>),
}
impl std::ops::Deref for RuleSource<'_> {
    type Target = Rules;
    fn deref(&self) -> &Rules {
        match self {
            Self::Borrowed(rules) => rules,
            #[cfg(all(
                feature = "wasm",
                any(not(target_arch = "wasm32"), feature = "runtime-catalog")
            ))]
            Self::Shared(rules) => rules,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Game<'catalog> {
    forked: bool,
    fork_pool: Rc<RefCell<Vec<super::forks::Storage>>>,
    pub(crate) rules: RuleSource<'catalog>,
    pub(super) attack_classes: Rc<super::exchange::AttackClasses>,
    pub(crate) board: Rc<Board>,
    pub(crate) terrain: Rc<[usize]>,
    pub(crate) owners: Vec<Option<Side>>,
    pub(crate) connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) floors: Rc<Vec<Option<String>>>,
    pub(crate) units: Vec<Unit>,
    pub(crate) map_id: String,
    pub(crate) map_revision: u32,
    pub(crate) cols: i32,
    pub(crate) rows: i32,
    pub(crate) round: u32,
    pub(crate) turn: Side,
    pub(crate) gold: [i32; 2],
    pub(crate) winner: Option<Side>,
    pub(crate) next_id: u32,
    pub(crate) combat_rng: u32,
    pub(crate) log: Vec<String>,
    pub(super) log_spare: RefCell<Vec<String>>,
    pub(crate) presentation: bool,
    pub(super) marker_spare: Vec<String>,
    pub(crate) campaign: Option<CampaignState>,
    pub(crate) last_combat: Option<LastCombat>,
    pub(super) last_combat_spare: Option<LastCombat>,
    #[cfg(feature = "wasm")]
    pub(crate) combat_animation: super::playback::Playback,
    pub(crate) combat_present: bool,
    pub(crate) armies: Option<[String; 2]>,
    pub(crate) pools: Option<Rc<[Vec<usize>; 2]>>,
    pub(crate) arena_first: Option<Side>,
    pub(crate) mission: Option<Rc<Mission>>,
    mission_revision: Option<[usize; 6]>,
    mission_cache: MissionCache,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) recordings: Option<Records>,
    pub(super) route_buffers: super::storage::Buffers<u16>,
    pub(super) reach_buffers: super::storage::Buffers<(usize, u16)>,
    pub(super) route_atlas: Rc<RefCell<super::movement::RouteAtlas>>,
    pub(super) occupied_route_cache: super::movement::OccupiedRouteCache,
    pub(super) route_cache:
        Rc<RefCell<HashMap<(usize, super::geometry::TileSet, u64), Rc<Vec<u16>>>>>,
    pub(super) terrain_revision: u64,
    pub(super) attack_geometry: super::geometry::AttackGeometry,
    pub(super) attack_masks: super::geometry::AttackMasks,
    pub(super) reach_cache: super::movement::ReachCache,
    pub(super) threat_reach_cache: super::movement::ThreatReachCache,
    pub(super) movement_masks: super::movement::MovementMasks,
    pub(super) route_order: Rc<RefCell<VecDeque<(usize, super::geometry::TileSet, u64)>>>,
    pub(super) occupancy: Vec<Option<usize>>,
    pub(crate) occupied_tiles: super::geometry::TileSet,
    pub(super) unit_indices: UnitIndices,
}
impl<'catalog> Game<'catalog> {
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(crate) fn from_state(rules: &'catalog Rules, state: State) -> Self {
        Self::from_source(RuleSource::Borrowed(rules), state)
    }
    pub(crate) fn from_source(rules: RuleSource<'catalog>, state: State) -> Self {
        super::exchange::prepare(rules.units.iter().map(|u| u.hp).max().unwrap_or(0));
        #[cfg(target_arch = "wasm32")]
        {
            super::damage::prepare();
            crate::sort::prepare(state.tiles.len());
        }
        crate::ai::prepare_storage(
            state.tiles.len(),
            rules
                .units
                .iter()
                .map(|u| u.attacks.len())
                .max()
                .unwrap_or(0),
        );
        let key = (
            rules.content_fingerprint.to_string(),
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
        let mut game = Self::from_source_with_board(rules, state, board);
        #[cfg(target_arch = "wasm32")]
        game.units
            .reserve(game.terrain.len().saturating_sub(game.units.len()));
        #[cfg(target_arch = "wasm32")]
        game.unit_indices
            .reserve(game.terrain.len().saturating_sub(game.unit_indices.len()));
        if let Some(cached) = cached {
            super::movement::prepare(
                game.terrain.len(),
                game.rules.units.iter().map(|u| u.r#move).max().unwrap_or(0),
                game.rules
                    .units
                    .iter()
                    .flat_map(|u| u.costs.iter().copied().flatten())
                    .max()
                    .unwrap_or(0) as usize,
            );
            game.route_buffers = cached.route_buffers;
            game.reach_buffers = cached.reach_buffers;
            game.route_cache = cached.routes;
            game.occupied_route_cache = cached.occupied_routes;
            game.route_atlas = cached.atlas;
            game.route_order = cached.route_order;
            game.attack_geometry = cached.attacks;
            game.attack_masks = cached.masks;
            game.reach_cache = cached.reach;
            game.threat_reach_cache = cached.threat_reach;
            game.movement_masks = cached.movement;
        } else {
            game.prepare_buffers();
            TERRAIN_POOL.with_borrow_mut(|(pool, order)| {
                if pool.len() >= if cfg!(target_arch = "wasm32") { 4 } else { 64 } {
                    if let Some(oldest) = order.pop_front() {
                        pool.remove(&oldest);
                    }
                }
                order.push_back(key.clone());
                pool.insert(
                    key,
                    SharedTerrain {
                        board: game.board.clone(),
                        route_buffers: game.route_buffers.clone(),
                        reach_buffers: game.reach_buffers.clone(),
                        routes: game.route_cache.clone(),
                        occupied_routes: game.occupied_route_cache.clone(),
                        atlas: game.route_atlas.clone(),
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
        #[cfg(target_arch = "wasm32")]
        {
            let tiles = game.terrain.len();
            if tiles <= 256 {
                let weapons = game
                    .rules
                    .units
                    .iter()
                    .map(|u| u.attacks.len())
                    .max()
                    .unwrap_or(0);
                let future = game
                    .rules
                    .rosters
                    .iter()
                    .map(|r| r.pool.len())
                    .max()
                    .unwrap_or(0);
                game.route_buffers
                    .borrow_mut()
                    .prepare(128 + 2 * (tiles + future) * weapons + tiles * 3, tiles);
                game.reach_buffers
                    .borrow_mut()
                    .prepare(256 + tiles * 2, tiles);
            }
        }
        game.prepare_campaign_storage();
        game.prepare_forks();
        game
    }
    #[cfg(target_arch = "wasm32")]
    pub(super) fn prepare_campaign_storage(&mut self) {
        let (count, bytes) = self.marker_capacity();
        if let Some(state) = &mut self.campaign {
            state.marks.reserve(count.saturating_sub(state.marks.len()));
            let mission = self.mission.as_ref().expect("campaign mission");
            let dialogue = mission.intro.len()
                + mission.victory.len()
                + mission
                    .events
                    .iter()
                    .map(|e| {
                        e.dialogue
                            .len()
                            .max(e.war_dialogue.as_ref().map_or(0, |d| d.len()))
                    })
                    .sum::<usize>()
                + mission
                    .refuge
                    .as_ref()
                    .map_or(0, |r| r.escalation.len() + r.war_victory.len())
                + self.rules.campaigns[&state.id].ending.len()
                + 1;
            state
                .dialogue
                .reserve(dialogue.saturating_sub(state.dialogue.len()));
        }
        while self.marker_spare.len() < count {
            self.marker_spare.push(String::with_capacity(bytes));
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub(super) fn marker_capacity(&self) -> (usize, usize) {
        let Some(mission) = &self.mission else {
            return (0, 0);
        };
        let count = self.terrain.len() + mission.events.len() + mission.waves.len() + 16;
        let bytes = mission
            .events
            .iter()
            .map(|e| e.id.len() + 6)
            .chain(mission.waves.iter().map(|w| w.id.len() + 5))
            .chain(self.board.ids.iter().map(|id| id.len() + 7))
            .chain(
                self.units
                    .iter()
                    .filter_map(|u| u.name.as_ref())
                    .map(|n| n.len() + 7),
            )
            .chain(
                self.campaign
                    .iter()
                    .flat_map(|c| c.marks.iter().map(String::len)),
            )
            .max()
            .unwrap_or(32)
            .max(32);
        (count, bytes)
    }
    pub(super) fn marker(&mut self, prefix: &str, value: &str) -> String {
        let mut buffer = self
            .marker_spare
            .pop()
            .unwrap_or_else(|| String::with_capacity(prefix.len() + value.len()));
        buffer.clear();
        buffer.push_str(prefix);
        buffer.push_str(value);
        buffer
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn prepare_campaign_storage(&mut self) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn prepare_forks(&self) {}
    #[cfg(target_arch = "wasm32")]
    pub(super) fn prepare_forks(&self) {
        let mut pool = self.fork_pool.borrow_mut();
        // At most 32 rescue candidates coexist, plus active planning/forecast drafts.
        let count = 40;
        pool.resize_with(count, super::forks::Storage::default);
        for storage in pool.iter_mut() {
            storage.copy(self, false);
        }
    }
    fn prepare_buffers(&mut self) {
        self.route_atlas = Rc::new(RefCell::new(super::movement::RouteAtlas::new(
            &self.rules,
            &self.terrain,
            &self.board,
        )));
        let tiles = self.terrain.len();
        #[cfg(target_arch = "wasm32")]
        self.units.reserve(tiles.saturating_sub(self.units.len()));
        #[cfg(target_arch = "wasm32")]
        self.unit_indices
            .reserve(tiles.saturating_sub(self.unit_indices.len()));
        self.route_buffers = super::storage::SharedBuffers::shared(0, tiles);
        self.reach_buffers = super::storage::SharedBuffers::shared(0, tiles);
        self.route_cache.borrow_mut().reserve(128);
        self.occupied_route_cache.borrow_mut().reserve(256);
        self.route_order.borrow_mut().reserve(128);
        self.reach_cache
            .borrow_mut()
            .reserve(if cfg!(target_arch = "wasm32") {
                256
            } else {
                1024
            });
        self.threat_reach_cache
            .borrow_mut()
            .reserve(if cfg!(target_arch = "wasm32") {
                512
            } else {
                4096
            });
        #[cfg(target_arch = "wasm32")]
        self.prepare_masks();
        super::movement::prepare(
            tiles,
            self.rules.units.iter().map(|u| u.r#move).max().unwrap_or(0),
            self.rules
                .units
                .iter()
                .flat_map(|u| u.costs.iter().copied().flatten())
                .max()
                .unwrap_or(0) as usize,
        );
    }
    fn from_source_with_board(rules: RuleSource<'catalog>, state: State, board: Rc<Board>) -> Self {
        let units: Vec<Unit> = state
            .units
            .iter()
            .map(|unit| Unit {
                id: unit.id.strip_prefix('u').unwrap().parse().unwrap(),
                kind: rules.types[&unit.r#type],
                side: unit.side,
                tile: board.indices[&unit.tile],
                hp: unit.hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                name: unit.name.as_deref().map(Into::into),
                role: unit.role.clone(),
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
            })
            .collect();
        let terrain = state
            .tiles
            .iter()
            .map(|tile| rules.terrains[&tile.terrain])
            .collect();
        let owners = state.tiles.iter().map(|tile| tile.owner).collect();
        let connections = Rc::new(
            state
                .tiles
                .iter()
                .map(|tile| tile.connections.clone())
                .collect(),
        );
        let combat_present = state.last_combat.is_some();
        let dead_ends = Rc::new(
            state
                .tiles
                .iter()
                .map(|tile| tile.dead_ends.clone())
                .collect(),
        );
        let last_combat = state.last_combat.flatten();
        let floors = Rc::new(state.tiles.iter().map(|tile| tile.floor.clone()).collect());
        let route_atlas = Rc::new(RefCell::new(super::movement::RouteAtlas::default()));
        let attack_classes = super::exchange::attack_classes(&rules);
        let mut game = Self {
            forked: false,
            fork_pool: Rc::new(RefCell::new(Vec::new())),
            rules,
            attack_classes,
            board,
            terrain,
            owners,
            connections,
            dead_ends,
            floors,
            units,
            map_id: state.map_id,
            map_revision: state.map_revision,
            cols: state.cols,
            rows: state.rows,
            round: state.round,
            turn: state.turn,
            gold: [state.gold.player, state.gold.enemy],
            winner: state.winner,
            next_id: state.next_id,
            combat_rng: state.combat_rng.unwrap_or(0x6d2b79f5),
            log: {
                let mut log = state.log;
                log.reserve(8usize.saturating_sub(log.len()));
                for line in &mut log {
                    line.reserve(1024usize.saturating_sub(line.len()));
                }
                log
            },
            log_spare: RefCell::new((0..11).map(|_| String::with_capacity(1024)).collect()),
            presentation: true,
            marker_spare: Vec::new(),
            campaign: state.campaign,
            last_combat,
            last_combat_spare: Some(LastCombat {
                attacker_id: String::with_capacity(11),
                attack_id: String::with_capacity(128),
                sound: String::with_capacity(128),
            }),
            #[cfg(feature = "wasm")]
            combat_animation: super::playback::Playback::new(0),
            combat_present,
            armies: state.armies,
            pools: None,
            arena_first: None,
            mission: None,
            mission_revision: None,
            mission_cache: Rc::new(RefCell::new(Vec::with_capacity(16))),
            #[cfg(not(target_arch = "wasm32"))]
            recordings: None,
            route_buffers: super::storage::SharedBuffers::shared(0, 0),
            reach_buffers: super::storage::SharedBuffers::shared(0, 0),
            route_cache: Rc::new(RefCell::new(HashMap::default())),
            occupied_route_cache: Rc::new(RefCell::new(HashMap::default())),
            route_atlas,
            terrain_revision: 0,
            attack_geometry: Rc::new(RefCell::new(HashMap::default())),
            attack_masks: super::geometry::AttackMasks::default(),
            reach_cache: super::movement::ReachCache::default(),
            threat_reach_cache: super::movement::ThreatReachCache::default(),
            movement_masks: super::movement::MovementMasks::default(),
            route_order: Rc::new(RefCell::new(VecDeque::new())),
            occupancy: vec![],
            occupied_tiles: super::geometry::TileSet::new(0),
            unit_indices: UnitIndices::default(),
        };
        #[cfg(feature = "wasm")]
        {
            game.combat_animation = super::playback::Playback::new(
                game.rules
                    .units
                    .iter()
                    .flat_map(|u| u.attacks.iter())
                    .map(|a| usize::from(a.count))
                    .max()
                    .unwrap_or(0)
                    * 2,
            );
        }
        game.roster_changed();
        game.refresh_mission();
        game
    }
    #[cfg(test)]
    pub(crate) fn new(rules: &'catalog Rules, map: &str) -> Self {
        Self::new_source(RuleSource::Borrowed(rules), map)
    }
    pub(super) fn new_source(rules: RuleSource<'catalog>, map: &str) -> Self {
        let state = rules
            .maps
            .iter()
            .filter(|((id, _), _)| id == map)
            .max_by_key(|((_, revision), _)| revision)
            .unwrap()
            .1
            .instantiate();
        Self::from_source(rules, state)
    }
    /// Materialize the protocol snapshot without changing battle state.
    pub(crate) fn state(&self) -> State {
        State {
            armies: self.armies.clone(),
            map_id: self.map_id.clone(),
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: Gold {
                player: self.gold[0],
                enemy: self.gold[1],
            },
            winner: self.winner,
            next_id: self.next_id,
            combat_rng: Some(self.combat_rng),
            log: self.log.clone(),
            tiles: self.tiles(),
            units: self.public_units(),
            campaign: self.campaign.clone(),
            last_combat: self.combat_present.then(|| self.last_combat.clone()),
        }
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
                terrain: self.rules.terrain_names[self.terrain[index]].to_string(),
                owner: self.owners[index],
                connections: self.connections[index].clone(),
                dead_ends: self.dead_ends[index].clone(),
                floor: self.floors[index].clone(),
            })
            .collect()
    }
    pub(crate) fn public_unit(&self, unit: &Unit) -> PublicUnit {
        PublicUnit {
            id: format!("u{}", unit.id),
            r#type: self.rules.units[unit.kind].id.to_string(),
            side: unit.side,
            tile: self.board.ids[unit.tile].clone(),
            hp: unit.hp,
            moved: unit.moved,
            attacked: unit.attacked,
            experience: unit.experience,
            name: unit.name.as_deref().map(str::to_owned),
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
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub(crate) fn unit(&self, id: u32) -> Option<&Unit> {
        self.index(id).map(|index| &self.units[index])
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub(crate) fn at(&self, tile: usize) -> Option<&Unit> {
        self.occupancy
            .get(tile)
            .copied()
            .flatten()
            .and_then(|index| self.units.get(index))
            .filter(|unit| unit.tile == tile)
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
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
    pub(crate) fn terrain_is(&self, tile: usize, terrain: Terrain) -> bool {
        self.rules.terrain_types[self.terrain[tile]] == terrain
    }
    pub(super) fn take_log_buffer(&self) -> String {
        let mut buffer = self.log_spare.borrow_mut().pop().unwrap_or_default();
        buffer.clear();
        buffer
    }
    pub(crate) fn record(&mut self, message: String) {
        if !self.presentation {
            return;
        }
        if self.log.len() == 8 {
            self.log_spare.borrow_mut().push(self.log.pop().unwrap());
        }
        self.log.insert(0, message);
    }
    pub(crate) fn add(&mut self, kind: usize, side: Side, tile: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
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
        self.occupancy.resize(self.terrain.len(), None);
        self.occupancy.fill(None);
        self.occupied_tiles = super::geometry::TileSet::new(self.terrain.len());
        self.unit_indices.clear();
        for (index, unit) in self.units.iter().enumerate() {
            self.occupancy[unit.tile] = Some(index);
            self.occupied_tiles.insert(unit.tile);
            self.unit_indices.insert(unit.id, index);
        }
    }
    /// Change terrain and invalidate only terrain-dependent route results.
    #[cfg(test)]
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        if self.terrain[tile] != terrain {
            Rc::make_mut(&mut self.terrain)[tile] = terrain;
            Rc::make_mut(&mut self.connections)[tile] = None;
            Rc::make_mut(&mut self.dead_ends)[tile] = None;
            self.attack_geometry = Rc::new(RefCell::new(HashMap::default()));
            self.attack_masks = super::geometry::AttackMasks::default();
            self.reach_cache = super::movement::ReachCache::default();
            self.threat_reach_cache = super::movement::ThreatReachCache::default();
            self.movement_masks = super::movement::MovementMasks::default();
            self.terrain_revision = self.terrain_revision.wrapping_add(1);
            self.route_cache = Rc::new(RefCell::new(HashMap::default()));
            self.occupied_route_cache = Rc::new(RefCell::new(HashMap::default()));
            self.route_atlas = Rc::new(RefCell::new(super::movement::RouteAtlas::new(
                &self.rules,
                &self.terrain,
                &self.board,
            )));
            self.route_order = Rc::new(RefCell::new(VecDeque::new()));
        }
    }

    pub(crate) fn fallen(&self, name: &str) -> bool {
        self.campaign.as_ref().is_some_and(|state| {
            state
                .fallen_companions
                .as_ref()
                .is_some_and(|names| names.iter().any(|n| n == name))
                || state.marked_prefixed("fallen:", name)
        })
    }
    pub(crate) fn refresh_mission(&mut self) {
        let Some(state) = &self.campaign else {
            self.mission = None;
            self.mission_revision = None;
            return;
        };
        let definition = &self.rules.campaigns[&state.id];
        // Engine progress lists are append-only within a chapter. Restore and
        // chapter transitions construct a new Game, so these lengths identify
        // every change that can affect the resolved mission.
        let revision = [
            definition as *const Campaign as usize,
            state.mission,
            state.marks.len(),
            state.choices.as_ref().map_or(0, Vec::len),
            state.fallen_companions.as_ref().map_or(0, Vec::len),
            state.escalated() as usize,
        ];
        if self.mission_revision == Some(revision) {
            return;
        }
        self.mission_revision = Some(revision);
        let without_dain = state.chosen("refuge-war")
            || state.chosen("green-dain-dead")
            || state.marked("dain-fell");
        let base = &definition.missions[state.mission];
        let mut losses = smallvec::SmallVec::new();
        if let Some(patches) = &base.companion_losses {
            let original = state.fallen_companions.as_deref().unwrap_or_default();
            for name in original.iter().map(String::as_str).chain(
                state.marks.iter().enumerate().filter_map(|(index, mark)| {
                    let name = mark.strip_prefix("fallen:")?;
                    (!original.iter().any(|n| n == name)
                        && !state.marks[..index].iter().any(|m| m == mark))
                    .then_some(name)
                }),
            ) {
                if let Some(index) = patches.keys().position(|key| key.as_ref() == name) {
                    losses.push(index);
                }
            }
        }
        let key = MissionKey {
            campaign: revision[0],
            stage: state.mission,
            without_dain,
            escalated: state.escalated(),
            losses,
        };
        if let Some((_, mission)) = self
            .mission_cache
            .borrow()
            .iter()
            .find(|(cached, _)| cached == &key)
        {
            self.mission = Some(mission.clone());
            return;
        }
        let mut resolved = base.clone();
        if without_dain {
            if let Some(patch) = &base.without_dain {
                resolved.apply(patch);
            }
        }
        if let Some(patches) = &base.companion_losses {
            for &index in &key.losses {
                resolved.apply(patches.values().nth(index).expect("mission loss patch"));
            }
        }
        if key.escalated {
            if let Some(refuge) = resolved.refuge.clone() {
                resolved.briefing = refuge.war_briefing;
                resolved.hints = refuge.war_hints;
            }
        }
        let mission = Rc::new(resolved);
        let mut cache = self.mission_cache.borrow_mut();
        if cache.len() == 16 {
            cache.remove(0);
        }
        cache.push((key, mission.clone()));
        self.mission = Some(mission);
    }
}

impl<'catalog> Game<'catalog> {
    pub(crate) fn planning_fork(&self) -> Self {
        let mut draft = self.forecast_fork();
        draft.combat_rng = 0x6d2b79f5;
        draft
    }
    #[cfg(test)]
    pub(crate) fn fork(&self) -> Self {
        self.fork_mode(self.presentation)
    }
    pub(crate) fn forecast_fork(&self) -> Self {
        self.fork_mode(false)
    }
    fn fork_mode(&self, presentation: bool) -> Self {
        let mut storage = self.fork_pool.borrow_mut().pop().unwrap_or_default();
        storage.copy(self, presentation);
        Self {
            forked: true,
            fork_pool: self.fork_pool.clone(),
            rules: self.rules.clone(),
            attack_classes: self.attack_classes.clone(),
            board: self.board.clone(),
            terrain: self.terrain.clone(),
            owners: storage.owners,
            connections: self.connections.clone(),
            dead_ends: self.dead_ends.clone(),
            floors: self.floors.clone(),
            units: storage.units,
            map_id: storage.map_id,
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold,
            winner: self.winner,
            next_id: self.next_id,
            combat_rng: self.combat_rng,
            log: storage.log,
            log_spare: RefCell::new(Vec::new()),
            presentation,
            marker_spare: storage.marker_spare,
            campaign: storage.campaign,
            last_combat: if presentation {
                self.last_combat.clone()
            } else {
                None
            },
            last_combat_spare: None,
            #[cfg(feature = "wasm")]
            combat_animation: super::playback::Playback::new(0),
            combat_present: presentation && self.combat_present,
            armies: storage.armies,
            pools: self.pools.clone(),
            arena_first: self.arena_first,
            mission: self.mission.clone(),
            mission_revision: self.mission_revision,
            mission_cache: self.mission_cache.clone(),
            #[cfg(not(target_arch = "wasm32"))]
            recordings: None,
            route_buffers: self.route_buffers.clone(),
            reach_buffers: self.reach_buffers.clone(),
            route_cache: self.route_cache.clone(),
            occupied_route_cache: self.occupied_route_cache.clone(),
            route_atlas: self.route_atlas.clone(),
            terrain_revision: self.terrain_revision,
            attack_geometry: self.attack_geometry.clone(),
            attack_masks: self.attack_masks.clone(),
            reach_cache: self.reach_cache.clone(),
            threat_reach_cache: self.threat_reach_cache.clone(),
            movement_masks: self.movement_masks.clone(),
            route_order: self.route_order.clone(),
            occupancy: storage.occupancy,
            occupied_tiles: self.occupied_tiles.clone(),
            unit_indices: storage.indices,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn action(&mut self, value: ActionRecord) {
        if let Some(records) = &mut self.recordings {
            records.actions.push(value);
        }
    }
    pub(crate) fn promotion_options(&self, unit: &Unit) -> smallvec::SmallVec<[usize; 4]> {
        if self.winner.is_some() || unit.side != self.turn || unit.moved || unit.attacked {
            return smallvec::SmallVec::new();
        }
        let rule = &self.rules.units[unit.kind];
        if rule
            .experience_needed
            .is_none_or(|needed| unit.experience.unwrap_or(0) < needed)
        {
            return smallvec::SmallVec::new();
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
        let name = self.presentation.then(|| {
            self.units[index]
                .name
                .clone()
                .unwrap_or_else(|| previous.name.as_ref().into())
        });
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
        game_log!(
            self,
            format!(
                "{} advanced to {}.",
                name.as_ref().expect("live promotion"),
                self.rules.units[kind].name
            )
        );
        #[cfg(not(target_arch = "wasm32"))]
        game_action!(
            self,
            ActionRecord::Advance {
                unit: format!("u{id}"),
                unit_type: self.rules.units[kind].id.to_string(),
            }
        );
    }

    pub(crate) fn recruitment_options(&self, side: Side) -> smallvec::SmallVec<[usize; 32]> {
        if let Some(pools) = &self.pools {
            return pools[side.index()].iter().copied().collect();
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
            return if side == Side::Player {
                &mission.recruits
            } else {
                &mission.enemy_recruits
            }
            .iter()
            .map(|name| self.kind(name))
            .collect();
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
    pub(crate) fn placement_tiles(
        &self,
        leader: &Unit,
        kind: usize,
        price: bool,
    ) -> smallvec::SmallVec<[usize; 8]> {
        if !self.rules.units[leader.kind].leader
            || self.rules.units[kind].leader
            || !self.recruitment_options(leader.side).contains(&kind)
            || self.winner.is_some()
            || leader.side != self.turn
            || !self.terrain_is(leader.tile, Terrain::Keep)
            || self.owners[leader.tile] != Some(leader.side)
            || (price && self.gold[leader.side.index()] < self.rules.units[kind].cost)
        {
            return smallvec::SmallVec::new();
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
        self.gold[leader.side.index()] -= self.rules.units[kind].cost;
        self.add(kind, leader.side, tile);
        let unit = self.units.last_mut().unwrap();
        unit.moved = true;
        unit.attacked = true;
        game_log!(
            self,
            format!(
                "{} recruited a {}. It acts next turn.",
                if leader.side == Side::Player {
                    "You"
                } else {
                    "Enemy"
                },
                self.rules.unit_log_names[kind]
            )
        );
        #[cfg(not(target_arch = "wasm32"))]
        game_action!(
            self,
            ActionRecord::Recruit {
                unit: format!("u{id}"),
                unit_type: self.rules.units[kind].id.to_string(),
                tile: self.board.ids[tile].clone(),
            }
        );
    }
}

impl Drop for Game<'_> {
    fn drop(&mut self) {
        if self.forked {
            let pool = self.fork_pool.clone();
            pool.borrow_mut().push(super::forks::Storage::take(self));
        }
    }
}
