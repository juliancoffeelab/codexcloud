//! Data-only validation of durable undo changes. No gameplay or AI is executed.
use crate::{data::*, protocol::*};
use serde_json::Value;

fn invalid(message: impl ToString) -> EngineError {
    EngineError::InvalidSave {
        message: message.to_string(),
    }
}
fn apply(state: &mut Value, change: UndoChange) -> Result<(), EngineError> {
    let (path, replacement) = match change {
        UndoChange::Set { path, value } => (path, Some(value)),
        UndoChange::Remove { path } => (path, None),
    };
    let (key, parents) = path
        .split_last()
        .ok_or_else(|| invalid("empty undo path"))?;
    let mut parent = state;
    for part in parents {
        parent = match parent {
            Value::Object(object) => object.get_mut(part),
            Value::Array(array) => part
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get_mut(index)),
            _ => None,
        }
        .ok_or_else(|| invalid("invalid undo path"))?;
    }
    match parent {
        Value::Object(object) => match replacement {
            Some(value) => {
                object.insert(key.clone(), value);
            }
            None => {
                if object.remove(key).is_none() {
                    return Err(invalid("missing undo field"));
                }
            }
        },
        Value::Array(array) => {
            let index = key.parse::<usize>().map_err(invalid)?;
            let slot = array
                .get_mut(index)
                .ok_or_else(|| invalid("invalid undo index"))?;
            *slot = replacement.ok_or_else(|| invalid("undo cannot create array holes"))?;
        }
        _ => return Err(invalid("invalid undo parent")),
    }
    Ok(())
}

pub(crate) fn validate(rules: &Catalogue, request: UndoValidation) -> Result<(), EngineError> {
    let round = request.state.round;
    let map = request.state.map_id.clone();
    if request.state.turn != TurnPhase::Player || request.state.winner.is_some() {
        return Err(invalid("undo requires an active player turn"));
    }
    let mut value = serde_json::to_value(request.state).map_err(invalid)?;
    for step in request.history.into_iter().rev() {
        if step.round != round {
            return Err(invalid("undo crosses a turn boundary"));
        }
        for change in step.changes {
            apply(&mut value, change)?;
        }
        let state: State = serde_json::from_value(value.clone()).map_err(invalid)?;
        if state.round != round
            || state.map_id != map
            || state.turn != TurnPhase::Player
            || state.winner.is_some()
            || !state.units.iter().any(|unit| unit.id == step.unit_id)
        {
            return Err(invalid("incoherent undo state"));
        }
        hex_command_battle::validate_state(rules, &state).map_err(invalid)?;
    }
    Ok(())
}
