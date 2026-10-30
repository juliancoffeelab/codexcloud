//! Authored mission objectives, event triggers and campaign definitions.
use std::borrow::Cow;

use super::*;
use serde::{Deserialize, Serialize};
/// An authored unit placement, including optional named-companion metadata.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
pub struct StartingUnit {
    pub col: i32,
    pub row: i32,
    pub r#type: Cow<'static, str>,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}
/// An authored objective. Each variant requires the fields its rule consumes.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Goal {
    Defeat {
        label: Cow<'static, str>,
    },
    Escort {
        label: Cow<'static, str>,
        tile: Cow<'static, str>,
    },
    Escape {
        label: Cow<'static, str>,
        tile: Cow<'static, str>,
    },
    Control {
        label: Cow<'static, str>,
        tiles: Cow<'static, [Cow<'static, str>]>,
    },
    Seal {
        label: Cow<'static, str>,
        tiles: Cow<'static, [Cow<'static, str>]>,
        #[serde(rename = "unitType")]
        unit_type: Option<Cow<'static, str>>,
    },
    Event {
        label: Cow<'static, str>,
        tile: Option<Cow<'static, str>>,
        event: Cow<'static, str>,
    },
    Hold {
        label: Cow<'static, str>,
        tile: Cow<'static, str>,
        turns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        companion: Option<Cow<'static, str>>,
        #[serde(rename = "earliestRound")]
        earliest_round: u32,
        #[serde(rename = "startEvent")]
        start_event: Option<Cow<'static, str>>,
    },
    Vigil {
        label: Cow<'static, str>,
        tile: Cow<'static, str>,
        turns: u32,
        companion: Cow<'static, str>,
    },
    Refuge {
        label: Cow<'static, str>,
        tile: Cow<'static, str>,
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
    pub fn tile(&self) -> Option<&str> {
        match self {
            Self::Escort { tile, .. }
            | Self::Escape { tile, .. }
            | Self::Hold { tile, .. }
            | Self::Vigil { tile, .. }
            | Self::Refuge { tile, .. } => Some(tile.as_ref()),
            Self::Event { tile, .. } => tile.as_deref(),
            _ => None,
        }
    }
    /// Borrow the ordered sites of a multiple-site objective.
    pub fn tiles(&self) -> &[Cow<'static, str>] {
        match self {
            Self::Control { tiles, .. } | Self::Seal { tiles, .. } => tiles,
            _ => &[],
        }
    }
    /// Materialize the applicable tile IDs in their authored order.
    pub fn sites(&self) -> Vec<String> {
        self.tile()
            .map(|t| vec![t.to_string()])
            .unwrap_or_else(|| self.tiles().iter().map(ToString::to_string).collect())
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
    pub fn start_event(&self) -> Option<&str> {
        match self {
            Self::Hold { start_event, .. } => start_event.as_deref(),
            _ => None,
        }
    }
    /// Borrow the named companion required at a vigil or named hold site.
    pub fn companion(&self) -> Option<&str> {
        match self {
            Self::Vigil { companion, .. } => Some(companion.as_ref()),
            Self::Hold { companion, .. } => companion.as_deref(),
            _ => None,
        }
    }
    /// Borrow the optional unit-family requirement for sealing sites.
    pub fn unit_type(&self) -> Option<&str> {
        match self {
            Self::Seal { unit_type, .. } => unit_type.as_deref(),
            _ => None,
        }
    }
    /// Borrow the event marker required by an event objective.
    pub fn event(&self) -> Option<&str> {
        match self {
            Self::Event { event, .. } => Some(event.as_ref()),
            _ => None,
        }
    }
}
/// Campaign event activation; required coordinates and rounds are structural.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Trigger {
    Visit {
        tile: Cow<'static, str>,
    },
    Round {
        round: u32,
    },
    Control {
        tile: Cow<'static, str>,
        round: Option<u32>,
    },
}
/// A once-per-mission scripted event gated by a typed trigger and prior markers.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Cow<'static, str>,
    pub trigger: Trigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub requires_event: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub unless_event: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub before_round: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub label: Option<Cow<'static, str>>,
    pub dialogue: Cow<'static, [Dialogue]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub gold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub war_dialogue: Option<Cow<'static, [Dialogue]>>,
}
/// A reinforcement group released on or after its authored round.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase")]
pub struct Wave {
    pub id: Cow<'static, str>,
    pub round: u32,
    pub label: Cow<'static, str>,
    pub units: Cow<'static, [StartingUnit]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub requires_unsealed: Option<Cow<'static, str>>,
}
/// The peaceful and escalated outcomes of a refuge-defense mission.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase")]
pub struct Refuge {
    pub medicine_event: Cow<'static, str>,
    pub medicine_turns: u32,
    pub escalation: Cow<'static, [Dialogue]>,
    pub war_victory: Cow<'static, [Dialogue]>,
    pub war_briefing: Cow<'static, str>,
    pub war_hints: Cow<'static, [Cow<'static, str>]>,
}
/// Chapter content overrides applied after companion losses or story choices.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase")]
pub struct MissionPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub briefing: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub debrief: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub hints: Option<Cow<'static, [Cow<'static, str>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub intro: Option<Cow<'static, [Dialogue]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub recruits: Option<Cow<'static, [Cow<'static, str>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub reinforcements: Option<Cow<'static, [StartingUnit]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub victory: Option<Cow<'static, [Dialogue]>>,
}
/// A mission map reference and its authored starting roster.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
pub struct Layout {
    pub id: Cow<'static, str>,
    pub units: Cow<'static, [StartingUnit]>,
}
/// An authored chapter defining objectives, recruits, events and story feedback.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub map: Layout,
    #[serde(default)]
    pub subtitle: Cow<'static, str>,
    pub briefing: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub debrief: Option<Cow<'static, str>>,
    pub hints: Cow<'static, [Cow<'static, str>]>,
    #[serde(default)]
    pub required_companions: Cow<'static, [Cow<'static, str>]>,
    pub goals: Cow<'static, [Goal]>,
    pub recruits: Cow<'static, [Cow<'static, str>]>,
    pub enemy_recruits: Cow<'static, [Cow<'static, str>]>,
    pub gold: i32,
    pub enemy_gold: i32,
    #[serde(default)]
    pub defend_objectives: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub enemy_leader_post: Option<Cow<'static, str>>,
    pub intro: Cow<'static, [Dialogue]>,
    pub victory: Cow<'static, [Dialogue]>,
    pub events: Cow<'static, [Event]>,
    pub waves: Cow<'static, [Wave]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[cfg_attr(feature = "wasm", tsify(type = "Record<string, MissionPatch>"))]
    pub companion_losses: Option<Table<Cow<'static, str>, MissionPatch>>,
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
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
pub struct Campaign {
    #[serde(rename = "playerDiplomacy")]
    pub player_diplomacy: Cow<'static, [Cow<'static, str>]>,
    pub tagline: Cow<'static, str>,
    pub description: Cow<'static, str>,
    #[serde(rename = "shortDescription")]
    pub short_description: Cow<'static, str>,
    pub race: Race,
    pub id: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub leader: Cow<'static, str>,
    pub missions: Cow<'static, [Mission]>,
    pub ending: Cow<'static, [Dialogue]>,
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
    pub fn tile(&self) -> Option<&str> {
        match self {
            Self::Visit { tile } | Self::Control { tile, .. } => Some(tile.as_ref()),
            _ => None,
        }
    }
}

/// Displayable current authored map.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
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
