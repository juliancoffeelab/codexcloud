//! One evaluator shared by gameplay triggers and presentation resolution.
use crate::data::*;
use serde::{Deserialize, Serialize};

/// Devtool input for the same resolver used by live Rust battles.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationInput {
    #[serde(default)]
    pub factions: Vec<Faction>,
    pub tiles: Vec<Tile>,
    pub units: Vec<StartingUnit>,
    #[serde(default)]
    pub pending: Vec<SpawnUnit>,
    pub lighting: MapLighting,
    #[serde(default)]
    pub marks: Vec<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub memory: Vec<String>,
    pub round: u32,
}
/// Borrowed unit facts used by condition evaluation, independent of storage layout.
#[derive(Clone, Copy)]
pub struct ConditionUnit<'a> {
    pub faction: Option<&'a str>,
    pub side: Side,
    pub kind: &'a str,
    pub character: Option<&'a str>,
    pub ai_group: Option<&'a str>,
    pub position: Option<(i32, i32)>,
}
/// Conditions query current facts without materializing a serializable battlefield.
pub trait ConditionContext {
    fn factions(&self) -> &[Faction];
    fn owns(&self, id: &TileRef, side: &str) -> bool;
    fn position(&self, id: &TileRef) -> Option<(i32, i32)>;
    fn visible_at(&self, position: (i32, i32)) -> bool;
    fn any_unit(&self, predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool;
    fn any_pending(&self, predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool;
    fn flag(&self, id: &str) -> bool;
    fn marks(&self) -> &[String];
    fn round(&self) -> u32;
}
impl ConditionContext for PresentationInput {
    fn factions(&self) -> &[Faction] {
        &self.factions
    }
    fn owns(&self, id: &TileRef, side: &str) -> bool {
        self.tiles
            .iter()
            .any(|t| t.matches_reference(id) && t.owner.as_deref() == Some(side))
    }
    fn position(&self, id: &TileRef) -> Option<(i32, i32)> {
        self.tiles
            .iter()
            .find(|t| t.matches_reference(id))
            .map(|t| (t.col, t.row))
    }
    fn visible_at(&self, position: (i32, i32)) -> bool {
        self.tiles
            .iter()
            .find(|tile| (tile.col, tile.row) == position)
            .is_some_and(|tile| {
                tile.fog.as_ref().is_none_or(|rule| {
                    (rule.remember
                        && self
                            .memory
                            .iter()
                            .any(|key| key.strip_prefix("fog:") == Some(tile.id.as_str())))
                        || matches_fog(&rule.condition, self, Some(&TileRef::Hex(tile.hex())))
                })
            })
    }
    #[inline(always)]
    fn any_unit(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.units.iter().any(|u| {
            predicate(ConditionUnit {
                faction: u.faction.as_deref(),
                side: u.side,
                kind: &u.r#type,
                character: u.character.as_deref(),
                ai_group: u.ai_group.as_deref(),
                position: Some((u.tile.col, u.tile.row)),
            })
        })
    }
    #[inline(always)]
    fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.pending.iter().any(|u| {
            predicate(ConditionUnit {
                faction: u.faction.as_deref(),
                side: u.side,
                kind: &u.r#type,
                character: u.character.as_deref(),
                ai_group: None,
                position: None,
            })
        })
    }
    fn flag(&self, id: &str) -> bool {
        self.choices.iter().any(|v| v == id)
    }
    fn marks(&self) -> &[String] {
        &self.marks
    }
    fn round(&self) -> u32 {
        self.round
    }
}
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn matches(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
) -> bool {
    evaluate(condition, input, target, true)
}
/// Visibility predicates cannot recursively query their own fog rules.
pub fn matches_fog(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
) -> bool {
    evaluate(condition, input, target, false)
}
fn evaluate(
    condition: &Condition,
    input: &impl ConditionContext,
    target: Option<&TileRef>,
    allow_visibility: bool,
) -> bool {
    let unit_matches =
        |unit: ConditionUnit<'_>, side, kind: &Option<String>, character: &Option<String>| {
            faction_matches_side(input.factions(), unit.faction, unit.side, side)
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
                    faction_matches_side(input.factions(), unit.faction, unit.side, *side)
                        && (unit_types.is_empty()
                            || unit_types.iter().any(|kind| kind == unit.kind))
                        && unit
                            .position
                            .is_some_and(|position| input.visible_at(position))
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
            .all(|c| evaluate(c, input, target, allow_visibility)),
        Condition::Any { conditions } => conditions
            .iter()
            .any(|c| evaluate(c, input, target, allow_visibility)),
    }
}

/// Presentation rules read the same borrowed context as gameplay conditions.
pub trait PresentationContext: ConditionContext {
    fn lighting(&self) -> &MapLighting;
    fn visit_lights(&self, visitor: impl FnMut(&Hex, i32, i32, &TileLight));
    fn visit_fog(&self, visitor: impl FnMut(&Hex, &ConditionRule));
    fn fog_index(&self, id: &Hex) -> usize;
    fn fog_remembered(&self, id: &Hex) -> bool;
}
/// Render current facts without cloning or updating durable condition memory.
pub fn render(input: &impl PresentationContext, memory: &[String]) -> BattlefieldPresentation {
    let lighting = input.lighting();
    let mut lights = Vec::new();
    input.visit_lights(|id, col, row, light| {
        let active = light.when.as_ref().is_none_or(|rule| {
            (rule.remember && memory.iter().any(|m| m == &format!("light:{id}")))
                || matches(&rule.condition, input, Some(&TileRef::Hex(*id)))
        });
        if active {
            lights.push(light.at(col, row));
        }
    });
    let mut fog = Vec::new();
    input.visit_fog(|id, rule| {
        let remembered = rule.remember && input.fog_remembered(id);
        if !remembered && !matches(&rule.condition, input, Some(&TileRef::Hex(*id))) {
            fog.push(*id);
        }
    });
    BattlefieldPresentation {
        lighting: BattlefieldLighting {
            time: lighting.time.clone(),
            lights,
        },
        fog,
    }
}
/// Discover new facts without copying existing memory or constructing visual output.
pub fn discover(
    input: &impl PresentationContext,
    memory: &[String],
    known: &mut [u64],
) -> Vec<String> {
    let mut added = Vec::new();
    input.visit_lights(|id, _, _, light| {
        if let Some(rule) = light.when.as_ref().filter(|r| r.remember) {
            if !memory.iter().any(|m| m == &format!("light:{id}"))
                && matches(&rule.condition, input, Some(&TileRef::Hex(*id)))
            {
                added.push(format!("light:{id}"));
            }
        }
    });
    input.visit_fog(|id, rule| {
        if rule.remember {
            let i = input.fog_index(id);
            let bit = 1u64 << (i % 64);
            if known[i / 64] & bit == 0 && matches(&rule.condition, input, Some(&TileRef::Hex(*id)))
            {
                added.push(format!("fog:{id}"));
                known[i / 64] |= bit;
            }
        }
    });
    added
}

/// Returns resolved visuals plus durable facts; callers decide whether to persist facts.
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn resolve(input: &PresentationInput) -> (BattlefieldPresentation, Vec<String>) {
    let mut memory = input.memory.clone();
    let mut rule_met = |rule: &ConditionRule, key: String, target: &TileRef| {
        let satisfied = matches(&rule.condition, input, Some(target));
        if rule.remember {
            if memory.contains(&key) {
                return true;
            }
            if satisfied {
                memory.push(key);
            }
        }
        satisfied
    };
    let lighting = BattlefieldLighting {
        time: input.lighting.time,
        lights: input
            .tiles
            .iter()
            .filter_map(|tile| {
                let light = tile.light.as_ref()?;
                light
                    .when
                    .as_ref()
                    .is_none_or(|rule| {
                        rule_met(
                            rule,
                            format!("light:{}", tile.id),
                            &TileRef::Hex(tile.hex()),
                        )
                    })
                    .then(|| light.at(tile.col, tile.row))
            })
            .collect(),
    };
    let fog = input
        .tiles
        .iter()
        .filter(|tile| {
            tile.fog.as_ref().is_some_and(|rule| {
                !rule_met(rule, format!("fog:{}", tile.id), &TileRef::Hex(tile.hex()))
            })
        })
        .map(Tile::hex)
        .collect();
    (BattlefieldPresentation { lighting, fog }, memory)
}

fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
