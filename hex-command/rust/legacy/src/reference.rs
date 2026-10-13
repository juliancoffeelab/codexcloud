//! The reference uses the original bounded worker pool and reporting-thread disposal.
use crate::{
    ai::profiling,
    data::Rules,
    simulation::{SimulationError, cli, matches, types::MatchConfig},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

#[derive(Deserialize)]
#[serde(untagged)]
enum Request {
    Batch {
        configs: Vec<MatchConfig>,
        workers: usize,
    },
    Match(MatchConfig),
}
fn envelope(rules: &Rules, payload: Value) -> Value {
    json!({"frozenCommit":"b1f553de69c0de8a340088c0a9ed42b2149fb27e", "contentFingerprint":rules.content_fingerprint,"payload":payload})
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let rules = Rules::load_from(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data"))?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        profiling::reset();
        let request: Request = serde_json::from_str(&line)?;
        match request {
            Request::Batch { configs, workers } => {
                let directory = TemporaryOutput::new()?;
                let result = cli::reference_batch(
                    &rules,
                    &configs,
                    workers,
                    directory.0.join("report.json"),
                );
                let profile = profiling::snapshot();
                if result.is_ok() {
                    let indexes: BTreeMap<_, _> = configs
                        .iter()
                        .enumerate()
                        .map(|(index, config)| {
                            Ok((
                                serde_json::to_string(&serde_json::to_value(config)?)?,
                                index,
                            ))
                        })
                        .collect::<Result<_, serde_json::Error>>()?;
                    let report: Value =
                        serde_json::from_reader(fs::File::open(directory.0.join("report.json"))?)?;
                    let elapsed: Vec<_> = report["outcomes"]
                        .as_array()
                        .ok_or("Missing reference outcomes")?
                        .iter()
                        .map(|o| o["elapsedMs"].clone())
                        .collect();
                    for line in
                        BufReader::new(fs::File::open(directory.0.join("report.details.jsonl"))?)
                            .lines()
                    {
                        let result: Value = serde_json::from_str(&line?)?;
                        let index = *indexes
                            .get(&serde_json::to_string(&result["config"])?)
                            .ok_or("Unexpected frozen match configuration")?;
                        let response = envelope(
                            &rules,
                            json!({"index":index,"result":result,"elapsedMs":elapsed[index]}),
                        );
                        serde_json::to_writer(&mut output, &response)?;
                        writeln!(output)?;
                        output.flush()?;
                    }
                }
                let payload = match result {
                    Ok(()) => json!({"done":true,"profile":profile}),
                    Err(error) => json!({"error":error.to_string()}),
                };
                serde_json::to_writer(&mut output, &envelope(&rules, payload))?;
            }
            Request::Match(config) => {
                let result = matches::run(&rules, config)
                    .map(|result| json!({"result":result,"profile":profiling::snapshot()}))
                    .unwrap_or_else(|error: SimulationError| json!({"error":error.to_string()}));
                serde_json::to_writer(&mut output, &envelope(&rules, result))?;
            }
        }
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}

/// Intermediate original JSONL is transport storage; the host retains the complete journal.
struct TemporaryOutput(PathBuf);
impl TemporaryOutput {
    fn new() -> io::Result<Self> {
        let name = format!(
            "hex-command-reference-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let directory = std::env::temp_dir().join(name);
        fs::create_dir(&directory)?;
        Ok(Self(directory))
    }
}
impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
