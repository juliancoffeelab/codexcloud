//! Decode finite vocabularies without copying JavaScript strings onto the heap.
use super::*;
use ai::{Blocks, CampaignBlock, Diplomacy, Planner, Policy, Recruiter};
fn get(value: &JsValue, field: &str) -> JsValue {
    js_sys::Reflect::get(value, &JsValue::from_str(field)).unwrap_or(JsValue::UNDEFINED)
}
fn equal(value: &JsValue, text: &str) -> bool {
    value == &JsValue::from_str(text)
}
fn vocabulary<T: Copy>(value: &JsValue, entries: &[(&str, T)]) -> Option<T> {
    entries
        .iter()
        .find_map(|(text, kind)| equal(value, text).then_some(*kind))
}
fn known_fields(value: &JsValue, allowed: &[&str]) -> bool {
    value.is_object()
        && !value.is_null()
        && !js_sys::Array::is_array(value)
        && js_sys::Object::keys(&js_sys::Object::from(value.clone()))
            .iter()
            .all(|key| allowed.iter().any(|name| equal(&key, name)))
}
pub(super) fn options(value: &JsValue) -> Result<AiOptions, EngineError> {
    fn parse(value: &JsValue) -> Option<AiOptions> {
        if !known_fields(
            value,
            &["policy", "diplomacy", "blocks", "recruitmentLimit"],
        ) {
            return None;
        }
        let policy = vocabulary(
            &get(value, "policy"),
            &[
                ("classic", Policy::Classic),
                ("rush", Policy::Rush),
                ("balanced", Policy::Balanced),
                ("rotated", Policy::Rotated),
                ("situational", Policy::Situational),
                ("predictor", Policy::Predictor),
            ],
        )?;
        let diplomacy = vocabulary(
            &get(value, "diplomacy"),
            &[
                ("restrained", Diplomacy::Restrained),
                ("pragmatic", Diplomacy::Pragmatic),
            ],
        )?;
        let limit = get(value, "recruitmentLimit");
        let recruitment_limit = if limit.is_null() || limit.is_undefined() {
            None
        } else {
            let n = limit.as_f64()?;
            if n < 0. || n > usize::MAX as f64 || n.fract() != 0. {
                return None;
            }
            Some(n as usize)
        };
        let block_value = get(value, "blocks");
        let blocks = if block_value.is_null() || block_value.is_undefined() {
            None
        } else {
            if !known_fields(&block_value, &["recruiter", "campaign", "planner"]) {
                return None;
            }
            Some(Blocks {
                recruiter: vocabulary(
                    &get(&block_value, "recruiter"),
                    &[
                        ("classic", Recruiter::Classic),
                        ("rotation", Recruiter::Rotation),
                        ("frontline", Recruiter::Frontline),
                        ("balanced", Recruiter::Balanced),
                        ("situational", Recruiter::Situational),
                        ("predictor", Recruiter::Predictor),
                        ("aggressive-predictor", Recruiter::AggressivePredictor),
                    ],
                )?,
                campaign: vocabulary(
                    &get(&block_value, "campaign"),
                    &[
                        ("arena", CampaignBlock::Arena),
                        ("mission", CampaignBlock::Mission),
                    ],
                )?,
                planner: vocabulary(
                    &get(&block_value, "planner"),
                    &[
                        ("classic", Planner::Classic),
                        ("full", Planner::Full),
                        ("queued", Planner::Queued),
                    ],
                )?,
            })
        };
        Some(AiOptions {
            policy,
            diplomacy,
            recruitment_limit,
            blocks,
        })
    }
    // Preserve the canonical decoder's diagnostics for invalid requests.
    #[cfg(not(target_arch = "wasm32"))]
    {
        parse(value)
            .map(Ok)
            .unwrap_or_else(|| decode(Ts::<AiOptions>::new_unchecked(value.clone())))
    }
    #[cfg(target_arch = "wasm32")]
    {
        parse(value).ok_or_else(||EngineError::InvalidRequest{message:"Invalid AI options: expected policy, diplomacy, supported block names, and an optional nonnegative integer recruitmentLimit".into()})
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) struct Catalog {
    kinds: js_sys::Map,
    weapons: Vec<js_sys::Map>,
}
#[cfg(target_arch = "wasm32")]
impl Catalog {
    pub fn new(rules: &Rules) -> Self {
        let kinds = js_sys::Map::new();
        let weapons = rules
            .units
            .iter()
            .enumerate()
            .map(|(index, unit)| {
                kinds.set(
                    &JsValue::from_str(&unit.id),
                    &JsValue::from_f64(index as f64),
                );
                let map = js_sys::Map::new();
                for (index, attack) in unit.attacks.iter().enumerate() {
                    map.set(
                        &JsValue::from_str(&attack.id),
                        &JsValue::from_f64(index as f64),
                    );
                }
                map
            })
            .collect();
        Self { kinds, weapons }
    }
}
#[cfg(target_arch = "wasm32")]
pub(super) fn tiles(game: &Game<'_>) -> js_sys::Map {
    let map = js_sys::Map::new();
    for (index, id) in game.board.ids.iter().enumerate() {
        map.set(&JsValue::from_str(id), &JsValue::from_f64(index as f64));
    }
    map
}
/// Valid requests resolve JavaScript string keys through persistent index tables.
/// The canonical decoder still owns diagnostics for malformed input/unknown IDs.
#[cfg(target_arch = "wasm32")]
pub(super) fn operation(
    value: &JsValue,
    catalog: &Catalog,
    tiles: &js_sys::Map,
    game: &Game<'_>,
) -> Option<Operation> {
    let unit = |key| {
        let n = get(value, key).as_f64()?;
        (n >= 0. && n <= u32::MAX as f64 && n.fract() == 0.).then_some(UnitId(n as u32))
    };
    let index = |key, map: &js_sys::Map| map.get(&get(value, key)).as_f64().map(|n| n as usize);
    let optional = |key| {
        let v = get(value, key);
        if v.is_null() || v.is_undefined() {
            Some(None)
        } else {
            v.as_bool().map(Some)
        }
    };
    let kind = get(value, "kind");
    let action = if equal(&kind, "move") {
        Action::Move {
            unit: unit("unit")?,
            tile: index("tile", tiles)?,
        }
    } else if equal(&kind, "attack") {
        let unit = unit("unit")?;
        let actor = game.unit(unit.0)?;
        Action::Attack {
            unit,
            target: unit_id(value, "target")?,
            weapon: index("weapon", &catalog.weapons[actor.kind])?,
        }
    } else if equal(&kind, "recruit") {
        Action::Recruit {
            unit: unit("unit")?,
            kind: index("type", &catalog.kinds)?,
            tile: index("tile", tiles)?,
        }
    } else if equal(&kind, "advance") {
        Action::Advance {
            unit: unit("unit")?,
            kind: index("type", &catalog.kinds)?,
        }
    } else if equal(&kind, "endTurn") {
        Action::EndTurn
    } else if equal(&kind, "setUnitIntent") {
        let record = get(value, "record_retreat");
        Action::SetUnitIntent {
            unit: unit("unit")?,
            retreating: optional("retreating")?,
            rebuilding: optional("rebuilding")?,
            attacked: get(value, "attacked").as_bool()?,
            record_retreat: if record.is_undefined() {
                false
            } else {
                record.as_bool()?
            },
        }
    } else {
        let command = if equal(&kind, "dismissDialogue") {
            Command::DismissDialogue
        } else if equal(&kind, "previousDialogue") {
            Command::PreviousDialogue
        } else if equal(&kind, "skipDialogue") {
            Command::SkipDialogue
        } else if equal(&kind, "retryMission") {
            Command::RetryMission
        } else if equal(&kind, "recallMission") {
            let value = get(value, "units");
            if !js_sys::Array::is_array(&value) {
                return None;
            }
            let array = js_sys::Array::from(&value);
            let mut units = Vec::with_capacity(array.length() as usize);
            for value in array.iter() {
                let n = value.as_f64()?;
                if n < 0. || n > u32::MAX as f64 || n.fract() != 0. {
                    return None;
                }
                units.push(UnitId(n as u32));
            }
            Command::RecallMission { units }
        } else {
            return None;
        };
        return Some(Operation::Wire(command));
    };
    Some(Operation::Planned(action))
}
#[cfg(target_arch = "wasm32")]
fn unit_id(value: &JsValue, key: &str) -> Option<UnitId> {
    let n = get(value, key).as_f64()?;
    (n >= 0. && n <= u32::MAX as f64 && n.fract() == 0.).then_some(UnitId(n as u32))
}
