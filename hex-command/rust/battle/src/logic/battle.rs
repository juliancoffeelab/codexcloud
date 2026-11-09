//! Compact battle sessions borrowing catalog rules and indexing the live roster.
use super::event_state::{CompiledMission, EventProgress};

use crate::{data::*, model::StateFrame};
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub struct Board {
    pub ids: Vec<Hex>,
    pub locations: Vec<Option<String>>,
    pub positions: Vec<(i32, i32)>,
    pub indices: HashMap<Hex, usize>,
    named_locations: HashMap<String, usize>,
    pub adjacent: Vec<Vec<usize>>,
    pub adjacent_bits: Vec<u128>,
    pub distances: Vec<u16>,
}
impl Board {
    pub fn new(tiles: &[Tile]) -> Rc<Self> {
        let ids: Vec<Hex> = tiles.iter().map(Tile::hex).collect();
        let positions: Vec<_> = tiles.iter().map(|tile| (tile.col, tile.row)).collect();
        let locations = tiles.iter().map(|tile| tile.location.clone()).collect();
        let indices = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.clone(), index))
            .collect();
        let named_locations = tiles
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| tile.location.as_ref().map(|name| (name.clone(), index)))
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
            locations,
            positions,
            indices,
            named_locations,
            adjacent,
            adjacent_bits,
            distances,
        })
    }
    /// Resolve a source location reference; command lookup uses coordinate indices only.
    #[inline]
    pub(crate) fn resolve(&self, reference: &TileRef) -> Option<usize> {
        match reference {
            TileRef::Hex(hex) => self.indices.get(hex).copied(),
            TileRef::Location(name) => self.named_locations.get(name).copied(),
            TileRef::Binding(_) => None,
        }
    }
    pub fn distance(&self, first: usize, second: usize) -> u16 {
        self.distances[first * self.ids.len() + second]
    }
}
pub fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
// A generated layout is reused by many policy/race pairings. These caches
// depend on catalog rules, geometry and terrain, never on a game's mutable state.
type TerrainKey = (String, Vec<(String, Option<String>, i32, i32, String)>);
#[derive(Clone)]
struct SharedTerrain {
    board: Rc<Board>,
    routes: Rc<RefCell<HashMap<super::movement::RouteKey, Rc<Vec<u16>>>>>,
    route_order: Rc<RefCell<VecDeque<super::movement::RouteKey>>>,
    attacks: super::geometry::AttackGeometry,
    masks: super::geometry::AttackMasks,
    reach: super::movement::ReachCache,
    threat_reach: super::movement::ThreatReachCache,
    movement: super::movement::MovementMasks,
    regions: super::movement::RegionCache,
    origin_runs: super::geometry::OriginRunCache,
}
thread_local! {
    static TERRAIN_POOL: RefCell<(HashMap<TerrainKey, SharedTerrain>, VecDeque<TerrainKey>)> = RefCell::new((HashMap::default(), VecDeque::new()));
}
#[derive(Clone, Debug)]
struct MissionKey {
    definition: CampaignRules,
    mission: usize,
}
impl PartialEq for MissionKey {
    fn eq(&self, other: &Self) -> bool {
        self.mission == other.mission && self.definition.same_definition(&other.definition)
    }
}
impl Eq for MissionKey {}
impl std::hash::Hash for MissionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.definition.identity(), state);
        std::hash::Hash::hash(&self.mission, state);
    }
}
thread_local! {
    // Strong definition ownership prevents address reuse from aliasing cached content.
    static MISSION_POOL: RefCell<HashMap<MissionKey, Rc<CompiledMission>>> = RefCell::default();
}
/// Derived data for the current battle revision; refreshed after successful commands.
#[derive(Clone, Debug)]
struct TileGeometry {
    connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    floors: Rc<Vec<Option<String>>>,
    fog: Rc<Vec<Option<std::sync::Arc<ConditionRule>>>>,
    light: Rc<Vec<Option<std::sync::Arc<TileLight>>>>,
}

/// Immutable planning facts are shared; a mutation detaches only its branch.
#[derive(Clone, Debug)]
pub(crate) struct Shared<T: Clone>(Rc<T>);
impl<T: Clone> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
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
/// Owned live data; no catalog borrows and no wire representation.
#[derive(Debug)]
pub struct BattleState {
    pub(crate) scope: crate::BattleScope,
    pub(crate) factions: Rc<Vec<Faction>>,
    /// Turn-local controller scope, never persisted.
    pub(crate) active_faction: Option<String>,
    pub(crate) board: Rc<Board>,
    pub(crate) terrain: Vec<usize>,
    pub(crate) owners: Shared<Vec<Option<String>>>,
    pub(crate) connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) fog: Rc<Vec<Option<std::sync::Arc<ConditionRule>>>>,
    pub(crate) light: Rc<Vec<Option<std::sync::Arc<TileLight>>>>,
    pub(crate) condition_memory: Rc<Vec<String>>,
    /// Derived tile-indexed remembered fog; never serialized.
    pub(crate) fog_memory: Vec<u64>,
    pub(super) traversal: Option<(u32, Vec<usize>)>,
    pub(crate) event_deaths: Vec<super::events::DeathOccurrence>,
    pub(super) event_departure: Option<Unit>,
    pub(super) event_opening: bool,
    pub(crate) floors: Rc<Vec<Option<String>>>,
    pub(crate) units: Vec<Unit>,
    pub(crate) ground_items: Vec<GroundItemState>,
    pub(crate) names: std::sync::Arc<Names>,
    pub(crate) inventories: std::sync::Arc<Inventories>,
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
    pub(crate) scripts: Option<Shared<EventProgress>>,
    pub(crate) last_combat: Option<LastCombat>,
    #[cfg(feature = "wasm")]
    pub(crate) combat_animation: Option<crate::model::CombatAnimation>,
    pub(crate) combat_present: bool,
    pub(crate) armies: Option<[String; 2]>,
    pub(crate) pools: Option<[Vec<usize>; 2]>,
    pub(crate) arena_first: Option<Side>,
    pub(crate) mission: Option<Rc<CompiledMission>>,
    /// The selected immutable definition; absent only for a non-campaign game.
    pub(super) campaign_definition: Option<CampaignRules>,
    mission_key: Option<MissionKey>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) recordings: Option<Records>,
    pub(super) route_cache: Rc<RefCell<HashMap<super::movement::RouteKey, Rc<Vec<u16>>>>>,
    pub(super) terrain_revision: u64,
    pub(super) attack_geometry: super::geometry::AttackGeometry,
    pub(super) attack_masks: super::geometry::AttackMasks,
    pub(super) reach_cache: super::movement::ReachCache,
    pub(super) threat_reach_cache: super::movement::ThreatReachCache,
    pub(super) movement_masks: super::movement::MovementMasks,
    pub(super) movement_regions: super::movement::RegionCache,
    pub(super) origin_runs: super::geometry::OriginRunCache,
    pub(super) route_order: Rc<RefCell<VecDeque<super::movement::RouteKey>>>,
    pub(super) occupancy: Vec<Option<usize>>,
    pub(crate) occupied_tiles: super::geometry::TileSet,
    pub(super) unit_indices: HashMap<u32, usize>,
}
#[derive(Debug)]
pub(crate) struct Game<'catalog> {
    pub(crate) rules: &'catalog Rules,
    pub(crate) mechanics: &'catalog UnitMechanics,
    pub(crate) runtime: BattleState,
}
impl std::ops::Deref for Game<'_> {
    type Target = BattleState;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
impl std::ops::DerefMut for Game<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}
impl<'catalog> Game<'catalog> {
    #[inline]
    pub(crate) fn name(&self, key: Option<NameKey>) -> Option<&str> {
        self.runtime.names.resolve(key)
    }
    pub(crate) fn intern_name(&mut self, value: &str) -> NameKey {
        if let Some(key) = self.runtime.names.find(value) {
            key
        } else {
            std::sync::Arc::make_mut(&mut self.runtime.names).intern(value.to_owned())
        }
    }
    #[inline]
    pub(crate) fn inventory(&self, unit: &Unit) -> impl ExactSizeIterator<Item = &Item> + Clone {
        self.runtime.inventories.items(unit.inventory)
    }
    pub(crate) fn set_inventory(&mut self, id: u32, items: Vec<Item>) {
        let inventory =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).load(items, self.mechanics);
        let index = self.index(id).unwrap();
        self.runtime.units[index].inventory = inventory;
    }
    pub(crate) fn give_item(&mut self, id: u32, item: Item) {
        let item =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).add_item(item, self.mechanics);
        self.give_existing_item(id, item);
    }
    pub(crate) fn give_existing_item(&mut self, id: u32, item: ItemId) {
        let index = self.index(id).unwrap();
        let old = self.runtime.units[index].inventory;
        let inventory =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).edit(old, None, Some(item));
        self.runtime.units[index].inventory = inventory;
    }
    pub(crate) fn drop_item(&mut self, id: u32, slot: usize) -> ItemId {
        let index = self.index(id).unwrap();
        let old = self.runtime.units[index].inventory;
        let item = self.runtime.inventories.item_ids(old)[slot];
        let inventory =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).edit(old, Some(slot), None);
        self.runtime.units[index].inventory = inventory;
        item
    }
    #[inline]
    pub(crate) fn source_weapon(&self, weapon: &WeaponRules) -> &Weapon {
        if weapon.source < self.mechanics.weapons.len() {
            let (kind, slot) = self.mechanics.weapon_sources[weapon.source];
            &self.rules.units[kind].attacks[slot]
        } else {
            self.runtime
                .inventories
                .source(weapon.source, self.mechanics)
        }
    }
    #[inline]
    pub(crate) fn weapon_id(&self, key: WeaponKey) -> String {
        self.mechanics.weapon_id(key)
    }
    pub(crate) fn from_state(rules: &'catalog Rules, state: State) -> Self {
        let definition = state
            .campaign
            .as_ref()
            .map(|state| &rules.campaigns[&state.id]);
        Self::from_state_with_campaign(rules, state, definition)
    }
    pub(crate) fn from_state_with_campaign(
        rules: &'catalog Rules,
        state: State,
        definition: Option<&CampaignRules>,
    ) -> Self {
        let key = (
            rules.content_fingerprint.clone(),
            state
                .tiles
                .iter()
                .map(|t| {
                    (
                        t.id.clone(),
                        t.location.clone(),
                        t.col,
                        t.row,
                        t.terrain.clone(),
                    )
                })
                .collect(),
        );
        let cached = TERRAIN_POOL.with_borrow(|(pool, _)| pool.get(&key).cloned());
        let board = cached
            .as_ref()
            .map_or_else(|| Board::new(&state.tiles), |c| c.board.clone());
        let mut game = Self::from_state_with_board_and_campaign(rules, state, board, definition);
        if let Some(cached) = cached {
            game.route_cache = cached.routes;
            game.route_order = cached.route_order;
            game.attack_geometry = cached.attacks;
            game.attack_masks = cached.masks;
            game.reach_cache = cached.reach;
            game.threat_reach_cache = cached.threat_reach;
            game.movement_masks = cached.movement;
            game.movement_regions = cached.regions;
            game.origin_runs = cached.origin_runs;
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
                        regions: game.movement_regions.clone(),
                        origin_runs: game.origin_runs.clone(),
                    },
                );
            });
        }
        game
    }

    fn from_state_with_board_and_campaign(
        rules: &'catalog Rules,
        state: State,
        board: Rc<Board>,
        definition: Option<&CampaignRules>,
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
        let mut light = Vec::with_capacity(n);
        for tile in tiles {
            terrain.push(rules.terrains[&tile.terrain]);
            owners.push(tile.owner);
            connections.push(tile.connections);
            walls.push(tile.walls);
            dead_ends.push(tile.dead_ends);
            floors.push(tile.floor);
            fog.push(tile.fog);
            light.push(tile.light);
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
            light: Rc::new(light),
        };
        Self::from_parts(rules, state, board, definition, terrain, owners, geometry)
    }

    fn from_parts(
        rules: &'catalog Rules,
        mut state: StateFrame,
        board: Rc<Board>,
        campaign_definition: Option<&CampaignRules>,
        terrain: Vec<usize>,
        owners: Vec<Option<String>>,
        geometry: TileGeometry,
    ) -> Self {
        let saved_scripts = state
            .campaign
            .as_mut()
            .and_then(|campaign| campaign.scripts.take());
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
                    .and_then(|id| id.split_once(','))
                    .and_then(|(col, row)| Some(Hex::new(col.parse().ok()?, row.parse().ok()?)))
                    .and_then(|hex| board.indices.get(&hex))
                {
                    fog_memory[i / 64] |= 1u64 << (i % 64);
                }
            }
        }
        let mechanics = rules.units.mechanics();
        let mut names = Names::default();
        let mut inventories = Inventories::default();
        let units: Vec<Unit> = state
            .units
            .into_iter()
            .map(|unit| Unit {
                id: unit.id.strip_prefix('u').unwrap().parse().unwrap(),
                kind: rules.types[&unit.r#type],
                side: unit.side,
                tile: board.indices[&unit.tile],
                hp: unit.hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                role: unit.role,
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
                faction: unit.faction.map(|value| names.intern(value)),
                ai_group: unit.ai_group.map(|value| names.intern(value)),
                character: unit.character.map(|value| names.intern(value)),
                inventory: inventories.load(unit.items, mechanics),
            })
            .collect();
        let ground_items = state
            .ground_items
            .into_iter()
            .map(|ground| GroundItemState {
                tile: board.indices[&ground.tile],
                item: inventories.add_item(ground.item, mechanics),
            })
            .collect();
        let combat_present = state.last_combat.is_some();
        let last_combat = state.last_combat.flatten();
        let mut game = Self {
            rules,
            mechanics,
            runtime: BattleState {
                scope: crate::BattleScope::new(),
                factions: Rc::new(state.factions),
                ground_items,
                names: std::sync::Arc::new(names),
                inventories: std::sync::Arc::new(inventories),
                active_faction: None,
                fog: geometry.fog,
                light: geometry.light,
                condition_memory: Rc::new(state.condition_memory),
                fog_memory,
                traversal: None,
                event_deaths: vec![],
                event_departure: None,
                event_opening: false,
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
                combat_rng: state.combat_rng,
                log: state.log,
                campaign: state.campaign.map(Into::into),
                scripts: None,
                last_combat,
                #[cfg(feature = "wasm")]
                combat_animation: None,
                combat_present,
                armies: state.armies,
                pools: None,
                arena_first: None,
                mission: None,
                campaign_definition: campaign_definition.cloned(),
                mission_key: None,
                #[cfg(not(target_arch = "wasm32"))]
                recordings: None,
                route_cache: Rc::new(RefCell::new(HashMap::default())),
                terrain_revision: 0,
                attack_geometry: Rc::new(RefCell::new(HashMap::default())),
                attack_masks: super::geometry::AttackMasks::default(),
                reach_cache: super::movement::ReachCache::default(),
                threat_reach_cache: super::movement::ThreatReachCache::default(),
                movement_masks: super::movement::MovementMasks::default(),
                movement_regions: super::movement::RegionCache::default(),
                origin_runs: super::geometry::OriginRunCache::default(),
                route_order: Rc::new(RefCell::new(VecDeque::new())),
                occupancy: vec![],
                occupied_tiles: super::geometry::TileSet::new(0),
                unit_indices: HashMap::default(),
            },
        };
        game.roster_changed();
        game.refresh_mission();
        if let Some(saved) = saved_scripts {
            game.scripts = Some(
                EventProgress::restore(
                    game.mission.as_ref().expect("restored mission").clone(),
                    saved,
                )
                .into(),
            );
        }
        game.remember_conditions();
        game
    }
    #[cfg(test)]
    pub(crate) fn new(rules: &'catalog Rules, map: &str) -> Self {
        Self::new_with_campaign(rules, map, None)
    }
    pub(crate) fn new_with_campaign(
        rules: &'catalog Rules,
        map: &str,
        definition: Option<&CampaignRules>,
    ) -> Self {
        let state = rules
            .maps
            .iter()
            .filter(|((id, _), _)| id == map)
            .max_by_key(|((_, revision), _)| revision)
            .unwrap()
            .1
            .clone();
        Self::from_state_with_campaign(rules, state, definition)
    }
    pub(crate) fn campaign_snapshot(&self) -> Option<CampaignState> {
        self.runtime
            .campaign
            .as_deref()
            .cloned()
            .map(|mut campaign| {
                campaign.scripts = self
                    .runtime
                    .scripts
                    .as_deref()
                    .map(|state| state.saved().clone());
                campaign
            })
    }
    /// Materialize the protocol snapshot without changing battle state.
    pub(crate) fn state(&self) -> State {
        self.state_with_tiles(self.tiles())
    }
    pub(crate) fn state_with_tiles(&self, tiles: Vec<Tile>) -> State {
        State {
            factions: self.runtime.factions.as_ref().clone(),
            ground_items: self
                .runtime
                .ground_items
                .iter()
                .map(|ground| GroundItem {
                    tile: self.runtime.board.ids[ground.tile].clone(),
                    item: self.runtime.inventories.item(ground.item).clone(),
                })
                .collect(),
            condition_memory: self.runtime.condition_memory.as_ref().clone(),
            armies: self.runtime.armies.clone(),
            map_id: self.runtime.map_id.clone(),
            map_revision: self.runtime.map_revision,
            cols: self.runtime.cols,
            rows: self.runtime.rows,
            round: self.runtime.round,
            turn: self.runtime.turn,
            gold: self.runtime.gold.clone(),
            winner: self.runtime.winner,
            next_id: self.runtime.next_id,
            combat_rng: self.runtime.combat_rng,
            log: self.runtime.log.clone(),
            tiles,
            units: self.public_units(),
            campaign: self.campaign_snapshot(),
            last_combat: self
                .runtime
                .combat_present
                .then(|| self.runtime.last_combat.clone()),
        }
    }

    /// Patch the authoritative terrain only where current runtime values differ.
    #[cfg(feature = "wasm")]
    pub(crate) fn patch_tiles(&self, tiles: &mut [Tile]) -> Vec<usize> {
        debug_assert_eq!(tiles.len(), self.runtime.board.ids.len());
        let mut changed = Vec::new();
        for (i, tile) in tiles.iter_mut().enumerate() {
            let Tile {
                id: _,
                location: _,
                col,
                row,
                terrain,
                owner,
                connections,
                dead_ends,
                walls,
                floor,
                fog,
                light,
            } = tile;
            let position = self.runtime.board.positions[i];
            let kind = &self.rules.terrain_names[self.runtime.terrain[i]];
            if *col != position.0
                || *row != position.1
                || terrain != kind
                || *owner != self.runtime.owners[i]
                || *connections != self.runtime.connections[i]
                || *dead_ends != self.runtime.dead_ends[i]
                || *walls != self.runtime.walls[i]
                || *floor != self.runtime.floors[i]
                || *fog != self.runtime.fog[i]
                || *light != self.runtime.light[i]
            {
                *col = position.0;
                *row = position.1;
                terrain.clone_from(kind);
                owner.clone_from(&self.runtime.owners[i]);
                connections.clone_from(&self.runtime.connections[i]);
                dead_ends.clone_from(&self.runtime.dead_ends[i]);
                walls.clone_from(&self.runtime.walls[i]);
                floor.clone_from(&self.runtime.floors[i]);
                fog.clone_from(&self.runtime.fog[i]);
                light.clone_from(&self.runtime.light[i]);
                changed.push(i);
            }
        }
        changed
    }
    pub(crate) fn tiles(&self) -> Vec<Tile> {
        self.runtime
            .board
            .ids
            .iter()
            .enumerate()
            .map(|(index, id)| Tile {
                id: id.to_string(),
                location: self.runtime.board.locations[index].clone(),
                col: self.runtime.board.positions[index].0,
                row: self.runtime.board.positions[index].1,
                terrain: self.rules.terrain_names[self.runtime.terrain[index]].clone(),
                owner: self.runtime.owners[index].clone(),
                connections: self.runtime.connections[index].clone(),
                walls: self.runtime.walls[index].clone(),
                dead_ends: self.runtime.dead_ends[index].clone(),
                floor: self.runtime.floors[index].clone(),
                fog: self.runtime.fog[index].clone(),
                light: self.runtime.light[index].clone(),
            })
            .collect()
    }
    pub(crate) fn public_unit(&self, unit: &Unit) -> PublicUnit {
        PublicUnit {
            faction: self.name(unit.faction).map(str::to_owned),
            items: self.inventory(unit).cloned().collect(),
            ai_group: self.name(unit.ai_group).map(str::to_owned),
            id: format!("u{}", unit.id),
            r#type: self.rules.units[unit.kind].id.clone(),
            side: unit.side,
            tile: self.runtime.board.ids[unit.tile].clone(),
            hp: unit.hp,
            moved: unit.moved,
            attacked: unit.attacked,
            experience: unit.experience,
            character: self.name(unit.character).map(str::to_owned),
            role: unit.role.clone(),
            retreating: unit.retreating,
            rebuilding: unit.rebuilding,
        }
    }
    pub(crate) fn public_units(&self) -> Vec<PublicUnit> {
        self.runtime
            .units
            .iter()
            .map(|unit| self.public_unit(unit))
            .collect()
    }
    #[inline]
    pub(crate) fn unit(&self, id: u32) -> Option<&Unit> {
        self.index(id).map(|index| &self.runtime.units[index])
    }
    #[inline]
    pub(crate) fn at(&self, tile: usize) -> Option<&Unit> {
        self.runtime
            .occupancy
            .get(tile)
            .copied()
            .flatten()
            .and_then(|index| self.runtime.units.get(index))
            .filter(|unit| unit.tile == tile)
    }
    #[inline]
    pub(crate) fn index(&self, id: u32) -> Option<usize> {
        self.runtime
            .unit_indices
            .get(&id)
            .copied()
            .filter(|&index| self.runtime.units.get(index).is_some_and(|u| u.id == id))
    }
    #[cfg(test)]
    pub(crate) fn site_at(&self, col: i32, row: i32) -> usize {
        self.runtime
            .board
            .positions
            .iter()
            .position(|p| *p == (col, row))
            .unwrap()
    }
    #[inline]
    pub(crate) fn site(&self, id: &Hex) -> usize {
        *self.runtime.board.indices.get(id).expect("validated hex")
    }
    pub(crate) fn character_name(&self, id: &str) -> Option<&str> {
        self.campaign_definition
            .as_ref()
            .or_else(|| self.rules.campaign_for_map(&self.runtime.map_id))
            .and_then(|campaign| campaign.character(id))
            .map(|c| c.name.as_str())
    }
    pub(crate) fn unit_name<'a>(&'a self, unit: &'a Unit) -> &'a str {
        self.name(unit.character)
            .and_then(|id| self.character_name(id))
            .unwrap_or(&self.rules.units[unit.kind].name)
    }
    #[inline]
    pub(crate) fn kind(&self, name: &str) -> usize {
        self.rules.types[name]
    }
    #[inline]
    pub(crate) fn terrain_role(&self, tile: usize) -> TerrainRole {
        self.rules.terrain_roles[self.runtime.terrain[tile]]
    }
    #[inline]
    pub(crate) fn role_is(&self, tile: usize, role: TerrainRole) -> bool {
        self.terrain_role(tile) == role
    }
    /// Capability is authored once and exported through the catalog for UI discovery too.
    #[inline]
    pub(crate) fn is_keep(&self, tile: usize) -> bool {
        self.role_is(tile, TerrainRole::Keep)
    }
    pub(crate) fn record(&mut self, message: String) {
        self.runtime.log.insert(0, message);
        self.runtime.log.truncate(8);
    }
    pub(crate) fn add(&mut self, kind: usize, side: Side, tile: usize) -> u32 {
        let id = self.runtime.next_id;
        self.runtime.next_id += 1;
        self.runtime.units.push(Unit {
            id,
            kind,
            side,
            tile,
            hp: self.rules.units[kind].hp,
            moved: false,
            attacked: false,
            experience: Some(0),
            role: None,
            retreating: None,
            rebuilding: None,
            faction: None,
            ai_group: None,
            character: None,
            inventory: InventoryId(0),
        });
        self.runtime
            .unit_indices
            .insert(id, self.runtime.units.len() - 1);
        self.runtime.occupancy[tile] = Some(self.runtime.units.len() - 1);
        self.runtime.occupied_tiles.insert(tile);
        id
    }
    /// Refresh dense lookup tables after an internal roster transformation.
    pub(crate) fn roster_changed(&mut self) {
        self.runtime.occupancy = vec![None; self.runtime.terrain.len()];
        self.runtime.occupied_tiles = super::geometry::TileSet::new(self.runtime.terrain.len());
        self.runtime.unit_indices.clear();
        for (index, unit) in self.runtime.units.iter().enumerate() {
            self.runtime.occupancy[unit.tile] = Some(index);
            self.runtime.occupied_tiles.insert(unit.tile);
            self.runtime.unit_indices.insert(unit.id, index);
        }
    }
    /// Change terrain and invalidate only terrain-dependent route results.
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        if self.runtime.terrain[tile] != terrain {
            self.runtime.terrain[tile] = terrain;
            Rc::make_mut(&mut self.runtime.connections)[tile] = None;
            Rc::make_mut(&mut self.runtime.walls)[tile] = None;
            Rc::make_mut(&mut self.runtime.dead_ends)[tile] = None;
            self.runtime.attack_geometry = Rc::new(RefCell::new(HashMap::default()));
            self.runtime.attack_masks = super::geometry::AttackMasks::default();
            self.runtime.reach_cache = super::movement::ReachCache::default();
            self.runtime.threat_reach_cache = super::movement::ThreatReachCache::default();
            self.runtime.movement_masks = super::movement::MovementMasks::default();
            self.runtime.movement_regions = super::movement::RegionCache::default();
            self.runtime.origin_runs = super::geometry::OriginRunCache::default();
            self.runtime.terrain_revision = self.runtime.terrain_revision.wrapping_add(1);
            self.runtime.route_cache = Rc::new(RefCell::new(HashMap::default()));
            self.runtime.route_order = Rc::new(RefCell::new(VecDeque::new()));
        }
    }

    pub(crate) fn campaign_definition(&self) -> &CampaignRules {
        self.campaign_definition
            .as_ref()
            .expect("campaign game has a definition")
    }
    pub(crate) fn refresh_mission(&mut self) {
        let Some(state) = &self.runtime.campaign else {
            self.runtime.mission = None;
            self.mission_key = None;
            return;
        };
        let definition = self.campaign_definition();
        if self.mission_key.as_ref().is_some_and(|key| {
            key.mission == state.mission && key.definition.same_definition(definition)
        }) {
            return;
        }
        let key = MissionKey {
            definition: definition.clone(),
            mission: state.mission,
        };
        let mission = MISSION_POOL.with_borrow_mut(|pool| {
            if let Some(mission) = pool.get(&key) {
                return mission.clone();
            }
            let mission = Rc::new(CompiledMission::new(
                definition.missions[state.mission].clone(),
            ));
            if pool.len() >= 128 {
                pool.clear();
            }
            pool.insert(key.clone(), mission.clone());
            mission
        });
        self.mission_key = Some(key);
        self.runtime.mission = Some(mission);
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
            rules: self.rules,
            mechanics: self.mechanics,
            runtime: BattleState {
                scope: self.runtime.scope.clone(),
                factions: self.runtime.factions.clone(),
                ground_items: self.runtime.ground_items.clone(),
                names: self.runtime.names.clone(),
                inventories: self.runtime.inventories.clone(),
                active_faction: self.runtime.active_faction.clone(),
                condition_memory: self.runtime.condition_memory.clone(),
                fog_memory: self.runtime.fog_memory.clone(),
                traversal: None,
                event_deaths: vec![],
                event_departure: None,
                event_opening: false,
                fog: self.runtime.fog.clone(),
                light: self.runtime.light.clone(),
                board: self.runtime.board.clone(),
                terrain: self.runtime.terrain.clone(),
                owners: self.runtime.owners.clone(),
                connections: self.runtime.connections.clone(),
                walls: self.runtime.walls.clone(),
                dead_ends: self.runtime.dead_ends.clone(),
                floors: self.runtime.floors.clone(),
                units: self.runtime.units.clone(),
                map_id: self.runtime.map_id.clone(),
                map_revision: self.runtime.map_revision,
                cols: self.runtime.cols,
                rows: self.runtime.rows,
                round: self.runtime.round,
                turn: self.runtime.turn,
                gold: self.runtime.gold.clone(),
                winner: self.runtime.winner,
                next_id: self.runtime.next_id,
                combat_rng: self.runtime.combat_rng,
                log: self.runtime.log.clone(),
                campaign: self.runtime.campaign.clone(),
                scripts: self.runtime.scripts.clone(),
                last_combat: self.runtime.last_combat.clone(),
                #[cfg(feature = "wasm")]
                combat_animation: None,
                combat_present: self.runtime.combat_present,
                armies: self.runtime.armies.clone(),
                pools: self.runtime.pools.clone(),
                arena_first: self.runtime.arena_first,
                mission: self.runtime.mission.clone(),
                campaign_definition: self.campaign_definition.clone(),
                mission_key: self.mission_key.clone(),
                #[cfg(not(target_arch = "wasm32"))]
                recordings: None,
                route_cache: self.runtime.route_cache.clone(),
                terrain_revision: self.runtime.terrain_revision,
                attack_geometry: self.runtime.attack_geometry.clone(),
                attack_masks: self.runtime.attack_masks.clone(),
                reach_cache: self.runtime.reach_cache.clone(),
                threat_reach_cache: self.runtime.threat_reach_cache.clone(),
                movement_masks: self.runtime.movement_masks.clone(),
                movement_regions: self.runtime.movement_regions.clone(),
                origin_runs: self.runtime.origin_runs.clone(),
                route_order: self.runtime.route_order.clone(),
                occupancy: self.runtime.occupancy.clone(),
                occupied_tiles: self.runtime.occupied_tiles.clone(),
                unit_indices: self.runtime.unit_indices.clone(),
            },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn action(&mut self, value: ActionRecord) {
        if let Some(records) = &mut self.runtime.recordings {
            records.actions.push(value);
        }
    }
    pub(crate) fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        if self.runtime.winner.is_some()
            || unit.side != self.runtime.turn
            || unit.moved
            || unit.attacked
        {
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
        assert!(
            self.promotion_options(&self.runtime.units[index])
                .contains(&kind)
        );
        let previous = &self.rules.units[self.runtime.units[index].kind];
        let name = self.unit_name(&self.runtime.units[index]).to_owned();
        let wounds = previous.hp - self.runtime.units[index].hp;
        let experience = self.runtime.units[index]
            .experience
            .unwrap_or(0)
            .saturating_sub(previous.experience_needed.unwrap_or(0));
        let unit = &mut self.runtime.units[index];
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
        self.evaluate_campaign(false, None);
    }

    pub(crate) fn recruitment_options(&self, side: Side) -> Vec<usize> {
        self.recruitment_options_for_faction(self.faction_for_phase(side))
    }
    pub(crate) fn recruitment_options_for_faction(&self, id: &str) -> Vec<usize> {
        self.recruitment_pool_for_faction(id).collect()
    }
    #[inline]
    pub(crate) fn recruitment_pool_for(
        &self,
        leader: &Unit,
    ) -> super::recruit_pool::RecruitPool<'_> {
        self.recruitment_pool_for_faction(self.unit_faction(leader))
    }
    fn recruitment_pool_for_faction(&self, id: &str) -> super::recruit_pool::RecruitPool<'_> {
        use super::recruit_pool::{PoolEntries, RecruitPool};
        let entries = if let Some(pool) = self
            .runtime
            .scripts
            .as_deref()
            .and_then(|s| s.recruitment.get(id))
        {
            PoolEntries::Recruits(pool.iter())
        } else if let Some(mission) = &self.runtime.mission {
            PoolEntries::Recruits(
                mission
                    .recruitment
                    .get(id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
                    .iter(),
            )
        } else if let Some(pools) = &self.runtime.pools
            && matches!(id, "player" | "enemy")
        {
            PoolEntries::Kinds(pools[if id == "player" { 0 } else { 1 }].iter())
        } else if let Some(armies) = &self.runtime.armies
            && matches!(id, "player" | "enemy")
        {
            PoolEntries::Names(
                self.rules
                    .rosters
                    .iter()
                    .find(|r| r.id == armies[if id == "player" { 0 } else { 1 }])
                    .expect("validated skirmish faction")
                    .pool
                    .iter(),
            )
        } else {
            PoolEntries::Recruits(
                self.faction(id)
                    .and_then(|f| f.recruits.as_deref())
                    .unwrap_or(&[])
                    .iter(),
            )
        };
        RecruitPool {
            rules: self.rules,
            entries,
        }
    }
    pub(crate) fn recruitment_options_for(&self, leader: &Unit) -> Vec<usize> {
        self.recruitment_options_for_faction(self.unit_faction(leader))
    }
    pub(crate) fn recruit_cost(&self, leader: &Unit, kind: usize) -> i32 {
        self.recruit_cost_for_faction(self.unit_faction(leader), kind)
    }
    pub(crate) fn recruit_cost_for_faction(&self, id: &str, kind: usize) -> i32 {
        let unit = &self.rules.units[kind].id;
        if let Some(mission) = &self.runtime.mission {
            if let Some(entry) = self
                .scripts
                .as_deref()
                .and_then(|s| s.recruitment.get(id))
                .and_then(|v| v.iter().find(|r| &r.unit == unit))
            {
                return entry.cost.unwrap_or(self.rules.units[kind].cost);
            }
            let base = mission
                .recruitment
                .get(id)
                .into_iter()
                .flatten()
                .find(|entry| &entry.unit == unit);
            if let Some(entry) = base {
                return entry.cost.unwrap_or(self.rules.units[kind].cost);
            }
            return self.rules.units[kind].cost;
        }
        self.faction(id)
            .and_then(|f| f.recruits.as_ref())
            .and_then(|pool| pool.iter().find(|entry| &entry.unit == unit))
            .and_then(|entry| entry.cost)
            .unwrap_or(self.rules.units[kind].cost)
    }
    pub(crate) fn placement_tiles(
        &self,
        leader: &Unit,
        kind: usize,
        price: bool,
    ) -> crate::model::PlacementTiles {
        if !leader.is_leader()
            || !self
                .recruitment_pool_for(leader)
                .any(|option| option == kind)
            || self.runtime.winner.is_some()
            || !self.may_act(leader)
            || !self.is_keep(leader.tile)
            || self.runtime.owners[leader.tile].as_deref() != Some(self.unit_faction(leader))
            || (price && self.gold_for_unit(leader) < self.recruit_cost(leader, kind))
        {
            return crate::model::PlacementTiles::default();
        }
        self.runtime.board.adjacent[leader.tile]
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
        self.runtime.gold[&faction] -= cost;
        self.add(kind, leader.side, tile);
        let unit = self.runtime.units.last_mut().unwrap();
        if unit.faction != leader.faction {
            unit.faction = leader.faction.clone();
        }
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
            tile: self.runtime.board.ids[tile].clone(),
            cost: Some(cost),
        });
        self.evaluate_campaign(false, None);
    }
}
