use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Player,
    Enemy,
}
impl Side {
    pub fn index(self) -> usize {
        if self == Self::Player { 0 } else { 1 }
    }
    pub fn other(self) -> Self {
        if self == Self::Player {
            Self::Enemy
        } else {
            Self::Player
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Melee,
    Ranged,
}
impl Category {
    pub fn index(self) -> usize {
        if self == Self::Melee { 0 } else { 1 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageType {
    Blade,
    Pierce,
    Impact,
}
impl DamageType {
    pub fn index(self) -> usize {
        match self {
            Self::Blade => 0,
            Self::Pierce => 1,
            Self::Impact => 2,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub damage: i32,
    pub damage_type: DamageType,
    pub category: Category,
    pub range: u8,
    pub sound: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageModifiers {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl DamageModifiers {
    pub fn get(&self, damage: DamageType) -> i32 {
        match damage {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitRule {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub r#move: usize,
    pub damage: i32,
    pub range: u8,
    pub cost: i32,
    pub attacks: Vec<Weapon>,
    pub damage_modifiers: DamageModifiers,
    pub race: String,
    #[serde(default)]
    pub leader: bool,
    #[serde(default)]
    pub healing: i32,
    #[serde(default)]
    pub level: u32,
    pub family: String,
    pub experience_needed: Option<u32>,
    #[serde(default)]
    pub advances_to: Vec<String>,
    pub costs: Vec<Option<u16>>,
    pub environment: Vec<[[i32; 3]; 2]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUnit {
    pub id: String,
    pub r#type: String,
    pub side: Side,
    pub tile: String,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experience: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retreating: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuilding: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: u32,
    pub kind: usize,
    pub side: Side,
    pub tile: usize,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    pub experience: Option<u32>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub id: String,
    pub col: i32,
    pub row: i32,
    pub terrain: String,
    pub owner: Option<Side>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gold {
    pub player: i32,
    pub enemy: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCombat {
    pub attacker_id: String,
    pub attack_id: String,
    pub sound: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Veteran {
    pub r#type: String,
    pub experience: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub speaker: String,
    pub portrait: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counter {
    pub held_turns: u32,
    pub last_counted_round: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefugeCounter {
    pub held_turns: u32,
    pub last_counted_round: u32,
    pub escalated: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignState {
    pub id: String,
    pub mission: usize,
    pub recall: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub veterans: Option<Vec<Veteran>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_veterans: Option<Vec<Veteran>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallen_companions: Option<Vec<String>>,
    pub carry_gold: i32,
    pub marks: Vec<String>,
    pub dialogue: Arc<Vec<Dialogue>>,
    pub dialogue_index: usize,
    pub outcome: Option<String>,
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refuge: Option<RefugeCounter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vigil: Option<Counter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holds: Option<BTreeMap<String, Counter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
}
impl CampaignState {
    pub fn marked(&self, mark: &str) -> bool {
        self.marks.iter().any(|m| m == mark)
    }
    pub fn chosen(&self, choice: &str) -> bool {
        self.choices
            .as_ref()
            .is_some_and(|list| list.iter().any(|c| c == choice))
    }
    pub fn escalated(&self) -> bool {
        self.refuge.as_ref().is_some_and(|c| c.escalated)
    }
    pub fn say(&mut self, lines: impl IntoIterator<Item = Dialogue>) {
        Arc::make_mut(&mut self.dialogue).extend(lines);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub map_id: String,
    pub map_revision: u32,
    pub cols: i32,
    pub rows: i32,
    pub round: u32,
    pub turn: Side,
    pub gold: Gold,
    pub winner: Option<Side>,
    pub next_id: u32,
    pub log: Vec<String>,
    pub tiles: Vec<Tile>,
    pub units: Vec<PublicUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignState>,
    #[serde(
        default,
        deserialize_with = "present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_combat: Option<Value>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct StartingUnit {
    pub col: i32,
    pub row: i32,
    pub r#type: String,
    pub side: Side,
    pub name: Option<String>,
    pub role: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub kind: String,
    pub label: String,
    pub tile: Option<String>,
    #[serde(default)]
    pub tiles: Vec<String>,
    pub turns: Option<u32>,
    pub earliest_round: Option<u32>,
    pub start_event: Option<String>,
    pub companion: Option<String>,
    pub unit_type: Option<String>,
    pub event: Option<String>,
}
impl Goal {
    pub fn sites(&self) -> Vec<String> {
        if let Some(tile) = &self.tile {
            vec![tile.clone()]
        } else {
            self.tiles.clone()
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct Trigger {
    pub kind: String,
    pub tile: Option<String>,
    pub round: Option<u32>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub trigger: Trigger,
    pub requires_event: Option<String>,
    pub unless_event: Option<String>,
    pub before_round: Option<u32>,
    pub label: Option<String>,
    pub dialogue: Vec<Dialogue>,
    pub gold: Option<i32>,
    pub war_dialogue: Option<Vec<Dialogue>>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wave {
    pub id: String,
    pub round: u32,
    pub label: String,
    pub units: Vec<StartingUnit>,
    pub requires_unsealed: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Refuge {
    pub medicine_event: String,
    pub medicine_turns: u32,
    pub escalation: Vec<Dialogue>,
    pub war_victory: Vec<Dialogue>,
    pub war_briefing: String,
    pub war_hints: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPatch {
    pub briefing: Option<String>,
    pub debrief: Option<String>,
    pub hints: Option<Vec<String>>,
    pub intro: Option<Vec<Dialogue>>,
    pub recruits: Option<Vec<String>>,
    pub reinforcements: Option<Vec<StartingUnit>>,
    pub victory: Option<Vec<Dialogue>>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Layout {
    pub id: String,
    pub units: Vec<StartingUnit>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub map: Layout,
    pub briefing: String,
    pub debrief: Option<String>,
    pub hints: Vec<String>,
    #[serde(default)]
    pub required_companions: Vec<String>,
    pub goals: Vec<Goal>,
    pub recruits: Vec<String>,
    pub enemy_recruits: Vec<String>,
    pub gold: i32,
    pub enemy_gold: i32,
    #[serde(default)]
    pub defend_objectives: bool,
    pub enemy_leader_post: Option<String>,
    pub intro: Vec<Dialogue>,
    pub victory: Vec<Dialogue>,
    pub events: Vec<Event>,
    pub waves: Vec<Wave>,
    pub companion_losses: Option<BTreeMap<String, MissionPatch>>,
    pub refuge: Option<Refuge>,
    pub without_dain: Option<MissionPatch>,
}
impl Mission {
    pub fn apply(&mut self, patch: &MissionPatch) {
        if let Some(value) = &patch.briefing {
            self.briefing = value.clone();
        }
        if let Some(value) = &patch.debrief {
            self.debrief = Some(value.clone());
        }
        if let Some(value) = &patch.hints {
            self.hints = value.clone();
        }
        if let Some(value) = &patch.intro {
            self.intro = value.clone();
        }
        if let Some(value) = &patch.victory {
            self.victory = value.clone();
        }
        if let Some(value) = &patch.recruits {
            self.recruits = value.clone();
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub leader: String,
    pub missions: Vec<Arc<Mission>>,
    pub ending: Vec<Dialogue>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleData {
    pub terrain_names: Vec<String>,
    pub terrain_definitions: Vec<Value>,
    pub units: Vec<UnitRule>,
    pub maps: Vec<State>,
    pub campaigns: HashMap<String, Campaign>,
}
#[derive(Debug)]
pub struct Rules {
    pub terrain_names: Vec<String>,
    pub terrain_labels: Vec<String>,
    pub units: Vec<UnitRule>,
    pub types: HashMap<String, usize>,
    pub terrains: HashMap<String, usize>,
    pub maps: HashMap<(String, u32), State>,
    pub campaigns: HashMap<String, Campaign>,
}
impl Rules {
    pub fn load() -> Arc<Self> {
        let data: RuleData =
            serde_json::from_str(include_str!("../data/rules.json")).expect("authored rules");
        let rules = Self {
            types: data
                .units
                .iter()
                .enumerate()
                .map(|(index, rule)| (rule.id.clone(), index))
                .collect(),
            terrains: data
                .terrain_names
                .iter()
                .enumerate()
                .map(|(index, id)| (id.clone(), index))
                .collect(),
            terrain_labels: data
                .terrain_definitions
                .iter()
                .map(|rule| rule["name"].as_str().unwrap().to_owned())
                .collect(),
            terrain_names: data.terrain_names,
            units: data.units,
            maps: data
                .maps
                .into_iter()
                .map(|state| ((state.map_id.clone(), state.map_revision), state))
                .collect(),
            campaigns: data.campaigns,
        };
        Arc::new(rules)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub label: String,
    pub current: u32,
    pub total: u32,
    pub done: bool,
    pub tiles: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objective {
    pub id: String,
    pub tiles: Vec<String>,
    pub weight: f64,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct Records {
    pub actions: Vec<Value>,
    pub losses: Vec<PublicUnit>,
    pub combats: Vec<Value>,
}

fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}
