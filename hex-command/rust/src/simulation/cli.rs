//! Native schedule, bounded scoped worker pool, reporting and replay.
use super::{
    checkpoint::{IndexedOutcome, Progress},
    error::SimulationError,
    matches,
    records::{DetailRecord, EncodedRecord, ResultJournal, read_detail, sha256},
    report,
    schedule::{
        DEFAULT_ARENA_SEEDS, DEFAULT_CAMPAIGN_SEEDS, Mode, ScheduleOptions, WithSelection, schedule,
    },
    types::{MatchConfig, MatchResult},
};
use crate::{
    ai::{Diplomacy, Policy},
    data::Rules,
};
use clap::{Parser, ValueEnum};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Parser)]
#[command(
    name = "hex-command-sim",
    about = "Run combat simulations and generate their report"
)]
#[command(group = clap::ArgGroup::new("batch").multiple(true).conflicts_with("protocol"))]
struct SimulationArgs {
    /// Workload to run.
    #[arg(group = "batch", long, value_enum, default_value = "full")]
    mode: Mode,
    /// Add a custom tactic; - selects the workload default for that block.
    #[arg(group = "batch", long, value_name = "RECRUITER,CAMPAIGNER,PLANNER", value_parser = WithSelection::parse, allow_hyphen_values = true)]
    with: Option<WithSelection>,
    /// Seed count override for both workloads (defaults: arena 50, campaign 100).
    #[arg(group = "batch", long, value_parser = clap::value_parser!(u32).range(1..=10000))]
    seeds: Option<u32>,
    /// First seed.
    #[arg(group = "batch", long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(0..=2147483647))]
    seed: u32,
    /// Arena tactics; also filters default campaign players.
    #[arg(group = "batch", long, value_enum, value_delimiter = ',', action = clap::ArgAction::Set)]
    policies: Option<Vec<Policy>>,
    /// Recruitment roster identifiers, separated by commas.
    #[arg(group = "batch", long, value_delimiter = ',', action = clap::ArgAction::Set)]
    armies: Option<Vec<String>>,
    /// Arena landscape identifiers, separated by commas.
    #[arg(group = "batch", long, value_delimiter = ',', action = clap::ArgAction::Set)]
    landscapes: Option<Vec<String>>,
    /// Campaign identifiers, separated by commas.
    #[arg(group = "batch", long, value_delimiter = ',', action = clap::ArgAction::Set)]
    campaigns: Option<Vec<String>>,
    /// Campaign player tactics; custom requires --with.
    #[arg(group = "batch", long, value_delimiter = ',', action = clap::ArgAction::Set, value_parser = campaign_participants())]
    campaign_players: Option<Vec<String>>,
    /// Campaign enemy tactics; custom requires --with.
    #[arg(group = "batch", long, value_delimiter = ',', action = clap::ArgAction::Set, value_parser = campaign_participants())]
    campaign_enemies: Option<Vec<String>>,
    /// Override both round caps (otherwise arena 30, campaign 100 per stage).
    #[arg(group = "batch", long, value_parser = clap::value_parser!(u32).range(1..=1000))]
    rounds: Option<u32>,
    /// Override recruits per phase on both sides (otherwise policy defaults).
    #[arg(group = "batch", long, value_parser = clap::value_parser!(u32).range(0..=20))]
    recruits: Option<u32>,
    /// Filter the campaign's authored diplomacy modes.
    #[arg(group = "batch", long, value_enum, value_delimiter = ',', action = clap::ArgAction::Set)]
    diplomacy: Option<Vec<Diplomacy>>,
    /// Worker count, defaulting to the available CPU quota.
    #[arg(group = "batch", long, default_value_t = default_workers() as u32, value_parser = clap::value_parser!(u32).range(1..=8))]
    workers: u32,
    /// Output directory or .json file; otherwise use a fresh artifacts directory.
    #[arg(group = "batch", long, value_name = "PATH")]
    out: Option<PathBuf>,
    /// Keep raw results without generating the report.
    #[arg(group = "batch", long, conflicts_with = "baseline")]
    no_report: bool,
    /// Lossless gameplay journal format.
    #[arg(group = "batch", long, value_enum, default_value = "jsonl")]
    details: DetailFormat,
    /// Compare every selected arena tactic with every other tactic.
    #[arg(group = "batch", long, conflicts_with_all = ["vs_classic", "vs"])]
    all_pairs: bool,
    /// Legacy shorthand for --vs classic.
    #[arg(group = "batch", long, conflicts_with = "self_play")]
    vs_classic: bool,
    /// Fixed arena opponent (defaults to Balanced); campaigns keep their own opponent.
    #[arg(group = "batch", long, value_enum, conflicts_with_all = ["vs_classic", "all_pairs", "self_play"])]
    vs: Option<Policy>,
    /// Run matching arena tactics against themselves; requires --mode arena.
    #[arg(group = "batch", long)]
    self_play: bool,
    /// Compare with a saved run with identical configurations; repeatable.
    #[arg(group = "batch", long, value_name = "PATH")]
    baseline: Vec<PathBuf>,
    /// Record policy decision and complete-phase CPU times.
    #[arg(group = "batch", long)]
    profile_turns: bool,
    /// Analyze battlefield presence, protection, objectives, support and leader risk.
    #[arg(group = "batch", long, conflicts_with = "no_report")]
    unit_usage: bool,
    /// Also retain raw match results.
    #[arg(group = "batch", long)]
    trace: bool,
    /// Verify a saved report's details and results with strict fingerprints.
    #[arg(
        group = "batch",
        long,
        value_name = "PATH",
        conflicts_with = "schedule_only"
    )]
    replay: Option<PathBuf>,
    /// Authored catalog root; otherwise use HEX_COMMAND_DATA_ROOT or this checkout.
    #[arg(long, value_name = "PATH")]
    data_dir: Option<PathBuf>,
    /// Print configurations without running matches.
    #[arg(group = "batch", long)]
    schedule_only: bool,
    /// Read and write JSON match requests/results, one per line.
    #[arg(long, group = "protocol")]
    stdio: bool,
    /// Read and write saved-position phase profiling requests/results.
    #[arg(long, group = "protocol")]
    phase_stdio: bool,
    /// Print the compiled content and engine SHA256 fingerprints.
    #[arg(long, group = "protocol")]
    fingerprint: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DetailFormat {
    Jsonl,
    Arrow,
}

fn campaign_participants() -> clap::builder::PossibleValuesParser {
    clap::builder::PossibleValuesParser::new(
        Policy::ALL
            .into_iter()
            .map(Policy::as_str)
            .chain(["custom"]),
    )
}

#[derive(Debug)]
struct CliOptions {
    schedule: ScheduleOptions,
    workers: usize,
    out: PathBuf,
    trace: bool,
    arrow: bool,
    replay: Option<PathBuf>,
    schedule_only: bool,
    profile_turns: bool,
    unit_usage: bool,
    no_report: bool,
    baselines: Vec<PathBuf>,
}

// Catalog membership and duplicate selections are domain validation, after Clap
// has handled argument syntax, types, ranges, conflicts and help.
fn selection<T: Clone + PartialEq>(
    input: Option<Vec<T>>,
    flag: &str,
    available: &[T],
) -> Result<Vec<T>, SimulationError> {
    let selected = input.unwrap_or_else(|| available.to_vec());
    if selected.is_empty()
        || selected
            .iter()
            .enumerate()
            .any(|(i, entry)| !available.contains(entry) || selected[..i].contains(entry))
    {
        return Err(format!("Invalid {flag}: expected unique available entries").into());
    }
    Ok(selected)
}

impl SimulationArgs {
    fn resolve(self, rules: &Rules) -> Result<CliOptions, SimulationError> {
        let mut options = ScheduleOptions::defaults(rules)?;
        options.with = self.with;
        options.mode = self.mode;
        options.first_seed = self.seed;
        options.seeds = self.seeds.unwrap_or(if self.mode == Mode::Campaign {
            DEFAULT_CAMPAIGN_SEEDS
        } else {
            DEFAULT_ARENA_SEEDS
        });
        options.campaign_seeds = self.seeds.unwrap_or(DEFAULT_CAMPAIGN_SEEDS);
        options.self_play = self.self_play;
        options.opponent = if self.mode == Mode::Campaign || self.all_pairs || self.self_play {
            None
        } else {
            Some(if self.vs_classic {
                Policy::Classic
            } else {
                self.vs.unwrap_or(Policy::Balanced)
            })
        };
        if self.self_play && self.mode != Mode::Arena {
            return Err("--self-play requires --mode arena".into());
        }
        options.max_rounds = self.rounds.unwrap_or(if self.mode == Mode::Campaign {
            rules.campaign_defaults.default_max_rounds
        } else {
            rules.arena.default_max_rounds
        });
        options.campaign_max_rounds = self
            .rounds
            .unwrap_or(rules.campaign_defaults.default_max_rounds);
        options.recruit_limit = self.recruits.map(|n| n as usize);
        options.armies = selection(self.armies, "--armies", &options.armies)?;
        options.landscapes = selection(self.landscapes, "--landscapes", &options.landscapes)?;
        options.campaigns = selection(self.campaigns, "--campaigns", &options.campaigns)?;
        options.policies = selection(self.policies, "--policies", &Policy::ALL)?;
        let mut participant_ids: Vec<_> =
            Policy::ALL.iter().map(|p| p.as_str().to_owned()).collect();
        if self.with.is_some() {
            participant_ids.push("custom".into());
        }
        for (input, flag, destination, custom) in [
            (
                self.campaign_players,
                "--campaign-players",
                &mut options.campaign_players,
                &mut options.campaign_custom_player,
            ),
            (
                self.campaign_enemies,
                "--campaign-enemies",
                &mut options.campaign_enemies,
                &mut options.campaign_custom_enemy,
            ),
        ] {
            if let Some(input) = input {
                let selected = selection(Some(input), flag, &participant_ids)?;
                *custom = selected.iter().any(|id| id == "custom");
                *destination = Some(
                    selected
                        .into_iter()
                        .filter(|id| id != "custom")
                        .map(|id| Policy::from_str(&id, false).map_err(SimulationError::from))
                        .collect::<Result<_, _>>()?,
                );
            }
        }
        if self.mode != Mode::Arena {
            if options.campaign_players.is_none() {
                if self.with.is_some() {
                    options.campaign_custom_player = true;
                    options.campaign_players = Some(vec![]);
                } else {
                    let players: Vec<_> = options
                        .policies
                        .iter()
                        .copied()
                        .filter(|p| {
                            matches!(
                                p,
                                Policy::Balanced
                                    | Policy::Rotated
                                    | Policy::Situational
                                    | Policy::Predictor
                            )
                        })
                        .collect();
                    if players.is_empty() {
                        return Err("No campaign simulation players selected; use --mode arena or explicit --campaign-players".into());
                    }
                    options.campaign_players = Some(players);
                }
            }
            options
                .campaign_enemies
                .get_or_insert_with(|| vec![Policy::Classic]);
        }
        options.diplomacy = selection(self.diplomacy, "--diplomacy", &options.diplomacy)?;
        let mut out = self.out.unwrap_or_else(|| {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            PathBuf::from(format!(
                "artifacts/simulation-{timestamp}-{}",
                std::process::id()
            ))
        });
        if out.extension().and_then(|ext| ext.to_str()) != Some("json") {
            out = out.join("report.json");
        }
        Ok(CliOptions {
            schedule: options,
            workers: self.workers as usize,
            out,
            trace: self.trace,
            arrow: matches!(self.details, DetailFormat::Arrow),
            replay: self.replay,
            schedule_only: self.schedule_only,
            profile_turns: self.profile_turns,
            unit_usage: self.unit_usage,
            no_report: self.no_report,
            baselines: self.baseline,
        })
    }
}

pub fn default_workers() -> usize {
    let mut count = thread::available_parallelism().map_or(1, usize::from);
    if let Ok(cpu_max) = fs::read_to_string("/sys/fs/cgroup/cpu.max") {
        let fields: Vec<_> = cpu_max.split_whitespace().collect();
        if fields.len() == 2 && fields[0] != "max" {
            if let (Ok(quota), Ok(period)) =
                (fields[0].parse::<usize>(), fields[1].parse::<usize>())
            {
                if period > 0 {
                    count = count.min((quota / period).max(1));
                }
            }
        }
    }
    count.clamp(1, 8)
}
pub fn fingerprints(rules: &Rules) -> Value {
    json!({"content":rules.content_fingerprint,"engine":env!("HEX_ENGINE_FINGERPRINT")})
}
fn report_fingerprint(rules: &Rules) -> Result<String, SimulationError> {
    Ok(sha256(&serde_json::to_vec(&fingerprints(rules))?))
}
fn stdio(rules: &Rules) -> Result<(), SimulationError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let config: MatchConfig = serde_json::from_str(&line)?;
        let result = matches::run(rules, config)?;
        serde_json::to_writer(&mut output, &result)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PhaseRequest {
    /// Include selected adaptive recruiter score components before running this phase.
    #[serde(default)]
    inspect_recruitment: bool,
    #[serde(default)]
    blocks: Option<crate::ai::Blocks>,
    state: crate::data::State,
    policy: crate::ai::Policy,
    seed: u32,
    #[serde(default)]
    recruitment_limit: usize,
    #[serde(default)]
    goals: Option<Vec<crate::data::Objective>>,
    #[serde(default)]
    diplomacy: crate::ai::Diplomacy,
    #[serde(default)]
    advance_turn: bool,
    #[serde(default)]
    begin_turn: bool,
    #[serde(default)]
    arena: Option<super::types::ArenaConfig>,
}
/// Replay one own phase from a saved position, for fixed-input policy profiling.
fn phase_stdio(rules: &Rules) -> Result<(), SimulationError> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: PhaseRequest = serde_json::from_str(&line)?;
        serde_json::to_writer(&mut output, &saved_phase(rules, request)?)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
fn saved_phase(rules: &Rules, request: PhaseRequest) -> Result<Value, SimulationError> {
    let arena = request
        .arena
        .as_ref()
        .map(|config| super::matches::create_arena(rules, config))
        .transpose()?;
    if let Some(arena) = &arena {
        if request.state.campaign.is_some() {
            return Err(SimulationError::InvalidConfig(
                "Arena phase contains a campaign".into(),
            ));
        }
        crate::logic::validate_state_geometry(rules, &request.state, &arena.state())
            .map_err(|error| SimulationError::InvalidConfig(error.to_string()))?;
    } else {
        crate::logic::validate_state(rules, &request.state)
            .map_err(|error| SimulationError::InvalidConfig(error.to_string()))?;
    }
    let mut game = crate::logic::Game::from_state(rules, request.state);
    if let Some(arena) = arena {
        game.pools = arena.pools;
        game.arena_first = arena.arena_first;
    }
    if request.advance_turn && request.begin_turn {
        return Err(SimulationError::InvalidConfig(
            "Select one turn-start operation".into(),
        ));
    }
    if request.begin_turn {
        game.begin_turn(game.turn);
    } else if request.advance_turn {
        game.end_turn();
    }
    if request.recruitment_limit > 1_000_000
        || request.goals.iter().flatten().any(|goal| {
            !goal.weight.is_finite()
                || goal.weight <= 0.0
                || goal.tiles.iter().any(|tile| !game.board.ids.contains(tile))
                || goal.actor.as_ref().is_some_and(|actor| {
                    actor
                        .strip_prefix('u')
                        .and_then(|id| id.parse::<u32>().ok())
                        .is_none_or(|id| id == 0 || id >= game.next_id)
                })
        })
    {
        return Err(SimulationError::InvalidConfig(
            "Invalid planning budget or objective".into(),
        ));
    }
    let scores = request.inspect_recruitment.then(|| {
        crate::ai::recruiter::diagnostics(
            &game,
            request.blocks.unwrap_or(request.policy.blocks()).recruiter,
        )
    });
    game.recordings = Some(crate::data::Records::default());
    game.play_blocks(
        request.policy,
        request.blocks,
        request.seed,
        request.recruitment_limit,
        request.goals.as_deref(),
        request.diplomacy,
    )?;
    let mut response = json!({"final":game.state(),"records":game.recordings});
    if let Some(scores) = scores {
        response["recruitmentScores"] = serde_json::to_value(scores)?;
    }
    Ok(response)
}
fn read_report(path: &Path) -> Result<Value, SimulationError> {
    let bytes = fs::read(path)?;
    let bytes = if path.extension().and_then(|extension| extension.to_str()) == Some("gz") {
        let mut output = Vec::new();
        flate2::read::GzDecoder::new(bytes.as_slice()).read_to_end(&mut output)?;
        output
    } else {
        bytes
    };
    Ok(serde_json::from_slice(&bytes)?)
}
fn replay(rules: &Rules, path: &Path, fingerprint: &str) -> Result<(), SimulationError> {
    let saved = read_report(path)?;
    if saved["fingerprint"] != fingerprint {
        return Err(SimulationError::Replay("Rules/policies/runner changed; replay on the recorded source before comparing outcomes".into()));
    }
    let outcomes = saved["outcomes"]
        .as_array()
        .ok_or("Replay report outcomes missing")?;
    for (index, outcome) in outcomes.iter().enumerate() {
        let expected = if let Some(reference) = outcome.get("detail") {
            let reference: DetailRecord = serde_json::from_value(reference.clone())?;
            read_detail(path.parent().unwrap_or(Path::new(".")), &reference)?
        } else {
            outcome.clone()
        };
        let config: MatchConfig = serde_json::from_value(expected["config"].clone())?;
        let actual = report::detailed_summary(&matches::run(rules, config)?)?;
        if actual != expected {
            return Err(SimulationError::Replay(format!(
                "Replay mismatch at game {}: {}",
                index + 1,
                expected["config"]
            )));
        }
        if (index + 1) % 10 == 0 {
            eprintln!("Verified {} games", index + 1);
        }
    }
    println!(
        "{}",
        json!({"verified":true,"games":outcomes.len(),"fingerprint":fingerprint})
    );
    Ok(())
}
struct Completed<Output> {
    index: usize,
    result: Output,
    elapsed_ms: f64,
}
struct PreparedMatch {
    details: EncodedRecord,
    compact: Value,
    trace: Option<EncodedRecord>,
}
fn execute_schedule<F>(
    rules: &Rules,
    configs: &[MatchConfig],
    workers: usize,
    trace: bool,
    arrow: bool,
    mut append: F,
) -> Result<(), SimulationError>
where
    F: FnMut(usize, PreparedMatch, f64) -> Result<(), SimulationError>,
{
    let prepare = |result: MatchResult, elapsed: f64| {
        let summary = report::borrowed_summary(&result, true)?;
        let details = if arrow {
            EncodedRecord::from_bytes(super::arrow::encode(&result, &summary)?)
        } else {
            EncodedRecord::new(&summary)?
        };
        // The journal thread supplies the final file and byte offset.
        let compact = report::compact_summary(
            &summary,
            DetailRecord {
                file: String::new(),
                offset: 0,
                bytes: 0,
                sha256: String::new(),
            },
            elapsed,
        )?;
        let trace = if trace {
            Some(EncodedRecord::new(&result)?)
        } else {
            None
        };
        Ok::<_, SimulationError>(PreparedMatch {
            details,
            compact,
            trace,
        })
    };
    execute_jobs(
        configs,
        workers,
        &|config| {
            let start = Instant::now();
            let result = matches::run(rules, config.clone())?;
            prepare(result, start.elapsed().as_secs_f64() * 1000.0)
        },
        |index, result, elapsed| append(index, result, elapsed),
    )
}
fn execute_jobs<Config, Output, Execute, Append>(
    configs: &[Config],
    workers: usize,
    execute: &Execute,
    mut append: Append,
) -> Result<(), SimulationError>
where
    Config: Sync,
    Output: Send,
    Execute: Fn(&Config) -> Result<Output, SimulationError> + Sync,
    Append: FnMut(usize, Output, f64) -> Result<(), SimulationError>,
{
    let next = AtomicUsize::new(0);
    let (sender, receiver) =
        mpsc::sync_channel::<Result<Completed<Output>, SimulationError>>(workers.max(1));
    thread::scope(|scope| {
        let mut handles = vec![];
        for _ in 0..workers.min(configs.len()) {
            let sender = sender.clone();
            let next = &next;
            handles.push(scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(config) = configs.get(index) else {
                        break;
                    };
                    let start = Instant::now();
                    let result = execute(config).map(|result| Completed {
                        index,
                        result,
                        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                    });
                    if sender.send(result).is_err() {
                        break;
                    }
                }
            }));
        }
        drop(sender);
        let mut result = Ok(());
        // Flush completed games immediately; schedule index restores canonical order in report.
        // An append/worker failure cancels new jobs but drains all in-flight sends before
        // joining workers. A worker blocked by the bounded channel can therefore finish.
        for completed in receiver.iter() {
            match completed {
                Ok(completed) if result.is_ok() => {
                    result = append(completed.index, completed.result, completed.elapsed_ms);
                }
                Err(error) if result.is_ok() => {
                    result = Err(error);
                }
                _ => {}
            }
            if result.is_err() {
                next.store(configs.len(), Ordering::Relaxed);
            }
        }
        for handle in handles {
            if handle.join().is_err() && result.is_ok() {
                result = Err(SimulationError::Worker(
                    "Native simulation worker failed".into(),
                ));
            }
        }
        result
    })
}
fn sibling(path: &Path, suffix: &str) -> Result<PathBuf, SimulationError> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Invalid report filename")?;
    Ok(path.with_file_name(format!("{stem}{suffix}")))
}
fn write_json<T: serde::Serialize>(
    path: &Path,
    value: &T,
    pretty: bool,
) -> Result<(), SimulationError> {
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };
    bytes.push(b'\n');
    // Atomic checkpoint replacement avoids leaving truncated JSON after interrupted writes.
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, &bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn run_batch(rules: &Rules, options: CliOptions, fingerprint: &str) -> Result<(), SimulationError> {
    let configs = schedule(&options.schedule)?;
    if options.schedule_only {
        println!(
            "{}",
            json!({"mode":options.schedule.mode,"games":configs.len(),"workers":options.workers,"configs":configs})
        );
        return Ok(());
    }
    let output = if options.out.is_absolute() {
        options.out.clone()
    } else {
        std::env::current_dir()?.join(&options.out)
    };
    fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))?;
    let partial = sibling(&output, ".partial.json")?;
    let suffix = if options.arrow {
        ".details.arrow"
    } else {
        ".details.jsonl"
    };
    let mut journal = ResultJournal::create(&sibling(&output, suffix)?)?;
    let mut index_journal = ResultJournal::create(&sibling(&output, ".outcomes.jsonl")?)?;
    let mut trace = if options.trace {
        Some(ResultJournal::create(&sibling(&output, ".trace.jsonl")?)?)
    } else {
        None
    };
    let start = Instant::now();
    let mut outcomes: Vec<Option<Value>> = vec![None; configs.len()];
    let mut completed = 0;
    write_json(
        &partial,
        &Progress::new(
            "running",
            options.schedule.mode,
            fingerprint,
            0,
            configs.len(),
            0.0,
            &index_journal,
            &journal,
            trace.as_ref(),
            None,
        ),
        false,
    )?;
    let result = execute_schedule(
        rules,
        &configs,
        options.workers,
        options.trace,
        options.arrow,
        |index, result, _elapsed| {
            let reference = journal.append_encoded(&result.details)?;
            if let (Some(trace), Some(encoded)) = (&mut trace, &result.trace) {
                trace.append_encoded(encoded)?;
            }
            let mut compact = result.compact;
            compact["detail"] = serde_json::to_value(reference)?;
            index_journal.append(&IndexedOutcome {
                index,
                outcome: &compact,
            })?;
            outcomes[index] = Some(compact);
            completed += 1;
            if completed % 10 == 0 {
                journal.flush()?;
                if let Some(trace) = &mut trace {
                    trace.flush()?;
                }
                index_journal.flush()?;
                write_json(
                    &partial,
                    &Progress::new(
                        "running",
                        options.schedule.mode,
                        fingerprint,
                        completed,
                        configs.len(),
                        start.elapsed().as_secs_f64(),
                        &index_journal,
                        &journal,
                        trace.as_ref(),
                        None,
                    ),
                    false,
                )?;
                eprintln!("{} games; {:.1}s", completed, start.elapsed().as_secs_f64());
            }
            Ok(())
        },
    );
    journal.flush()?;
    if let Some(trace) = &mut trace {
        trace.flush()?;
    }
    index_journal.flush()?;
    if let Err(error) = result {
        let message = error.to_string();
        write_json(
            &partial,
            &Progress::new(
                "failed",
                options.schedule.mode,
                fingerprint,
                completed,
                configs.len(),
                start.elapsed().as_secs_f64(),
                &index_journal,
                &journal,
                trace.as_ref(),
                Some(&message),
            ),
            false,
        )?;
        return Err(error);
    }
    let source_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .unwrap_or_else(|| "unknown".into());
    let trace_files = trace.as_ref().map(|trace| vec![trace.file.clone()]);
    let outcomes: Vec<_> = outcomes
        .into_iter()
        .map(|outcome| {
            outcome.ok_or_else(|| SimulationError::Worker("Missing match result".into()))
        })
        .collect::<Result<_, _>>()?;
    let mut report = report::build_report(
        &options.schedule,
        fingerprint,
        &source_commit,
        start.elapsed().as_secs_f64(),
        outcomes,
        trace_files,
    )?;
    if options.arrow {
        report["detailFormat"] = Value::String(super::arrow::FORMAT.into());
    }
    report["unitFamilies"] = serde_json::to_value(
        rules
            .units
            .iter()
            .map(|unit| (unit.id.clone(), unit.family))
            .collect::<std::collections::BTreeMap<_, _>>(),
    )?;
    report["supportTelemetryVersion"] = json!(1);
    report["combatTelemetryVersion"] = json!(1);
    report["leaderAuditVersion"] = json!(1);
    report["unitProfiles"] = serde_json::to_value(
        rules
            .units
            .iter()
            .map(|u| {
                (
                    u.id.clone(),
                    json!({
                        "name":u.name,"family":u.family,"race":u.race,"leader":u.leader,"hp":u.hp,
                        "level":u.level,"cost":u.cost,"move":u.r#move,"role":u.role,
                        "healing":u.healing,"thornAura":u.thorn_aura,"thorns":u.thorns
                    }),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>(),
    )?;
    if options.profile_turns {
        report["turnTiming"] = crate::ai::profiling::snapshot().unwrap();
    }
    super::damage::write(rules, output.parent().unwrap_or(Path::new(".")))?;
    write_json(&output, &report, true)?;
    if !options.no_report {
        fs::write(sibling(&output, ".md")?, report::markdown(&report))?;
    }
    write_json(
        &partial,
        &Progress::new(
            "complete",
            options.schedule.mode,
            fingerprint,
            completed,
            configs.len(),
            start.elapsed().as_secs_f64(),
            &index_journal,
            &journal,
            trace.as_ref(),
            None,
        ),
        false,
    )?;
    let mut summary = json!({"output":output,"games":completed,"elapsedSeconds":report["elapsedSeconds"],"rows":report["rows"]});
    if let Some(timing) = report.get("turnTiming") {
        summary["turnTiming"] = timing.clone();
    }
    println!("{summary}");
    if !options.no_report {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut analyzer = Command::new("python3");
        analyzer
            .arg(root.join(".agents/skills/combat-simulation/scripts/report-combat.py"))
            .arg(&output)
            .arg("--out")
            .arg(output.parent().unwrap().join("analytics"));
        if options.unit_usage {
            analyzer.arg("--unit-usage");
        }
        for baseline in &options.baselines {
            analyzer.arg("--baseline").arg(baseline);
        }
        let status = analyzer.status().map_err(|error| {
            SimulationError::InvalidConfig(format!(
                "Cannot start report analyzer: {error}. Raw results remain at {}",
                output.display()
            ))
        })?;
        if !status.success() {
            return Err(format!(
                "Report analyzer failed ({status}); raw results remain at {}",
                output.display()
            )
            .into());
        }
    }
    Ok(())
}
pub fn run(args: &[String]) -> Result<(), SimulationError> {
    let args = match SimulationArgs::try_parse_from(
        std::iter::once("hex-command-sim").chain(args.iter().map(String::as_str)),
    ) {
        Ok(args) => args,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            error.print()?;
            return Ok(());
        }
        Err(error) => return Err(SimulationError::InvalidConfig(error.to_string())),
    };
    let rules = if let Some(root) = &args.data_dir {
        Rules::load_from(root)?
    } else {
        Rules::load()?
    };
    if args.fingerprint {
        println!("{}", fingerprints(&rules));
        return Ok(());
    }
    if args.stdio {
        return stdio(&rules);
    }
    if args.phase_stdio {
        return phase_stdio(&rules);
    }
    let options = args.resolve(&rules)?;
    if options.profile_turns {
        crate::ai::profiling::enable();
    }
    let fingerprint = report_fingerprint(&rules)?;
    if let Some(path) = &options.replay {
        return replay(&rules, path, &fingerprint);
    }
    run_batch(&rules, options, &fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arena_opponent_defaults_to_balanced_without_changing_campaign_enemy() {
        let rules = Rules::load().unwrap();
        let options = SimulationArgs::try_parse_from(["sim"])
            .unwrap()
            .resolve(&rules)
            .unwrap();
        assert_eq!(options.schedule.seeds, 50);
        assert_eq!(options.schedule.campaign_seeds, 100);
        for (mode, arena_seeds) in [("arena", 50), ("campaign", 100), ("full", 50)] {
            let args = ["sim", "--mode", mode];
            let defaults = SimulationArgs::try_parse_from(args)
                .unwrap()
                .resolve(&rules)
                .unwrap();
            assert_eq!(defaults.schedule.seeds, arena_seeds);
            assert_eq!(defaults.schedule.campaign_seeds, 100);
            let override_seeds =
                SimulationArgs::try_parse_from(args.into_iter().chain(["--seeds", "7"]))
                    .unwrap()
                    .resolve(&rules)
                    .unwrap();
            assert_eq!(override_seeds.schedule.seeds, 7);
            assert_eq!(override_seeds.schedule.campaign_seeds, 7);
        }

        assert_eq!(options.schedule.opponent, Some(Policy::Balanced));
        assert_eq!(
            options.schedule.campaign_enemies,
            Some(vec![Policy::Classic])
        );
        let legacy = SimulationArgs::try_parse_from(["sim", "--vs-classic"])
            .unwrap()
            .resolve(&rules)
            .unwrap();
        assert_eq!(legacy.schedule.opponent, Some(Policy::Classic));
        assert!(
            SimulationArgs::try_parse_from(["sim", "--vs", "balanced", "--all-pairs"]).is_err()
        );
    }
    #[test]
    fn saved_phase_restores_generated_arenas_and_validates_geometry() {
        let rules = Rules::load().unwrap();
        let config: super::super::types::ArenaConfig = serde_json::from_value(json!({
            "seed": 7, "maxRounds": 30, "recruitLimit": 0, "landscape": "open",
            "first": "player", "player": "balanced", "enemy": "balanced",
            "armies": {"player": "coastal", "enemy": "human"}
        }))
        .unwrap();
        let mut expected = super::super::matches::create_arena(&rules, &config).unwrap();
        let state = expected.state();
        let request = |state| {
            serde_json::from_value::<PhaseRequest>(json!({
                "state": state, "arena": config, "policy": "balanced", "seed": 7,
                "beginTurn": true, "recruitmentLimit": 1
            }))
            .unwrap()
        };
        expected.begin_turn(expected.turn);
        expected.recordings = Some(crate::data::Records::default());
        expected
            .play_blocks(
                crate::ai::Policy::Balanced,
                None,
                7,
                1,
                None,
                crate::ai::Diplomacy::Pragmatic,
            )
            .unwrap();
        assert_eq!(
            saved_phase(&rules, request(state.clone())).unwrap(),
            json!({"final": expected.state(), "records": expected.recordings})
        );
        let mut inspected = request(state.clone());
        inspected.inspect_recruitment = true;
        let mut diagnostic = saved_phase(&rules, inspected).unwrap();
        assert!(
            !diagnostic
                .as_object_mut()
                .unwrap()
                .remove("recruitmentScores")
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            diagnostic,
            json!({"final":expected.state(),"records":expected.recordings})
        );
        let mut invalid = state.clone();
        invalid.tiles[0].col = -1;
        assert!(saved_phase(&rules, request(invalid)).is_err());
        let mut invalid = request(state);
        invalid.advance_turn = true;
        assert!(saved_phase(&rules, invalid).is_err());
    }
    #[test]
    fn append_failure_drains_in_flight_workers() {
        let jobs: Vec<_> = (0..32).collect();
        let mut appends = 0;
        let result = execute_jobs(&jobs, 2, &|job| Ok(*job), |_, _, _| {
            appends += 1;
            Err(SimulationError::Worker("injected journal failure".into()))
        });
        assert_eq!(appends, 1);
        assert!(
            matches!(result, Err(SimulationError::Worker(message)) if message == "injected journal failure")
        );
    }
    #[test]
    fn execution_failure_stops_before_appending() {
        let jobs = [0, 1, 2, 3];
        let mut appends = 0;
        let result = execute_jobs(
            &jobs,
            2,
            &|_: &i32| -> Result<i32, SimulationError> {
                Err(SimulationError::Worker("injected execution failure".into()))
            },
            |_, _, _| {
                appends += 1;
                Ok(())
            },
        );
        assert_eq!(appends, 0);
        assert!(
            matches!(result, Err(SimulationError::Worker(message)) if message == "injected execution failure")
        );
    }
}
