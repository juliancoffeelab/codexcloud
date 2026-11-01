//! Decode the finite browser wire contract without a general value visitor.
//! Domain validation remains in the engine, shared with native callers.
use super::*;
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, String>;
type Decoded<T> = std::result::Result<T, EngineError>;
pub(super) trait Decode: Tsify + Sized {
    fn decode(value: Ts<Self>) -> Decoded<Self>;
}
macro_rules! decoder {
    ($type:ty,$read:ident) => {
        impl Decode for $type {
            fn decode(value: Ts<Self>) -> Decoded<Self> {
                $read(&value.js_value()).map_err(|message| EngineError::InvalidRequest { message })
            }
        }
    };
}
decoder!(Start, start);
decoder!(SavedGame, saved);
decoder!(Command, command);
#[cfg(feature = "runtime-catalog")]
impl Decode for CatalogDocuments {
    fn decode(value: Ts<Self>) -> Decoded<Self> {
        value
            .to_rust()
            .map_err(|error| EngineError::InvalidRequest {
                message: error.to_string(),
            })
    }
}
#[cold]
#[inline(never)]
fn invalid(message: impl Into<String>) -> String {
    message.into()
}
#[cold]
#[inline(never)]
fn field_error(name: &str, error: String) -> String {
    format!("{name}: {error}")
}
// Match struct deserialization: only own enumerable fields belong to a payload.
fn own(value: &JsValue, key: &str) -> Result<bool> {
    let descriptor = js_sys::Reflect::get_own_property_descriptor(
        value.unchecked_ref::<js_sys::Object>(),
        &JsValue::from_str(key),
    )
    .map_err(|_| invalid(format!("cannot read field `{key}`")))?;
    if descriptor.is_undefined() {
        return Ok(false);
    }
    js_sys::Reflect::get(&descriptor, &JsValue::from_str("enumerable"))
        .map(|flag| flag.as_bool() == Some(true))
        .map_err(|_| invalid("cannot read field descriptor"))
}
fn get(value: &JsValue, key: &str) -> Result<JsValue> {
    if !own(value, key)? {
        return Ok(JsValue::UNDEFINED);
    }
    js_sys::Reflect::get(value, &JsValue::from_str(key))
        .map_err(|_| invalid(format!("cannot read field `{key}`")))
}
macro_rules! f {
    ($v:expr,$key:literal,$parse:expr) => {{ $parse(&get($v, $key)?).map_err(|error| field_error($key, error))? }};
}
fn object(value: &JsValue) -> Result<()> {
    if value.is_object() && !value.is_null() && !js_sys::Array::is_array(value) {
        Ok(())
    } else {
        Err(invalid("expected an object"))
    }
}
fn fields(value: &JsValue, allowed: &[&str]) -> Result<()> {
    object(value)?;
    for key in js_sys::Object::keys(&js_sys::Object::from(value.clone())).iter() {
        let key = text(&key)?;
        if !allowed.contains(&key.as_str()) {
            return Err(invalid(format!("unknown field `{key}`")));
        }
    }
    Ok(())
}
#[inline(never)]
fn text(value: &JsValue) -> Result<String> {
    value
        .as_string()
        .ok_or_else(|| invalid("expected a string"))
}
#[inline(never)]
fn boolean(value: &JsValue) -> Result<bool> {
    value.as_bool().ok_or_else(|| invalid("expected a boolean"))
}
#[inline(never)]
fn uint(value: &JsValue) -> Result<u32> {
    match value.as_f64() {
        Some(n) if n >= 0. && n <= u32::MAX as f64 && n.fract() == 0. => Ok(n as u32),
        _ => Err(invalid("expected an unsigned 32-bit integer")),
    }
}
#[inline(never)]
fn integer(value: &JsValue) -> Result<i32> {
    match value.as_f64() {
        Some(n) if n >= i32::MIN as f64 && n <= i32::MAX as f64 && n.fract() == 0. => Ok(n as i32),
        _ => Err(invalid("expected a signed 32-bit integer")),
    }
}
#[inline(never)]
fn side(value: &JsValue) -> Result<Side> {
    if value == &JsValue::from_str("player") {
        Ok(Side::Player)
    } else if value == &JsValue::from_str("enemy") {
        Ok(Side::Enemy)
    } else {
        Err(invalid("expected `player` or `enemy`"))
    }
}
#[inline(never)]
fn role(value: &JsValue) -> Result<Role> {
    if value == &JsValue::from_str("leader") {
        Ok(Role::Leader)
    } else if value == &JsValue::from_str("courier") {
        Ok(Role::Courier)
    } else {
        Err(invalid("expected `leader` or `courier`"))
    }
}
#[inline(never)]
fn optional<T>(value: &JsValue, parse: fn(&JsValue) -> Result<T>) -> Result<Option<T>> {
    if value.is_null() || value.is_undefined() {
        Ok(None)
    } else {
        parse(value).map(Some)
    }
}
#[inline(never)]
fn array<T>(value: &JsValue, parse: fn(&JsValue) -> Result<T>) -> Result<Vec<T>> {
    if !js_sys::Array::is_array(value) {
        return Err(invalid("expected an array"));
    }
    let values = js_sys::Array::from(value);
    let mut output = Vec::with_capacity(values.length() as usize);
    for index in 0..values.length() {
        output.push(
            parse(&values.get(index)).map_err(|error| field_error(&index.to_string(), error))?,
        );
    }
    Ok(output)
}
fn strings(v: &JsValue) -> Result<Vec<String>> {
    array(v, text)
}
fn opt_text(v: &JsValue) -> Result<Option<String>> {
    optional(v, text)
}
fn opt_uint(v: &JsValue) -> Result<Option<u32>> {
    optional(v, uint)
}
fn opt_bool(v: &JsValue) -> Result<Option<bool>> {
    optional(v, boolean)
}
fn opt_side(v: &JsValue) -> Result<Option<Side>> {
    optional(v, side)
}
fn opt_role(v: &JsValue) -> Result<Option<Role>> {
    optional(v, role)
}
fn pair(v: &JsValue) -> Result<[String; 2]> {
    strings(v)?
        .try_into()
        .map_err(|_| invalid("expected exactly two faction identifiers"))
}
fn opt_pair(v: &JsValue) -> Result<Option<[String; 2]>> {
    optional(v, pair)
}
fn start(v: &JsValue) -> Result<Start> {
    object(v)?;
    let kind = f!(v, "kind", text);
    match kind.as_str() {
        "scenario" => Ok(Start::Scenario {
            map: f!(v, "map", text),
            revision: f!(v, "revision", opt_uint),
            armies: f!(v, "armies", opt_pair),
            first: f!(v, "first", opt_side),
        }),
        "campaign" => Ok(Start::Campaign {
            id: f!(v, "id", text),
        }),
        _ => Err(invalid("unknown start kind")),
    }
}
fn saved(v: &JsValue) -> Result<SavedGame> {
    fields(v, &["seed", "state"])?;
    Ok(SavedGame {
        seed: f!(v, "seed", uint),
        state: f!(v, "state", state),
    })
}
fn gold(v: &JsValue) -> Result<Gold> {
    object(v)?;
    Ok(Gold {
        player: f!(v, "player", integer),
        enemy: f!(v, "enemy", integer),
    })
}
fn state(v: &JsValue) -> Result<State> {
    object(v)?;
    let combat = get(v, "lastCombat")?;
    let present = own(v, "lastCombat")?;
    Ok(State {
        armies: f!(v, "armies", opt_pair),
        map_id: f!(v, "mapId", text),
        map_revision: f!(v, "mapRevision", uint),
        cols: f!(v, "cols", integer),
        rows: f!(v, "rows", integer),
        round: f!(v, "round", uint),
        turn: f!(v, "turn", side),
        gold: f!(v, "gold", gold),
        winner: f!(v, "winner", opt_side),
        next_id: f!(v, "nextId", uint),
        combat_rng: f!(v, "combatRng", opt_uint),
        log: f!(v, "log", strings),
        tiles: f!(v, "tiles", |v| array(v, tile)),
        units: f!(v, "units", |v| array(v, unit)),
        campaign: f!(v, "campaign", |v| optional(v, campaign)),
        last_combat: if present {
            Some(optional(&combat, last_combat)?)
        } else {
            None
        },
    })
}
#[inline(never)]
fn direction(v: &JsValue) -> Result<HexDirection> {
    match text(v)?.as_str() {
        "e" => Ok(HexDirection::E),
        "se" => Ok(HexDirection::Se),
        "sw" => Ok(HexDirection::Sw),
        "w" => Ok(HexDirection::W),
        "nw" => Ok(HexDirection::Nw),
        "ne" => Ok(HexDirection::Ne),
        _ => Err(invalid("unknown hex direction")),
    }
}
fn directions(v: &JsValue) -> Result<Option<Vec<HexDirection>>> {
    optional(v, |v| array(v, direction))
}
fn tile(v: &JsValue) -> Result<Tile> {
    object(v)?;
    Ok(Tile {
        id: f!(v, "id", text),
        col: f!(v, "col", integer),
        row: f!(v, "row", integer),
        terrain: f!(v, "terrain", text),
        owner: f!(v, "owner", opt_side),
        connections: f!(v, "connections", directions),
        dead_ends: f!(v, "deadEnds", directions),
        floor: f!(v, "floor", opt_text),
    })
}
fn unit(v: &JsValue) -> Result<PublicUnit> {
    object(v)?;
    Ok(PublicUnit {
        id: f!(v, "id", text),
        r#type: f!(v, "type", text),
        side: f!(v, "side", side),
        tile: f!(v, "tile", text),
        hp: f!(v, "hp", integer),
        moved: f!(v, "moved", boolean),
        attacked: f!(v, "attacked", boolean),
        experience: f!(v, "experience", opt_uint),
        name: f!(v, "name", opt_text),
        role: f!(v, "role", opt_role),
        retreating: f!(v, "retreating", opt_bool),
        rebuilding: f!(v, "rebuilding", opt_bool),
    })
}
fn last_combat(v: &JsValue) -> Result<LastCombat> {
    object(v)?;
    Ok(LastCombat {
        attacker_id: f!(v, "attackerId", text),
        attack_id: f!(v, "attackId", text),
        sound: f!(v, "sound", text),
    })
}
fn veteran(v: &JsValue) -> Result<Veteran> {
    object(v)?;
    Ok(Veteran {
        r#type: f!(v, "type", text),
        experience: f!(v, "experience", uint),
        name: f!(v, "name", opt_text),
        role: f!(v, "role", opt_role),
    })
}
fn dialogue(v: &JsValue) -> Result<Dialogue> {
    object(v)?;
    Ok(Dialogue {
        speaker: f!(v, "speaker", text).into(),
        portrait: f!(v, "portrait", text).into(),
        text: f!(v, "text", text).into(),
    })
}
fn counter(v: &JsValue) -> Result<Counter> {
    object(v)?;
    Ok(Counter {
        held_turns: f!(v, "heldTurns", uint),
        last_counted_round: f!(v, "lastCountedRound", uint),
    })
}
fn refuge(v: &JsValue) -> Result<RefugeCounter> {
    object(v)?;
    Ok(RefugeCounter {
        held_turns: f!(v, "heldTurns", uint),
        last_counted_round: f!(v, "lastCountedRound", uint),
        escalated: f!(v, "escalated", boolean),
    })
}
fn holds(v: &JsValue) -> Result<BTreeMap<String, Counter>> {
    object(v)?;
    let mut output = BTreeMap::new();
    for key in js_sys::Object::keys(&js_sys::Object::from(v.clone())).iter() {
        let key = text(&key)?;
        output.insert(key.clone(), counter(&get(v, &key)?)?);
    }
    Ok(output)
}
fn campaign(v: &JsValue) -> Result<CampaignState> {
    object(v)?;
    let mission = f!(v, "mission", uint) as usize;
    let dialogue_index = f!(v, "dialogueIndex", uint) as usize;
    Ok(CampaignState {
        id: f!(v, "id", text),
        mission,
        recall: f!(v, "recall", strings),
        veterans: f!(v, "veterans", |v| optional(v, |v| array(v, veteran))),
        story_veterans: f!(v, "storyVeterans", |v| optional(v, |v| array(v, veteran))),
        fallen_companions: f!(v, "fallenCompanions", |v| optional(v, strings)),
        carry_gold: f!(v, "carryGold", integer),
        marks: f!(v, "marks", strings),
        dialogue: f!(v, "dialogue", |v| array(v, dialogue)),
        dialogue_index,
        outcome: f!(v, "outcome", opt_text),
        completed: f!(v, "completed", boolean),
        refuge: f!(v, "refuge", |v| optional(v, refuge)),
        vigil: f!(v, "vigil", |v| optional(v, counter)),
        holds: f!(v, "holds", |v| optional(v, holds)),
        choices: f!(v, "choices", |v| optional(v, strings)),
    })
}
fn id(v: &JsValue) -> Result<UnitId> {
    uint(v).map(UnitId)
}
fn command(v: &JsValue) -> Result<Command> {
    object(v)?;
    let kind = f!(v, "kind", text);
    match kind.as_str() {
        "move" => Ok(Command::Move {
            unit: f!(v, "unit", id),
            tile: TileId(f!(v, "tile", text)),
        }),
        "attack" => Ok(Command::Attack {
            unit: f!(v, "unit", id),
            target: f!(v, "target", id),
            weapon: WeaponId(f!(v, "weapon", text)),
        }),
        "recruit" => Ok(Command::Recruit {
            unit: f!(v, "unit", id),
            kind: UnitTypeId(f!(v, "type", text)),
            tile: TileId(f!(v, "tile", text)),
        }),
        "advance" => Ok(Command::Advance {
            unit: f!(v, "unit", id),
            kind: UnitTypeId(f!(v, "type", text)),
        }),
        "endTurn" => Ok(Command::EndTurn),
        "dismissDialogue" => Ok(Command::DismissDialogue),
        "previousDialogue" => Ok(Command::PreviousDialogue),
        "skipDialogue" => Ok(Command::SkipDialogue),
        "retryMission" => Ok(Command::RetryMission),
        "recallMission" => Ok(Command::RecallMission {
            units: f!(v, "units", |v| array(v, id)),
        }),
        "setUnitIntent" => {
            let record = get(v, "record_retreat")?;
            Ok(Command::SetUnitIntent {
                unit: f!(v, "unit", id),
                retreating: f!(v, "retreating", opt_bool),
                rebuilding: f!(v, "rebuilding", opt_bool),
                attacked: f!(v, "attacked", boolean),
                record_retreat: if record.is_undefined() {
                    false
                } else {
                    boolean(&record)?
                },
            })
        }
        _ => Err(invalid("unknown command kind")),
    }
}
