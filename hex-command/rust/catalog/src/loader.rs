//! Typed parsing and reference resolution. Authored semantic validation belongs in catalog tests.
use crate::schema;
use crate::world::{CampaignSource, MapSource};
use crate::*;
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::path::{Component, Path};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    terrain: Text,
    concealment_kinds: Text,
    races: Vec<Text>,
    units: Vec<Text>,
    abilities: Vec<Text>,
    environment_effects: Vec<Text>,
    items: Vec<Text>,
    scenarios: Vec<Text>,
    campaigns: Vec<Text>,
    simulation: SimulationPaths,
    recruitment: Text,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulationPaths {
    #[serde(rename = "rosters")]
    rosters: Text,
    #[serde(rename = "arena")]
    arena: Text,
    #[serde(rename = "campaign")]
    campaign: Text,
    #[serde(rename = "rl")]
    rl: Text,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitSource {
    sight: SightContribution,
    abilities: Vec<Text>,
    id: Text,
    name: Text,
    hp: i32,
    r#move: usize,
    cost: i32,
    attacks: Vec<schema::Weapon>,
    damage_modifiers: DamageModifiers,
    race: Race,
    role: UnitRole,
    terrain_modifiers: std::collections::BTreeMap<TerrainRole, TerrainStatModifiers>,
    level: u32,
    family: Family,
    experience_needed: u32,
    advances_to: Vec<Text>,
    ability: Text,
    description: Text,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TerrainSource {
    id: Terrain,
    name: Text,
    description: Text,
    role: TerrainRole,
    cost: u16,
    passable: bool,
    capturable: bool,
    blocks_shots: bool,
    income: i32,
    damage_modifiers: EnvironmentalModifiers,
    environment_effects: Vec<Text>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemSource {
    id: Text,
    name: Text,
    description: Text,
    attacks: Vec<schema::Weapon>,
    #[serde(deserialize_with = "crate::schema::explicit_option")]
    armor: Option<schema::ItemArmor>,
    abilities: Vec<Text>,
    sight: SightContribution,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioSource {
    tags: Vec<Text>,
    characters: Vec<Character>,
    map: Text,
    roster: Text,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterSource {
    id: Text,
    map: Text,
    roster: Text,
    story: Text,
    scripts: Text,
}
fn document<T: DeserializeOwned>(bundle: &JsonBundle, path: &str) -> Result<T, ContentError> {
    let text = bundle
        .get(path)
        .ok_or_else(|| ContentError::new(path, "missing authored document"))?;
    serde_json::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))
}
fn sibling(path: &str, name: &str) -> Text {
    let joined = Path::new(path)
        .parent()
        .expect("document has directory")
        .join(name);
    let mut resolved = std::path::PathBuf::new();
    for part in joined.components() {
        match part {
            Component::Normal(v) => resolved.push(v),
            Component::ParentDir => {
                assert!(resolved.pop(), "authored reference stays within data root")
            }
            Component::CurDir => {}
            _ => panic!("authored reference must be relative"),
        }
    }
    resolved
        .to_str()
        .expect("authored paths are UTF-8")
        .replace('\\', "/")
}
fn resolve_abilities(
    ids: &[Text],
    all: &[schema::AbilityDefinition],
) -> Vec<schema::AbilityDefinition> {
    ids.iter()
        .map(|id| {
            all.iter()
                .find(|a| a.id == *id)
                .expect("authored ability reference resolves")
                .clone()
        })
        .collect()
}
pub(crate) fn load_from(root: impl AsRef<Path>) -> Result<Catalogue, ContentError> {
    fn read(root: &Path, directory: &Path, bundle: &mut JsonBundle) -> Result<(), ContentError> {
        for entry in std::fs::read_dir(directory)
            .map_err(|e| ContentError::new(directory.display().to_string(), e.to_string()))?
        {
            let path = entry
                .map_err(|e| ContentError::new(directory.display().to_string(), e.to_string()))?
                .path();
            if path.is_dir() {
                read(root, &path, bundle)?
            } else if path.extension().is_some_and(|x| x == "json") {
                let name = path
                    .strip_prefix(root)
                    .expect("document below root")
                    .to_str()
                    .expect("authored path UTF-8")
                    .replace('\\', "/");
                bundle.insert(
                    name,
                    std::fs::read_to_string(&path).map_err(|e| {
                        ContentError::new(path.display().to_string(), e.to_string())
                    })?,
                );
            }
        }
        Ok(())
    }
    let root = root.as_ref();
    let mut bundle = JsonBundle::new();
    read(root, root, &mut bundle)?;
    parse(&bundle)
}
pub(crate) fn parse(bundle: &JsonBundle) -> Result<Catalogue, ContentError> {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let instance_identity = NEXT
        .fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |id| id.checked_add(1),
        )
        .expect("Catalogue instance identity space");
    let mut digest = Sha256::new();
    for (path, source) in bundle {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(source.as_bytes());
    }
    let content_fingerprint = digest.finalize().into();
    let manifest: Manifest = document(bundle, "manifest.json")?;
    if manifest.schema_version != 6 {
        return Err(ContentError::new(
            "manifest.json",
            "unsupported authored schemaVersion",
        ));
    }
    let concealment =
        document::<Vec<schema::ConcealmentKindDefinition>>(bundle, &manifest.concealment_kinds)?
            .into_iter()
            .map(|source| crate::ConcealmentDefinition { source })
            .collect();
    let abilities = manifest
        .abilities
        .iter()
        .map(|p| document(bundle, p))
        .collect::<Result<Vec<schema::AbilityDefinition>, _>>()?;
    let environment = manifest
        .environment_effects
        .iter()
        .map(|p| document(bundle, p).map(|source| EnvironmentDefinition { source }))
        .collect::<Result<Vec<_>, _>>()?;
    let races = manifest
        .races
        .iter()
        .map(|p| {
            let source: schema::RaceDefinition = document(bundle, p)?;
            Ok(RaceDefinition {
                abilities: resolve_abilities(&source.abilities, &abilities),
                source,
            })
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    let sources: Vec<TerrainSource> = document(bundle, &manifest.terrain)?;
    let terrain = sources
        .into_iter()
        .map(|s| {
            let effects = s
                .environment_effects
                .iter()
                .flat_map(|id| {
                    &environment
                        .iter()
                        .find(|e| e.id() == id)
                        .expect("terrain environmental reference resolves")
                        .source
                        .effects
                })
                .map(|effect| match effect {
                    schema::EnvironmentEffect::Unit { effect } => effect.clone(),
                    _ => panic!("terrain-type effect must be a unit contribution"),
                })
                .collect();
            TerrainDefinition {
                instance_identity,
                source: schema::TerrainDefinition {
                    id: s.id,
                    name: s.name,
                    description: s.description,
                    role: s.role,
                    cost: s.cost,
                    passable: s.passable,
                    capturable: s.capturable,
                    blocks_shots: s.blocks_shots,
                    income: s.income,
                    damage_modifiers: s.damage_modifiers,
                    effects,
                },
            }
        })
        .collect();
    let items = manifest
        .items
        .iter()
        .map(|p| {
            let s: ItemSource = document(bundle, p)?;
            let effects = resolve_abilities(&s.abilities, &abilities)
                .into_iter()
                .flat_map(|a| a.effects)
                .collect();
            Ok(ItemDefinition {
                instance_identity,
                source: schema::Item {
                    id: s.id,
                    name: s.name,
                    description: s.description,
                    attacks: s.attacks,
                    armor: s.armor,
                    sight: s.sight,
                    effects,
                },
            })
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    let units = manifest
        .units
        .iter()
        .map(|p| {
            let s: UnitSource = document(bundle, p)?;
            let race = races
                .iter()
                .find(|r| r.id() == s.race)
                .expect("authored unit race resolves");
            Ok(UnitDefinition {
                instance_identity,
                max_hp: s.hp,
                max_xp: s.experience_needed,
                movement: s.r#move,
                price: s.cost,
                metadata: schema::UnitMetadata {
                    id: s.id,
                    name: s.name,
                    description: s.description,
                    ability: s.ability,
                    family: s.family,
                    race: s.race,
                    role: s.role,
                    level: s.level,
                    advances_to: s.advances_to,
                },
                modifiers: StatModifiers {
                    sight: s.sight,
                    damage_modifiers: s.damage_modifiers,
                    terrain_modifiers: s.terrain_modifiers,
                },
                abilities: resolve_abilities(&s.abilities, &abilities),
                attacks: s.attacks,
                racial: race.source.clone(),
                racial_abilities: race.abilities.clone(),
            })
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    let mut maps = Vec::new();
    for p in &manifest.scenarios {
        let source: ScenarioSource = document(bundle, p)?;
        maps.push(MapDefinition {
            source: document(bundle, &sibling(p, &source.map))?,
            tags: source.tags,
            roster: document(bundle, &sibling(p, &source.roster))?,
            characters: source.characters,
        });
    }
    let mut campaigns = Vec::new();
    for p in &manifest.campaigns {
        let source: CampaignSource = document(bundle, p)?;
        let mut chapters = Vec::new();
        for stage in &source.stages {
            let path = sibling(p, stage);
            let chapter: ChapterSource = document(bundle, &path)?;
            let map: MapSource = document(bundle, &sibling(&path, &chapter.map))?;
            let id = map.map_id.clone();
            let map_revision = map.map_revision;
            maps.push(MapDefinition {
                source: map,
                tags: source.tags.clone(),
                roster: document(bundle, &sibling(&path, &chapter.roster))?,
                characters: source.characters.clone(),
            });
            chapters.push(ChapterDefinition {
                id: chapter.id,
                map: id,
                map_revision,
                story: document(bundle, &sibling(&path, &chapter.story))?,
                scripts: document(bundle, &sibling(&path, &chapter.scripts))?,
            });
        }
        campaigns.push(CampaignDefinition { source, chapters });
    }
    let simulation = SimulationSettings {
        rosters: document(bundle, &manifest.simulation.rosters)?,
        arena: document(bundle, &manifest.simulation.arena)?,
        campaign: document(bundle, &manifest.simulation.campaign)?,
        training: document(bundle, &manifest.simulation.rl)?,
    };
    let recruiter_weights = document(bundle, &manifest.recruitment)?;
    let _ = manifest.concealment_kinds;
    Ok(Catalogue {
        instance_identity,
        combat_preparations: std::sync::Mutex::new(rustc_hash::FxHashMap::default()),
        meta: CatalogueMeta {
            recruiter_weights,
            simulation,
        },
        content_fingerprint,
        units,
        races,
        terrain,
        items,
        abilities: abilities
            .into_iter()
            .map(|source| AbilityDefinition { source })
            .collect(),
        environment,
        maps,
        campaigns,
        concealment,
    })
}
