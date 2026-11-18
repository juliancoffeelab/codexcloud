//! One evaluator shared by gameplay triggers and presentation resolution.
use crate::data::Text;
use crate::data::*;
use crate::logic::battle::hex_distance;
use serde::{Deserialize, Serialize};

/// Complete current state for a devtool preview, with explicit hypothetical story facts.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "wasm")]
pub struct PresentationInput {
    pub observer: Text,
    pub state: State,
    pub pending: Vec<PreviewPending>,
    pub marks: Vec<Text>,
    pub choices: Vec<Text>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "wasm")]
pub struct PreviewPending {
    pub faction: Text,
    pub unit_type: Text,
    pub character: Option<Text>,
    pub ai_group: Option<Text>,
}
/// Borrowed unit facts used by condition evaluation, independent of storage layout.
#[derive(Clone, Copy)]
pub struct ConditionUnit<'a> {
    pub faction: &'a str,
    pub kind: &'a str,
    pub character: Option<&'a str>,
    pub ai_group: Option<&'a str>,
    pub position: Option<(i32, i32)>,
}
/// Conditions query current facts without materializing a serializable battlefield.
pub trait ConditionContext {
    fn factions(&self) -> &[Faction];
    fn observer_faction(&self) -> Option<&str>;
    fn matches_side(
        &self,
        unit: ConditionUnit<'_>,
        side: ConditionSide,
        observer: Option<&str>,
    ) -> bool {
        if side == ConditionSide::Observer {
            unit.faction == observer.expect("validated observer selector")
        } else {
            faction_matches_side(self.factions(), unit.faction, side)
        }
    }
    fn visible_unit(&self, unit: ConditionUnit<'_>, observer: &str) -> bool {
        unit.position
            .is_some_and(|position| self.visible_at(position, observer))
    }
    fn owns(&self, id: &TileRef, side: &str) -> bool;
    fn position(&self, id: &TileRef) -> Option<(i32, i32)>;
    fn visible_at(&self, position: (i32, i32), observer: &str) -> bool;
    fn any_unit(&self, predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool;
    fn any_pending(&self, predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool;
    fn flag(&self, id: &str) -> bool;
    fn marks(&self) -> &[Text];
    fn round(&self) -> u32;
}
pub fn matches(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
) -> bool {
    matches_for(condition, input, target, input.observer_faction())
}
/// A world rule's observer is authored independently of the querying viewpoint.
pub fn rule_matches(
    rule: &ConditionRule,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
) -> bool {
    matches_for(&rule.condition, input, target, rule.observer.as_deref())
}
pub fn matches_for(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
    observer: Option<&str>,
) -> bool {
    evaluate(condition, input, target, true, observer)
}
/// Visibility predicates cannot recursively query their own fog rules.
pub fn matches_fog(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
) -> bool {
    evaluate(condition, input, target, false, input.observer_faction())
}
fn evaluate(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
    allow_visibility: bool,
    observer: Option<&str>,
) -> bool {
    let unit_matches =
        |unit: ConditionUnit<'_>, side, kind: &Option<Text>, character: &Option<Text>| {
            input.matches_side(unit, side, observer)
                && kind.as_deref().is_none_or(|kind| unit.kind == kind)
                && character
                    .as_deref()
                    .is_none_or(|character| unit.character == Some(character))
        };
    match condition {
        Condition::Owner { tile, side } => tile
            .as_ref()
            .or(target)
            .is_some_and(|id| input.owns(id, side)),
        Condition::Proximity {
            tile,
            radius,
            side,
            unit_type,
            unit_types,
            character,
        } => {
            let Some(position) = tile.as_ref().or(target).and_then(|id| input.position(id)) else {
                return false;
            };
            input.any_unit(|unit| {
                unit_matches(unit, *side, unit_type, character)
                    && (unit_types.is_empty() || unit_types.iter().any(|kind| kind == unit.kind))
                    && unit
                        .position
                        .is_some_and(|p| hex_distance(position, p) <= *radius)
            })
        }
        Condition::VisibleUnits { side, unit_types } => {
            allow_visibility
                && input.any_unit(|unit| {
                    input.matches_side(unit, *side, observer)
                        && (unit_types.is_empty()
                            || unit_types.iter().any(|kind| kind == unit.kind))
                        && input
                            .visible_unit(unit, observer.expect("validated visibility observer"))
                })
        }
        Condition::HasUnits {
            ai_group,
            side,
            unit_type,
            character,
        } => input.any_unit(|unit| {
            unit_matches(unit, *side, unit_type, character)
                && ai_group
                    .as_deref()
                    .is_none_or(|group| unit.ai_group == Some(group))
        }),
        Condition::NoUnits {
            side,
            unit_type,
            character,
        } => {
            !input.any_unit(|unit| unit_matches(unit, *side, unit_type, character))
                && !input.any_pending(|unit| unit_matches(unit, *side, unit_type, character))
        }
        Condition::Choice { choice } => input.flag(choice),
        Condition::Flag { flag } => input.flag(flag),
        Condition::Event { event } => input
            .marks()
            .iter()
            .any(|mark| mark.strip_prefix("event:") == Some(event)),
        Condition::Round { round } => input.round() >= *round,
        Condition::All { conditions } => conditions
            .iter()
            .all(|c| evaluate(c, input, target, allow_visibility, observer)),
        Condition::Any { conditions } => conditions
            .iter()
            .any(|c| evaluate(c, input, target, allow_visibility, observer)),
    }
}

/// Presentation rules read the same borrowed context as gameplay conditions.
pub trait PresentationContext: ConditionContext {
    fn unlit_tiles(&self) -> Vec<Hex>;
    fn visit_lights(&self, visitor: impl FnMut(&Hex, i32, i32, &TileLight));
    fn visit_tiles(&self, visitor: impl FnMut(&Hex));
    fn fog_hidden(&self, id: &Hex) -> bool;
}
/// Render current facts without cloning or updating durable condition memory.
pub fn render(input: &impl PresentationContext, memory: &[Text]) -> BattlefieldPresentation {
    let mut lights = Vec::new();
    input.visit_lights(|id, col, row, light| {
        let active = light.when.as_ref().is_none_or(|rule| {
            (rule.remember && memory.iter().any(|m| m == &format!("light:{id}")))
                || rule_matches(rule, input, Some(&TileRef::Hex(*id)))
        });
        if active {
            lights.push(light.at(col, row));
        }
    });
    let mut fog = Vec::new();
    input.visit_tiles(|id| {
        if input.fog_hidden(id) {
            fog.push(*id);
        }
    });
    BattlefieldPresentation {
        lighting: BattlefieldLighting {
            unlit_tiles: input.unlit_tiles(),
            lights,
        },
        fog,
        visible_units: Vec::new(),
        unit_appearances: Vec::new(),
    }
}
/// Discover new facts without copying existing memory or constructing visual output.
pub fn discover(input: &impl PresentationContext, memory: &[Text]) -> Vec<Text> {
    let mut added = Vec::new();
    input.visit_lights(|id, _, _, light| {
        if let Some(rule) = light.when.as_ref().filter(|r| r.remember) {
            if !memory.iter().any(|m| m == &format!("light:{id}"))
                && rule_matches(rule, input, Some(&TileRef::Hex(*id)))
            {
                added.push(format!("light:{id}"));
            }
        }
    });
    added
}
