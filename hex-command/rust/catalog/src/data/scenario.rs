//! Authored mission objectives, event triggers and campaign definitions.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// An authored unit placement, including optional named-companion metadata.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[serde(deny_unknown_fields)]
pub struct StartingUnit {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    pub tile: Hex,
    pub r#type: String,
    pub side: Side,
    #[serde(default, rename = "aiGroup", skip_serializing_if = "Option::is_none")]
    pub ai_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}
/// Select recipients by unit-instance metadata, independent of their current type.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct UnitFilter {
    pub side: Side,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
}
impl UnitFilter {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn accepts(&self, unit: &Unit, names: &Names) -> bool {
        unit.side == self.side
            && self
                .role
                .as_ref()
                .is_none_or(|role| unit.role.as_ref() == Some(role))
            && self
                .character
                .as_ref()
                .is_none_or(|name| names.resolve(unit.character) == Some(name.as_str()))
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ItemReward {
    pub recipient: UnitFilter,
    pub item: Item,
}
/// Remove matching units at a safe destination without killing them.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Evacuation {
    pub tile: TileRef,
    pub units: UnitFilter,
}
/// An authored objective. Each variant requires the fields its rule consumes.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Goal {
    Defeat {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
    },
    Escort {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: TileRef,
    },
    Escape {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: TileRef,
    },
    Control {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tiles: Vec<TileRef>,
    },
    Seal {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tiles: Vec<TileRef>,
        #[serde(rename = "unitType")]
        unit_type: Option<String>,
    },
    Event {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: Option<TileRef>,
        event: String,
    },
    Hold {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: TileRef,
        turns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        companion: Option<String>,
        #[serde(default, rename = "unitTypes", skip_serializing_if = "Vec::is_empty")]
        unit_types: Vec<String>,
        #[serde(default)]
        once: bool,
        #[serde(rename = "earliestRound")]
        earliest_round: u32,
        #[serde(rename = "startEvent")]
        start_event: Option<String>,
    },
    Vigil {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: TileRef,
        turns: u32,
        companion: String,
    },
    Refuge {
        label: String,
        #[serde(
            default,
            rename = "visibleWhen",
            skip_serializing_if = "Option::is_none"
        )]
        visible_when: Option<Condition>,
        tile: TileRef,
        turns: u32,
    },
}
impl Goal {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn visible_when(&self) -> Option<&Condition> {
        match self {
            Self::Defeat { visible_when, .. }
            | Self::Escort { visible_when, .. }
            | Self::Escape { visible_when, .. }
            | Self::Control { visible_when, .. }
            | Self::Seal { visible_when, .. }
            | Self::Event { visible_when, .. }
            | Self::Hold { visible_when, .. }
            | Self::Vigil { visible_when, .. }
            | Self::Refuge { visible_when, .. } => visible_when.as_ref(),
        }
    }
    /// Return the structural category without interpreting a string tag.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn kind(&self) -> GoalKind {
        match self {
            Self::Defeat { .. } => GoalKind::Defeat,
            Self::Escort { .. } => GoalKind::Escort,
            Self::Escape { .. } => GoalKind::Escape,
            Self::Control { .. } => GoalKind::Control,
            Self::Seal { .. } => GoalKind::Seal,
            Self::Event { .. } => GoalKind::Event,
            Self::Hold { .. } => GoalKind::Hold,
            Self::Vigil { .. } => GoalKind::Vigil,
            Self::Refuge { .. } => GoalKind::Refuge,
        }
    }
    /// Borrow the authored objective label.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn label(&self) -> &str {
        match self {
            Self::Defeat { label, .. }
            | Self::Escort { label, .. }
            | Self::Escape { label, .. }
            | Self::Control { label, .. }
            | Self::Seal { label, .. }
            | Self::Event { label, .. }
            | Self::Hold { label, .. }
            | Self::Vigil { label, .. }
            | Self::Refuge { label, .. } => label,
        }
    }
    /// Borrow the single site required by this objective or trigger, when present.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn tile(&self) -> Option<&TileRef> {
        match self {
            Self::Escort { tile, .. }
            | Self::Escape { tile, .. }
            | Self::Hold { tile, .. }
            | Self::Vigil { tile, .. }
            | Self::Refuge { tile, .. } => Some(tile),
            Self::Event { tile, .. } => tile.as_ref(),
            _ => None,
        }
    }
    /// Borrow the ordered sites of a multiple-site objective.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn tiles(&self) -> &[TileRef] {
        match self {
            Self::Control { tiles, .. } | Self::Seal { tiles, .. } => tiles,
            _ => &[],
        }
    }
    /// Materialize the applicable tile IDs in their authored order.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn sites(&self) -> Vec<TileRef> {
        self.tile()
            .map(|t| vec![t.clone()])
            .unwrap_or_else(|| self.tiles().to_vec())
    }
    /// Return the defended-turn requirement for timed objectives.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn turns(&self) -> Option<u32> {
        match self {
            Self::Hold { turns, .. } | Self::Vigil { turns, .. } | Self::Refuge { turns, .. } => {
                Some(*turns)
            }
            _ => None,
        }
    }
    /// Return the earliest round on which a hold objective can complete.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn earliest_round(&self) -> Option<u32> {
        match self {
            Self::Hold { earliest_round, .. } => Some(*earliest_round),
            _ => None,
        }
    }
    /// Borrow the event that enables counting for a hold objective.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn start_event(&self) -> Option<&String> {
        match self {
            Self::Hold { start_event, .. } => start_event.as_ref(),
            _ => None,
        }
    }
    /// Borrow the named companion required at a vigil or named hold site.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn companion(&self) -> Option<&String> {
        match self {
            Self::Vigil { companion, .. } => Some(companion),
            Self::Hold { companion, .. } => companion.as_ref(),
            _ => None,
        }
    }
    /// Borrow the optional unit-family requirement for sealing sites.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn unit_type(&self) -> Option<&String> {
        match self {
            Self::Seal { unit_type, .. } => unit_type.as_ref(),
            _ => None,
        }
    }
    /// Borrow the event marker required by an event objective.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn event(&self) -> Option<&String> {
        match self {
            Self::Event { event, .. } => Some(event),
            _ => None,
        }
    }
}
/// Campaign event activation; required coordinates and rounds are structural.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Trigger {
    Visit { tile: TileRef },
    Condition { condition: Condition },
    Round { round: u32 },
    Held { tile: TileRef, turns: u32 },
    Control { tile: TileRef, round: Option<u32> },
}
/// A stable member identity prevents repeated or skipped spawns after restoration.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SpawnUnit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    pub id: String,
    pub r#type: String,
    pub side: Side,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct SpawnGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    #[serde(default, rename = "aiGroup", skip_serializing_if = "Option::is_none")]
    pub ai_group: Option<String>,
    pub area: Vec<TileRef>,
    pub units: Vec<SpawnUnit>,
}
/// A once-per-mission scripted event gated by a typed trigger and prior markers.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Event {
    /// Recruitment unlocked by this event, derived from its durable marker.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub recruitment: BTreeMap<String, Vec<FactionRecruit>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evacuate: Option<Evacuation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemReward>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ground_items: Vec<super::GroundItem>,
    pub id: String,
    pub trigger: Trigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unless_event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unless_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_round: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub dialogue: Vec<Dialogue>,
    /// Once activated, unplaced members remain pending until an area tile fits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn: Option<SpawnGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gold: Option<i32>,
    /// Durable story outcome carried across chapters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub war_dialogue: Option<Vec<Dialogue>>,
}
/// A reinforcement group released on or after its authored round.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Wave {
    /// Shown only when at least one reinforcement actually arrives.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dialogue: Vec<Dialogue>,
    pub id: String,
    pub round: u32,
    pub label: String,
    pub units: Vec<StartingUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_unsealed: Option<TileRef>,
}
/// A named survivor enters the next mission automatically, outside recall slots.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CompanionArrival {
    pub character: String,
    pub tile: Hex,
}

/// The peaceful and escalated outcomes of a refuge-defense mission.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Refuge {
    pub medicine_event: String,
    pub medicine_turns: u32,
    pub escalation: Vec<Dialogue>,
    pub war_victory: Vec<Dialogue>,
    pub war_briefing: String,
    pub war_hints: Vec<String>,
}
/// Chapter content overrides applied after companion losses or story choices.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct MissionPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub briefing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debrief: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro: Option<Vec<Dialogue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recruitment: Option<BTreeMap<String, Vec<FactionRecruit>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reinforcements: Option<Vec<StartingUnit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub victory: Option<Vec<Dialogue>>,
}
/// A mission map reference and its authored starting roster.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Layout {
    pub id: String,
    pub units: Vec<StartingUnit>,
}
/// Local orders constrain only members of the authored group.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum GroupOrder {
    Hold {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        area: Vec<TileRef>,
        #[serde(rename = "attackRadius")]
        attack_radius: u16,
    },
}
/// Independent mission-controlled enemies; activation and goals do not require leaders.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct EnemyGroup {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<GroupOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<ConditionRule>,
}
/// A scenario-authored defeat rule evaluated through the shared condition vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct LossCondition {
    pub label: String,
    pub condition: Condition,
    pub message: String,
}

/// An authored chapter defining objectives, recruits, events and story feedback.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Mission {
    #[serde(default)]
    pub item_ids: Vec<String>,
    #[serde(default)]
    pub scripted_events: Vec<ScriptEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lose_conditions: Vec<LossCondition>,
    pub map: Layout,
    #[serde(default)]
    pub subtitle: String,
    pub briefing: ConditionalText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debrief: Option<ConditionalText>,
    pub hints: Vec<ConditionalHint>,
    #[serde(default)]
    pub required_companions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arrivals: Vec<CompanionArrival>,
    pub goals: Vec<Goal>,
    /// Authored side objectives, displayed separately and never required for victory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub optional_goals: Vec<Goal>,
    pub recruitment: BTreeMap<String, Vec<FactionRecruit>>,
    pub gold: Gold,
    #[serde(default)]
    pub defend_objectives: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_leader_post: Option<TileRef>,
    pub intro: Vec<Speech>,
    pub victory: Vec<Speech>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub enemy_groups: Vec<EnemyGroup>,
    pub waves: Vec<Wave>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_losses: Option<BTreeMap<String, MissionPatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refuge: Option<Refuge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub without_dain: Option<MissionPatch>,
}
impl Mission {
    /// Named companions may arrive through encounters as well as the opening roster.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn companion_ids(&self) -> impl Iterator<Item = &str> {
        self.map
            .units
            .iter()
            .filter(|u| u.side == Side::Player && u.role != Some(Role::Leader))
            .filter_map(|u| u.character.as_deref())
            .chain(
                self.events
                    .iter()
                    .filter_map(|e| e.spawn.as_ref())
                    .flat_map(|group| &group.units)
                    .filter(|u| u.side == Side::Player && u.role != Some(Role::Leader))
                    .filter_map(|u| u.character.as_deref()),
            )
            .chain(self.arrivals.iter().map(|a| a.character.as_str()))
            .chain(
                self.scripted_events
                    .iter()
                    .flat_map(|e| e.actions_recursive())
                    .filter_map(|a| match a {
                        EventAction::Spawn(s) if s.faction == "player" => Some(&s.units),
                        _ => None,
                    })
                    .flatten()
                    .filter_map(|u| match u {
                        SpawnEntry::Member(m) => m.character.as_deref(),
                        _ => None,
                    }),
            )
    }
    /// Shared progress rules apply to required and optional goals alike.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn all_goals(&self) -> impl Iterator<Item = &Goal> {
        self.goals.iter().chain(&self.optional_goals)
    }
}
/// An ordered set of authored chapters and their final dialogue.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Campaign {
    pub human: bool,
    pub tagline: String,
    pub description: String,
    #[serde(rename = "shortDescription")]
    pub short_description: String,
    pub race: Race,
    pub id: String,
    pub name: String,
    pub leader: String,
    pub characters: Vec<Character>,
    pub missions: Vec<Mission>,
    pub ending: Vec<Speech>,
}
impl Campaign {
    pub fn character(&self, id: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }
}
impl Trigger {
    /// Return the structural category without interpreting a string tag.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn kind(&self) -> TriggerKind {
        match self {
            Self::Visit { .. } => TriggerKind::Visit,
            Self::Condition { .. } => TriggerKind::Condition,
            Self::Round { .. } => TriggerKind::Round,
            Self::Control { .. } => TriggerKind::Control,
            Self::Held { .. } => TriggerKind::Held,
        }
    }
    /// Borrow the single site required by this objective or trigger, when present.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn tile(&self) -> Option<&TileRef> {
        match self {
            Self::Visit { tile } | Self::Held { tile, .. } | Self::Control { tile, .. } => {
                Some(tile)
            }
            _ => None,
        }
    }
}

/// Displayable current authored map.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct MapDefinition {
    pub id: String,
    pub revision: u32,
    pub name: String,
    pub description: String,
    pub human: bool,
    pub cols: i32,
    pub rows: i32,
    pub terrain: Vec<Tile>,
    pub units: Vec<StartingUnit>,
    pub lighting: MapLighting,
    pub presentation: BattlefieldPresentation,
}
