//! Typed assembly of schema-v2 scenario/stage components. Legacy schema-v1
//! snapshots remain readable for the preserved simulation reference catalogs.
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
    #[serde(rename = "name")]
    _name: Option<String>,
    #[serde(rename = "description")]
    _description: Option<String>,
    #[serde(rename = "revision")]
    _revision: Option<u32>,
}
impl MapDocument {
    fn initial(self, roster: InitialRoster) -> InitialState {
        InitialState {
            map_id: self.map_id,
            map_revision: self.map_revision,
            cols: self.cols,
            rows: self.rows,
            tiles: self.tiles,
            gold: roster.gold,
            units: roster.units,
            round: initial_round(),
            turn: initial_turn(),
            winner: None,
            next_id: None,
            log: None,
            campaign: None,
            last_combat: None,
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
    required_companions: Vec<String>,
    #[serde(default)]
    defend_objectives: bool,
    enemy_leader_post: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageStory {
    #[serde(rename = "subtitle")]
    _subtitle: Option<String>,
    briefing: String,
    debrief: Option<String>,
    hints: Vec<String>,
    intro: Vec<Dialogue>,
    victory: Vec<Dialogue>,
    #[serde(rename = "legacyDialogue")]
    _legacy_dialogue: Option<Vec<Dialogue>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageScripts {
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
pub(super) fn stage_path(schema: u32, campaign: &str, stage: &str) -> Result<String, ContentError> {
    if schema == 1 {
        checked_path(stage)?;
        Ok(stage.to_owned())
    } else {
        sibling(campaign, stage)
    }
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
    if !matches!(manifest.schema_version, 1 | 2) {
        return Err(ContentError::new(
            "manifest.json",
            "unsupported schemaVersion",
        ));
    }
    for path in [
        &manifest.terrain,
        &manifest.races,
        &manifest.simulation.rosters,
        &manifest.simulation.arena,
        &manifest.simulation.campaign,
    ]
    .into_iter()
    .chain(manifest.units.paths())
    .chain(&manifest.scenarios)
    .chain(&manifest.campaigns)
    {
        read(&mut bundle, &mut source, path)?;
    }
    if manifest.schema_version == 2 {
        for path in &manifest.scenarios {
            let documents: ScenarioDocuments = parse(&bundle, path)?;
            for component in [&documents.map, &documents.roster] {
                read(&mut bundle, &mut source, &sibling(path, component)?)?;
            }
        }
    }
    for campaign_path in &manifest.campaigns {
        let campaign: CampaignFile = parse(&bundle, campaign_path)?;
        for stage in campaign.stages {
            let path = stage_path(manifest.schema_version, campaign_path, &stage)?;
            read(&mut bundle, &mut source, &path)?;
            if manifest.schema_version == 2 {
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
    }
    Ok(bundle)
}
pub(super) fn scenario(
    bundle: &JsonBundle,
    path: &str,
) -> Result<(InitialState, String), ContentError> {
    let documents: ScenarioDocuments = parse(bundle, path)?;
    let map: MapDocument = parse(bundle, &sibling(path, &documents.map)?)?;
    let roster_path = sibling(path, &documents.roster)?;
    let roster: InitialRoster = parse(bundle, &roster_path)?;
    Ok((map.initial(roster), roster_path))
}
pub(super) fn stage(bundle: &JsonBundle, path: &str) -> Result<StageFile, ContentError> {
    let documents: StageDocuments = parse(bundle, path)?;
    let map: MapDocument = parse(bundle, &sibling(path, &documents.map)?)?;
    let roster: StageRoster = parse(bundle, &sibling(path, &documents.roster)?)?;
    let objectives: StageObjectives = parse(bundle, &sibling(path, &documents.objectives)?)?;
    let story: StageStory = parse(bundle, &sibling(path, &documents.story)?)?;
    let scripts: StageScripts = parse(bundle, &sibling(path, &documents.scripts)?)?;
    let layout = Layout {
        id: map.map_id.clone(),
        units: roster.units,
    };
    Ok(StageFile {
        id: documents.id,
        state: map.initial(roster.initial),
        mission: Mission {
            map: layout,
            briefing: story.briefing,
            debrief: story.debrief,
            hints: story.hints,
            intro: story.intro,
            victory: story.victory,
            required_companions: objectives.required_companions,
            goals: objectives.goals,
            defend_objectives: objectives.defend_objectives,
            enemy_leader_post: objectives.enemy_leader_post,
            recruits: roster.recruits,
            enemy_recruits: roster.enemy_recruits,
            gold: roster.gold,
            enemy_gold: roster.enemy_gold,
            events: scripts.events,
            waves: scripts.waves,
            companion_losses: scripts.companion_losses,
            refuge: scripts.refuge,
            without_dain: scripts.without_dain,
        },
    })
}
