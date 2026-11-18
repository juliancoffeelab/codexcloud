//! Compact battle sessions borrowing catalog rules and indexing the live roster.
use super::event_state::{CompiledMission, EventProgress};
use crate::CampaignProgram;

use crate::{data::*, model::StateFrame};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub struct Board {
    pub ids: Vec<Hex>,
    pub locations: Vec<Option<String>>,
    pub positions: Vec<(i32, i32)>,
    pub indices: FxHashMap<Hex, usize>,
    named_locations: FxHashMap<String, usize>,
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
    #[inline]
    pub(crate) fn tile_at(&self, hex: Hex) -> Option<usize> {
        self.indices.get(&hex).copied()
    }
    #[inline]
    pub(crate) fn named_tile(&self, name: &str) -> Option<usize> {
        self.named_locations.get(name).copied()
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
type TerrainKey = ([u8; 32], Vec<(String, Option<String>, i32, i32, String)>);
#[derive(Clone)]
struct SharedTerrain {
    board: Rc<Board>,
    routes: Rc<RefCell<FxHashMap<super::movement::RouteKey, Rc<Vec<u16>>>>>,
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
    static TERRAIN_POOL: RefCell<(FxHashMap<TerrainKey, SharedTerrain>, VecDeque<TerrainKey>)> = RefCell::new((FxHashMap::default(), VecDeque::new()));
}
#[derive(Clone, Debug)]
struct MissionKey {
    definition: CampaignProgram,
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
    static MISSION_POOL: RefCell<FxHashMap<MissionKey, Rc<CompiledMission>>> = RefCell::default();
}
/// Derived data for the current battle revision; refreshed after successful commands.
#[derive(Clone, Debug)]
struct TileGeometry {
    connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    floors: Rc<Vec<Option<String>>>,
    tile_perception: Rc<Vec<TilePerception>>,
    tile_effects: Rc<Vec<Vec<ScopedEffect>>>,
    light: Rc<Vec<Option<std::sync::Arc<TileLight>>>>,
}

/// Immutable planning facts are shared; a mutation detaches only its branch.
#[derive(Clone, Debug)]
pub(crate) struct Shared<T: Clone>(Rc<T>);
impl<T: Clone> Shared<T> {
    pub(crate) fn into_inner(self) -> T {
        Rc::unwrap_or_clone(self.0)
    }
}
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
#[derive(Clone, Debug)]
pub(crate) struct PositionState {
    pub(crate) effect_sources: Rc<super::effect_index::EffectSourceIndex>,
    pub(crate) weapon_ids: Rc<RefCell<super::definition_index::WeaponIdentities>>,
    pub(crate) map_modifiers: Rc<StatModifiers>,
    pub(crate) lighting: Rc<MapLighting>,
    pub(crate) scope: crate::BattleScope,
    pub(crate) character_names: Rc<FxHashMap<String, String>>,
    pub(crate) campaign_battle: bool,
    pub(crate) tracks_path: bool,
    pub(crate) traversal: Option<(u32, Vec<usize>)>,
    pub(crate) mechanical_deaths: Vec<Unit>,
    pub(crate) resolved_recruitment: Rc<std::collections::BTreeMap<String, Vec<FactionRecruit>>>,
    pub(crate) terrain_emitters: Rc<Vec<usize>>,
    pub(crate) perception_sensitive: bool,
    pub(crate) observed_light: Rc<super::geometry::TileSet>,
    pub(crate) ambient_illumination: Rc<Vec<Illumination>>,
    pub(crate) fixed_lights: Rc<Vec<super::perception::ResolvedFixedLight>>,
    pub(crate) factions: Rc<Vec<Faction>>,
    pub(crate) vision_groups: Rc<Vec<VisionGroupState>>,
    pub(crate) unit_perception: Shared<Vec<UnitPerceptionFacts>>,
    /// Turn-local controller scope, never persisted.
    pub(crate) active_faction: Option<String>,
    pub(crate) board: Rc<Board>,
    pub(crate) terrain: Vec<usize>,
    pub(crate) owners: Shared<Vec<Option<String>>>,
    pub(crate) connections: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) walls: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) dead_ends: Rc<Vec<Option<Vec<HexDirection>>>>,
    pub(crate) tile_perception: Rc<Vec<TilePerception>>,
    pub(crate) tile_effects: Rc<Vec<Vec<ScopedEffect>>>,
    pub(crate) light: Rc<Vec<Option<std::sync::Arc<TileLight>>>>,
    pub(crate) condition_memory: Rc<Vec<String>>,
    /// Derived tile-indexed remembered fog; never serialized.
    pub(crate) fog_memory: Rc<FxHashMap<String, Vec<u64>>>,
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
    pub(crate) turn: TurnPhase,
    pub(crate) gold: Gold,
    pub(crate) winner: Option<BattleResult>,
    pub(crate) next_id: u32,
    pub(crate) combat_rng: u32,
    pub(crate) log: Vec<String>,
    pub(crate) last_combat: Option<LastCombat>,
    #[cfg(feature = "wasm")]
    pub(crate) combat_animation: Option<crate::model::CombatAnimation>,
    pub(crate) combat_present: bool,
    pub(crate) armies: Option<[String; 2]>,
    pub(crate) pools: Option<[Vec<usize>; 2]>,
    pub(crate) arena_first: Option<TurnPhase>,
    /// The selected immutable definition; absent only for a non-campaign game.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) recordings: Option<Shared<Records>>,
    pub(super) route_cache: Rc<RefCell<FxHashMap<super::movement::RouteKey, Rc<Vec<u16>>>>>,
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
    pub(super) unit_indices: FxHashMap<u32, usize>,
}
/// Opaque owned live storage used by native hosts between catalog loans.
#[derive(Debug)]
pub struct BattleState {
    pub(crate) position: PositionState,
    pub(crate) story: CampaignRuntime,
}
/// Physical state has no campaign interpreter or narrative history.
pub(crate) struct BattlePosition<'catalog> {
    pub(crate) catalogue: &'catalog Catalogue,
    pub(crate) definitions: Rc<super::definition_index::DefinitionIndex<'catalog>>,
    pub(crate) runtime: PositionState,
}
impl std::fmt::Debug for BattlePosition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BattlePosition")
            .field("content_fingerprint", &self.catalogue.content_fingerprint)
            .field("runtime", &self.runtime)
            .finish()
    }
}
#[derive(Clone, Debug)]
pub(crate) struct CampaignRuntime {
    pub(crate) planning_revision: u64,
    pub(crate) event_deaths: Vec<super::events::DeathOccurrence>,
    pub(super) event_departure: Option<Unit>,
    pub(super) event_opening: bool,
    pub(crate) campaign: Option<Shared<CampaignState>>,
    pub(crate) scripts: Option<Shared<EventProgress>>,
    pub(crate) mission: Option<Rc<CompiledMission>>,
    pub(super) campaign_definition: Option<CampaignProgram>,
    mission_key: Option<MissionKey>,
}
/// Authoritative physical state and campaign/event runtime.
#[derive(Debug)]
pub(crate) struct LiveBattle<'catalog> {
    pub(crate) position: BattlePosition<'catalog>,
    pub(crate) story: CampaignRuntime,
}
impl<'catalog> std::ops::Deref for LiveBattle<'catalog> {
    type Target = BattlePosition<'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.position
    }
}
impl std::ops::DerefMut for LiveBattle<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.position
    }
}
impl std::ops::Deref for BattlePosition<'_> {
    type Target = PositionState;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
impl std::ops::DerefMut for BattlePosition<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}
impl<'catalog> LiveBattle<'catalog> {
    pub(crate) fn from_map(catalogue: &'catalog Catalogue, map: &MapDefinition) -> Self {
        Self::from_state(catalogue, super::setup::initial_state(catalogue, map))
    }

    pub(crate) fn from_state(rules: &'catalog Catalogue, state: State) -> Self {
        let definition = state
            .campaign
            .as_ref()
            .map(|state| CampaignProgram::compile(rules, rules.campaign(&state.id), None));
        Self::from_state_with_campaign(rules, state, definition.as_ref())
    }
    pub(crate) fn from_state_with_campaign(
        rules: &'catalog Catalogue,
        state: State,
        definition: Option<&CampaignProgram>,
    ) -> Self {
        let key = (
            rules.content_fingerprint,
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
        rules: &'catalog Catalogue,
        state: State,
        board: Rc<Board>,
        definition: Option<&CampaignProgram>,
    ) -> Self {
        let (state, tiles) = StateFrame::split(state);
        let n = tiles.len();
        let definitions = Rc::new(super::definition_index::DefinitionIndex::new(rules));
        let mut terrain = Vec::with_capacity(n);
        let mut owners = Vec::with_capacity(n);
        let mut connections = Vec::with_capacity(n);
        let mut walls = Vec::with_capacity(n);
        let mut dead_ends = Vec::with_capacity(n);
        let mut floors = Vec::with_capacity(n);
        let mut tile_perception = Vec::with_capacity(n);
        let mut tile_effects = Vec::with_capacity(n);
        let mut light = Vec::with_capacity(n);
        for tile in tiles {
            terrain.push(definitions.terrain_name(&tile.terrain));
            owners.push(tile.owner);
            connections.push(tile.connections);
            walls.push(tile.walls);
            dead_ends.push(tile.dead_ends);
            floors.push(tile.floor);
            tile_effects.push(tile.effects);
            tile_perception.push(tile.perception);
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
            tile_perception: Rc::new(tile_perception),
            tile_effects: Rc::new(tile_effects),
            light: Rc::new(light),
        };
        Self::from_parts(rules, state, board, definition, terrain, owners, geometry)
    }

    fn from_parts(
        rules: &'catalog Catalogue,
        mut state: StateFrame,
        board: Rc<Board>,
        campaign_definition: Option<&CampaignProgram>,
        terrain: Vec<usize>,
        owners: Vec<Option<String>>,
        geometry: TileGeometry,
    ) -> Self {
        let definitions = Rc::new(super::definition_index::DefinitionIndex::new(rules));
        let character_names = campaign_definition
            .map(|c| {
                c.characters()
                    .iter()
                    .map(|v| (v.id.clone(), v.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let saved_scripts = state
            .campaign
            .as_mut()
            .and_then(|campaign| campaign.scripts.take());
        let fog_memory = state
            .vision_groups
            .iter()
            .map(|group| {
                let mut bits = vec![0; terrain.len().div_ceil(64)];
                for hex in &group.explored {
                    let i = board.indices[hex];
                    bits[i / 64] |= 1u64 << (i % 64);
                }
                (group.id.clone(), bits)
            })
            .collect();
        let fog_memory = Rc::new(fog_memory);
        let mut names = Names::default();
        names.intern("player".into());
        names.intern("enemy".into());
        for faction in &state.factions {
            names.intern(faction.id.clone());
        }
        let mut inventories = Inventories::default();
        let mut unit_perception = vec![UnitPerceptionFacts::default()];
        let units: Vec<Unit> = state
            .units
            .into_iter()
            .map(|unit| {
                let perception = PerceptionId(unit_perception.len());
                unit_perception.push(UnitPerceptionFacts {
                    revealed_to: unit
                        .revealed_to
                        .iter()
                        .map(|id| {
                            VisionGroupKey(
                                state
                                    .vision_groups
                                    .iter()
                                    .position(|g| &g.id == id)
                                    .expect("validated reveal group"),
                            )
                        })
                        .collect(),
                    effects: unit.perception_effects,
                });
                Unit {
                    field_available: true,
                    perception,
                    id: unit.id.strip_prefix('u').unwrap().parse().unwrap(),
                    kind: definitions.kind(&unit.r#type),
                    phase: state
                        .factions
                        .iter()
                        .find(|f| f.id == unit.faction)
                        .expect("validated unit faction")
                        .phase,
                    tile: board.indices[&unit.tile],
                    hp: unit.hp,
                    moved: unit.moved,
                    attack_spent: unit.attack_spent,
                    experience: unit.experience,
                    role: unit.role,
                    retreating: unit.retreating,
                    rebuilding: unit.rebuilding,
                    faction: names.intern(unit.faction.clone()),
                    vision_group: VisionGroupKey(
                        state
                            .vision_groups
                            .iter()
                            .position(|g| {
                                g.id == state
                                    .factions
                                    .iter()
                                    .find(|f| f.id == unit.faction)
                                    .expect("validated unit faction")
                                    .vision_group
                            })
                            .expect("validated faction group"),
                    ),
                    ai_group: unit.ai_group.map(|value| names.intern(value)),
                    character: unit.character.map(|value| names.intern(value)),
                    inventory: inventories.load(unit.items),
                }
            })
            .collect();
        let ground_items = state
            .ground_items
            .into_iter()
            .map(|ground| GroundItemState {
                tile: board.indices[&ground.tile],
                item: inventories.add_item(ground.item),
            })
            .collect();
        let combat_present = state.last_combat.is_some();
        let last_combat = state.last_combat.flatten();
        let observed_light = Rc::new(super::geometry::TileSet::new(terrain.len()));
        let ambient_illumination = Rc::new(
            geometry
                .tile_perception
                .iter()
                .map(|p| match p.illumination {
                    Some(level) => level,
                    None => match state.lighting.time {
                        TimeOfDay::Day => Illumination::Lit,
                        TimeOfDay::Night => Illumination::Unlit,
                    },
                })
                .collect(),
        );
        let terrain_emitters = Rc::new(
            (0..geometry.light.len())
                .filter(|&i| geometry.light[i].is_some())
                .collect(),
        );
        let mut game = Self {
            position: BattlePosition {
                catalogue: rules,
                definitions,
                runtime: PositionState {
                    effect_sources: Rc::new(super::effect_index::EffectSourceIndex::new()),
                    weapon_ids: Rc::new(RefCell::new(
                        super::definition_index::WeaponIdentities::default(),
                    )),
                    map_modifiers: Rc::new(state.map_modifiers.clone()),
                    lighting: Rc::new(state.lighting.clone()),
                    scope: crate::BattleScope::new(),
                    character_names: Rc::new(character_names),
                    campaign_battle: state.campaign.is_some(),
                    tracks_path: false,
                    traversal: None,
                    mechanical_deaths: vec![],
                    terrain_emitters,
                    perception_sensitive: true,
                    observed_light,
                    ambient_illumination,
                    fixed_lights: Rc::new(vec![]),
                    resolved_recruitment: Rc::new(Default::default()),
                    factions: Rc::new(state.factions),
                    vision_groups: Rc::new(state.vision_groups),
                    unit_perception: unit_perception.into(),
                    ground_items,
                    names: std::sync::Arc::new(names),
                    inventories: std::sync::Arc::new(inventories),
                    active_faction: None,
                    tile_perception: geometry.tile_perception,
                    tile_effects: geometry.tile_effects,
                    light: geometry.light,
                    condition_memory: Rc::new(state.condition_memory),
                    fog_memory,
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
                    last_combat,
                    #[cfg(feature = "wasm")]
                    combat_animation: None,
                    combat_present,
                    armies: state.armies,
                    pools: None,
                    arena_first: None,
                    #[cfg(not(target_arch = "wasm32"))]
                    recordings: None,
                    route_cache: Rc::new(RefCell::new(FxHashMap::default())),
                    terrain_revision: 0,
                    attack_geometry: Rc::new(RefCell::new(FxHashMap::default())),
                    attack_masks: super::geometry::AttackMasks::default(),
                    reach_cache: super::movement::ReachCache::default(),
                    threat_reach_cache: super::movement::ThreatReachCache::default(),
                    movement_masks: super::movement::MovementMasks::default(),
                    movement_regions: super::movement::RegionCache::default(),
                    origin_runs: super::geometry::OriginRunCache::default(),
                    route_order: Rc::new(RefCell::new(VecDeque::new())),
                    occupancy: vec![],
                    occupied_tiles: super::geometry::TileSet::new(0),
                    unit_indices: FxHashMap::default(),
                },
            },
            story: CampaignRuntime {
                planning_revision: 0,
                event_deaths: vec![],
                event_departure: None,
                event_opening: false,
                campaign: state.campaign.map(Into::into),
                scripts: None,
                mission: None,
                campaign_definition: campaign_definition.cloned(),
                mission_key: None,
            },
        };
        game.roster_changed();
        game.refresh_mission();
        if let Some(saved) = saved_scripts {
            game.story.scripts = Some(
                EventProgress::restore(
                    game.story
                        .mission
                        .as_ref()
                        .expect("restored mission")
                        .clone(),
                    saved,
                )
                .into(),
            );
        }
        game.sync_recruitment();
        game.refresh_planning_facts();
        game.remember_conditions();
        game
    }
    #[cfg(test)]
    pub(crate) fn new(rules: &'catalog Catalogue, map: &str) -> Self {
        Self::new_with_campaign(rules, map, None)
    }
    pub(crate) fn new_with_campaign(
        rules: &'catalog Catalogue,
        map: &str,
        definition: Option<&CampaignProgram>,
    ) -> Self {
        let map = rules
            .maps()
            .filter(|candidate| candidate.id() == map)
            .max_by_key(|map| map.revision())
            .expect("selected map exists in catalogue");
        let state = super::setup::initial_state(rules, map);
        Self::from_state_with_campaign(rules, state, definition)
    }
    pub(crate) fn campaign_snapshot(&self) -> Option<CampaignState> {
        self.story.campaign.as_deref().cloned().map(|mut campaign| {
            campaign.scripts = self
                .story
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
            map_modifiers: self.runtime.map_modifiers.as_ref().clone(),
            lighting: self.runtime.lighting.as_ref().clone(),
            vision_groups: self.runtime.vision_groups.as_ref().clone(),
            factions: self.runtime.factions.as_ref().clone(),
            ground_items: self
                .runtime
                .ground_items
                .iter()
                .map(|ground| GroundItem {
                    tile: self.runtime.board.ids[ground.tile].clone(),
                    item: self.runtime.inventories.item(ground.item).to_owned(),
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

    /// Capability is authored once and exported through the catalog for UI discovery too.

    /// Refresh dense lookup tables after an internal roster transformation.

    /// Change terrain and invalidate only terrain-dependent route results.

    pub(crate) fn campaign_definition(&self) -> &CampaignProgram {
        self.story
            .campaign_definition
            .as_ref()
            .expect("campaign game has a definition")
    }
    pub(crate) fn refresh_mission(&mut self) {
        let Some(state) = &self.story.campaign else {
            self.story.mission = None;
            self.story.mission_key = None;
            return;
        };
        let definition = self.campaign_definition();
        if self.story.mission_key.as_ref().is_some_and(|key| {
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
                definition.chapters()[state.mission].clone(),
            ));
            if pool.len() >= 128 {
                pool.clear();
            }
            pool.insert(key.clone(), mission.clone());
            mission
        });
        self.story.mission_key = Some(key);
        self.story.mission = Some(mission);
    }
}

impl<'catalog> LiveBattle<'catalog> {}

impl<'catalog> LiveBattle<'catalog> {
    pub(crate) fn fork(&self) -> Self {
        Self {
            position: self.position.fork(),
            story: self.story.clone(),
        }
    }
    pub(crate) fn planning_fork(&self) -> Self {
        let mut copy = self.fork();
        copy.position.runtime.combat_rng = 0x6d2b79f5;
        copy
    }
}
impl<'catalog> BattlePosition<'catalog> {
    pub(crate) fn fork(&self) -> Self {
        let mut runtime = self.runtime.clone();
        #[cfg(not(target_arch = "wasm32"))]
        {
            runtime.recordings = None;
        }
        #[cfg(feature = "wasm")]
        {
            runtime.combat_animation = None;
        }
        Self {
            catalogue: self.catalogue,
            definitions: self.definitions.clone(),
            runtime,
        }
    }
    pub(crate) fn planning_fork(&self) -> Self {
        let mut copy = self.fork();
        copy.runtime.combat_rng = 0x6d2b79f5;
        copy
    }
}

impl<'catalog> BattlePosition<'catalog> {
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
    pub(crate) fn inventory(
        &self,
        unit: &Unit,
    ) -> impl ExactSizeIterator<Item = &ItemDefinition> + Clone {
        self.runtime
            .inventories
            .items(unit.inventory)
            .map(|id| self.catalogue.item(id))
    }
    pub(crate) fn set_inventory(&mut self, id: u32, items: Vec<String>) {
        self.runtime.perception_sensitive |= !items.is_empty();
        let inventory = std::sync::Arc::make_mut(&mut self.runtime.inventories).load(items);
        let index = self.index(id).unwrap();
        self.runtime.units[index].inventory = inventory;
        self.refresh_effect_sources();
    }
    pub(crate) fn give_item(&mut self, id: u32, item: String) {
        let item = std::sync::Arc::make_mut(&mut self.runtime.inventories).add_item(item);
        self.give_existing_item(id, item);
    }
    pub(crate) fn give_existing_item(&mut self, id: u32, item: ItemId) {
        self.runtime.perception_sensitive = true;
        let index = self.index(id).unwrap();
        let old = self.runtime.units[index].inventory;
        let inventory =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).edit(old, None, Some(item));
        self.runtime.units[index].inventory = inventory;
        self.refresh_effect_sources();
    }
    pub(crate) fn drop_item(&mut self, id: u32, slot: usize) -> ItemId {
        let index = self.index(id).unwrap();
        let old = self.runtime.units[index].inventory;
        let item = self.runtime.inventories.item_ids(old)[slot];
        let inventory =
            std::sync::Arc::make_mut(&mut self.runtime.inventories).edit(old, Some(slot), None);
        self.runtime.units[index].inventory = inventory;
        self.refresh_effect_sources();
        item
    }
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
                perception,
                light,
                effects,
            } = tile;
            let position = self.runtime.board.positions[i];
            let kind = self.terrain_definition(i).id().as_str();
            if *col != position.0
                || *row != position.1
                || terrain != kind
                || *owner != self.runtime.owners[i]
                || *connections != self.runtime.connections[i]
                || *dead_ends != self.runtime.dead_ends[i]
                || *walls != self.runtime.walls[i]
                || *floor != self.runtime.floors[i]
                || *perception != self.runtime.tile_perception[i]
                || *effects != self.runtime.tile_effects[i]
                || *light != self.runtime.light[i]
            {
                *col = position.0;
                *row = position.1;
                *terrain = kind.to_owned();
                owner.clone_from(&self.runtime.owners[i]);
                connections.clone_from(&self.runtime.connections[i]);
                dead_ends.clone_from(&self.runtime.dead_ends[i]);
                walls.clone_from(&self.runtime.walls[i]);
                floor.clone_from(&self.runtime.floors[i]);
                perception.clone_from(&self.runtime.tile_perception[i]);
                light.clone_from(&self.runtime.light[i]);
                effects.clone_from(&self.runtime.tile_effects[i]);
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
                terrain: self.terrain_definition(index).id().as_str().to_owned(),
                owner: self.runtime.owners[index].clone(),
                connections: self.runtime.connections[index].clone(),
                walls: self.runtime.walls[index].clone(),
                dead_ends: self.runtime.dead_ends[index].clone(),
                floor: self.runtime.floors[index].clone(),
                perception: self.runtime.tile_perception[index].clone(),
                light: self.runtime.light[index].clone(),
                effects: self.runtime.tile_effects[index].clone(),
            })
            .collect()
    }
    pub(crate) fn public_unit(&self, unit: &Unit) -> PublicUnit {
        PublicUnit {
            revealed_to: self.unit_perception[unit.perception.0]
                .revealed_to
                .iter()
                .map(|g| self.vision_groups[g.0].id.clone())
                .collect(),
            perception_effects: self.unit_perception[unit.perception.0].effects.clone(),
            faction: self.runtime.names.get(unit.faction).to_owned(),
            items: self
                .runtime
                .inventories
                .items(unit.inventory)
                .map(str::to_owned)
                .collect(),
            ai_group: self.name(unit.ai_group).map(str::to_owned),
            id: format!("u{}", unit.id),
            r#type: self.definitions.unit(unit.kind).id().to_owned(),
            tile: self.runtime.board.ids[unit.tile].clone(),
            hp: unit.hp,
            moved: unit.moved,
            attack_spent: unit.attack_spent,
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
    pub(crate) fn unit_name<'a>(&'a self, unit: &'a Unit) -> &'a str {
        self.name(unit.character)
            .and_then(|id| self.runtime.character_names.get(id).map(String::as_str))
            .unwrap_or_else(|| self.definitions.unit(unit.kind).name())
    }
    #[inline]
    pub(crate) fn kind(&self, name: &str) -> usize {
        self.definitions.kind(name)
    }
    #[inline]
    pub(crate) fn terrain_definition(&self, tile: usize) -> &TerrainDefinition {
        self.definitions.terrain(self.runtime.terrain[tile])
    }
    #[inline]
    pub(crate) fn is_keep(&self, tile: usize) -> bool {
        self.terrain_definition(tile).allows_recruitment()
    }
    pub(crate) fn record(&mut self, message: String) {
        self.runtime.log.insert(0, message);
        self.runtime.log.truncate(8);
    }
    pub(crate) fn prospective(&self, kind: usize, faction: NameKey, tile: usize) -> Unit {
        let faction_id = self.names.get(faction);
        let faction_index = self
            .factions
            .iter()
            .position(|f| f.id == faction_id)
            .expect("known prospective faction");
        let offset =
            (faction_index * self.catalogue.units().count() + kind) * self.terrain.len() + tile + 1;
        let id = self
            .next_id
            .checked_add(u32::try_from(offset).expect("prospective identity range"))
            .expect("prospective identity range");
        Unit {
            kind,
            tile,
            hp: self.definitions.unit(kind).max_hp,
            id,
            inventory: InventoryId(0),
            experience: Some(0),
            role: None,
            character: None,
            ai_group: None,
            moved: true,
            attack_spent: true,
            field_available: true,
            perception: PerceptionId(0),
            vision_group: self.vision_group(faction_id),
            phase: self
                .faction(faction_id)
                .expect("known prospective faction")
                .phase,
            faction,
            retreating: None,
            rebuilding: None,
        }
    }
    pub(crate) fn promotion(&self, unit: &Unit, kind: usize) -> Unit {
        let mut result = *unit;
        result.kind = kind;
        result.hp = (self.definitions.unit(kind).max_hp
            - (self.definitions.unit(unit.kind).max_hp - unit.hp))
            .max(1);
        result
    }
    pub(crate) fn recruit_cost(&self, leader: &Unit, kind: usize) -> i32 {
        self.recruit_cost_for_faction(self.unit_faction(leader), kind)
    }
    pub(crate) fn recruit_cost_for_faction(&self, faction: &str, kind: usize) -> i32 {
        let definition = self.definitions.unit(kind);
        let pool = match self.resolved_recruitment.get(faction) {
            Some(pool) => Some(pool.as_slice()),
            None => self.faction(faction).and_then(|f| f.recruits.as_deref()),
        };
        let price = pool
            .and_then(|pool| pool.iter().find(|entry| entry.unit == definition.id()))
            .and_then(|entry| entry.cost);
        definition.recruitment_price(price)
    }
    pub(crate) fn add(&mut self, kind: usize, faction: NameKey, tile: usize) -> u32 {
        self.runtime.perception_sensitive = true;
        let id = self.runtime.next_id;
        self.runtime.next_id += 1;
        let perception = self.fresh_perception();
        let mut unit = self.prospective(kind, faction, tile);
        unit.id = id;
        unit.perception = perception;
        unit.moved = false;
        unit.attack_spent = false;
        self.runtime.units.push(unit);
        self.runtime
            .unit_indices
            .insert(id, self.runtime.units.len() - 1);
        self.runtime.occupancy[tile] = Some(self.runtime.units.len() - 1);
        self.runtime.occupied_tiles.insert(tile);
        self.refresh_effect_sources();
        id
    }
    pub(crate) fn roster_changed(&mut self) {
        self.runtime.occupancy = vec![None; self.runtime.terrain.len()];
        self.runtime.occupied_tiles = super::geometry::TileSet::new(self.runtime.terrain.len());
        self.runtime.unit_indices.clear();
        for (index, unit) in self.runtime.units.iter().enumerate() {
            self.runtime.occupancy[unit.tile] = Some(index);
            self.runtime.occupied_tiles.insert(unit.tile);
            self.runtime.unit_indices.insert(unit.id, index);
        }
        self.refresh_effect_sources();
    }
    pub(crate) fn refresh_ambient_illumination(&mut self) {
        self.runtime.ambient_illumination = Rc::new(
            self.tile_perception
                .iter()
                .map(|p| match p.illumination {
                    Some(level) => level,
                    None => match self.lighting.time {
                        TimeOfDay::Day => Illumination::Lit,
                        TimeOfDay::Night => Illumination::Unlit,
                    },
                })
                .collect(),
        );
    }
    pub(crate) fn invalidate_terrain_queries(&mut self) {
        self.refresh_effect_sources();
        self.runtime.attack_geometry = Rc::new(RefCell::new(FxHashMap::default()));
        self.runtime.attack_masks = super::geometry::AttackMasks::default();
        self.runtime.reach_cache = super::movement::ReachCache::default();
        self.runtime.threat_reach_cache = super::movement::ThreatReachCache::default();
        self.runtime.movement_masks = super::movement::MovementMasks::default();
        self.runtime.movement_regions = super::movement::RegionCache::default();
        self.runtime.origin_runs = super::geometry::OriginRunCache::default();
        self.runtime.terrain_revision = self.runtime.terrain_revision.wrapping_add(1);
        self.runtime.route_cache = Rc::new(RefCell::new(FxHashMap::default()));
        self.runtime.route_order = Rc::new(RefCell::new(VecDeque::new()));
    }
    pub(crate) fn set_terrain(&mut self, tile: usize, terrain: usize) {
        if self.runtime.terrain[tile] != terrain {
            self.runtime.terrain[tile] = terrain;
            Rc::make_mut(&mut self.runtime.connections)[tile] = None;
            Rc::make_mut(&mut self.runtime.walls)[tile] = None;
            Rc::make_mut(&mut self.runtime.dead_ends)[tile] = None;
            self.invalidate_terrain_queries();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn action(&mut self, value: ActionRecord) {
        if let Some(records) = &mut self.runtime.recordings {
            records.actions.push(value);
        }
    }
    pub(crate) fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        if self.runtime.winner.is_some() || !self.may_act(unit) || unit.moved || unit.attack_spent {
            return vec![];
        }
        let definition = self.definitions.unit(unit.kind);
        if definition.max_xp == 0 || unit.experience.expect("resolved XP") < definition.max_xp {
            return vec![];
        }
        definition
            .promotion_ids()
            .iter()
            .map(|name| self.kind(name))
            .filter(|&kind| {
                let promoted = self.promotion(unit, kind);
                self.with_capabilities_at(&promoted, unit.tile, &self.units, &[], |caps| {
                    caps.step_cost(self.terrain_definition(unit.tile)).is_some()
                })
            })
            .collect()
    }

    pub(crate) fn advance(&mut self, id: u32, kind: usize) {
        self.runtime.perception_sensitive = true;
        let index = self.index(id).unwrap();
        assert!(
            self.promotion_options(&self.runtime.units[index])
                .contains(&kind)
        );
        let current = self.runtime.units[index];
        let name = self.unit_name(&current).to_owned();
        let experience = current
            .experience
            .expect("resolved unit experience")
            .saturating_sub(self.definitions.unit(current.kind).max_xp);
        let promoted = self.promotion(&current, kind);
        let unit = &mut self.runtime.units[index];
        *unit = promoted;
        unit.experience = Some(experience);
        unit.moved = true;
        unit.attack_spent = true;
        self.refresh_effect_sources();
        self.record(format!(
            "{} advanced to {}.",
            name,
            self.definitions.unit(kind).name()
        ));
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Advance {
            unit: format!("u{id}"),
            unit_type: self.definitions.unit(kind).id().to_owned(),
        });
    }
    pub(crate) fn recruitment_options_for_faction(&self, id: &str) -> Vec<usize> {
        self.recruitment_pool_for_faction(id).collect()
    }
    #[inline]
    pub(crate) fn recruitment_pool_for(
        &self,
        leader: &Unit,
    ) -> super::recruit_pool::RecruitPool<'_, 'catalog> {
        self.recruitment_pool_for_faction(self.unit_faction(leader))
    }
    fn recruitment_pool_for_faction(
        &self,
        id: &str,
    ) -> super::recruit_pool::RecruitPool<'_, 'catalog> {
        use super::recruit_pool::{PoolEntries, RecruitPool};
        let entries = if let Some(pool) = self.runtime.resolved_recruitment.get(id) {
            PoolEntries::Recruits(pool.iter())
        } else if let Some(pools) = &self.runtime.pools
            && matches!(id, "player" | "enemy")
        {
            PoolEntries::Kinds(pools[if id == "player" { 0 } else { 1 }].iter())
        } else if let Some(armies) = &self.runtime.armies
            && matches!(id, "player" | "enemy")
        {
            PoolEntries::Names(
                self.catalogue
                    .meta()
                    .simulation
                    .rosters
                    .iter()
                    .find(|roster| roster.id == armies[if id == "player" { 0 } else { 1 }])
                    .expect("selected army exists in authored rosters")
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
            definitions: &self.definitions,
            entries,
        }
    }
    pub(crate) fn recruitment_options_for(&self, leader: &Unit) -> Vec<usize> {
        self.recruitment_options_for_faction(self.unit_faction(leader))
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
            .filter(|&tile| {
                let recruit = self.prospective(kind, leader.faction, tile);
                self.at(tile).is_none()
                    && self.with_capabilities_at(&recruit, tile, &self.units, &[], |caps| {
                        caps.step_cost(self.terrain_definition(tile)).is_some()
                    })
            })
            .collect()
    }
    pub(crate) fn recruit(&mut self, id: u32, kind: usize, tile: usize) {
        let leader = self.unit(id).unwrap().clone();
        assert!(self.placement_tiles(&leader, kind, true).contains(&tile));
        let faction = self.unit_faction(&leader).to_owned();
        let cost = self.recruit_cost(&leader, kind);
        self.runtime.gold[&faction] -= cost;
        self.add(kind, leader.faction, tile);
        let unit = self.runtime.units.last_mut().unwrap();
        unit.faction = leader.faction;
        unit.vision_group = leader.vision_group;
        unit.moved = true;
        unit.attack_spent = true;
        self.record(format!(
            "{} recruited a {}. It acts next turn.",
            if self.unit_faction(&leader) == "player" {
                "You"
            } else {
                "Enemy"
            },
            self.definitions.unit(kind).name().to_lowercase()
        ));
        #[cfg(not(target_arch = "wasm32"))]
        self.action(ActionRecord::Recruit {
            unit: format!("u{id}"),
            unit_type: self.definitions.unit(kind).id().to_owned(),
            tile: self.runtime.board.ids[tile].clone(),
            cost: Some(cost),
        });
    }
}
