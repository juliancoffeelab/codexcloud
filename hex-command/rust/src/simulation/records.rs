//! Append-only result journals retain full turns without holding the entire batch in memory.
use super::error::SimulationError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Checksum-protected byte range locating one complete JSONL journal record.
pub struct DetailRecord {
    pub file: String,
    pub offset: u64,
    pub bytes: u64,
    pub sha256: String,
}
/// Return a lowercase hexadecimal SHA-256 digest of exact record bytes.
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// Append-only writer whose indexes remain valid as further matches finish.
pub struct ResultJournal {
    writer: File,
    pub file: String,
    offset: u64,
    scratch: Vec<u8>,
}
impl ResultJournal {
    /// Create an empty journal and reset its byte cursor.
    pub fn create(path: &Path) -> Result<Self, SimulationError> {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid journal filename")?
            .to_owned();
        Ok(Self {
            writer: File::create(path)?,
            file,
            offset: 0,
            scratch: Vec::new(),
        })
    }
    /// Serialize one record, append a newline, and return its complete byte index.
    pub fn append<T: Serialize>(&mut self, value: &T) -> Result<DetailRecord, SimulationError> {
        self.scratch.clear();
        serde_json::to_writer(&mut self.scratch, value)?;
        self.scratch.push(b'\n');
        self.writer.write_all(&self.scratch)?;
        let record = DetailRecord {
            file: self.file.clone(),
            offset: self.offset,
            bytes: self.scratch.len() as u64,
            sha256: sha256(&self.scratch),
        };
        self.offset = self
            .offset
            .checked_add(record.bytes)
            .ok_or("Journal size overflow")?;
        Ok(record)
    }
    /// Number of complete record bytes appended to this journal.
    pub fn byte_len(&self) -> u64 {
        self.offset
    }
    /// Flush buffered output before exposing completed journal references.
    pub fn flush(&mut self) -> Result<(), SimulationError> {
        self.writer.flush()?;
        Ok(())
    }
}
/// Read one indexed record, validating its path, byte range and checksum first.
pub fn read_detail(directory: &Path, record: &DetailRecord) -> Result<Value, SimulationError> {
    let path = PathBuf::from(&record.file);
    if path.components().count() != 1
        || path.file_name().and_then(|name| name.to_str()) != Some(record.file.as_str())
        || record.bytes == 0
    {
        return Err(SimulationError::Replay(
            "Invalid result journal reference".into(),
        ));
    }
    let mut reader = File::open(directory.join(path))?;
    let end = record
        .offset
        .checked_add(record.bytes)
        .ok_or("Invalid journal byte range")?;
    if end > reader.metadata()?.len() {
        return Err(SimulationError::Replay(format!(
            "Incomplete result journal record: {}",
            record.file
        )));
    }
    let size = usize::try_from(record.bytes).map_err(|_| "Journal byte range too large")?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| "Journal byte range allocation failed")?;
    bytes.resize(size, 0);
    reader.seek(SeekFrom::Start(record.offset))?;
    reader.read_exact(&mut bytes)?;
    if sha256(&bytes) != record.sha256 {
        return Err(SimulationError::Replay(format!(
            "Result journal checksum mismatch: {}",
            record.file
        )));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_ranges_and_checksum_are_checked() {
        let directory = std::env::temp_dir().join(format!("hex-journal-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("details.jsonl");
        let mut journal = ResultJournal::create(&path).unwrap();
        let first = journal.append(&serde_json::json!({"name":"é"})).unwrap();
        let mut second = journal
            .append(&serde_json::json!({"actions":["move"]}))
            .unwrap();
        assert_eq!(second.offset, first.bytes);
        journal.flush().unwrap();
        assert_eq!(read_detail(&directory, &first).unwrap()["name"], "é");
        second.sha256 = "corrupt".into();
        assert!(read_detail(&directory, &second).is_err());
        second.file = "../details.jsonl".into();
        assert!(read_detail(&directory, &second).is_err());
        drop(journal);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
