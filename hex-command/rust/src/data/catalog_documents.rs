//! Typed assembly of current scenario/stage components.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioDocuments {
    map: String,
    roster: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageDocuments {
    id: String,
    map: String,
    roster: String,
    objectives: String,
    story: String,
    scripts: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MapDocument {
    #[serde(default)]
    factions: Vec<Faction>,
    map_id: String,
    map_revision: u32,
    cols: i32,
    rows: i32,
    tiles: Vec<Tile>,
    #[serde(default)]
    lighting: BattlefieldLighting,
    #[serde(default)]
    hidden: bool,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "description")]
    description: Option<String>,
    #[serde(rename = "revision")]
    _revision: Option<u32>,
}
impl MapDocument {
    fn initial(self, roster: InitialRoster) -> InitialState {
        InitialState {
            factions: self.factions,
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitialRoster {
    gold: Gold,
    units: Vec<InitialUnit>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageRoster {
    initial: InitialRoster,
    units: Vec<StartingUnit>,
    recruits: Vec<String>,
    enemy_recruits: Vec<String>,
    gold: i32,
    enemy_gold: i32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageObjectives {
    goals: Vec<Goal>,
    #[serde(default)]
    optional_goals: Vec<Goal>,
    #[serde(default)]
    required_companions: Vec<String>,
    #[serde(default)]
    defend_objectives: bool,
    enemy_leader_post: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageStory {
    #[serde(rename = "subtitle")]
    subtitle: Option<String>,
    briefing: String,
    debrief: Option<String>,
    hints: Vec<String>,
    intro: Vec<Dialogue>,
    victory: Vec<Dialogue>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageScripts {
    #[serde(default)]
    enemy_groups: Vec<EnemyGroup>,
    events: Vec<Event>,
    waves: Vec<Wave>,
    companion_losses: Option<BTreeMap<String, MissionPatch>>,
    refuge: Option<Refuge>,
    without_dain: Option<MissionPatch>,
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
pub(super) fn stage_path(campaign: &str, stage: &str) -> Result<String, ContentError> {
    sibling(campaign, stage)
}
/// Load precisely the documents referenced by the manifest; unrelated files
/// never affect native-versus-embedded catalog fingerprints.
pub(super) fn collect_bundle(
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
    if manifest.schema_version != 2 {
        return Err(ContentError::new(
            "manifest.json",
            "unsupported schemaVersion",
        ));
    }
    for path in [
        &manifest.terrain,
        &manifest.simulation.rosters,
        &manifest.simulation.arena,
        &manifest.simulation.campaign,
    ]
    .into_iter()
    .chain(manifest.recruitment.iter())
    .chain(&manifest.races)
    .chain(&manifest.units)
    .chain(&manifest.scenarios)
    .chain(&manifest.arena_scenarios)
    .chain(&manifest.campaigns)
    {
        read(&mut bundle, &mut source, path)?;
    }
    for path in manifest.scenarios.iter().chain(&manifest.arena_scenarios) {
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
            for component in [
                &documents.map,
                &documents.roster,
                &documents.objectives,
                &documents.story,
                &documents.scripts,
            ] {
                read(&mut bundle, &mut source, &sibling(&path, component)?)?;
            }
        }
    }
    Ok(bundle)
}
pub(super) fn scenario(
    bundle: &JsonBundle,
    path: &str,
) -> Result<(InitialState, String), ContentError> {
    let documents: ScenarioDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    map.lighting
        .validate(&map.tiles, &[])
        .map_err(|message| ContentError::new(&map_path, message))?;
    for tile in &map.tiles {
        if let Some(rule) = &tile.fog {
            rule.condition
                .validate(&map.tiles, &[], true)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let roster_path = sibling(path, &documents.roster)?;
    let roster: InitialRoster = parse(bundle, &roster_path)?;
    Ok((map.initial(roster), roster_path))
}
pub(super) fn stage(bundle: &JsonBundle, path: &str) -> Result<StageFile, ContentError> {
    let documents: StageDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    let roster: StageRoster = parse(bundle, &sibling(path, &documents.roster)?)?;
    let objectives: StageObjectives = parse(bundle, &sibling(path, &documents.objectives)?)?;
    let story: StageStory = parse(bundle, &sibling(path, &documents.story)?)?;
    let scripts: StageScripts = parse(bundle, &sibling(path, &documents.scripts)?)?;
    map.lighting
        .validate(&map.tiles, &scripts.events)
        .map_err(|message| ContentError::new(&map_path, message))?;
    for tile in &map.tiles {
        if let Some(rule) = &tile.fog {
            rule.condition
                .validate(&map.tiles, &scripts.events, true)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let layout = Layout {
        id: map.map_id.clone(),
        units: roster.units,
    };
    Ok(StageFile {
        id: documents.id,
        state: map.initial(roster.initial),
        mission: Mission {
            map: layout,
            subtitle: story.subtitle.unwrap_or_default(),
            briefing: story.briefing,
            debrief: story.debrief,
            hints: story.hints,
            intro: story.intro,
            victory: story.victory,
            required_companions: objectives.required_companions,
            goals: objectives.goals,
            optional_goals: objectives.optional_goals,
            defend_objectives: objectives.defend_objectives,
            enemy_leader_post: objectives.enemy_leader_post,
            recruits: roster.recruits,
            enemy_recruits: roster.enemy_recruits,
            gold: roster.gold,
            enemy_gold: roster.enemy_gold,
            events: scripts.events,
            enemy_groups: scripts.enemy_groups,
            waves: scripts.waves,
            companion_losses: scripts.companion_losses,
            refuge: scripts.refuge,
            without_dain: scripts.without_dain,
        },
    })
}

/// Preserve authored display text and placements while the catalog still owns the documents.
pub(super) fn map_metadata(
    bundle: &JsonBundle,
    descriptor: &str,
    state: &State,
    units: Option<Vec<StartingUnit>>,
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
                    .find(|tile| tile.id == unit.tile)
                    .ok_or_else(|| {
                        ContentError::new(descriptor, format!("invalid unit placement {}", unit.id))
                    })?;
                Ok(StartingUnit {
                    items: unit.items.clone(),
                    ai_group: unit.ai_group.clone(),
                    faction: unit.faction.clone(),
                    col: tile.col,
                    row: tile.row,
                    r#type: unit.r#type.clone(),
                    side: unit.side,
                    name: unit.name.clone(),
                    role: unit.role,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?,
    };
    let presentation =
        crate::logic::conditions::resolve(&crate::logic::conditions::PresentationInput {
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
        hidden: metadata.hidden,
        cols: state.cols,
        rows: state.rows,
        terrain: state.tiles.clone(),
        units,
        lighting: metadata.lighting,
    })
}
