//! Typed authored JSON catalog. Native loading is rooted at the crate's data directory;
//! browser callers can supply the same named JSON documents without filesystem access.
use super::*;
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
    path::{Component, Path},
};
#[path = "catalog_documents.rs"]
mod documents;
#[cfg(any(test, feature = "wasm"))]
include!(concat!(env!("OUT_DIR"), "/catalog_documents.rs"));

/// Relative document paths mapped to their complete UTF-8 JSON source.
pub type JsonBundle = BTreeMap<String, String>;
#[derive(Debug)]
/// A catalog error carrying the authoring document and a concrete diagnostic.
pub struct ContentError {
    pub path: String,
    pub message: String,
}
/// Compatibility name for a catalog loading/validation failure.
pub type CatalogError = ContentError;
impl ContentError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
impl std::error::Error for ContentError {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    terrain: String,
    races: DefinitionDocuments,
    units: DefinitionDocuments,
    scenarios: Vec<String>,
    campaigns: Vec<String>,
    simulation: SimulationPaths,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DefinitionDocuments {
    Legacy(String),
    Individual(Vec<String>),
}
impl DefinitionDocuments {
    fn paths(&self) -> &[String] {
        match self {
            Self::Legacy(path) => std::slice::from_ref(path),
            Self::Individual(paths) => paths,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulationPaths {
    rosters: String,
    arena: String,
    campaign: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// An arena roster, authored independently of story campaign recruitment.
pub struct ArmyRoster {
    pub id: String,
    pub leader: String,
    pub screen: String,
    pub pool: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A generated arena landscape and its required territory hold duration.
pub struct LandscapeConfig {
    pub id: String,
    pub hold_turns: u32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored arena dimensions and default per-match execution limits.
pub struct ArenaDefaults {
    pub landscapes: Vec<LandscapeConfig>,
    pub territory: Vec<String>,
    /// Minimum total gold value per side, including its leader and two starters.
    pub starting_budget: i32,
    pub default_max_rounds: u32,
    pub default_recruit_limit: usize,
    pub policies: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Campaign schedule dimensions and default attempt execution limits.
pub struct CampaignDefaults {
    pub default_max_rounds: u32,
    pub default_recruit_limit: usize,
    pub diplomacy: Vec<String>,
    pub policies: Vec<String>,
    pub campaigns: Vec<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct EnvironmentalModifiers {
    melee: i32,
    ranged: i32,
    blade: i32,
    pierce: i32,
    impact: i32,
}
impl EnvironmentalModifiers {
    fn value(&self, category: usize, damage: usize) -> i32 {
        [self.melee, self.ranged][category] + [self.blade, self.pierce, self.impact][damage]
    }
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TerrainModifier {
    #[serde(default)]
    cost: i32,
    passable: Option<bool>,
    #[serde(default)]
    damage_modifiers: EnvironmentalModifiers,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TerrainDefinition {
    id: Terrain,
    name: String,
    cost: u16,
    passable: bool,
    description: String,
    #[serde(default)]
    damage_modifiers: EnvironmentalModifiers,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RaceDefinition {
    id: Race,
    name: String,
    terrain_modifiers: HashMap<Terrain, TerrainModifier>,
    damage_modifiers: DamageModifiers,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitSpecification {
    id: String,
    name: String,
    hp: i32,
    r#move: usize,
    cost: i32,
    attacks: Vec<Weapon>,
    damage_modifiers: DamageModifiers,
    race: Race,
    role: UnitRole,
    #[serde(default)]
    terrain_modifiers: HashMap<Terrain, TerrainModifier>,
    #[serde(default)]
    leader: bool,
    #[serde(default)]
    healing: i32,
    level: u32,
    family: Family,
    experience_needed: u32,
    advances_to: Vec<String>,
    ability: String,
    description: String,
}
#[derive(Deserialize)]
struct CampaignFile {
    id: String,
    name: String,
    tagline: String,
    description: String,
    race: Race,
    leader: String,
    stages: Vec<String>,
    ending: Vec<Dialogue>,
}
#[derive(Deserialize)]
struct StageFile {
    id: String,
    state: InitialState,
    #[serde(flatten)]
    mission: Mission,
}

/// Authored initial placements accept omitted runtime defaults. Full reference
/// snapshots remain compatible, but active content derives new troops from rules.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InitialUnit {
    id: Option<String>,
    r#type: String,
    side: Side,
    tile: Option<String>,
    col: Option<i32>,
    row: Option<i32>,
    hp: Option<i32>,
    #[serde(default)]
    moved: bool,
    #[serde(default)]
    attacked: bool,
    #[serde(default = "initial_experience")]
    experience: Option<u32>,
    name: Option<String>,
    role: Option<Role>,
    retreating: Option<bool>,
    rebuilding: Option<bool>,
}
fn initial_experience() -> Option<u32> {
    Some(0)
}
fn initial_round() -> u32 {
    1
}
fn initial_turn() -> Side {
    Side::Player
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InitialState {
    map_id: String,
    map_revision: u32,
    cols: i32,
    rows: i32,
    #[serde(default = "initial_round")]
    round: u32,
    #[serde(default = "initial_turn")]
    turn: Side,
    gold: Gold,
    winner: Option<Side>,
    next_id: Option<u32>,
    log: Option<Vec<String>>,
    tiles: Vec<Tile>,
    units: Vec<InitialUnit>,
    campaign: Option<CampaignState>,
    #[serde(default, deserialize_with = "initial_last_combat")]
    last_combat: Option<Option<LastCombat>>,
}
fn initial_last_combat<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<LastCombat>>, D::Error> {
    Option::<LastCombat>::deserialize(deserializer).map(Some)
}
impl InitialState {
    fn resolve(
        self,
        rules: &[UnitRule],
        types: &HashMap<String, usize>,
        path: &str,
    ) -> Result<State, ContentError> {
        let mut units = Vec::with_capacity(self.units.len());
        let mut next_id = 1;
        for (index, unit) in self.units.into_iter().enumerate() {
            reference(&unit.r#type, types, path)?;
            let rule = &rules[types[&unit.r#type]];
            let hp = unit.hp.unwrap_or(rule.hp);
            if hp <= 0 || hp > rule.hp {
                return Err(ContentError::new(
                    path,
                    format!("invalid initial HP for {}", unit.r#type),
                ));
            }
            let id = unit.id.unwrap_or_else(|| format!("u{}", index + 1));
            let number = id
                .strip_prefix('u')
                .and_then(|id| id.parse::<u32>().ok())
                .filter(|id| *id > 0)
                .ok_or_else(|| {
                    ContentError::new(path, format!("invalid initial unit identifier {id}"))
                })?;
            next_id = next_id.max(
                number
                    .checked_add(1)
                    .ok_or_else(|| ContentError::new(path, "initial unit identifier overflow"))?,
            );
            let tile = match (unit.tile, unit.col, unit.row) {
                (Some(tile), None, None) => tile,
                (None, Some(col), Some(row)) => format!("{col},{row}"),
                _ => {
                    return Err(ContentError::new(
                        path,
                        "initial units require col and row or one legacy tile identifier",
                    ));
                }
            };
            units.push(PublicUnit {
                id,
                r#type: unit.r#type,
                side: unit.side,
                tile,
                hp,
                moved: unit.moved,
                attacked: unit.attacked,
                experience: unit.experience,
                name: unit.name,
                role: unit.role,
                retreating: unit.retreating,
                rebuilding: unit.rebuilding,
            });
        }
        let authored_next = self.next_id.unwrap_or(next_id);
        if authored_next < next_id {
            return Err(ContentError::new(
                path,
                "nextId must exceed all initial unit identifiers",
            ));
        }
        Ok(State {
            map_id: self.map_id,
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            round: self.round,
            turn: self.turn,
            gold: self.gold,
            winner: self.winner,
            next_id: authored_next,
            log: self.log.unwrap_or_else(|| {
                vec!["Capture villages for gold. Defeat the enemy commander.".to_owned()]
            }),
            tiles: self.tiles,
            units,
            campaign: self.campaign,
            last_combat: self.last_combat,
        })
    }
}

fn parse<T: DeserializeOwned>(bundle: &JsonBundle, path: &str) -> Result<T, ContentError> {
    checked_path(path)?;
    let source = bundle
        .get(path)
        .ok_or_else(|| ContentError::new(path, "missing catalog document"))?;
    serde_json::from_str(source).map_err(|error| ContentError::new(path, error.to_string()))
}
fn checked_path(path: &str) -> Result<(), ContentError> {
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(ContentError::new(
            path,
            "catalog paths must be relative without traversal",
        ));
    }
    Ok(())
}
fn identifier(id: &str, path: &str) -> Result<(), ContentError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err(ContentError::new(
            path,
            format!("invalid identifier {id:?}"),
        ));
    }
    Ok(())
}
fn unique<'a>(ids: impl IntoIterator<Item = &'a str>, path: &str) -> Result<(), ContentError> {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(ContentError::new(
                path,
                format!("duplicate identifier {id}"),
            ));
        }
    }
    Ok(())
}
fn reference(id: &str, types: &HashMap<String, usize>, path: &str) -> Result<(), ContentError> {
    if !types.contains_key(id) {
        return Err(ContentError::new(path, format!("unknown unit {id}")));
    }
    Ok(())
}

impl Rules {
    /// Native data root never depends on the process working directory.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Result<Self, ContentError> {
        let root = std::env::var_os("HEX_COMMAND_DATA_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("data"));
        Self::load_from(root)
    }
    /// Load the compiled WASM catalog, or report that an explicit bundle is needed.
    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Result<Self, ContentError> {
        #[cfg(feature = "wasm")]
        {
            Self::embedded()
        }
        #[cfg(not(feature = "wasm"))]
        {
            Err(ContentError::new(
                "manifest.json",
                "WebAssembly callers must supply a JSON bundle",
            ))
        }
    }
    /// Read and validate a native catalog rooted at an explicit filesystem path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from(root: impl AsRef<Path>) -> Result<Self, ContentError> {
        let root = root.as_ref();
        documents::collect_bundle(|path| {
            checked_path(path)?;
            std::fs::read_to_string(root.join(path))
                .map_err(|error| ContentError::new(path, error.to_string()))
        })
        .and_then(|bundle| Self::from_json_bundle(&bundle))
    }
    /// Load the bundled catalog for focused tests and WASM deployment.
    #[cfg(any(test, feature = "wasm"))]
    pub fn embedded() -> Result<Self, ContentError> {
        Self::from_json_bundle(&embedded_bundle())
    }
    /// Validate named documents and resolve inheritance without filesystem access.
    pub fn from_json_bundle(bundle: &JsonBundle) -> Result<Self, ContentError> {
        let manifest: Manifest = parse(bundle, "manifest.json")?;
        if !matches!(manifest.schema_version, 1 | 2) {
            return Err(ContentError::new(
                "manifest.json",
                "unsupported schemaVersion",
            ));
        }
        let terrain: Vec<TerrainDefinition> = parse(bundle, &manifest.terrain)?;
        let races: Vec<(String, RaceDefinition)> = match &manifest.races {
            DefinitionDocuments::Legacy(path) => parse::<Vec<RaceDefinition>>(bundle, path)?
                .into_iter()
                .map(|race| (path.clone(), race))
                .collect(),
            DefinitionDocuments::Individual(paths) => {
                unique(paths.iter().map(String::as_str), "manifest.json")?;
                paths
                    .iter()
                    .map(|path| parse(bundle, path).map(|race| (path.clone(), race)))
                    .collect::<Result<_, _>>()?
            }
        };
        let specs: Vec<(String, UnitSpecification)> = match &manifest.units {
            DefinitionDocuments::Legacy(path) if manifest.schema_version == 1 => {
                parse::<Vec<UnitSpecification>>(bundle, path)?
                    .into_iter()
                    .map(|spec| (path.clone(), spec))
                    .collect()
            }
            DefinitionDocuments::Individual(paths) if manifest.schema_version == 2 => {
                unique(paths.iter().map(String::as_str), "manifest.json")?;
                paths
                    .iter()
                    .map(|path| parse(bundle, path).map(|spec| (path.clone(), spec)))
                    .collect::<Result<_, _>>()?
            }
            _ => {
                return Err(ContentError::new(
                    "manifest.json",
                    "units must match the catalog schemaVersion",
                ));
            }
        };
        if terrain.is_empty() || races.is_empty() || specs.is_empty() {
            return Err(ContentError::new(
                "manifest.json",
                "terrain, races and units must be nonempty",
            ));
        }
        unique(terrain.iter().map(|t| t.id.as_str()), &manifest.terrain)?;
        let mut seen_races = HashSet::new();
        for (path, race) in &races {
            if !seen_races.insert(race.id) {
                return Err(ContentError::new(
                    path,
                    format!("duplicate race {}", race.id),
                ));
            }
        }
        let mut seen_units = HashSet::new();
        for (path, spec) in &specs {
            if !seen_units.insert(&spec.id) {
                return Err(ContentError::new(
                    path,
                    format!("duplicate identifier {}", spec.id),
                ));
            }
        }
        let terrain_names: Vec<String> = terrain.iter().map(|t| t.id.as_str().to_owned()).collect();
        let terrains: HashMap<String, usize> = terrain_names
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        let types: HashMap<String, usize> = specs
            .iter()
            .enumerate()
            .map(|(i, (_, u))| (u.id.clone(), i))
            .collect();
        let race_by_id: HashMap<Race, &RaceDefinition> =
            races.iter().map(|(_, r)| (r.id, r)).collect();
        let validate_modifiers = |modifiers: &HashMap<Terrain, TerrainModifier>,
                                  path: &str|
         -> Result<(), ContentError> {
            for id in modifiers.keys() {
                if !terrains.contains_key(id.as_str()) {
                    return Err(ContentError::new(path, format!("unknown terrain {id}")));
                }
            }
            Ok(())
        };
        for t in &terrain {
            if t.name.is_empty() || t.description.is_empty() || t.cost == 0 {
                return Err(ContentError::new(
                    &manifest.terrain,
                    "terrain requires name, description and positive cost",
                ));
            }
        }
        for (path, race) in &races {
            if race.name.is_empty() {
                return Err(ContentError::new(path, "race name is required"));
            }
            validate_modifiers(&race.terrain_modifiers, path)?;
        }
        let mut units = Vec::with_capacity(specs.len());
        for (path, spec) in specs {
            identifier(&spec.id, &path)?;
            let race = race_by_id
                .get(&spec.race)
                .ok_or_else(|| ContentError::new(&path, format!("unknown race {}", spec.race)))?;
            if spec.name.is_empty()
                || spec.ability.is_empty()
                || spec.description.is_empty()
                || spec.hp <= 0
                || spec.r#move == 0
                || spec.cost < 0
                || spec.level == 0
                || spec.attacks.is_empty()
            {
                return Err(ContentError::new(
                    &path,
                    format!("invalid required stats for {}", spec.id),
                ));
            }
            unique(spec.attacks.iter().map(|w| w.id.as_str()), &path)?;
            for attack in &spec.attacks {
                identifier(&attack.id, &path)?;
                if attack.name.is_empty()
                    || attack.sound.is_empty()
                    || attack.range == 0
                    || attack.damage < 0
                {
                    return Err(ContentError::new(
                        &path,
                        format!("invalid weapon {}", attack.id),
                    ));
                }
            }
            for next in &spec.advances_to {
                reference(next, &types, &path)?;
            }
            validate_modifiers(&spec.terrain_modifiers, &path)?;
            let mut costs = Vec::new();
            let mut environment = Vec::new();
            for t in &terrain {
                let inherited = race.terrain_modifiers.get(&t.id);
                let individual = spec.terrain_modifiers.get(&t.id);
                let passable = individual
                    .and_then(|m| m.passable)
                    .or_else(|| inherited.and_then(|m| m.passable))
                    .unwrap_or(t.passable);
                let cost = (i32::from(t.cost)
                    + inherited.map_or(0, |m| m.cost)
                    + individual.map_or(0, |m| m.cost))
                .max(1);
                costs.push(if passable {
                    Some(
                        u16::try_from(cost)
                            .map_err(|_| ContentError::new(&path, "movement cost overflows u16"))?,
                    )
                } else {
                    None
                });
                let mut modifiers = [[0; 3]; 2];
                for (category, row) in modifiers.iter_mut().enumerate() {
                    for (damage, value) in row.iter_mut().enumerate() {
                        *value = t.damage_modifiers.value(category, damage)
                            + inherited.map_or(0, |m| m.damage_modifiers.value(category, damage))
                            + individual.map_or(0, |m| m.damage_modifiers.value(category, damage));
                    }
                }
                environment.push(modifiers);
            }
            units.push(UnitRule {
                id: spec.id,
                name: spec.name,
                ability: spec.ability,
                description: spec.description,
                hp: spec.hp,
                r#move: spec.r#move,
                damage: spec.attacks[0].damage,
                range: spec.attacks.iter().map(|w| w.range).max().unwrap(),
                cost: spec.cost,
                attacks: spec.attacks,
                damage_modifiers: DamageModifiers {
                    blade: race.damage_modifiers.blade + spec.damage_modifiers.blade,
                    pierce: race.damage_modifiers.pierce + spec.damage_modifiers.pierce,
                    impact: race.damage_modifiers.impact + spec.damage_modifiers.impact,
                },
                race: spec.race,
                role: spec.role,
                leader: spec.leader,
                healing: spec.healing,
                level: spec.level,
                family: spec.family,
                experience_needed: Some(spec.experience_needed),
                advances_to: spec.advances_to,
                costs,
                environment,
            });
        }
        let mut maps = HashMap::new();
        let mut campaigns = HashMap::new();
        let mut used_paths = HashSet::new();
        for path in &manifest.scenarios {
            if !used_paths.insert(path.clone()) {
                return Err(ContentError::new(path, "duplicate catalog path"));
            }
            let (initial, roster_path) = if manifest.schema_version == 1 {
                (parse::<InitialState>(bundle, path)?, path.clone())
            } else {
                documents::scenario(bundle, path)?
            };
            let state = initial.resolve(&units, &types, &roster_path)?;
            insert_map(&mut maps, state, &terrains, &types, path)?;
        }
        for path in &manifest.campaigns {
            if !used_paths.insert(path.clone()) {
                return Err(ContentError::new(path, "duplicate catalog path"));
            }
            let source: CampaignFile = parse(bundle, path)?;
            identifier(&source.id, path)?;
            reference(&source.leader, &types, path)?;
            if source.name.is_empty()
                || source.tagline.is_empty()
                || source.description.is_empty()
                || source.stages.is_empty()
            {
                return Err(ContentError::new(
                    path,
                    "campaign name and stages are required",
                ));
            }
            let mut missions = Vec::new();
            let mut stage_ids = HashSet::new();
            for stage_path in source.stages {
                let stage_path = documents::stage_path(manifest.schema_version, path, &stage_path)?;
                if !used_paths.insert(stage_path.clone()) {
                    return Err(ContentError::new(&stage_path, "duplicate catalog path"));
                }
                let stage: StageFile = if manifest.schema_version == 1 {
                    parse(bundle, &stage_path)?
                } else {
                    documents::stage(bundle, &stage_path)?
                };
                let state = stage.state.resolve(&units, &types, &stage_path)?;
                identifier(&stage.id, &stage_path)?;
                if !stage_ids.insert(stage.id.clone())
                    || stage.id != state.map_id
                    || stage.id != stage.mission.map.id
                {
                    return Err(ContentError::new(
                        &stage_path,
                        "duplicate or mismatched stage/map identifier",
                    ));
                }
                validate_mission(&stage.mission, &state, &types, &stage_path)?;
                insert_map(&mut maps, state, &terrains, &types, &stage_path)?;
                missions.push(stage.mission);
            }
            let id = source.id.clone();
            if campaigns
                .insert(
                    id,
                    Campaign {
                        id: source.id,
                        name: source.name,
                        tagline: source.tagline,
                        description: source.description,
                        race: source.race,
                        leader: source.leader,
                        missions,
                        ending: source.ending,
                    },
                )
                .is_some()
            {
                return Err(ContentError::new(path, "duplicate campaign identifier"));
            }
        }
        let rosters: Vec<ArmyRoster> = parse(bundle, &manifest.simulation.rosters)?;
        let arena: ArenaDefaults = parse(bundle, &manifest.simulation.arena)?;
        let campaign_defaults: CampaignDefaults = parse(bundle, &manifest.simulation.campaign)?;
        validate_simulation(&rosters, &arena, &campaign_defaults, &types)?;
        unique(
            campaign_defaults.campaigns.iter().map(String::as_str),
            "simulation/campaign.json",
        )?;
        for id in &campaign_defaults.campaigns {
            if !campaigns.contains_key(id) {
                return Err(ContentError::new(
                    "simulation/campaign.json",
                    format!("unknown campaign {id}"),
                ));
            }
        }
        Ok(Self {
            content_fingerprint: content_fingerprint(bundle),
            terrain_types: terrain.iter().map(|t| t.id).collect(),
            terrain_names,
            terrain_labels: terrain.into_iter().map(|t| t.name).collect(),
            units,
            types,
            terrains,
            maps,
            campaigns,
            rosters,
            arena,
            campaign_defaults,
        })
    }
}

/// SHA-256 of the exact supplied documents, in lexicographic path order.
/// Formatting changes deliberately change the fingerprint alongside rule edits.
pub fn content_fingerprint(bundle: &JsonBundle) -> String {
    let mut digest = Sha256::new();
    for (path, source) in bundle {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(source.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn insert_map(
    maps: &mut HashMap<(String, u32), State>,
    state: State,
    terrains: &HashMap<String, usize>,
    types: &HashMap<String, usize>,
    path: &str,
) -> Result<(), ContentError> {
    identifier(&state.map_id, path)?;
    if state.map_revision == 0 || state.cols <= 0 || state.rows <= 0 || state.tiles.is_empty() {
        return Err(ContentError::new(
            path,
            "map revision, dimensions and tiles must be positive",
        ));
    }
    unique(state.tiles.iter().map(|t| t.id.as_str()), path)?;
    unique(state.units.iter().map(|u| u.id.as_str()), path)?;
    unique(state.units.iter().map(|u| u.tile.as_str()), path)?;
    let mut coordinates = HashSet::new();
    let tiles: HashSet<&str> = state.tiles.iter().map(|t| t.id.as_str()).collect();
    for tile in &state.tiles {
        if !terrains.contains_key(&tile.terrain) {
            return Err(ContentError::new(
                path,
                format!("unknown terrain {}", tile.terrain),
            ));
        }
        if tile.id != format!("{},{}", tile.col, tile.row)
            || tile.col < 0
            || tile.col >= state.cols
            || tile.row < 0
            || tile.row >= state.rows
            || !coordinates.insert((tile.col, tile.row))
        {
            return Err(ContentError::new(
                path,
                format!("invalid tile coordinates {}", tile.id),
            ));
        }
    }
    for unit in &state.units {
        reference(&unit.r#type, types, path)?;
        if !tiles.contains(unit.tile.as_str()) || unit.hp <= 0 {
            return Err(ContentError::new(
                path,
                format!("invalid unit placement {}", unit.id),
            ));
        }
    }
    let key = (state.map_id.clone(), state.map_revision);
    if maps.insert(key, state).is_some() {
        return Err(ContentError::new(
            path,
            "duplicate map identifier and revision",
        ));
    }
    Ok(())
}
fn validate_mission(
    mission: &Mission,
    state: &State,
    types: &HashMap<String, usize>,
    path: &str,
) -> Result<(), ContentError> {
    let tiles: HashSet<&str> = state.tiles.iter().map(|t| t.id.as_str()).collect();
    let tile_ref = |id: &str| -> Result<(), ContentError> {
        if !tiles.contains(id) {
            return Err(ContentError::new(path, format!("unknown tile {id}")));
        }
        Ok(())
    };
    unique(mission.events.iter().map(|e| e.id.as_str()), path)?;
    unique(mission.waves.iter().map(|w| w.id.as_str()), path)?;
    let events: HashSet<&str> = mission.events.iter().map(|e| e.id.as_str()).collect();
    let event_ref = |id: &str| -> Result<(), ContentError> {
        if !events.contains(id) {
            return Err(ContentError::new(path, format!("unknown event {id}")));
        }
        Ok(())
    };
    let starting = |units: &[StartingUnit]| -> Result<(), ContentError> {
        for unit in units {
            reference(&unit.r#type, types, path)?;
            tile_ref(&format!("{},{}", unit.col, unit.row))?;
        }
        Ok(())
    };
    starting(&mission.map.units)?;
    for id in mission.recruits.iter().chain(&mission.enemy_recruits) {
        reference(id, types, path)?;
    }
    if let Some(id) = &mission.enemy_leader_post {
        tile_ref(id)?;
    }
    if mission.briefing.is_empty() || mission.goals.is_empty() {
        return Err(ContentError::new(
            path,
            "mission briefing and goals are required",
        ));
    }
    for goal in &mission.goals {
        for site in goal.sites() {
            tile_ref(&site)?;
        }
        if let Some(id) = goal.unit_type() {
            reference(id, types, path)?;
        }
        if let Some(id) = goal.event() {
            event_ref(id)?;
        }
        if let Some(id) = goal.start_event() {
            event_ref(id)?;
        }
        if goal.turns() == Some(0) {
            return Err(ContentError::new(path, "goal turns must be positive"));
        }
    }
    for event in &mission.events {
        identifier(&event.id, path)?;
        match &event.trigger {
            Trigger::Visit { tile } | Trigger::Control { tile, .. } => tile_ref(tile)?,
            Trigger::Round { round } => {
                if *round == 0 {
                    return Err(ContentError::new(path, "event round must be positive"));
                }
            }
        }
        for id in event.requires_event.iter().chain(&event.unless_event) {
            event_ref(id)?;
        }
    }
    for wave in &mission.waves {
        identifier(&wave.id, path)?;
        starting(&wave.units)?;
        if let Some(id) = &wave.requires_unsealed {
            tile_ref(id)?;
        }
    }
    if let Some(refuge) = &mission.refuge {
        event_ref(&refuge.medicine_event)?;
    }
    let validate_patch = |patch: &MissionPatch| -> Result<(), ContentError> {
        if let Some(recruits) = &patch.recruits {
            for id in recruits {
                reference(id, types, path)?;
            }
        }
        if let Some(units) = &patch.reinforcements {
            starting(units)?;
        }
        Ok(())
    };
    if let Some(patches) = &mission.companion_losses {
        for patch in patches.values() {
            validate_patch(patch)?;
        }
    }
    if let Some(patch) = &mission.without_dain {
        validate_patch(patch)?;
    }
    Ok(())
}
fn validate_simulation(
    rosters: &[ArmyRoster],
    arena: &ArenaDefaults,
    campaign: &CampaignDefaults,
    types: &HashMap<String, usize>,
) -> Result<(), ContentError> {
    let path = "simulation/rosters.json";
    unique(rosters.iter().map(|r| r.id.as_str()), path)?;
    if rosters.is_empty() {
        return Err(ContentError::new(path, "rosters must be nonempty"));
    }
    for roster in rosters {
        identifier(&roster.id, path)?;
        reference(&roster.leader, types, path)?;
        reference(&roster.screen, types, path)?;
        unique(roster.pool.iter().map(String::as_str), path)?;
        if !roster.pool.contains(&roster.screen) {
            return Err(ContentError::new(path, "screen must belong to roster pool"));
        }
        for id in &roster.pool {
            reference(id, types, path)?;
        }
    }
    let path = "simulation/arena.json";
    unique(arena.landscapes.iter().map(|l| l.id.as_str()), path)?;
    unique(arena.territory.iter().map(String::as_str), path)?;
    if arena.landscapes.is_empty()
        || arena.territory.is_empty()
        || arena.starting_budget <= 0
        || arena.default_max_rounds == 0
        || campaign.default_max_rounds == 0
    {
        return Err(ContentError::new(
            path,
            "landscapes, territory, starting budget and round limits must be nonempty/positive",
        ));
    }
    for landscape in &arena.landscapes {
        if !["open", "wooded", "crossings", "islands"].contains(&landscape.id.as_str())
            || landscape.hold_turns == 0
        {
            return Err(ContentError::new(path, "invalid landscape or holdTurns"));
        }
    }
    for tile in &arena.territory {
        let parts: Vec<_> = tile.split(',').collect();
        if parts.len() != 2
            || parts
                .iter()
                .any(|p| p.parse::<u8>().map_or(true, |n| n > 8))
        {
            return Err(ContentError::new(
                path,
                format!("invalid arena territory tile {tile}"),
            ));
        }
    }
    for (ids, allowed, path) in [
        (
            &arena.policies,
            ["classic", "rush", "mixed"].as_slice(),
            "simulation/arena.json",
        ),
        (
            &campaign.policies,
            ["classic", "rush", "mixed"].as_slice(),
            "simulation/campaign.json",
        ),
        (
            &campaign.diplomacy,
            ["pragmatic", "restrained"].as_slice(),
            "simulation/campaign.json",
        ),
    ] {
        unique(ids.iter().map(String::as_str), path)?;
        if ids.is_empty() || ids.iter().any(|id| !allowed.contains(&id.as_str())) {
            return Err(ContentError::new(
                path,
                "unknown or missing simulation strategy",
            ));
        }
    }
    Ok(())
}

/// The authored documents are also available without native filesystem APIs.
#[cfg(any(test, feature = "wasm"))]
pub fn embedded_bundle() -> JsonBundle {
    documents::collect_bundle(|path| {
        EMBEDDED_DOCUMENTS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, source)| (*source).to_owned())
            .ok_or_else(|| ContentError::new(path, "missing embedded catalog document"))
    })
    .expect("compiled catalog document references")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_catalog_resolves_races_and_preserves_revisions() {
        let rules = Rules::embedded().unwrap();
        let skeleton = &rules.units[rules.types["skeleton"]];
        assert_eq!(skeleton.damage_modifiers.pierce, -2);
        let boar = &rules.units[rules.types["boar"]];
        assert_eq!(boar.costs[rules.terrains["forest"]], Some(1));
        assert_eq!(boar.costs[rules.terrains["ford"]], Some(3));
        assert_eq!(boar.costs[rules.terrains["water"]], None);
        assert_eq!(
            boar.environment[rules.terrains["forest"]],
            [[-1, -1, -1], [-2, -2, -2]]
        );
        let reefguard = &rules.units[rules.types["reefguard"]];
        assert_eq!(reefguard.costs[rules.terrains["water"]], Some(1));
        assert!(rules.maps.contains_key(&("river".to_owned(), 1)));
        assert!(rules.maps.contains_key(&("river".to_owned(), 2)));
        assert!(rules.maps.contains_key(&("ashen-refuge".to_owned(), 1)));
        assert!(rules.maps.contains_key(&("ashen-refuge".to_owned(), 2)));
        assert_eq!(
            rules.campaigns["lantern-road"].missions[0].map.id,
            "lantern-ferry"
        );
    }
    #[test]
    fn invalid_references_and_duplicates_report_source_document() {
        let mut bundle = embedded_bundle();
        let units = bundle.get_mut("game/units/warden.json").unwrap();
        *units = units.replacen("\"warden\"", "\"adept\"", 1);
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "game/units/adept.json");
        assert!(error.message.contains("duplicate"));
        let mut bundle = embedded_bundle();
        let roster = bundle.get_mut("simulation/rosters.json").unwrap();
        *roster = roster.replacen("\"commander\"", "\"missing-unit\"", 1);
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert!(error.message.contains("unknown unit missing-unit"));
    }
    #[test]
    fn missing_required_stats_and_documents_are_errors() {
        let mut bundle = embedded_bundle();
        let units = bundle.get_mut("game/units/warden.json").unwrap();
        let mut document: serde_json::Value = serde_json::from_str(units).unwrap();
        document.as_object_mut().unwrap().remove("hp");
        *units = serde_json::to_string(&document).unwrap();
        assert!(
            Rules::from_json_bundle(&bundle)
                .unwrap_err()
                .message
                .contains("missing field")
        );
        let mut bundle = embedded_bundle();
        bundle.remove("game/races/merfolk.json");
        assert_eq!(
            Rules::from_json_bundle(&bundle).unwrap_err().path,
            "game/races/merfolk.json"
        );
    }
    #[test]
    fn roles_are_required_and_race_errors_identify_the_individual_document() {
        let mut bundle = embedded_bundle();
        let unit = bundle.get_mut("game/units/scout.json").unwrap();
        *unit = unit.replace("\"role\": \"flank\",", "");
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "game/units/scout.json");
        assert!(error.message.contains("role"));
        let mut bundle = embedded_bundle();
        let race = bundle.get_mut("game/races/human.json").unwrap();
        *race = race.replace("\"id\": \"human\"", "\"id\": \"merfolk\"");
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "game/races/human.json");
        assert!(error.message.contains("duplicate race"));
    }
    #[test]
    fn manifest_rejects_traversal_and_unsupported_schema() {
        let mut bundle = embedded_bundle();
        let manifest = bundle.get_mut("manifest.json").unwrap();
        *manifest = manifest.replace("game/terrain.json", "../terrain.json");
        assert!(
            Rules::from_json_bundle(&bundle)
                .unwrap_err()
                .message
                .contains("traversal")
        );
        let mut bundle = embedded_bundle();
        let manifest = bundle.get_mut("manifest.json").unwrap();
        *manifest = manifest.replace("\"schemaVersion\": 2", "\"schemaVersion\": 99");
        assert!(
            Rules::from_json_bundle(&bundle)
                .unwrap_err()
                .message
                .contains("schemaVersion")
        );
    }
    #[test]
    fn metadata_and_exact_document_fingerprint_are_preserved() {
        let bundle = embedded_bundle();
        let rules = Rules::from_json_bundle(&bundle).unwrap();
        let adept = &rules.units[rules.types["adept"]];
        assert_eq!(adept.ability, "Stone spark");
        assert!(adept.description.contains("Ranged Impact"));
        let campaign = &rules.campaigns["lantern-road"];
        assert!(!campaign.tagline.is_empty());
        assert!(!campaign.description.is_empty());
        assert_eq!(campaign.race, Race::Human);
        assert_eq!(rules.content_fingerprint, content_fingerprint(&bundle));
        assert_eq!(rules.content_fingerprint.len(), 64);
        let mut reformatted = bundle.clone();
        reformatted
            .get_mut("game/units/warden.json")
            .unwrap()
            .push('\n');
        let changed = Rules::from_json_bundle(&reformatted).unwrap();
        assert_ne!(rules.content_fingerprint, changed.content_fingerprint);
        assert_eq!(adept.ability, changed.units[changed.types["adept"]].ability);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_reload_reads_metadata_edits_without_rebuilding() {
        struct Directory(std::path::PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let root = Directory(std::env::temp_dir().join(format!(
            "hex-command-catalog-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        )));
        for (path, source) in embedded_bundle() {
            let target = root.0.join(path);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, source).unwrap();
        }
        let original = Rules::load_from(&root.0).unwrap();
        let path = root.0.join("game/units/adept.json");
        let edited =
            std::fs::read_to_string(&path)
                .unwrap()
                .replacen("Stone spark", "Revised spark", 1);
        std::fs::write(path, edited).unwrap();
        let commander = root.0.join("game/units/commander.json");
        let original_hp = original.units[original.types["commander"]].hp;
        let revised_hp = original_hp + 3;
        let mut edited: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&commander).unwrap()).unwrap();
        edited["hp"] = serde_json::json!(revised_hp);
        std::fs::write(commander, serde_json::to_vec(&edited).unwrap()).unwrap();
        let reloaded = Rules::load_from(&root.0).unwrap();
        assert_eq!(
            reloaded.units[reloaded.types["adept"]].ability,
            "Revised spark"
        );
        assert_eq!(
            original.units[original.types["adept"]].ability,
            "Stone spark"
        );
        assert_ne!(original.content_fingerprint, reloaded.content_fingerprint);
        assert_eq!(original.rosters[0].pool, reloaded.rosters[0].pool);
        let initial = reloaded.maps[&("mirror".to_owned(), 1)].clone();
        assert_eq!(initial.units[0].r#type, "commander");
        assert_eq!(initial.units[0].hp, revised_hp);
        let battle = crate::logic::Battle::restore(&reloaded, initial).unwrap();
        assert_eq!(battle.snapshot().units[0].hp, revised_hp);
        assert_eq!(
            original.maps[&("mirror".to_owned(), 1)].units[0].hp,
            original_hp
        );
    }
    #[test]
    fn initial_placements_allow_wounds_but_reject_excess_hp() {
        let mut bundle = embedded_bundle();
        let map = bundle
            .get_mut("game/scenario/mirror-v1/roster.json")
            .unwrap();
        *map = map.replacen(
            "\"type\": \"commander\",",
            "\"type\": \"commander\",\n      \"hp\": 7,",
            1,
        );
        let rules = Rules::from_json_bundle(&bundle).unwrap();
        let state = &rules.maps[&("mirror".to_owned(), 1)];
        assert_eq!(state.units[0].hp, 7);
        assert_eq!(state.units[1].hp, rules.units[rules.types["soldier"]].hp);
        assert_eq!(state.units[0].id, "u1");
        assert_eq!(state.units[0].experience, Some(0));
        assert!(!state.units[0].moved);
        assert!(!state.units[0].attacked);
        let map = bundle
            .get_mut("game/scenario/mirror-v1/roster.json")
            .unwrap();
        *map = map.replace("\"hp\": 7,", "\"hp\": 99,");
        let error = Rules::from_json_bundle(&bundle).unwrap_err();
        assert_eq!(error.path, "game/scenario/mirror-v1/roster.json");
        assert!(error.message.contains("initial HP"));
    }

    #[test]
    fn component_references_report_missing_files_and_reject_traversal() {
        let mut bundle = embedded_bundle();
        let path = "game/campaigns/borrowed-tide/stages/tide-wreck/objectives.json";
        bundle.remove(path);
        assert_eq!(Rules::from_json_bundle(&bundle).unwrap_err().path, path);
        let mut bundle = embedded_bundle();
        let descriptor = bundle
            .get_mut("game/scenario/mirror-v1/scenario.json")
            .unwrap();
        *descriptor = descriptor.replace("roster.json", "../roster.json");
        assert!(
            Rules::from_json_bundle(&bundle)
                .unwrap_err()
                .message
                .contains("traversal")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_and_embedded_load_only_manifest_referenced_documents() {
        let native = Rules::load_from(Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap();
        assert_eq!(
            native.content_fingerprint,
            Rules::embedded().unwrap().content_fingerprint
        );
        // Unreferenced files must not create a different catalog on WASM.
        let mut sources = embedded_bundle();
        sources.insert("game/units/unlisted.json".into(), "{}".into());
        let selected = documents::collect_bundle(|path| {
            sources
                .get(path)
                .cloned()
                .ok_or_else(|| ContentError::new(path, "missing document"))
        })
        .unwrap();
        assert!(!selected.contains_key("game/units/unlisted.json"));
        assert_eq!(native.content_fingerprint, content_fingerprint(&selected));
    }
}
