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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScenarioSource {
    tags: Vec<Text>,
    characters: Vec<Character>,
    map: Text,
    roster: Text,
    fixed_armies: bool,
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
    if manifest.schema_version != 7 {
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
            fixed_armies: source.fixed_armies,
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
                fixed_armies: true,
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
    let training = &simulation.training;
    let valid_allocation = |group: &Text, map: &Text, reward: &Text| {
        training.sampling_weights.contains_key(group)
            && training
                .map_weights
                .get(map)
                .is_some_and(|w| w.is_finite() && *w > 0.)
            && training.reward_profiles.contains_key(reward)
    };
    if training.reward_profiles.is_empty()
        || training
            .reward_profiles
            .values()
            .any(|r| !r.win.is_finite() || r.win <= 0. || !r.chapter.is_finite() || r.chapter < 0.)
        || training
            .map_weights
            .values()
            .any(|w| !w.is_finite() || *w <= 0.)
    {
        return Err(ContentError::new(
            &manifest.simulation.rl,
            "invalid reward profiles or map weights",
        ));
    }
    if !training.adaptive_sampling.valid() {
        return Err(ContentError::new(
            &manifest.simulation.rl,
            "invalid adaptiveSampling settings",
        ));
    }
    if training.sampling_weights.is_empty()
        || training
            .sampling_weights
            .values()
            .any(|w| !w.is_finite() || *w <= 0.)
        || !training.sampling_weights.contains_key("arena")
        || training.arenas.iter().any(|a| {
            !a.pretrain_imitator.valid()
                || !a.condition_weight.is_finite()
                || a.condition_weight <= 0.
                || matches!(a.pretrain_imitator, crate::PretrainImitator::Path { .. })
                || !valid_allocation(&a.sampling_group, &a.sampling_map, &a.reward_profile)
                || !simulation.arena.scenarios.iter().any(|s| s.id == a.id)
                || a.conditions.as_ref().is_some_and(|c| {
                    c.max_rounds == 0
                        || c.first.is_empty()
                        || c.first
                            .iter()
                            .any(|p| !matches!(p.as_str(), "player" | "ai"))
                        || c.objectives.is_empty()
                        || c.objectives
                            .iter()
                            .any(|o| !training.objectives.contains(o))
                        || c.opponents.is_empty()
                        || c.opponents.iter().any(|o| !training.opponents.contains(o))
                        || c.recruiters.is_empty()
                        || c.recruiters
                            .iter()
                            .any(|r| !training.recruiters.contains(r))
                })
        })
    {
        return Err(ContentError::new(
            &manifest.simulation.rl,
            "invalid sampling weights or arena pretrain_imitator",
        ));
    }
    for (campaign_id, variants) in &training.campaigns {
        for variant in variants {
            if !variant.pretrain_imitator.valid()
                || !variant.condition_weight.is_finite()
                || variant.condition_weight <= 0.
                || !valid_allocation(
                    &variant.sampling_group,
                    &variant.sampling_map,
                    &variant.reward_profile,
                )
                || variant.recruiters.as_ref().is_some_and(|rs| {
                    rs.is_empty() || rs.iter().any(|r| !training.recruiters.contains(r))
                })
                || !campaigns.iter().any(|c| c.id() == campaign_id)
            {
                return Err(ContentError::new(
                    &manifest.simulation.rl,
                    "unknown pretrain_imitator or samplingGroup",
                ));
            }
            let overlapping = variants
                .iter()
                .filter(|v| v.objective == variant.objective && v.diplomacy == variant.diplomacy)
                .filter(|v| {
                    v.opponents
                        .as_ref()
                        .unwrap_or(&training.opponents)
                        .iter()
                        .any(|o| {
                            variant
                                .opponents
                                .as_ref()
                                .unwrap_or(&training.opponents)
                                .contains(o)
                        })
                })
                .count();
            if overlapping != 1 {
                return Err(ContentError::new(
                    &manifest.simulation.rl,
                    "overlapping campaign training conditions",
                ));
            }
            if let crate::PretrainImitator::Path { path_to } = &variant.pretrain_imitator {
                let campaign = campaigns.iter().find(|c| c.id() == campaign_id);
                let valid_target = campaign.is_some_and(|c| {
                    c.chapters().len() == 1 && {
                        let chapter = &c.chapters()[0];
                        maps.iter()
                            .find(|m| {
                                m.id() == chapter.map_id() && m.revision() == chapter.map_revision()
                            })
                            .is_some_and(|m| {
                                m.tiles()
                                    .iter()
                                    .any(|t| t.col == path_to.col && t.row == path_to.row)
                            })
                    }
                });
                if !valid_target {
                    return Err(ContentError::new(
                        &manifest.simulation.rl,
                        "path_to requires an authored tile in a single-chapter campaign",
                    ));
                }
            }
            if let Some(opponents) = &variant.opponents {
                if opponents.is_empty()
                    || opponents.iter().enumerate().any(|(i, name)| {
                        !simulation.training.opponents.contains(name)
                            || opponents[..i].contains(name)
                    })
                {
                    return Err(ContentError::new(
                        &manifest.simulation.rl,
                        "campaign opponents must be a nonempty, unique subset of global opponents",
                    ));
                }
            }
        }
    }
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
