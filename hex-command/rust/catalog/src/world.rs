//! Authored setup and scripts. Runtime state is constructed and owned by battle.
use crate::schema;
use crate::*;
use serde::{Deserialize, Serialize};

pub struct EnvironmentDefinition {
    pub(crate) source: schema::EnvironmentEffectDefinition,
}
impl EnvironmentDefinition {
    pub fn id(&self) -> &str {
        &self.source.id
    }
    pub fn name(&self) -> &str {
        &self.source.name
    }
    pub fn description(&self) -> &str {
        &self.source.description
    }
    pub fn effects(&self) -> &[schema::EnvironmentEffect] {
        &self.source.effects
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MapTile {
    pub id: Option<Text>,
    pub col: i32,
    pub row: i32,
    pub terrain: Terrain,
    pub owner: Option<Text>,
    pub connections: Option<Vec<HexDirection>>,
    pub dead_ends: Option<Vec<HexDirection>>,
    pub walls: Option<Vec<HexDirection>>,
    pub floor: Option<Text>,
    pub environment_effects: Vec<Text>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MapSource {
    pub map_id: Text,
    pub map_revision: u32,
    pub name: Text,
    pub description: Text,
    pub revision: Option<u32>,
    pub cols: i32,
    pub rows: i32,
    pub modifiers: StatModifiers,
    pub lighting: MapLighting,
    pub tiles: Vec<MapTile>,
}
pub struct MapDefinition {
    pub(crate) source: MapSource,
    pub(crate) tags: Vec<Text>,
    pub(crate) roster: InitialRoster,
    pub(crate) characters: Vec<Character>,
    pub(crate) fixed_armies: bool,
}
impl MapDefinition {
    pub fn id(&self) -> &str {
        &self.source.map_id
    }
    pub fn name(&self) -> &str {
        &self.source.name
    }
    pub fn description(&self) -> &str {
        &self.source.description
    }
    pub fn artwork_revision(&self) -> Option<u32> {
        self.source.revision
    }
    pub fn revision(&self) -> u32 {
        self.source.map_revision
    }
    pub fn tags(&self) -> &[Text] {
        &self.tags
    }
    pub fn fixed_armies(&self) -> bool {
        self.fixed_armies
    }
    pub fn dimensions(&self) -> (i32, i32) {
        (self.source.cols, self.source.rows)
    }
    pub fn tiles(&self) -> &[MapTile] {
        &self.source.tiles
    }
    pub fn modifiers(&self) -> &StatModifiers {
        &self.source.modifiers
    }
    pub fn lighting(&self) -> &MapLighting {
        &self.source.lighting
    }
    pub fn roster(&self) -> &InitialRoster {
        &self.roster
    }
    pub fn characters(&self) -> &[Character] {
        &self.characters
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InitialHealth {
    Extent(FullHealth),
    Wounded(i32),
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FullHealth {
    Full,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialUnit {
    pub id: Text,
    #[serde(rename = "type")]
    pub unit_type: Text,
    pub tile: Hex,
    pub hp: InitialHealth,
    pub perception_effects: Vec<PerceptionEffect>,
    pub moved: bool,
    pub attack_spent: bool,
    pub experience: u32,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
    pub ai_group: Option<Text>,
    pub character: Option<Text>,
    pub role: Option<Role>,
    pub items: Vec<Text>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitialArmy {
    #[serde(flatten)]
    pub faction: FactionSetup,
    pub gold: i32,
    pub units: Vec<InitialUnit>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InitialVisionGroup {
    pub id: Text,
    pub sight: SightContribution,
    pub explored: Vec<Hex>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialRoster {
    pub round: u32,
    pub turn: TurnPhase,
    pub log: Vec<Text>,
    pub combat_rng: u32,
    pub vision_groups: Vec<InitialVisionGroup>,
    pub factions: Vec<InitialArmy>,
    pub arrivals: Vec<schema::CompanionArrival>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageStory {
    pub subtitle: Text,
    pub briefing: ConditionalText,
    pub debrief: Option<ConditionalText>,
    pub hints: Vec<ConditionalHint>,
    pub intro: Vec<Speech>,
    pub victory: Vec<Speech>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageScripts {
    pub enemy_groups: Vec<EnemyGroup>,
    pub events: Vec<ScriptEvent>,
    pub required_companions: Vec<Text>,
    pub defend_objectives: bool,
    pub enemy_leader_post: Option<TileRef>,
}
pub struct ChapterDefinition {
    pub(crate) id: Text,
    pub(crate) map: Text,
    pub(crate) map_revision: u32,
    pub(crate) story: StageStory,
    pub(crate) scripts: StageScripts,
}
impl ChapterDefinition {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn map_id(&self) -> &str {
        &self.map
    }
    pub fn map_revision(&self) -> u32 {
        self.map_revision
    }
    pub fn story(&self) -> &StageStory {
        &self.story
    }
    pub fn scripts(&self) -> &StageScripts {
        &self.scripts
    }
}
pub struct CampaignDefinition {
    pub(crate) source: CampaignSource,
    pub(crate) chapters: Vec<ChapterDefinition>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CampaignSource {
    pub carryover: CampaignCarryover,
    pub tags: Vec<Text>,
    pub id: Text,
    pub name: Text,
    pub tagline: Text,
    pub description: Text,
    pub short_description: Text,
    pub race: Race,
    pub leader: Text,
    pub stages: Vec<Text>,
    pub characters: Vec<Character>,
    pub ending: Vec<Speech>,
}
impl CampaignDefinition {
    pub fn id(&self) -> &str {
        &self.source.id
    }
    pub fn name(&self) -> &str {
        &self.source.name
    }
    pub fn description(&self) -> &str {
        &self.source.description
    }
    pub fn tags(&self) -> &[Text] {
        &self.source.tags
    }
    pub fn race(&self) -> Race {
        self.source.race
    }
    pub fn leader_id(&self) -> &str {
        &self.source.leader
    }
    pub fn tagline(&self) -> &str {
        &self.source.tagline
    }
    pub fn short_description(&self) -> &str {
        &self.source.short_description
    }
    pub fn carryover(&self) -> &CampaignCarryover {
        &self.source.carryover
    }
    pub fn chapters(&self) -> &[ChapterDefinition] {
        &self.chapters
    }
    pub fn characters(&self) -> &[Character] {
        &self.source.characters
    }
    pub fn ending(&self) -> &[Speech] {
        &self.source.ending
    }
}

/// Authored presentation for a concealment kind, independent of current visibility.
pub struct ConcealmentDefinition {
    pub(crate) source: schema::ConcealmentKindDefinition,
}
impl ConcealmentDefinition {
    pub fn id(&self) -> ConcealmentKind {
        self.source.id
    }
    pub fn unit_appearance(&self) -> ConcealmentVisual {
        self.source.unit_appearance
    }
}
