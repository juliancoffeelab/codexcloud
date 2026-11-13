//! Typed assembly of current scenario/stage components.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioDocuments {
    human: bool,
    #[serde(default)]
    characters: Vec<Character>,
    map: String,
    roster: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageDocuments {
    id: String,
    map: String,
    roster: String,
    objectives: Option<String>,
    story: String,
    scripts: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MapDocument {
    map_id: String,
    map_revision: u32,
    cols: i32,
    rows: i32,
    #[serde(deserialize_with = "authored_tiles")]
    tiles: Vec<Tile>,
    #[serde(default)]
    lighting: MapLighting,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "description")]
    description: Option<String>,
    #[serde(rename = "revision")]
    _revision: Option<u32>,
}
/// Authoring names are optional; transport cells always have coordinate addresses.
/// This conversion runs only while loading source content, never while restoring saves.
fn authored_tiles<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Tile>, D::Error> {
    use serde::de::Error;
    let tiles = Vec::<serde_json::Map<String, serde_json::Value>>::deserialize(deserializer)?;
    tiles
        .into_iter()
        .map(|mut tile| {
            if tile.contains_key("location") {
                return Err(D::Error::custom(
                    "map tiles use optional id, not runtime location",
                ));
            }
            if let Some(id) = tile.remove("id") {
                let name = id
                    .as_str()
                    .ok_or_else(|| D::Error::custom("tile id must be a name"))?;
                identifier(name, "map tile").map_err(D::Error::custom)?;
                tile.insert("location".into(), id);
            }
            let col = tile
                .get("col")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| D::Error::custom("tile requires integer col"))?;
            let row = tile
                .get("row")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| D::Error::custom("tile requires integer row"))?;
            tile.insert("id".into(), format!("{col},{row}").into());
            serde_json::from_value(serde_json::Value::Object(tile)).map_err(D::Error::custom)
        })
        .collect()
}

impl MapDocument {
    fn initial(self, roster: InitialRoster) -> InitialState {
        InitialState {
            vision_groups: roster.vision_groups,
            factions: roster.factions,
            map_id: self.map_id,
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            tiles: self.tiles,
            gold: roster.gold,
            units: roster.units,
        }
    }
}
/// One authored army owns its controller, economy, recruitment and placements.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RosterDocument {
    #[serde(rename = "visionGroups")]
    vision_groups: Vec<VisionGroupState>,
    factions: Vec<RosterFaction>,
    #[serde(default)]
    arrivals: Vec<CompanionArrival>,
}
#[derive(Deserialize)]
struct RosterFaction {
    #[serde(flatten)]
    faction: Faction,
    gold: i32,
    units: Vec<RosterUnit>,
}
#[derive(Deserialize)]
struct RosterUnit {
    #[serde(default)]
    items: Vec<String>,
    #[serde(flatten)]
    unit: InitialUnit,
}
struct InitialRoster {
    vision_groups: Vec<VisionGroupState>,
    factions: Vec<Faction>,
    gold: Gold,
    units: Vec<InitialUnit>,
}
impl RosterDocument {
    fn resolve(
        self,
        path: &str,
        items: &HashMap<String, Item>,
    ) -> Result<(InitialRoster, Vec<StartingUnit>, Vec<CompanionArrival>), ContentError> {
        unique(
            self.factions.iter().map(|entry| entry.faction.id.as_str()),
            path,
        )?;
        let mut factions = Vec::new();
        let mut gold = BTreeMap::new();
        let mut initial = Vec::new();
        let mut placements = Vec::new();
        for entry in self.factions {
            let faction = entry.faction;
            if entry.gold < 0 || faction.recruits.is_none() {
                return Err(ContentError::new(
                    path,
                    "each faction requires nonnegative gold and a recruit list",
                ));
            }
            gold.insert(faction.id.clone(), entry.gold);
            for source in entry.units {
                let mut unit = source.unit;
                unit.items = source
                    .items
                    .iter()
                    .map(|id| {
                        items
                            .get(id)
                            .cloned()
                            .ok_or_else(|| ContentError::new(path, format!("unknown item {id}")))
                    })
                    .collect::<Result<_, _>>()?;
                if unit.side.is_some() || unit.faction.is_some() {
                    return Err(ContentError::new(
                        path,
                        "roster units inherit side and faction from their army",
                    ));
                }
                unit.side = Some(faction.phase);
                unit.faction = Some(faction.id.clone());
                placements.push(StartingUnit {
                    items: unit.items.clone(),
                    faction: unit.faction.clone(),
                    tile: unit.tile.clone(),
                    r#type: unit.r#type.clone(),
                    side: faction.phase,
                    ai_group: unit.ai_group.clone(),
                    character: unit.character.clone(),
                    role: unit.role,
                });
                initial.push(unit);
            }
            factions.push(faction);
        }
        Ok((
            InitialRoster {
                vision_groups: self.vision_groups,
                factions,
                gold: Gold(std::sync::Arc::new(gold)),
                units: initial,
            },
            placements,
            self.arrivals,
        ))
    }
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageObjectives {
    #[serde(default)]
    lose_conditions: Vec<LossCondition>,
    goals: Vec<Goal>,
    #[serde(default)]
    optional_goals: Vec<Goal>,
    #[serde(default)]
    required_companions: Vec<String>,
    #[serde(default)]
    defend_objectives: bool,
    enemy_leader_post: Option<TileRef>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageStory {
    #[serde(rename = "subtitle")]
    subtitle: Option<String>,
    briefing: ConditionalText,
    debrief: Option<ConditionalText>,
    hints: Vec<ConditionalHint>,
    intro: Vec<Speech>,
    victory: Vec<Speech>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageScripts {
    #[serde(default)]
    enemy_groups: Vec<EnemyGroup>,
    events: Vec<ScriptEvent>,
    #[serde(default)]
    required_companions: Vec<String>,
    #[serde(default)]
    defend_objectives: bool,
    #[serde(default)]
    enemy_leader_post: Option<TileRef>,
}

/// Component references are local to their descriptor, with traversal rejected.
fn sibling(descriptor: &str, component: &str) -> Result<String, ContentError> {
    checked_path(descriptor)?;
    checked_path(component)?;
    let path = Path::new(descriptor)
        .parent()
        .unwrap_or(Path::new(""))
        .join(component);
    let path = path
        .to_str()
        .ok_or_else(|| ContentError::new(descriptor, "invalid component path"))?
        .replace('\\', "/");
    checked_path(&path)?;
    Ok(path)
}
pub fn stage_path(campaign: &str, stage: &str) -> Result<String, ContentError> {
    sibling(campaign, stage)
}
/// Load precisely the documents referenced by the manifest; unrelated files
/// never affect native-versus-embedded catalog fingerprints.
pub fn collect_bundle(
    mut source: impl FnMut(&str) -> Result<String, ContentError>,
) -> Result<JsonBundle, ContentError> {
    fn read(
        bundle: &mut JsonBundle,
        source: &mut impl FnMut(&str) -> Result<String, ContentError>,
        path: &str,
    ) -> Result<(), ContentError> {
        checked_path(path)?;
        if !bundle.contains_key(path) {
            bundle.insert(path.to_owned(), source(path)?);
        }
        Ok(())
    }
    let mut bundle = JsonBundle::new();
    read(&mut bundle, &mut source, "manifest.json")?;
    let manifest: Manifest = parse(&bundle, "manifest.json")?;
    if manifest.schema_version != 3 {
        return Err(ContentError::new(
            "manifest.json",
            "unsupported schemaVersion",
        ));
    }
    for path in [
        &manifest.terrain,
        &manifest.concealment_kinds,
        &manifest.simulation.rosters,
        &manifest.simulation.arena,
        &manifest.simulation.campaign,
        &manifest.simulation.rl,
    ]
    .into_iter()
    .chain(manifest.recruitment.iter())
    .chain(&manifest.races)
    .chain(&manifest.units)
    .chain(&manifest.items)
    .chain(&manifest.scenarios)
    .chain(&manifest.campaigns)
    {
        read(&mut bundle, &mut source, path)?;
    }
    for path in &manifest.scenarios {
        let documents: ScenarioDocuments = parse(&bundle, path)?;
        for component in [&documents.map, &documents.roster] {
            read(&mut bundle, &mut source, &sibling(path, component)?)?;
        }
    }
    for campaign_path in &manifest.campaigns {
        let campaign: CampaignFile = parse(&bundle, campaign_path)?;
        for stage in campaign.stages {
            let path = stage_path(campaign_path, &stage)?;
            read(&mut bundle, &mut source, &path)?;
            let documents: StageDocuments = parse(&bundle, &path)?;
            if let Some(component) = &documents.objectives {
                read(&mut bundle, &mut source, &sibling(&path, component)?)?;
            }
            for component in [
                &documents.map,
                &documents.roster,
                &documents.story,
                &documents.scripts,
            ] {
                read(&mut bundle, &mut source, &sibling(&path, component)?)?;
            }
        }
    }
    Ok(bundle)
}
pub fn scenario(
    bundle: &JsonBundle,
    path: &str,
    items: &HashMap<String, Item>,
) -> Result<(InitialState, String, bool, Vec<Character>), ContentError> {
    let documents: ScenarioDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    map.lighting
        .validate(&map.tiles)
        .map_err(|message| ContentError::new(&map_path, message))?;
    for tile in &map.tiles {
        if let Some(light) = &tile.light {
            light
                .validate(&map.tiles, &[])
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let roster_path = sibling(path, &documents.roster)?;
    let roster: RosterDocument = parse(bundle, &roster_path)?;
    let (roster, _, _) = roster.resolve(&roster_path, items)?;
    validate_characters(&documents.characters, path)?;
    Ok((
        map.initial(roster),
        roster_path,
        documents.human,
        documents.characters,
    ))
}
pub fn stage(
    bundle: &JsonBundle,
    path: &str,
    items: &HashMap<String, Item>,
) -> Result<StageFile, ContentError> {
    let documents: StageDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    let roster_path = sibling(path, &documents.roster)?;
    let document: RosterDocument = parse(bundle, &roster_path)?;
    let (roster, placements, arrivals) = document.resolve(&roster_path, items)?;
    let gold = roster.gold.clone();
    let recruitment = roster
        .factions
        .iter()
        .map(|f| (f.id.clone(), f.recruits.clone().unwrap()))
        .collect();
    let objectives: StageObjectives = match &documents.objectives {
        Some(component) => parse(bundle, &sibling(path, component)?)?,
        None => StageObjectives::default(),
    };
    if !objectives.goals.is_empty()
        || !objectives.optional_goals.is_empty()
        || !objectives.required_companions.is_empty()
        || objectives.defend_objectives
        || objectives.enemy_leader_post.is_some()
    {
        return Err(ContentError::new(
            path,
            "legacy objective fields are unsupported; author objectives and protection in scripts",
        ));
    }
    let story: StageStory = parse(bundle, &sibling(path, &documents.story)?)?;
    let scripts: StageScripts = parse(bundle, &sibling(path, &documents.scripts)?)?;
    let events = vec![];
    let mut scripted_events = scripts.events;
    if scripted_events.is_empty() {
        return Err(ContentError::new(
            path,
            "chapters require scripted events; legacy execution is unsupported",
        ));
    }
    unique(scripted_events.iter().map(|e| e.id.as_str()), &map_path)?;
    scripted_events.sort_by_key(|e| std::cmp::Reverse(e.priority));
    for e in &mut scripted_events {
        e.when.index_holds();
        e.validate(
            &map.tiles,
            &roster.factions,
            &roster.vision_groups,
            items,
            &map_path,
        )?;
    }
    map.lighting
        .validate(&map.tiles)
        .map_err(|message| ContentError::new(&map_path, message))?;
    for tile in &map.tiles {
        if let Some(light) = &tile.light {
            light
                .validate(&map.tiles, &events)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let mut item_ids: Vec<String> = placements
        .iter()
        .flat_map(|u| u.items.iter().map(|i| i.id.clone()))
        .collect();
    for event in &scripted_events {
        for action in event.actions_recursive() {
            match action {
                EventAction::Give(v) => item_ids.push(v.item.clone()),
                EventAction::PlaceItem(v) => item_ids.push(v.item.clone()),
                EventAction::Spawn(v) => {
                    for u in &v.units {
                        item_ids.extend(u.member().items);
                    }
                }
                _ => {}
            }
        }
    }
    item_ids.sort();
    item_ids.dedup();
    let layout = Layout {
        id: map.map_id.clone(),
        units: placements,
    };
    Ok(StageFile {
        id: documents.id,
        state: map.initial(roster),
        mission: Mission {
            item_ids,
            scripted_events,
            lose_conditions: objectives.lose_conditions,
            map: layout,
            subtitle: story.subtitle.unwrap_or_default(),
            briefing: story.briefing,
            debrief: story.debrief,
            hints: story.hints,
            intro: story.intro,
            victory: story.victory,
            required_companions: scripts.required_companions,
            arrivals,
            goals: objectives.goals,
            optional_goals: objectives.optional_goals,
            defend_objectives: scripts.defend_objectives,
            enemy_leader_post: scripts.enemy_leader_post,
            recruitment,
            gold,
            events,
            enemy_groups: scripts.enemy_groups,
            waves: vec![],
            companion_losses: None,
            refuge: None,
            without_dain: None,
        },
    })
}

/// Preserve authored display text and placements while the catalog still owns the documents.
pub fn map_metadata(
    bundle: &JsonBundle,
    descriptor: &str,
    state: &State,
    units: Option<Vec<StartingUnit>>,
    human: bool,
) -> Result<MapDefinition, ContentError> {
    #[derive(Deserialize)]
    struct Reference {
        map: String,
    }
    let reference: Reference = parse(bundle, descriptor)?;
    let metadata: MapDocument = parse(bundle, &sibling(descriptor, &reference.map)?)?;
    let units = match units {
        Some(units) => units,
        None => state
            .units
            .iter()
            .map(|unit| {
                let tile = state
                    .tiles
                    .iter()
                    .find(|tile| tile.hex() == unit.tile)
                    .ok_or_else(|| {
                        ContentError::new(descriptor, format!("invalid unit placement {}", unit.id))
                    })?;
                Ok(StartingUnit {
                    items: unit.items.clone(),
                    ai_group: unit.ai_group.clone(),
                    faction: Some(unit.faction.clone()),
                    tile: tile.hex(),
                    r#type: unit.r#type.clone(),
                    side: unit.side,
                    character: unit.character.clone(),
                    role: unit.role,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?,
    };
    let presentation =
        crate::data::presentation::resolve(&crate::data::presentation::PresentationInput {
            observer: None,
            vision_groups: state.vision_groups.clone(),
            factions: state.factions.clone(),
            pending: vec![],
            tiles: state.tiles.clone(),
            units: units.clone(),
            lighting: metadata.lighting.clone(),
            marks: vec![],
            choices: vec![],
            memory: vec![],
            round: 1,
        })
        .0;
    Ok(MapDefinition {
        presentation,
        id: state.map_id.clone(),
        revision: state.map_revision,
        name: metadata.name.unwrap_or_else(|| state.map_id.clone()),
        description: metadata.description.unwrap_or_default(),
        human,
        cols: state.cols,
        rows: state.rows,
        terrain: state.tiles.clone(),
        units,
        lighting: metadata.lighting,
    })
}
