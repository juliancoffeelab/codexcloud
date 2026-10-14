//! Direct Arrow IPC records: lossless snapshot dictionaries and ordered phase references.
//! One independently readable IPC stream occupies each checksummed journal range.
use super::{error::SimulationError, summaries, types::*};
use crate::data::{CampaignState, Gold, Objective, Progress, PublicUnit, Records, Side};
use arrow_array::{
    Array, ArrayRef, BooleanArray, Int8Array, Int32Array, ListArray, RecordBatch, StringArray,
    StructArray, UInt32Array,
    builder::{Int8Builder, ListBuilder, StringBuilder, UInt32Builder},
};
use arrow_buffer::{NullBuffer, OffsetBuffer};
use arrow_ipc::{reader::StreamReader, writer::StreamWriter};
use arrow_schema::{Field, Schema};
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_json::{Map, Value};
use std::{io::Cursor, sync::Arc};

pub(crate) const FORMAT: &str = "arrow-ipc-snapshot-dictionaries-v2";

#[derive(Serialize)]
struct PhaseHeader<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    planning: Option<&'a crate::ai::orders::TurnDiagnostics>,
    side: Side,
    round: u32,
    #[serde(flatten)]
    records: &'a Records,
    living: &'a Sides<usize>,
    health: &'a Sides<i32>,
    gold: &'a Gold,
    goals: &'a [Progress],
    objectives: &'a [Objective],
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign: Option<&'a CampaignState>,
}
struct Dictionary<'a, T> {
    identities: FxHashMap<usize, u32>,
    values: Vec<&'a T>,
}
impl<T> Default for Dictionary<'_, T> {
    fn default() -> Self {
        Self {
            identities: FxHashMap::default(),
            values: vec![],
        }
    }
}
impl<'a, T> Dictionary<'a, T> {
    fn insert(&mut self, value: &'a Arc<T>) -> Result<u32, SimulationError> {
        let pointer = Arc::as_ptr(value) as usize;
        if let Some(&index) = self.identities.get(&pointer) {
            return Ok(index);
        }
        let index =
            u32::try_from(self.values.len()).map_err(|_| "Snapshot dictionary too large")?;
        self.values.push(value.as_ref());
        self.identities.insert(pointer, index);
        Ok(index)
    }
}
fn dictionary_column(values: ArrayRef, rows: usize) -> Result<ArrayRef, SimulationError> {
    let count = i32::try_from(values.len()).map_err(|_| "Snapshot dictionary too large")?;
    let offsets = std::iter::once(0)
        .chain(std::iter::repeat_n(count, rows))
        .collect::<Vec<_>>();
    let valid = std::iter::once(true)
        .chain(std::iter::repeat_n(false, rows - 1))
        .collect::<Vec<_>>();
    Ok(Arc::new(ListArray::try_new(
        Arc::new(Field::new("item", values.data_type().clone(), true)),
        OffsetBuffer::new(offsets.into()),
        values,
        Some(NullBuffer::from(valid)),
    )?))
}
fn unit_column(
    dictionary: Dictionary<'_, PublicUnit>,
    rows: usize,
) -> Result<ArrayRef, SimulationError> {
    let values = dictionary.values;
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from_iter_values(
            values.iter().map(|u| u.id.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            values.iter().map(|u| u.r#type.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(values.iter().map(|u| {
            if u.side == Side::Player {
                "player"
            } else {
                "enemy"
            }
        }))),
        Arc::new(StringArray::from_iter_values(
            values.iter().map(|u| u.tile.as_str()),
        )),
        Arc::new(Int32Array::from_iter_values(values.iter().map(|u| u.hp))),
        Arc::new(BooleanArray::from_iter(
            values.iter().map(|u| Some(u.moved)),
        )),
        Arc::new(BooleanArray::from_iter(
            values.iter().map(|u| Some(u.attacked)),
        )),
        Arc::new(UInt32Array::from_iter(values.iter().map(|u| u.experience))),
        Arc::new(StringArray::from_iter(
            values.iter().map(|u| u.name.as_deref()),
        )),
        Arc::new(StringArray::from_iter(
            values.iter().map(|u| u.role.as_ref().map(|r| r.as_str())),
        )),
        Arc::new(BooleanArray::from_iter(values.iter().map(|u| u.retreating))),
        Arc::new(BooleanArray::from_iter(values.iter().map(|u| u.rebuilding))),
    ];
    let names = [
        "id",
        "type",
        "side",
        "tile",
        "hp",
        "moved",
        "attacked",
        "experience",
        "name",
        "role",
        "retreating",
        "rebuilding",
    ];
    let fields = names
        .into_iter()
        .zip(&columns)
        .enumerate()
        .map(|(i, (name, column))| Field::new(name, column.data_type().clone(), i >= 7))
        .collect::<Vec<_>>();
    dictionary_column(
        Arc::new(StructArray::try_new(fields.into(), columns, None)?),
        rows,
    )
}
fn ownership_column(
    dictionary: Dictionary<'_, OwnershipSnapshot>,
    rows: usize,
) -> Result<ArrayRef, SimulationError> {
    let mut builder = ListBuilder::new(Int8Builder::new());
    for value in dictionary.values {
        for owner in &value.owners {
            builder
                .values()
                .append_option(owner.map(|s| s.index() as i8));
        }
        builder.append(true);
    }
    dictionary_column(Arc::new(builder.finish()), rows)
}

pub(crate) fn encode(
    result: &MatchResult,
    summary: &summaries::DetailedSummary<'_>,
) -> Result<Vec<u8>, SimulationError> {
    let mut header = StringBuilder::new();
    let mut chapter = UInt32Builder::new();
    let mut phase_json = StringBuilder::new();
    let mut before = ListBuilder::new(UInt32Builder::new());
    let mut units = ListBuilder::new(UInt32Builder::new());
    let mut ownership = UInt32Builder::new();
    let mut unit_dictionary = Dictionary::default();
    let mut ownership_dictionary = Dictionary::default();
    header.append_value(summaries::arrow_header(summary)?);
    chapter.append_null();
    phase_json.append_null();
    before.append(false);
    units.append(false);
    ownership.append_null();
    let chapters: Vec<&[Phase]> = match result {
        MatchResult::Arena(r) => vec![&r.phases],
        MatchResult::Campaign(r) => r.missions.iter().map(|m| m.phases.as_slice()).collect(),
    };
    let mut rows = 1;
    for (index, phases) in chapters.into_iter().enumerate() {
        for phase in phases {
            header.append_null();
            chapter.append_value(index as u32);
            phase_json.append_value(serde_json::to_string(&PhaseHeader {
                planning: phase.planning.as_ref(),
                side: phase.side,
                round: phase.round,
                records: &phase.records,
                living: &phase.living,
                health: &phase.health,
                gold: &phase.gold,
                goals: &phase.goals,
                objectives: &phase.objectives,
                campaign: phase.campaign.as_ref(),
            })?);
            for unit in &phase.before {
                before.values().append_value(unit_dictionary.insert(unit)?);
            }
            before.append(true);
            for unit in &phase.units {
                units.values().append_value(unit_dictionary.insert(unit)?);
            }
            units.append(true);
            ownership.append_value(ownership_dictionary.insert(&phase.ownership)?);
            rows += 1;
        }
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(header.finish()),
        Arc::new(chapter.finish()),
        Arc::new(phase_json.finish()),
        Arc::new(before.finish()),
        Arc::new(units.finish()),
        Arc::new(ownership.finish()),
        unit_column(unit_dictionary, rows)?,
        ownership_column(ownership_dictionary, rows)?,
    ];
    let names = [
        "header_json",
        "chapter",
        "phase_json",
        "before",
        "units",
        "ownership",
        "unit_snapshots",
        "ownership_snapshots",
    ];
    let schema = Arc::new(Schema::new_with_metadata(
        names
            .into_iter()
            .zip(&columns)
            .map(|(name, column)| Field::new(name, column.data_type().clone(), true))
            .collect::<Vec<_>>(),
        std::collections::HashMap::from([("hex-command.detail-format".into(), FORMAT.into())]),
    ));
    let batch = RecordBatch::try_new(schema.clone(), columns)?;
    let mut bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut bytes, &schema)?;
        writer.write(&batch)?;
        writer.finish()?;
    }
    Ok(bytes)
}

fn column<T: Array + 'static>(array: &dyn Array) -> Result<&T, SimulationError> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| "Unexpected Arrow column type".into())
}
fn unit_dictionary(array: &dyn Array) -> Result<Vec<Value>, SimulationError> {
    let list = column::<ListArray>(array)?;
    if list.len() == 0 || list.is_null(0) {
        return Err("Missing Arrow snapshot dictionary".into());
    }
    let values = list.value(0);
    let units = column::<StructArray>(values.as_ref())?;
    (0..units.len())
        .map(|row| {
            let mut object = Map::new();
            for (field, values) in units.fields().iter().zip(units.columns()) {
                if values.is_null(row) {
                    if !field.is_nullable() {
                        return Err("Null required unit field".into());
                    }
                    continue;
                }
                let value = match field.data_type() {
                    arrow_schema::DataType::Utf8 => {
                        Value::from(column::<StringArray>(values.as_ref())?.value(row))
                    }
                    arrow_schema::DataType::Int32 => {
                        Value::from(column::<Int32Array>(values.as_ref())?.value(row))
                    }
                    arrow_schema::DataType::UInt32 => {
                        Value::from(column::<UInt32Array>(values.as_ref())?.value(row))
                    }
                    arrow_schema::DataType::Boolean => {
                        Value::from(column::<BooleanArray>(values.as_ref())?.value(row))
                    }
                    _ => return Err("Unexpected unit field type".into()),
                };
                object.insert(field.name().clone(), value);
            }
            Ok(Value::Object(object))
        })
        .collect()
}
fn owners(array: &ListArray, index: usize, tiles: &Value) -> Result<Value, SimulationError> {
    if index >= array.len() || array.is_null(index) {
        return Err("Invalid ownership index".into());
    }
    let values = array.value(index);
    let owners = column::<Int8Array>(values.as_ref())?;
    let tiles = tiles.as_array().ok_or("Missing chapter tiles")?;
    if owners.len() != tiles.len() {
        return Err("Ownership length differs from chapter tiles".into());
    }
    let values = tiles
        .iter()
        .zip(owners.iter())
        .map(|(tile, owner)| {
            let owner = match owner {
                None => Value::Null,
                Some(0) => Value::from("player"),
                Some(1) => Value::from("enemy"),
                _ => return Err("Invalid owner side".into()),
            };
            Ok(serde_json::json!({"tile":tile["id"], "owner":owner}))
        })
        .collect::<Result<Vec<Value>, SimulationError>>()?;
    Ok(Value::Array(values))
}
fn roster(array: &ListArray, row: usize, dictionary: &[Value]) -> Result<Value, SimulationError> {
    if array.is_null(row) {
        return Err("Missing Arrow roster".into());
    }
    let values = array.value(row);
    let ids = column::<UInt32Array>(values.as_ref())?;
    let units = ids
        .iter()
        .map(|id| {
            dictionary
                .get(id.ok_or("Null roster index")? as usize)
                .cloned()
                .ok_or_else(|| "Invalid roster index".into())
        })
        .collect::<Result<Vec<Value>, SimulationError>>()?;
    Ok(Value::Array(units))
}
pub(crate) fn decode(bytes: &[u8]) -> Result<Value, SimulationError> {
    let mut reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    if reader
        .schema()
        .metadata()
        .get("hex-command.detail-format")
        .map(String::as_str)
        != Some(FORMAT)
    {
        return Err("Unknown Arrow gameplay format".into());
    }
    let batch = reader.next().ok_or("Missing Arrow gameplay batch")??;
    if reader.next().is_some() || batch.num_columns() != 8 || batch.num_rows() == 0 {
        return Err("Invalid Arrow gameplay batch".into());
    }
    let headers = column::<StringArray>(batch.column(0).as_ref())?;
    if headers.is_null(0) {
        return Err("Missing Arrow gameplay header".into());
    }
    let mut result: Value = serde_json::from_str(headers.value(0))?;
    let chapters = column::<UInt32Array>(batch.column(1).as_ref())?;
    let phases = column::<StringArray>(batch.column(2).as_ref())?;
    let before = column::<ListArray>(batch.column(3).as_ref())?;
    let units = column::<ListArray>(batch.column(4).as_ref())?;
    let ownership = column::<UInt32Array>(batch.column(5).as_ref())?;
    let unit_dictionary = unit_dictionary(batch.column(6).as_ref())?;
    let ownership_values = column::<ListArray>(batch.column(7).as_ref())?.value(0);
    let ownership_dictionary = column::<ListArray>(ownership_values.as_ref())?;
    let arena = result["kind"] == "arena";
    for row in 1..batch.num_rows() {
        if chapters.is_null(row) || phases.is_null(row) || ownership.is_null(row) {
            return Err("Missing Arrow phase".into());
        }
        let mut phase: Value = serde_json::from_str(phases.value(row))?;
        phase["before"] = roster(before, row, &unit_dictionary)?;
        phase["units"] = roster(units, row, &unit_dictionary)?;
        let tiles = if arena {
            &result["initial"]["tiles"]
        } else {
            result
                .get("missions")
                .and_then(Value::as_array)
                .and_then(|missions| missions.get(chapters.value(row) as usize))
                .and_then(|mission| mission.get("initial"))
                .and_then(|initial| initial.get("tiles"))
                .ok_or("Invalid campaign chapter index")?
        };
        phase["ownership"] = owners(ownership_dictionary, ownership.value(row) as usize, tiles)?;
        let target = if arena {
            if chapters.value(row) != 0 {
                return Err("Invalid arena chapter index".into());
            }
            &mut result["phases"]
        } else {
            result
                .get_mut("missions")
                .and_then(Value::as_array_mut)
                .and_then(|missions| missions.get_mut(chapters.value(row) as usize))
                .and_then(|mission| mission.get_mut("phases"))
                .ok_or("Invalid campaign chapter index")?
        };
        target
            .as_array_mut()
            .ok_or("Missing header phase array")?
            .push(phase);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ai::{Diplomacy, Policy},
        data::{Catalog, Role},
    };
    #[test]
    fn dictionaries_round_trip_full_arena_and_multiple_campaign_chapters() {
        let rules = Catalog::load().unwrap();
        let mut arena = super::super::matches::run_arena(
            &rules,
            ArenaConfig {
                seed: 17,
                armies: Armies {
                    player: "human".into(),
                    enemy: "woodland".into(),
                },
                landscape: "wooded".into(),
                player: Policy::Mixed,
                enemy: Policy::Classic,
                first: Side::Player,
                max_rounds: 1,
                recruit_limit: 0,
            },
        )
        .unwrap();
        // Cover optional absence, explicit false, zero XP and non-ASCII names.
        let unit = Arc::make_mut(&mut arena.phases[0].before[0]);
        unit.experience = None;
        unit.retreating = Some(false);
        unit.rebuilding = Some(true);
        unit.role = Some(Role::Courier);
        unit.name = Some("河😀".into());
        arena.phases[0].before.reverse();
        let arena = MatchResult::Arena(arena);
        let mut campaign = super::super::matches::run_campaign(
            &rules,
            CampaignConfig {
                seed: 17,
                diplomacy: Diplomacy::Restrained,
                campaign: "green-accord".into(),
                player: Policy::Mixed,
                enemy: Policy::Classic,
                max_rounds: 1,
                recruit_limit: 0,
            },
        )
        .unwrap();
        campaign.missions.push(campaign.missions[0].clone());
        for result in [arena, MatchResult::Campaign(campaign)] {
            let summary = summaries::borrowed_summary(&result, true).unwrap();
            let expected = serde_json::to_value(&summary).unwrap();
            let bytes = encode(&result, &summary).unwrap();
            assert_eq!(decode(&bytes).unwrap(), expected);
            assert!(decode(&bytes[..bytes.len() / 2]).is_err());
        }
    }
}
