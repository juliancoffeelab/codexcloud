//! Frozen native references keep their original engine, allocator and release build.
use super::{error::SimulationError, types::*};
use crate::{
    ai::{Policy, profiling},
    data::{Rules, Side},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
const CONTENT: &str = "96981ec4797122c6f715e0febdc9045a20d14562bf1389c9a5fe040163b35a17";
const REVISION: &str = "b1f553de69c0de8a340088c0a9ed42b2149fb27e";
struct Client {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}
impl Client {
    fn new() -> Result<Self, SimulationError> {
        let executable = std::env::current_exe()?.with_file_name(if cfg!(windows) {
            "hex-command-legacy.exe"
        } else {
            "hex-command-legacy"
        });
        let mut child = Command::new(&executable)
            .env(
                "HEX_COMMAND_AI_PROFILE",
                if profiling::is_enabled() { "1" } else { "0" },
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                SimulationError::Worker(format!(
                    "Cannot start frozen reference {}: {e}. Run npm run ai:rust:build.",
                    executable.display()
                ))
            })?;
        Ok(Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
        })
    }
    fn send(&mut self, request: &Value) -> Result<(), SimulationError> {
        serde_json::to_writer(&mut self.input, &request)?;
        writeln!(self.input)?;
        self.input.flush()?;
        Ok(())
    }
    fn receive(&mut self) -> Result<Value, SimulationError> {
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            return Err(SimulationError::Worker(
                "Frozen reference exited before returning a game".into(),
            ));
        }
        let mut response: Value = serde_json::from_str(&line)?;
        if response["frozenCommit"] != REVISION || response["contentFingerprint"] != CONTENT {
            return Err(SimulationError::Worker(
                "Frozen reference provenance mismatch".into(),
            ));
        }
        let payload = response["payload"].take();
        if let Some(error) = payload.get("error") {
            return Err(SimulationError::Worker(error.to_string()));
        }
        Ok(payload)
    }
    fn run(&mut self, config: &MatchConfig) -> Result<MatchResult, SimulationError> {
        self.send(&request(config)?)?;
        let mut response = self.receive()?;
        import_profile(&response["profile"])?;
        response["result"]["config"] = serde_json::to_value(config)?;
        Ok(serde_json::from_value(response["result"].take())?)
    }
}
fn request(config: &MatchConfig) -> Result<Value, SimulationError> {
    let mut value = serde_json::to_value(config)?;
    let (player, enemy) = policies(config);
    value["player"] = json!(player.legacy().ok_or("Current policy in frozen request")?);
    value["enemy"] = json!(enemy.legacy().ok_or("Current policy in frozen request")?);
    Ok(value)
}
fn import_profile(profile: &Value) -> Result<(), SimulationError> {
    if let Some(rows) = profile["rows"].as_array() {
        for row in rows {
            let side: Side = serde_json::from_value(row["side"].clone())?;
            let policy = Policy::ALL
                .into_iter()
                .find(|p| p.legacy().is_some_and(|name| row["policy"] == name))
                .ok_or("Unknown frozen policy in profile")?;
            profiling::import(policy, side, row);
        }
    }
    Ok(())
}
/// Use the frozen bounded batch pipeline; per-game IPC changes allocator reuse.
pub(super) fn execute_batch<F>(
    rules: &Rules,
    configs: &[(usize, &MatchConfig)],
    workers: usize,
    mut append: F,
) -> Result<(), SimulationError>
where
    F: FnMut(usize, MatchResult, f64) -> Result<(), SimulationError>,
{
    if rules.content_fingerprint != CONTENT {
        return Err("Frozen reference catalog differs: use --data-dir rust/legacy/data to reproduce the pinned baseline".into());
    }
    let requests = configs
        .iter()
        .map(|(_, c)| request(c))
        .collect::<Result<Vec<_>, _>>()?;
    let mut client = Client::new()?;
    client.send(&json!({"configs":requests,"workers":workers}))?;
    let mut seen = vec![false; configs.len()];
    loop {
        let mut response = client.receive()?;
        if response["done"] == true {
            if seen.iter().any(|complete| !complete) {
                return Err("Missing frozen match result".into());
            }
            import_profile(&response["profile"])?;
            return Ok(());
        }
        let index = response["index"]
            .as_u64()
            .ok_or("Invalid frozen match index")? as usize;
        let (original, config) = configs
            .get(index)
            .ok_or("Frozen match index out of range")?;
        if std::mem::replace(&mut seen[index], true) {
            return Err("Duplicate frozen match index".into());
        }
        response["result"]["config"] = serde_json::to_value(config)?;
        let elapsed = response["elapsedMs"]
            .as_f64()
            .ok_or("Missing frozen match timing")?;
        append(
            *original,
            serde_json::from_value(response["result"].take())?,
            elapsed,
        )?;
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
thread_local! {static CLIENT:RefCell<Option<Client>>=const{RefCell::new(None)};}
fn policies(config: &MatchConfig) -> (Policy, Policy) {
    match config {
        MatchConfig::Arena(c) => (c.player, c.enemy),
        MatchConfig::Campaign(c) => (c.player, c.enemy),
    }
}
pub(super) fn run(
    rules: &Rules,
    config: &MatchConfig,
) -> Option<Result<MatchResult, SimulationError>> {
    let (player, enemy) = policies(config);
    if player.legacy().is_none() && enemy.legacy().is_none() {
        return None;
    }
    Some((|| {
        if player.legacy().is_none() || enemy.legacy().is_none() {
            return Err("Frozen references support old-versus-old games; use --self-play to compare old and current policies in one report".into());
        }
        if rules.content_fingerprint != CONTENT {
            return Err("Frozen reference catalog differs: use --data-dir rust/legacy/data to reproduce the pinned baseline".into());
        }
        CLIENT.with(|cell| {
            let mut client = cell.borrow_mut();
            if client.is_none() {
                *client = Some(Client::new()?);
            }
            client.as_mut().unwrap().run(config)
        })
    })())
}
