use super::*;
use serde::{Deserialize, Serialize};
/// Surviving active or evacuated members selected by authored allegiance and identity.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurvivalSelection {
    pub company: Text,
    pub allies: bool,
    pub character: Option<Text>,
    pub role: Option<Role>,
    pub include_evacuated: bool,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnitMatch {
    pub character: Option<Text>,
    pub role: Option<Role>,
    pub side: Option<ConditionSide>,
    pub faction: Option<Text>,
    #[serde(rename = "unitType")]
    pub unit_type: Option<Text>,
    #[serde(rename = "unitTypes")]
    pub unit_types: Vec<Text>,
    #[serde(rename = "aiGroup")]
    pub ai_group: Option<Text>,
    pub retreating: Option<bool>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteSelection {
    pub tile: TileRef,
    #[serde(flatten)]
    pub units: UnitMatch,
    pub through: bool,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProximitySelection {
    pub tile: Option<TileRef>,
    pub target: Option<UnitMatch>,
    pub radius: u16,
    #[serde(flatten)]
    pub units: UnitMatch,
    pub through: bool,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HoldSelection {
    #[serde(skip)]
    pub predicate: u32,
    pub tile: TileRef,
    pub turns: u32,
    pub when: Option<Box<When>>,
    #[serde(flatten)]
    pub units: UnitMatch,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerSelection {
    pub tile: TileRef,
    pub faction: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnSelection {
    #[serde(skip)]
    pub predicate: u32,
    pub tile: TileRef,
    pub faction: Text,
    pub turns: Option<u32>,
    pub when: Option<Box<When>>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TurnSelection {
    Number(u32),
    Faction { number: u32, faction: Text },
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeathSelection {
    #[serde(flatten)]
    pub units: UnitMatch,
    pub initiated_by: Option<Text>,
    pub killed_by: Option<Text>,
    pub defending: Option<bool>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextVariant {
    pub when: Option<When>,
    pub text: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionalText {
    Text(Text),
    Variants(Vec<TextVariant>),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionalHint {
    Text(Text),
    Variant(TextVariant),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    Flag(Text),
    ObjectiveSatisfied(Text),
    Survives(SurvivalSelection),
    HasUnits(UnitMatch),
    NoUnits(UnitMatch),
    VisibleUnits(UnitMatch),
    Owner(OwnerSelection),
    Own(OwnSelection),
    Held(HoldSelection),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptLine {
    pub character: Option<Text>,
    pub speaker: Option<Text>,
    pub portrait: Option<Text>,
    pub text: Text,
    pub sound: Option<Text>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueBranch {
    pub with: Text,
    pub dialogue: Vec<Speech>,
    pub alternative: Vec<Speech>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Speech {
    Branch(DialogueBranch),
    Line(ScriptLine),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnMember {
    #[serde(rename = "type")]
    pub unit_type: Text,
    pub character: Option<Text>,
    pub role: Option<Role>,
    pub items: Vec<Text>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpawnEntry {
    Type(Text),
    Member(SpawnMember),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptSpawn {
    pub faction: Text,
    pub tile: Option<TileRef>,
    pub area: Vec<TileRef>,
    pub radius: u16,
    #[serde(rename = "aiGroup")]
    pub ai_group: Option<Text>,
    pub units: Vec<SpawnEntry>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GiveItem {
    pub item: Text,
    pub tile: Option<TileRef>,
    #[serde(flatten)]
    pub units: UnitMatch,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceItem {
    pub item: Text,
    pub tile: TileRef,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllegianceChange {
    pub units: UnitMatch,
    pub faction: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnitReference {
    Bound(Text),
    Selection(UnitMatch),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkSpec {
    #[serde(rename = "type")]
    pub kind: Text,
    pub tile: TileRef,
    pub light: Option<TileLight>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkSelection {
    #[serde(rename = "type")]
    pub kind: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitLoop {
    pub select: LoopSelection,
    pub actions: Vec<EventAction>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkLoop {
    pub select: MarkSelection,
    pub actions: Vec<EventAction>,
}
/// Text shorthand sets true; the explicit form can also clear a flag.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlagUpdate {
    Name(Text),
    Value { id: Text, state: bool },
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainChange {
    pub tile: TileRef,
    pub terrain: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldGrant {
    pub faction: Text,
    pub amount: i32,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecruitmentUnlock {
    pub faction: Text,
    pub units: Vec<FactionRecruit>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventAction {
    SetMapModifiers { modifiers: StatModifiers },
    SetTimeOfDay { time: TimeOfDay },
    AddConcealment(ConcealmentChange),
    RemoveConcealment(ConcealmentRemoval),
    GrantPerception(PerceptionGrant),
    RemovePerception(PerceptionRemoval),
    RevealTerrain(TerrainReveal),
    SetVisionGroup(VisionGroupAssignment),
    Spawn(ScriptSpawn),
    Give(GiveItem),
    PlaceItem(PlaceItem),
    ChangeAllegiance(AllegianceChange),
    DeleteUnit(UnitReference),
    SetFlag(FlagUpdate),
    Say(Vec<Speech>),
    Sound(Text),
    CreateMark(MarkSpec),
    RemoveMark(Text),
    ForEachUnit(UnitLoop),
    ForEachMark(MarkLoop),
    SetTerrain(TerrainChange),
    Gold(GoldGrant),
    UnlockRecruits(RecruitmentUnlock),
    Victory(Text),
    Defeat(Text),
    Log(Text),
    FailObjective(Text),
    CompleteObjective(Text),
    Recruits(RecruitmentUnlock),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerMarker {
    pub tiles: Vec<TileRef>,
    pub units: Option<UnitMatch>,
    pub visible_in_fog: bool,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventObjective {
    /// Exact faction assigned this objective, independent of controller or diplomacy.
    pub faction: Text,
    pub id: Option<Text>,
    #[serde(rename = "preserveLives")]
    pub preserve_lives: Option<UnitMatch>,
    pub label: Text,
    pub optional: bool,
    #[serde(rename = "visibleWhen")]
    pub visible_when: Option<When>,
    #[serde(rename = "availableWhen")]
    pub available_when: Option<When>,
    /// None is a permanent achievement; Some is a live requirement until victory.
    #[serde(rename = "satisfiedWhen")]
    pub satisfied_when: Option<When>,
    pub intent: EventIntent,
    #[serde(rename = "playerMarker")]
    pub player_marker: Option<PlayerMarker>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptEvent {
    pub observer: Option<Text>,
    pub id: Text,
    pub when: When,
    pub remembers: bool,
    pub priority: i32,
    pub objective: Option<EventObjective>,
    pub actions: Vec<EventAction>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopSelection {
    pub tile: Option<TileRef>,
    #[serde(flatten)]
    pub units: UnitMatch,
}
