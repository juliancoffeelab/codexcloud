//! General authored events and durable event facts; no campaign-specific behavior.
use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct UnitMatch {
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub side: Option<ConditionSide>,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default, rename = "unitType")]
    pub unit_type: Option<String>,
    #[serde(default, rename = "unitTypes")]
    pub unit_types: Vec<String>,
    #[serde(default, rename = "aiGroup")]
    pub ai_group: Option<String>,
    #[serde(default)]
    pub retreating: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SiteSelection {
    pub tile: TileRef,
    #[serde(flatten)]
    pub units: UnitMatch,
    #[serde(default)]
    pub through: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ProximitySelection {
    #[serde(default)]
    pub tile: Option<TileRef>,
    #[serde(default)]
    pub target: Option<UnitMatch>,
    pub radius: u16,
    #[serde(flatten)]
    pub units: UnitMatch,
    #[serde(default = "yes")]
    pub through: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct HoldSelection {
    #[serde(skip)]
    pub predicate: u32,
    pub tile: TileRef,
    pub turns: u32,
    #[serde(default)]
    pub when: Option<Box<When>>,
    #[serde(flatten)]
    pub units: UnitMatch,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct OwnerSelection {
    pub tile: TileRef,
    pub faction: String,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnSelection {
    #[serde(skip)]
    pub predicate: u32,
    pub tile: TileRef,
    pub faction: String,
    #[serde(default)]
    pub turns: Option<u32>,
    #[serde(default)]
    pub when: Option<Box<When>>,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum TurnSelection {
    Number(u32),
    Faction { number: u32, faction: String },
}
impl TurnSelection {
    pub fn number(&self) -> u32 {
        match self {
            Self::Number(n) | Self::Faction { number: n, .. } => *n,
        }
    }
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase")]
pub struct DeathSelection {
    #[serde(flatten)]
    pub units: UnitMatch,
    #[serde(default)]
    pub initiated_by: Option<String>,
    #[serde(default)]
    pub killed_by: Option<String>,
    #[serde(default)]
    pub defending: Option<bool>,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct TextVariant {
    #[serde(default)]
    pub when: Option<When>,
    pub text: String,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum ConditionalText {
    Text(String),
    Variants(Vec<TextVariant>),
}
impl ConditionalText {
    pub fn conditions(&self) -> Vec<&When> {
        match self {
            Self::Text(_) => vec![],
            Self::Variants(v) => v.iter().filter_map(|v| v.when.as_ref()).collect(),
        }
    }
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum ConditionalHint {
    Text(String),
    Variant(TextVariant),
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase")]
pub enum When {
    Always(bool),
    All(Vec<When>),
    Any(Vec<When>),
    Not(Box<When>),
    Stand(SiteSelection),
    Proximity(ProximitySelection),
    Killed(DeathSelection),
    Left(SiteSelection),
    Turn(TurnSelection),
    TurnAtLeast(u32),
    ChapterStarted(bool),
    Flag(String),
    ObjectiveSatisfied(String),
    HasUnits(UnitMatch),
    NoUnits(UnitMatch),
    VisibleUnits(UnitMatch),
    Owner(OwnerSelection),
    Own(OwnSelection),
    Held(HoldSelection),
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct ScriptLine {
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub speaker: Option<String>,
    #[serde(default)]
    pub portrait: Option<String>,
    pub text: String,
    #[serde(default)]
    pub sound: Option<String>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct DialogueBranch {
    pub with: String,
    pub dialogue: Vec<Speech>,
    #[serde(default)]
    pub alternative: Vec<Speech>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum Speech {
    Branch(DialogueBranch),
    Line(ScriptLine),
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct SpawnMember {
    #[serde(rename = "type")]
    pub unit_type: String,
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub items: Vec<String>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum SpawnEntry {
    Type(String),
    Member(SpawnMember),
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct ScriptSpawn {
    pub faction: String,
    #[serde(default)]
    pub tile: Option<TileRef>,
    #[serde(default)]
    pub area: Vec<TileRef>,
    #[serde(default = "spawn_radius")]
    pub radius: u16,
    #[serde(default, rename = "aiGroup")]
    pub ai_group: Option<String>,
    pub units: Vec<SpawnEntry>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct GiveItem {
    pub item: String,
    #[serde(default)]
    pub tile: Option<TileRef>,
    #[serde(flatten)]
    pub units: UnitMatch,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct PlaceItem {
    pub item: String,
    pub tile: TileRef,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct AllegianceChange {
    pub units: UnitMatch,
    pub faction: String,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(untagged)]
pub enum UnitReference {
    Bound(String),
    Selection(UnitMatch),
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct MarkSpec {
    #[serde(rename = "type")]
    pub kind: String,
    pub tile: TileRef,
    #[serde(default)]
    pub light: Option<TileLight>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct MarkSelection {
    #[serde(rename = "type")]
    pub kind: String,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct UnitLoop {
    pub select: LoopSelection,
    pub actions: Vec<EventAction>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct MarkLoop {
    pub select: MarkSelection,
    pub actions: Vec<EventAction>,
}

/// String shorthand sets true; the explicit form can also clear a flag.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum FlagUpdate {
    Name(String),
    Value { id: String, state: bool },
}
impl FlagUpdate {
    pub fn id(&self) -> &str {
        match self {
            Self::Name(id) | Self::Value { id, .. } => id,
        }
    }
    pub fn state(&self) -> bool {
        match self {
            Self::Name(_) => true,
            Self::Value { state, .. } => *state,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct TerrainChange {
    pub tile: TileRef,
    pub terrain: String,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct GoldGrant {
    pub faction: String,
    pub amount: i32,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct RecruitmentUnlock {
    pub faction: String,
    pub units: Vec<FactionRecruit>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase")]
pub enum EventAction {
    Spawn(ScriptSpawn),
    Give(GiveItem),
    PlaceItem(PlaceItem),
    ChangeAllegiance(AllegianceChange),
    DeleteUnit(UnitReference),
    SetFlag(FlagUpdate),
    Say(Vec<Speech>),
    Sound(String),
    CreateMark(MarkSpec),
    RemoveMark(String),
    ForEachUnit(UnitLoop),
    ForEachMark(MarkLoop),
    SetTerrain(TerrainChange),
    Gold(GoldGrant),
    UnlockRecruits(RecruitmentUnlock),
    Victory(String),
    Defeat(String),
    Log(String),
    FailObjective(String),
    CompleteObjective(String),
    Recruits(RecruitmentUnlock),
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase")]
pub enum EventIntent {
    Hold(HoldSelection),
    Own(OwnSelection),
    Escort(SiteSelection),
    GoTo(SiteSelection),
    Escape(SiteSelection),
    Approach(ProximitySelection),
    Capture(Vec<TileRef>),
    Defeat(UnitMatch),
}

/// Explicit player-facing destinations, independent of AI intent and interaction radius.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerMarker {
    #[serde(default)]
    pub tiles: Vec<TileRef>,
    #[serde(default)]
    pub units: Option<UnitMatch>,
    #[serde(default)]
    pub visible_in_fog: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct EventObjective {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "preserveLives")]
    pub preserve_lives: Option<UnitMatch>,
    pub label: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default, rename = "visibleWhen")]
    pub visible_when: Option<When>,
    #[serde(default, rename = "availableWhen")]
    pub available_when: Option<When>,
    /// None is a permanent achievement; Some is a live requirement until victory.
    #[serde(default, rename = "satisfiedWhen")]
    pub satisfied_when: Option<When>,
    pub intent: EventIntent,
    #[serde(default, rename = "playerMarker")]
    pub player_marker: Option<PlayerMarker>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct ScriptEvent {
    pub id: String,
    pub when: When,
    #[serde(default = "yes")]
    pub remembers: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub objective: Option<EventObjective>,
    pub actions: Vec<EventAction>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct MapMark {
    #[serde(default, rename = "lightRemembered")]
    pub light_remembered: bool,
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: String,
    pub tile: Hex,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<TileLight>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventHold {
    pub event: String,
    pub predicate: u32,
    pub turns: u32,
    pub last_round: u32,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct PendingEventSpawn {
    pub area: Vec<Hex>,
    pub faction: String,
    #[serde(default, rename = "aiGroup")]
    pub ai_group: Option<String>,
    pub units: Vec<SpawnMember>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventState {
    #[serde(skip)]
    pub active_indices: Vec<usize>,
    #[serde(skip)]
    pub active_mask: Vec<bool>,
    #[serde(default)]
    pub failed: Vec<String>,
    pub completed: Vec<String>,
    pub revealed: Vec<String>,
    pub recruitment: std::collections::BTreeMap<String, Vec<FactionRecruit>>,
    pub active: Vec<String>,
    pub holds: Vec<EventHold>,
    pub marks: Vec<MapMark>,
    pub pending: Vec<PendingEventSpawn>,
    pub next_mark: u32,
}

fn yes() -> bool {
    true
}
fn spawn_radius() -> u16 {
    2
}
impl SpawnEntry {
    pub fn member(&self) -> SpawnMember {
        match self {
            Self::Type(kind) => SpawnMember {
                unit_type: kind.clone(),
                character: None,
                role: None,
                items: vec![],
            },
            Self::Member(member) => member.clone(),
        }
    }
}
pub struct EventClock<'a> {
    pub predicate: u32,
    pub tile: &'a TileRef,
    pub turns: u32,
    pub units: Option<&'a UnitMatch>,
    pub faction: Option<&'a str>,
    pub when: Option<&'a When>,
}
impl When {
    pub fn state_only(&self) -> bool {
        match self {
            Self::All(v) | Self::Any(v) => v.iter().all(Self::state_only),
            Self::Not(v) => v.state_only(),
            Self::Stand(v) => !v.through,
            Self::Proximity(v) => !v.through,
            Self::Own(v) => v.turns.is_none() && v.when.as_ref().is_none_or(|w| w.state_only()),
            Self::Held(_)
            | Self::Killed(_)
            | Self::Left(_)
            | Self::Turn(_)
            | Self::ChapterStarted(_) => false,
            _ => true,
        }
    }

    pub fn visit_objective_refs(&self, visit: &mut impl FnMut(&str)) {
        match self {
            Self::ObjectiveSatisfied(id) => visit(id),
            Self::All(values) | Self::Any(values) => {
                for value in values {
                    value.visit_objective_refs(visit);
                }
            }
            Self::Not(value) => value.visit_objective_refs(visit),
            Self::Own(value) => {
                if let Some(when) = &value.when {
                    when.visit_objective_refs(visit);
                }
            }
            Self::Held(value) => {
                if let Some(when) = &value.when {
                    when.visit_objective_refs(visit);
                }
            }
            _ => {}
        }
    }

    fn requires_death(&self) -> bool {
        match self {
            Self::Killed(_) => true,
            Self::All(v) => v.iter().any(Self::requires_death),
            Self::Any(v) => !v.is_empty() && v.iter().all(Self::requires_death),
            _ => false,
        }
    }
    pub fn has_death(&self) -> bool {
        match self {
            Self::Killed(_) => true,
            Self::All(v) | Self::Any(v) => v.iter().any(Self::has_death),
            Self::Not(v) => v.has_death(),
            _ => false,
        }
    }
    // Match the durable counter numbering without depending on evaluation order.
    pub fn index_holds(&mut self) {
        fn visit(when: &mut When, next: &mut u32) {
            match when {
                When::Held(value) => {
                    value.predicate = *next;
                    *next += 1;
                }
                When::Own(value) if value.turns.is_some() => {
                    value.predicate = *next;
                    *next += 1;
                }
                When::All(values) | When::Any(values) => {
                    for value in values {
                        visit(value, next);
                    }
                }
                When::Not(value) => visit(value, next),
                _ => {}
            }
        }
        visit(self, &mut 0);
    }
    pub fn holds(&self) -> Vec<EventClock<'_>> {
        match self {
            Self::Held(v) => vec![EventClock {
                predicate: v.predicate,
                tile: &v.tile,
                turns: v.turns,
                units: Some(&v.units),
                faction: None,
                when: v.when.as_deref(),
            }],
            Self::Own(v) if v.turns.is_some() => vec![EventClock {
                predicate: v.predicate,
                tile: &v.tile,
                turns: v.turns.unwrap(),
                units: None,
                faction: Some(&v.faction),
                when: v.when.as_deref(),
            }],
            Self::All(v) | Self::Any(v) => v.iter().flat_map(Self::holds).collect(),
            Self::Not(v) => v.holds(),
            _ => vec![],
        }
    }
}
impl EventState {
    pub fn index_active(&mut self, events: &[ScriptEvent]) {
        let active: std::collections::HashSet<_> = self.active.iter().map(String::as_str).collect();
        self.active_indices = events
            .iter()
            .enumerate()
            .filter(|(_, event)| active.contains(event.id.as_str()))
            .map(|(index, _)| index)
            .collect();
        self.active_mask = vec![false; events.len()];
        for &index in &self.active_indices {
            self.active_mask[index] = true;
        }
    }
    pub fn retire(&mut self, index: usize) {
        self.active_mask[index] = false;
    }
    pub fn compact(&mut self, events: &[ScriptEvent]) {
        self.active_indices.retain(|&index| self.active_mask[index]);
        let active: std::collections::HashSet<_> = self
            .active_indices
            .iter()
            .map(|&index| events[index].id.as_str())
            .collect();
        self.active.retain(|id| active.contains(id.as_str()));
    }
    pub fn new(events: &[ScriptEvent]) -> Self {
        Self {
            active_indices: (0..events.len()).collect(),
            active_mask: vec![true; events.len()],
            failed: vec![],
            completed: vec![],
            revealed: vec![],
            recruitment: Default::default(),
            active: events.iter().map(|e| e.id.clone()).collect(),
            holds: vec![],
            marks: vec![],
            pending: vec![],
            next_mark: 1,
        }
    }
}

impl ScriptEvent {
    pub fn objective_id(&self) -> &str {
        self.objective
            .as_ref()
            .and_then(|o| o.id.as_deref())
            .unwrap_or(&self.id)
    }
    pub fn validate(
        &self,
        tiles: &[Tile],
        factions: &[Faction],
        items: &std::collections::HashMap<String, Item>,
        path: &str,
    ) -> Result<(), ContentError> {
        use super::catalog::{identifier, unique};
        identifier(&self.id, path)?;
        let fail = |s: String| ContentError::new(path, s);
        let tile = |id: &TileRef| -> Result<(), ContentError> {
            if tiles.iter().any(|t| t.matches_reference(id)) {
                Ok(())
            } else {
                Err(fail(format!("unknown event tile {id}")))
            }
        };
        let faction = |id: &str| {
            if factions.iter().any(|f| f.id == id) {
                Ok(())
            } else {
                Err(fail(format!("unknown event faction {id}")))
            }
        };
        let item = |id: &str| {
            if items.contains_key(id) {
                Ok(())
            } else {
                Err(fail(format!("unknown event item {id}")))
            }
        };
        fn when(
            value: &When,
            tile: &impl Fn(&TileRef) -> Result<(), ContentError>,
            faction: &impl Fn(&str) -> Result<(), ContentError>,
            path: &str,
        ) -> Result<(), ContentError> {
            match value {
                When::All(v) | When::Any(v) => {
                    if v.is_empty() {
                        return Err(ContentError::new(path, "empty event condition"));
                    }
                    for c in v {
                        when(c, tile, faction, path)?;
                    }
                }
                When::Not(v) => when(v, tile, faction, path)?,
                When::Stand(v) | When::Left(v) => tile(&v.tile)?,
                When::Proximity(v) => {
                    if v.tile.is_some() == v.target.is_some() {
                        return Err(ContentError::new(
                            path,
                            "proximity needs one tile or unit target",
                        ));
                    }
                    if let Some(id) = &v.tile {
                        tile(id)?;
                    }
                }
                When::Held(v) => {
                    tile(&v.tile)?;
                    if let Some(c) = &v.when {
                        if !c.state_only() {
                            return Err(ContentError::new(
                                path,
                                "clock gates must be state predicates",
                            ));
                        }
                        when(c, tile, faction, path)?;
                    }
                    if v.turns == 0 {
                        return Err(ContentError::new(path, "hold duration must be positive"));
                    }
                }
                When::Own(v) => {
                    tile(&v.tile)?;
                    faction(&v.faction)?;
                    if v.turns == Some(0) {
                        return Err(ContentError::new(
                            path,
                            "ownership duration must be positive",
                        ));
                    }
                    if let Some(c) = &v.when {
                        if !c.state_only() {
                            return Err(ContentError::new(
                                path,
                                "ownership gate must be a state predicate",
                            ));
                        }
                        when(c, tile, faction, path)?;
                    }
                }
                When::Turn(TurnSelection::Faction { faction: id, .. }) => faction(id)?,
                When::Owner(v) => {
                    tile(&v.tile)?;
                    faction(&v.faction)?;
                }
                When::Killed(v) => {
                    for id in v.initiated_by.iter().chain(&v.killed_by) {
                        faction(id)?;
                    }
                }
                When::Flag(id) | When::ObjectiveSatisfied(id) => identifier(id, path)?,
                When::ChapterStarted(false) => {
                    return Err(ContentError::new(
                        path,
                        "chapterStarted must be true; use not for negation",
                    ));
                }
                _ => {}
            }
            Ok(())
        }
        fn actions(
            values: &[EventAction],
            tile: &impl Fn(&TileRef) -> Result<(), ContentError>,
            faction: &impl Fn(&str) -> Result<(), ContentError>,
            item: &impl Fn(&str) -> Result<(), ContentError>,
            path: &str,
            bound_unit: bool,
            bound_mark: bool,
            death: bool,
        ) -> Result<(), ContentError> {
            let reference = |s: &TileRef| -> Result<(), ContentError> {
                match s {
                    TileRef::Binding(TileBinding::Unit) if !bound_unit => {
                        Err(ContentError::new(path, "unbound unit tile"))
                    }
                    TileRef::Binding(TileBinding::Mark) if !bound_mark => {
                        Err(ContentError::new(path, "unbound mark tile"))
                    }
                    TileRef::Binding(TileBinding::Death) if !death => Err(ContentError::new(
                        path,
                        "death tile requires a killed condition",
                    )),
                    TileRef::Binding(_) => Ok(()),
                    _ => tile(s),
                }
            };
            for action in values {
                match action {
                    EventAction::Spawn(v) => {
                        faction(&v.faction)?;
                        if v.tile.is_none() == v.area.is_empty() {
                            return Err(ContentError::new(
                                path,
                                "spawn requires one tile or a nonempty area",
                            ));
                        }
                        if v.units.is_empty() {
                            return Err(ContentError::new(path, "empty spawn"));
                        }
                        if let Some(t) = &v.tile {
                            reference(t)?;
                        }
                        for t in &v.area {
                            reference(t)?;
                        }
                        if v.area
                            .iter()
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                            != v.area.len()
                        {
                            return Err(ContentError::new(path, "duplicate spawn target"));
                        }
                        for u in &v.units {
                            for id in &u.member().items {
                                item(id)?;
                            }
                        }
                    }
                    EventAction::Give(v) => {
                        item(&v.item)?;
                        if let Some(t) = &v.tile {
                            reference(t)?;
                        }
                    }
                    EventAction::PlaceItem(v) => {
                        reference(&v.tile)?;
                        item(&v.item)?;
                    }
                    EventAction::ChangeAllegiance(v) => faction(&v.faction)?,
                    EventAction::DeleteUnit(UnitReference::Bound(v))
                        if v != "$unit" || !bound_unit =>
                    {
                        return Err(ContentError::new(
                            path,
                            "deleteUnit requires a selector or a bound unit",
                        ));
                    }
                    EventAction::SetFlag(update) => identifier(update.id(), path)?,
                    EventAction::CreateMark(v) => {
                        identifier(&v.kind, path)?;
                        reference(&v.tile)?;
                        if let Some(l) = &v.light {
                            let mut appearance = l.clone();
                            appearance.when = None;
                            appearance
                                .validate(&[], &[])
                                .map_err(|e| ContentError::new(path, e))?;
                        }
                    }
                    EventAction::RemoveMark(v) if v != "$mark" || !bound_mark => {
                        return Err(ContentError::new(path, "removeMark requires a bound mark"));
                    }
                    EventAction::ForEachUnit(v) => {
                        if let Some(t) = &v.select.tile {
                            reference(t)?;
                        }
                        actions(
                            &v.actions, tile, faction, item, path, true, bound_mark, death,
                        )?;
                    }
                    EventAction::ForEachMark(v) => {
                        identifier(&v.select.kind, path)?;
                        actions(
                            &v.actions, tile, faction, item, path, bound_unit, true, death,
                        )?;
                    }
                    EventAction::SetTerrain(v) => reference(&v.tile)?,
                    EventAction::Gold(v) => faction(&v.faction)?,
                    EventAction::UnlockRecruits(v) | EventAction::Recruits(v) => {
                        faction(&v.faction)?;
                        unique(v.units.iter().map(|r| r.unit.as_str()), path)?;
                        if v.units.iter().any(|r| r.cost.is_some_and(|c| c < 0)) {
                            return Err(ContentError::new(path, "negative recruitment cost"));
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        when(&self.when, &tile, &faction, path)?;
        actions(
            &self.actions,
            &tile,
            &faction,
            &item,
            path,
            false,
            false,
            self.when.requires_death(),
        )?;
        if let Some(o) = &self.objective {
            if let Some(id) = &o.id {
                identifier(id, path)?;
            }
            if o.label.trim().is_empty() || !self.remembers {
                return Err(fail(
                    "objectives require a label and a remembered event".into(),
                ));
            }
            if o.visible_when.is_none() {
                return Err(fail("objectives require an explicit visibleWhen".into()));
            }
            if let Some(c) = &o.available_when {
                if !c.state_only() {
                    return Err(fail(
                        "objective availability requires state predicates".into(),
                    ));
                }
                when(c, &tile, &faction, path)?;
            }
            if let Some(c) = &o.visible_when {
                if !c.state_only() {
                    return Err(fail("objective reveal requires state predicates".into()));
                }
                when(c, &tile, &faction, path)?;
            }
            if let Some(c) = &o.satisfied_when {
                if !c.state_only() {
                    return Err(fail(
                        "objective satisfaction requires state predicates".into(),
                    ));
                }
                when(c, &tile, &faction, path)?;
            }
            if let Some(marker) = &o.player_marker {
                if marker.tiles.is_empty() == marker.units.is_none() {
                    return Err(fail("player marker needs either tiles or units".into()));
                }
                for id in &marker.tiles {
                    tile(id)?;
                }
            }
            match &o.intent {
                EventIntent::Hold(v) => {
                    tile(&v.tile)?;
                    if v.turns == 0 {
                        return Err(fail("hold intent duration must be positive".into()));
                    }
                }
                EventIntent::Escort(v) | EventIntent::GoTo(v) | EventIntent::Escape(v) => {
                    tile(&v.tile)?
                }
                EventIntent::Capture(v) => {
                    if v.is_empty() {
                        return Err(fail("capture intent needs tiles".into()));
                    }
                    for t in v {
                        tile(t)?;
                    }
                }
                EventIntent::Approach(v) => {
                    when(&When::Proximity(v.clone()), &tile, &faction, path)?
                }
                EventIntent::Own(v) => {
                    tile(&v.tile)?;
                    faction(&v.faction)?;
                    if v.turns == Some(0) {
                        return Err(fail("ownership intent duration must be positive".into()));
                    }
                }
                EventIntent::Defeat(_) => {}
            }
        }
        Ok(())
    }

    pub fn selectors(&self) -> Vec<&UnitMatch> {
        fn conditions<'a>(w: &'a When, out: &mut Vec<&'a UnitMatch>) {
            match w {
                When::All(v) | When::Any(v) => {
                    for c in v {
                        conditions(c, out)
                    }
                }
                When::Not(v) => conditions(v, out),
                When::Stand(v) | When::Left(v) => out.push(&v.units),
                When::Held(v) => {
                    out.push(&v.units);
                    if let Some(c) = &v.when {
                        conditions(c, out);
                    }
                }
                When::Own(v) => {
                    if let Some(c) = &v.when {
                        conditions(c, out);
                    }
                }
                When::Proximity(v) => {
                    out.push(&v.units);
                    if let Some(s) = &v.target {
                        out.push(s);
                    }
                }
                When::Killed(v) => out.push(&v.units),
                When::HasUnits(v) | When::NoUnits(v) | When::VisibleUnits(v) => out.push(v),
                _ => {}
            }
        }
        fn actions<'a>(values: &'a [EventAction], out: &mut Vec<&'a UnitMatch>) {
            for a in values {
                match a {
                    EventAction::Give(v) => out.push(&v.units),
                    EventAction::ChangeAllegiance(v) => out.push(&v.units),
                    EventAction::DeleteUnit(UnitReference::Selection(v)) => out.push(v),
                    EventAction::ForEachUnit(v) => {
                        out.push(&v.select.units);
                        actions(&v.actions, out);
                    }
                    EventAction::ForEachMark(v) => actions(&v.actions, out),
                    _ => {}
                }
            }
        }
        let mut out = vec![];
        conditions(&self.when, &mut out);
        actions(&self.actions, &mut out);
        if let Some(o) = &self.objective {
            if let Some(w) = &o.available_when {
                conditions(w, &mut out);
            }
            if let Some(w) = &o.visible_when {
                conditions(w, &mut out);
            }
            if let Some(v) = o.player_marker.as_ref().and_then(|m| m.units.as_ref()) {
                out.push(v);
            }
            if let Some(w) = &o.satisfied_when {
                conditions(w, &mut out);
            }
            if let Some(v) = &o.preserve_lives {
                out.push(v);
            }
            match &o.intent {
                EventIntent::Hold(v) => out.push(&v.units),
                EventIntent::Escort(v) | EventIntent::GoTo(v) | EventIntent::Escape(v) => {
                    out.push(&v.units)
                }
                EventIntent::Approach(v) => {
                    out.push(&v.units);
                    if let Some(t) = &v.target {
                        out.push(t);
                    }
                }
                EventIntent::Defeat(v) => out.push(v),
                _ => {}
            }
        }
        out
    }
    pub fn validate_characters(
        &self,
        characters: &[Character],
        path: &str,
    ) -> Result<(), ContentError> {
        let check = |id: &str| {
            if characters.iter().any(|c| c.id == id) {
                Ok(())
            } else {
                Err(ContentError::new(path, format!("unknown character {id}")))
            }
        };
        fn speech(
            lines: &[Speech],
            check: &impl Fn(&str) -> Result<(), ContentError>,
            path: &str,
        ) -> Result<(), ContentError> {
            for line in lines {
                match line {
                    Speech::Branch(v) => {
                        check(&v.with)?;
                        speech(&v.dialogue, check, path)?;
                        speech(&v.alternative, check, path)?;
                    }
                    Speech::Line(v) => {
                        if let Some(id) = &v.character {
                            check(id)?;
                        } else if v.speaker.as_ref().is_none_or(|s| s.trim().is_empty()) {
                            return Err(ContentError::new(
                                path,
                                "dialogue needs a character or narrator",
                            ));
                        }
                    }
                }
            }
            Ok(())
        }
        fn actions(
            a: &[EventAction],
            check: &impl Fn(&str) -> Result<(), ContentError>,
            path: &str,
        ) -> Result<(), ContentError> {
            for v in a {
                match v {
                    EventAction::Say(v) => speech(v, check, path)?,
                    EventAction::Spawn(v) => {
                        for u in &v.units {
                            if let Some(id) = &u.member().character {
                                check(id)?;
                            }
                        }
                    }
                    EventAction::ForEachUnit(v) => actions(&v.actions, check, path)?,
                    EventAction::ForEachMark(v) => actions(&v.actions, check, path)?,
                    _ => {}
                }
            }
            Ok(())
        }
        for selector in self.selectors() {
            if let Some(id) = &selector.character {
                check(id)?;
            }
        }
        actions(&self.actions, &check, path)
    }
    pub fn actions_recursive(&self) -> Vec<&EventAction> {
        fn collect<'a>(actions: &'a [EventAction], out: &mut Vec<&'a EventAction>) {
            for a in actions {
                out.push(a);
                match a {
                    EventAction::ForEachUnit(v) => collect(&v.actions, out),
                    EventAction::ForEachMark(v) => collect(&v.actions, out),
                    _ => {}
                }
            }
        }
        let mut out = vec![];
        collect(&self.actions, &mut out);
        out
    }
}

impl When {
    pub fn uses_path(&self) -> bool {
        match self {
            Self::Proximity(v) => v.through,
            Self::Stand(v) => v.through,
            Self::All(v) | Self::Any(v) => v.iter().any(Self::uses_path),
            Self::Not(v) => v.uses_path(),
            _ => false,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct LoopSelection {
    #[serde(default)]
    pub tile: Option<TileRef>,
    #[serde(flatten)]
    pub units: UnitMatch,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedStory {
    pub briefing: String,
    pub debrief: Option<String>,
    pub hints: Vec<String>,
}
impl ConditionalText {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(t) => t.trim().is_empty(),
            Self::Variants(v) => v.is_empty() || v.iter().any(|v| v.text.trim().is_empty()),
        }
    }
}

impl Mission {
    pub fn story_conditions(&self) -> Vec<&When> {
        let mut out = self.briefing.conditions();
        if let Some(t) = &self.debrief {
            out.extend(t.conditions());
        }
        for h in &self.hints {
            if let ConditionalHint::Variant(v) = h {
                if let Some(w) = &v.when {
                    out.push(w);
                }
            }
        }
        out
    }
}
