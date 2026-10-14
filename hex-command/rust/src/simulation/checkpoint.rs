//! Constant-size progress snapshots plus an append-only compact outcome index.
use super::{error::SimulationError, records::ResultJournal, schedule::Mode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// One completed outcome, tagged with its canonical schedule position.
#[derive(Serialize)]
pub(crate) struct IndexedOutcome<'a> {
    pub index: usize,
    pub outcome: &'a Value,
}
#[derive(Serialize)]
struct Cursor<'a> {
    file: &'a str,
    bytes: u64,
}
impl<'a> From<&'a ResultJournal> for Cursor<'a> {
    fn from(journal: &'a ResultJournal) -> Self {
        Self {
            file: &journal.file,
            bytes: journal.byte_len(),
        }
    }
}
/// Small atomic progress snapshot; completed outcomes live only in the indexed journal.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Progress<'a> {
    schema: u8,
    status: &'a str,
    mode: Mode,
    fingerprint: &'a str,
    completed: usize,
    expected: usize,
    elapsed_seconds: f64,
    progress_format: &'static str,
    outcome_index: Cursor<'a>,
    details: Cursor<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Cursor<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}
impl<'a> Progress<'a> {
    pub fn new(
        status: &'a str,
        mode: Mode,
        fingerprint: &'a str,
        completed: usize,
        expected: usize,
        elapsed_seconds: f64,
        index: &'a ResultJournal,
        details: &'a ResultJournal,
        trace: Option<&'a ResultJournal>,
        error: Option<&'a str>,
    ) -> Self {
        Self {
            schema: 3,
            status,
            mode,
            fingerprint,
            completed,
            expected,
            elapsed_seconds,
            progress_format: "indexed-outcomes-jsonl-v1",
            outcome_index: index.into(),
            details: details.into(),
            trace: trace.map(Cursor::from),
            error,
        }
    }
}
/// Recover complete index lines in schedule order; an interrupted final line is ignored.
/// Each recovered compact outcome retains its checksummed full-detail journal reference.
pub fn recover_outcomes(
    path: &Path,
    expected: usize,
) -> Result<Vec<Option<Value>>, SimulationError> {
    #[derive(Deserialize)]
    struct Entry {
        index: usize,
        outcome: Value,
    }
    let mut outcomes = Vec::new();
    outcomes
        .try_reserve_exact(expected)
        .map_err(|_| "Recovery index allocation failed")?;
    outcomes.resize_with(expected, || None);
    let mut reader = BufReader::new(File::open(path)?);
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        if reader.read_until(b'\n', &mut bytes)? == 0 {
            break;
        }
        if bytes.last() != Some(&b'\n') {
            break;
        }
        let entry: Entry = serde_json::from_slice(&bytes)?;
        let slot = outcomes
            .get_mut(entry.index)
            .ok_or("Recovery index is outside the recorded schedule")?;
        if slot.is_some() {
            return Err("Duplicate outcome in recovery index".into());
        }
        *slot = Some(entry.outcome);
    }
    Ok(outcomes)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn complete_index_lines_recover_canonical_order_after_partial_write() {
        let path =
            std::env::temp_dir().join(format!("hex-index-recovery-{}.jsonl", std::process::id()));
        let mut journal = ResultJournal::create(&path).unwrap();
        let first = json!({"config":{"seed":2},"detail":{"file":"details.jsonl","offset":10,"bytes":20,"sha256":"saved"}});
        let second = json!({"config":{"seed":1},"detail":{"file":"details.jsonl","offset":0,"bytes":10,"sha256":"saved"}});
        journal
            .append(&IndexedOutcome {
                index: 1,
                outcome: &first,
            })
            .unwrap();
        journal
            .append(&IndexedOutcome {
                index: 0,
                outcome: &second,
            })
            .unwrap();
        drop(journal);
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        file.write_all(b"{\"index\":2,\"outcome\":").unwrap();
        drop(file);
        let recovered = recover_outcomes(&path, 3).unwrap();
        assert_eq!(recovered[0], Some(second));
        assert_eq!(recovered[1], Some(first));
        assert!(recovered[2].is_none());
        std::fs::remove_file(path).unwrap();
    }
}
