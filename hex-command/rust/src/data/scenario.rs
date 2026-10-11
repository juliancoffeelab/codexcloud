//! Authored mission objectives, event triggers and campaign definitions.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// An authored unit placement, including optional named-companion metadata.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartingUnit {
    pub col: i32,
    pub row: i32,
    pub r#type: String,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}
/// An authored objective. Each variant requires the fields its rule consumes.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Goal {
    Defeat {
        label: String,
    },
    Escort {
        label: String,
        tile: String,
    },
    Escape {
        label: String,
        tile: String,
    },
    Control {
        label: String,
        tiles: Vec<String>,
    },
    Seal {
        label: String,
        tiles: Vec<String>,
        #[serde(rename = "unitType")]
        unit_type: Option<String>,
    },
    Event {
        label: String,
        tile: Option<String>,
        event: String,
    },
    Hold {
        label: String,
        tile: String,
        turns: u32,
        #[serde(rename = "earliestRound")]
        earliest_round: u32,
        #[serde(rename = "startEvent")]
        start_event: Option<String>,
    },
    Vigil {
        label: String,
        tile: String,
        turns: u32,
        companion: String,
    },
    Refuge {
        label: String,
        tile: String,
        turns: u32,
    },
}
impl Goal {
    /// Return the structural category without interpreting a string tag.
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
    pub fn label(&self) -> &str {
        match self {
            Self::Defeat { label }
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
    pub fn tile(&self) -> Option<&String> {
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
    pub fn tiles(&self) -> &[String] {
        match self {
            Self::Control { tiles, .. } | Self::Seal { tiles, .. } => tiles,
            _ => &[],
        }
    }
    /// Materialize the applicable tile IDs in their authored order.
    pub fn sites(&self) -> Vec<String> {
        self.tile()
            .map(|t| vec![t.clone()])
            .unwrap_or_else(|| self.tiles().to_vec())
    }
    /// Return the defended-turn requirement for timed objectives.
    pub fn turns(&self) -> Option<u32> {
        match self {
            Self::Hold { turns, .. } | Self::Vigil { turns, .. } | Self::Refuge { turns, .. } => {
                Some(*turns)
            }
            _ => None,
        }
    }
    /// Return the earliest round on which a hold objective can complete.
    pub fn earliest_round(&self) -> Option<u32> {
        match self {
            Self::Hold { earliest_round, .. } => Some(*earliest_round),
            _ => None,
        }
    }
    /// Borrow the event that enables counting for a hold objective.
    pub fn start_event(&self) -> Option<&String> {
        match self {
            Self::Hold { start_event, .. } => start_event.as_ref(),
            _ => None,
        }
    }
    /// Borrow the named companion required at a vigil site.
    pub fn companion(&self) -> Option<&String> {
        match self {
            Self::Vigil { companion, .. } => Some(companion),
            _ => None,
        }
    }
    /// Borrow the optional unit-family requirement for sealing sites.
    pub fn unit_type(&self) -> Option<&String> {
        match self {
            Self::Seal { unit_type, .. } => unit_type.as_ref(),
            _ => None,
        }
    }
    /// Borrow the event marker required by an event objective.
    pub fn event(&self) -> Option<&String> {
        match self {
            Self::Event { event, .. } => Some(event),
            _ => None,
        }
    }
}
/// Campaign event activation; required coordinates and rounds are structural.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Trigger {
    Visit { tile: String },
    Round { round: u32 },
    Control { tile: String, round: Option<u32> },
}
/// A once-per-mission scripted event gated by a typed trigger and prior markers.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub trigger: Trigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub requires_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub unless_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub before_round: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub label: Option<String>,
    pub dialogue: Vec<Dialogue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub gold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub war_dialogue: Option<Vec<Dialogue>>,
}
/// A reinforcement group released on or after its authored round.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wave {
    pub id: String,
    pub round: u32,
    pub label: String,
    pub units: Vec<StartingUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub requires_unsealed: Option<String>,
}
/// The peaceful and escalated outcomes of a refuge-defense mission.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Refuge {
    pub medicine_event: String,
    pub medicine_turns: u32,
    pub escalation: Vec<Dialogue>,
    pub war_victory: Vec<Dialogue>,
    pub war_briefing: String,
    pub war_hints: Vec<String>,
}
/// Chapter content overrides applied after companion losses or story choices.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub briefing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub debrief: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub hints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub intro: Option<Vec<Dialogue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub recruits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub reinforcements: Option<Vec<StartingUnit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub victory: Option<Vec<Dialogue>>,
}
/// A mission map reference and its authored starting roster.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    pub id: String,
    pub units: Vec<StartingUnit>,
}
/// An authored chapter defining objectives, recruits, events and story feedback.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub map: Layout,
    #[serde(default)]
    pub subtitle: String,
    pub briefing: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub enemy_leader_post: Option<String>,
    pub intro: Vec<Dialogue>,
    pub victory: Vec<Dialogue>,
    pub events: Vec<Event>,
    pub waves: Vec<Wave>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub companion_losses: Option<BTreeMap<String, MissionPatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub refuge: Option<Refuge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub without_dain: Option<MissionPatch>,
}
impl Mission {
    /// Apply the authored fields of a story-dependent mission override.
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
/// An ordered set of authored chapters and their final dialogue.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub tagline: String,
    pub description: String,
    pub race: Race,
    pub id: String,
    pub name: String,
    pub leader: String,
    pub missions: Vec<Mission>,
    pub ending: Vec<Dialogue>,
}
impl Trigger {
    /// Return the structural category without interpreting a string tag.
    pub fn kind(&self) -> TriggerKind {
        match self {
            Self::Visit { .. } => TriggerKind::Visit,
            Self::Round { .. } => TriggerKind::Round,
            Self::Control { .. } => TriggerKind::Control,
        }
    }
    /// Borrow the single site required by this objective or trigger, when present.
    pub fn tile(&self) -> Option<&String> {
        match self {
            Self::Visit { tile } | Self::Control { tile, .. } => Some(tile),
            _ => None,
        }
    }
}

/// Displayable current authored map.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapDefinition {
    pub id: String,
    pub revision: u32,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub hidden: bool,
    pub cols: i32,
    pub rows: i32,
    pub terrain: Vec<Tile>,
    pub units: Vec<StartingUnit>,
}
