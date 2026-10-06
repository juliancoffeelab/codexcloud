//! Semantic dump comparison: object order is irrelevant, array order and all game fields matter.
use super::{
    error::SimulationError,
    records::{DetailRecord, read_detail},
};
use serde::Serialize;
use serde_json::{Number, Value};
use std::{fmt, path::Path};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// The first semantic field difference, retaining missing versus present values.
pub struct JsonDifference {
    /// JSON Pointer, with RFC 6901 escaping for object keys.
    pub path: String,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
    pub expected_present: bool,
    pub actual_present: bool,
}
impl fmt::Display for JsonDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = |value: &Option<Value>| {
            value
                .as_ref()
                .map_or_else(|| "<missing>".into(), Value::to_string)
        };
        write!(
            f,
            "{}: expected {}, actual {}",
            if self.path.is_empty() {
                "/"
            } else {
                &self.path
            },
            value(&self.expected),
            value(&self.actual)
        )
    }
}
/// Read checksummed comparison records, including absolute references in selected subsets.
/// Strict native replay continues to require journal basenames.
pub fn read_comparison_detail(
    directory: &Path,
    record: &DetailRecord,
) -> Result<Value, SimulationError> {
    let path = Path::new(&record.file);
    if path.is_absolute() {
        let mut reference = record.clone();
        reference.file = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid absolute comparison journal path")?
            .to_owned();
        return read_detail(
            path.parent()
                .ok_or("Invalid absolute comparison journal path")?,
            &reference,
        );
    }
    read_detail(directory, record)
}
/// Strip only derived summary metadata; endpoint states and phase records are untouched.
pub fn strip_derived_metadata(value: &mut Value) {
    fn strip(value: &mut Value) {
        if let Some(object) = value.as_object_mut() {
            for key in ["stateHash", "elapsedMs", "elapsedSeconds", "detail"] {
                object.remove(key);
            }
        }
    }
    strip(value);
    if let Some(missions) = value.get_mut("missions").and_then(Value::as_array_mut) {
        for mission in missions {
            strip(mission);
        }
    }
}
fn integer(number: &Number) -> Option<i128> {
    number
        .as_i64()
        .map(i128::from)
        .or_else(|| number.as_u64().map(i128::from))
}
fn numbers_equal(expected: &Number, actual: &Number) -> bool {
    match (integer(expected), integer(actual)) {
        (Some(left), Some(right)) => left == right,
        (None, None) => expected.as_f64() == actual.as_f64(),
        (int, float) => {
            let (int, float) = match int {
                Some(int) => (int, actual.as_f64()),
                None => (float.unwrap_or_default(), expected.as_f64()),
            };
            // JSON emitted by JavaScript writes 24 where Rust's typed f64 writes 24.0.
            // Only exact, safe integral equivalence is accepted; no epsilon is used.
            float.is_some_and(|float| {
                float.is_finite()
                    && float.fract() == 0.0
                    && float.abs() <= 9_007_199_254_740_992.0
                    && float as i128 == int
            })
        }
    }
}
/// Compare complete JSON values without object-order or numeric-spelling artifacts.
pub fn first_difference(expected: &Value, actual: &Value) -> Option<JsonDifference> {
    fn difference(path: &str, expected: Option<&Value>, actual: Option<&Value>) -> JsonDifference {
        JsonDifference {
            path: path.into(),
            expected: expected.cloned(),
            actual: actual.cloned(),
            expected_present: expected.is_some(),
            actual_present: actual.is_some(),
        }
    }
    fn pointer(path: &str, key: &str) -> String {
        format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"))
    }
    fn visit(expected: &Value, actual: &Value, path: &str) -> Option<JsonDifference> {
        match (expected, actual) {
            (Value::Object(left), Value::Object(right)) => {
                for (key, expected) in left {
                    let next = pointer(path, key);
                    match right.get(key) {
                        Some(actual) => {
                            if let Some(diff) = visit(expected, actual, &next) {
                                return Some(diff);
                            }
                        }
                        None => return Some(difference(&next, Some(expected), None)),
                    }
                }
                for (key, actual) in right {
                    if !left.contains_key(key) {
                        return Some(difference(&pointer(path, key), None, Some(actual)));
                    }
                }
                None
            }
            (Value::Array(left), Value::Array(right)) => {
                for index in 0..left.len().max(right.len()) {
                    let next = pointer(path, &index.to_string());
                    match (left.get(index), right.get(index)) {
                        (Some(expected), Some(actual)) => {
                            if let Some(diff) = visit(expected, actual, &next) {
                                return Some(diff);
                            }
                        }
                        (expected, actual) => return Some(difference(&next, expected, actual)),
                    }
                }
                None
            }
            (Value::Number(left), Value::Number(right)) if numbers_equal(left, right) => None,
            _ if expected == actual => None,
            _ => Some(difference(path, Some(expected), Some(actual))),
        }
    }
    visit(expected, actual, "")
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn numeric_semantics_ignore_only_exact_representation_differences() {
        assert!(first_difference(&json!({"weight":24}), &json!({"weight":24.0})).is_none());
        assert!(first_difference(&json!(24.0), &json!(24)).is_none());
        assert!(first_difference(&json!(0.1), &json!(0.10000000000000002)).is_some());
        assert!(
            first_difference(&json!(9007199254740992u64), &json!(9007199254740993u64)).is_some()
        );
    }
    #[test]
    fn keys_are_unordered_and_array_missing_null_differences_are_precise() {
        let left: Value = serde_json::from_str("{\"b\":2,\"a\":[1,2]}").unwrap();
        let right: Value = serde_json::from_str("{\"a\":[1,2],\"b\":2}").unwrap();
        assert!(first_difference(&left, &right).is_none());
        let difference = first_difference(&left, &json!({"a":[2,1],"b":2})).unwrap();
        assert_eq!(difference.path, "/a/0");
        let difference = first_difference(&json!({"a/b~c":null}), &json!({})).unwrap();
        assert_eq!(difference.path, "/a~1b~0c");
        assert!(difference.expected_present);
        assert!(!difference.actual_present);
    }
    #[test]
    fn ignored_summary_metadata_does_not_remove_any_state_fields() {
        let mut left = json!({"stateHash":"ts","elapsedMs":1,"detail":{},"phases":[{"round":1}],"final":{"stateHash":"state-field"},"missions":[{"stateHash":"ts","final":{"round":1}}]});
        let mut right = left.clone();
        right["stateHash"] = json!("rust");
        right["elapsedMs"] = json!(2);
        right["missions"][0]["stateHash"] = json!("rust");
        strip_derived_metadata(&mut left);
        strip_derived_metadata(&mut right);
        assert!(first_difference(&left, &right).is_none());
        right["final"]["stateHash"] = json!("different-state-field");
        assert_eq!(
            first_difference(&left, &right).unwrap().path,
            "/final/stateHash"
        );
    }
}
