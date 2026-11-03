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
    map_id: String,
    map_revision: u32,
    cols: i32,
    rows: i32,
    tiles: Vec<Tile>,
    #[serde(default)]
    lighting: MapLighting,
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
    factions: Vec<RosterFaction>,
    #[serde(default)]
    arrivals: Vec<CompanionArrival>,
}
#[derive(Deserialize)]
struct RosterFaction {
    #[serde(flatten)]
    faction: Faction,
    gold: i32,
    units: Vec<InitialUnit>,
}
struct InitialRoster {
    factions: Vec<Faction>,
    gold: Gold,
    units: Vec<InitialUnit>,
}
impl RosterDocument {
    fn resolve(
        self,
        path: &str,
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
            for mut unit in entry.units {
                if unit.side.is_some() || unit.faction.is_some() {
                    return Err(ContentError::new(
                        path,
                        "roster units inherit side and faction from their army",
                    ));
                }
                unit.side = Some(faction.phase);
                unit.faction = (!matches!(faction.id.as_str(), "player" | "enemy"))
                    .then(|| faction.id.clone());
                placements.push(StartingUnit {
                    items: unit.items.clone(),
                    faction: unit.faction.clone(),
                    col: unit.col,
                    row: unit.row,
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
                factions,
                gold: Gold(std::sync::Arc::new(gold)),
                units: initial,
            },
            placements,
            self.arrivals,
        ))
    }
}
#[derive(Deserialize)]
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
    rituals: Vec<Ritual>,
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
pub fn scenario(bundle: &JsonBundle, path: &str) -> Result<(InitialState, String), ContentError> {
    let documents: ScenarioDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    for tile in &map.tiles {
        if let Some(light) = &tile.light {
            light
                .validate(&map.tiles, &[])
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
        if let Some(rule) = &tile.fog {
            rule.condition
                .validate_presentation()
                .map_err(|message| ContentError::new(&map_path, message))?;
            rule.condition
                .validate(&map.tiles, &[], true)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let roster_path = sibling(path, &documents.roster)?;
    let roster: RosterDocument = parse(bundle, &roster_path)?;
    let (roster, _, _) = roster.resolve(&roster_path)?;
    Ok((map.initial(roster), roster_path))
}
pub fn stage(bundle: &JsonBundle, path: &str) -> Result<StageFile, ContentError> {
    let documents: StageDocuments = parse(bundle, path)?;
    let map_path = sibling(path, &documents.map)?;
    let map: MapDocument = parse(bundle, &map_path)?;
    let roster_path = sibling(path, &documents.roster)?;
    let document: RosterDocument = parse(bundle, &roster_path)?;
    let (roster, placements, arrivals) = document.resolve(&roster_path)?;
    let gold = roster.gold.clone();
    let recruitment = roster
        .factions
        .iter()
        .map(|f| (f.id.clone(), f.recruits.clone().unwrap()))
        .collect();
    let objectives: StageObjectives = parse(bundle, &sibling(path, &documents.objectives)?)?;
    let story: StageStory = parse(bundle, &sibling(path, &documents.story)?)?;
    let scripts: StageScripts = parse(bundle, &sibling(path, &documents.scripts)?)?;
    for tile in &map.tiles {
        if let Some(light) = &tile.light {
            light
                .validate(&map.tiles, &scripts.events)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
        if let Some(rule) = &tile.fog {
            rule.condition
                .validate_presentation()
                .map_err(|message| ContentError::new(&map_path, message))?;
            rule.condition
                .validate(&map.tiles, &scripts.events, true)
                .map_err(|message| ContentError::new(&map_path, message))?;
        }
    }
    let layout = Layout {
        id: map.map_id.clone(),
        units: placements,
    };
    Ok(StageFile {
        id: documents.id,
        state: map.initial(roster),
        mission: Mission {
            lose_conditions: objectives.lose_conditions,
            map: layout,
            subtitle: story.subtitle.unwrap_or_default(),
            briefing: story.briefing,
            debrief: story.debrief,
            hints: story.hints,
            intro: story.intro,
            victory: story.victory,
            required_companions: objectives.required_companions,
            arrivals,
            rituals: scripts.rituals,
            goals: objectives.goals,
            optional_goals: objectives.optional_goals,
            defend_objectives: objectives.defend_objectives,
            enemy_leader_post: objectives.enemy_leader_post,
            recruitment,
            gold,
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
pub fn map_metadata(
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
                    character: unit.character.clone(),
                    role: unit.role,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?,
    };
    let presentation =
        crate::data::presentation::resolve(&crate::data::presentation::PresentationInput {
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
