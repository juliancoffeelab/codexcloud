//! Native schedule, bounded scoped worker pool, reporting and replay.
use super::{
    checkpoint::{IndexedOutcome, Progress},
    error::SimulationError,
    matches,
    records::{DetailRecord, EncodedRecord, ResultJournal, read_detail, sha256},
    report,
    schedule::{Mode, ScheduleOptions, WithSelection, schedule},
    types::{MatchConfig, MatchResult},
};
use crate::data::Rules;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
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

const HELP: &str = "Usage: hex-command-sim [options]\n  --mode arena|campaign|full      default full\n  --with recruiter,campaigner,planner  add custom tactic; - uses workload defaults\n  --seeds N --seed N              count (default 10) and first seed (default 1)\n  --policies classic,rush,balanced,rotated\n  --armies human,woodland,undead,coastal\n  --landscapes open,wooded,crossings,islands\n  --campaigns lantern-road,green-accord,ashen-oath,borrowed-tide\n  --campaign-players balanced,rotated  independent campaign player tactics\n  --campaign-enemies classic       independent campaign enemy tactics\n  --rounds N --recruits N         bounded rounds (default arena 30, campaign 100), explicit recruits/phase override\n  --diplomacy pragmatic,restrained filter authored player diplomacy modes\n  --workers N                    scoped thread pool, CPU quota default, maximum 8\n  --out PATH                     Output directory or .json file (default fresh artifacts/simulation-* directory)\n  --no-report                  retain raw results without running the report analyzer\n  --details arrow|jsonl         lossless gameplay journal (default jsonl)\n  --all-pairs                    full arena policy matrix instead of facing Classic\n  --baseline PATH                compare report to a saved run (repeatable)\n  --vs-classic                   arena players face Classic (default); both seats/initiatives\n  --self-play                    arena games use the same policy on both sides\n  --profile-turns                report policy mean decision and complete-phase CPU time\n  --trace                        also retain raw match results\n  --replay PATH                  verify saved report details and results (strict fingerprint)\n  --data-dir PATH                authored JSON catalog root (HEX_COMMAND_DATA_ROOT also works)\n  --schedule-only                emit configurations without running any matches\n  --stdio                        JSON match request/result line protocol\n  --phase-stdio                  saved-position own-phase profiling protocol\n  --fingerprint                  compiled content and engine SHA256 fingerprints\nArena entrants have both seats and both initiatives; self-play has both initiatives.\nRecruitment defaults: Classic 2 per phase; other policies uncapped.\nCampaigns begin at chapter one, earn carryover and receive no retries. Caps remain unresolved.";
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
    no_report: bool,
    baselines: Vec<PathBuf>,
}
fn flag_integer(
    values: &BTreeMap<String, String>,
    flag: &str,
    fallback: u32,
    min: u32,
    max: u32,
) -> Result<u32, SimulationError> {
    let value = values
        .get(flag)
        .map_or(Ok(fallback), |value| value.parse::<u32>());
    match value {
        Ok(value) if (min..=max).contains(&value) => Ok(value),
        _ => Err(format!("{flag} must be {min}..{max}").into()),
    }
}
fn selection(
    values: &BTreeMap<String, String>,
    flag: &str,
    available: &[String],
) -> Result<Vec<String>, SimulationError> {
    let Some(input) = values.get(flag) else {
        return Ok(available.to_vec());
    };
    let selected: Vec<_> = input
        .split(',')
        .map(|entry| entry.trim().to_owned())
        .collect();
    let mut seen = HashSet::new();
    if selected.is_empty()
        || selected
            .iter()
            .any(|entry| !available.contains(entry) || !seen.insert(entry))
    {
        return Err(format!(
            "Invalid {flag}: expected unique entries from {}",
            available.join(",")
        )
        .into());
    }
    Ok(selected)
}
fn parse(args: &[String], rules: &Rules) -> Result<CliOptions, SimulationError> {
    let expanded: Vec<String> = args
        .iter()
        .flat_map(|arg| {
            if let Some((flag, value)) = arg.split_once('=') {
                vec![flag.to_owned(), value.to_owned()]
            } else {
                vec![arg.clone()]
            }
        })
        .collect();
    let args = &expanded;
    let mut values = BTreeMap::new();
    let mut switches = HashSet::new();
    let mut baselines = vec![];
    let mut index = 0;
    const VALUE_FLAGS: &[&str] = &[
        "--with",
        "--mode",
        "--seeds",
        "--seed",
        "--policies",
        "--campaign-players",
        "--campaign-enemies",
        "--armies",
        "--landscapes",
        "--campaigns",
        "--rounds",
        "--recruits",
        "--diplomacy",
        "--workers",
        "--out",
        "--replay",
        "--details",
        "--baseline",
    ];
    while index < args.len() {
        let flag = &args[index];
        if [
            "--trace",
            "--schedule-only",
            "--profile-turns",
            "--self-play",
            "--vs-classic",
            "--no-report",
            "--all-pairs",
        ]
        .contains(&flag.as_str())
        {
            switches.insert(flag.clone());
        } else if VALUE_FLAGS.contains(&flag.as_str()) {
            index += 1;
            let value = args
                .get(index)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("Missing value for {flag}"))?;
            if flag == "--baseline" {
                baselines.push(PathBuf::from(value));
            } else if values.insert(flag.clone(), value.clone()).is_some() {
                return Err(format!("Duplicate option: {flag}").into());
            }
        } else {
            return Err(format!("Unknown option: {flag}").into());
        }
        index += 1;
    }
    if switches.contains("--schedule-only") && values.contains_key("--replay") {
        return Err("--schedule-only cannot be combined with --replay".into());
    }
    if switches.contains("--no-report") && !baselines.is_empty() {
        return Err("--baseline requires report generation; remove --no-report".into());
    }
    if switches.contains("--all-pairs") && switches.contains("--vs-classic") {
        return Err("--all-pairs cannot be combined with --vs-classic".into());
    }
    let mut options = ScheduleOptions::defaults(rules)?;
    options.with = values
        .get("--with")
        .map(|value| WithSelection::parse(value))
        .transpose()?;
    options.mode = match values.get("--mode").map(String::as_str).unwrap_or("full") {
        "arena" => Mode::Arena,
        "campaign" => Mode::Campaign,
        "full" => Mode::Full,
        _ => return Err("Unknown simulation mode; use arena, campaign or full".into()),
    };
    options.first_seed = flag_integer(&values, "--seed", 1, 0, 2147483647)?;
    options.seeds = flag_integer(&values, "--seeds", 10, 1, 10000)?;
    options.self_play = switches.contains("--self-play");
    options.vs_classic = switches.contains("--vs-classic")
        || (options.mode != Mode::Campaign
            && !switches.contains("--all-pairs")
            && !options.self_play);
    if options.vs_classic && options.self_play {
        return Err("--vs-classic cannot be combined with --self-play".into());
    }
    if options.self_play && options.mode != Mode::Arena {
        return Err("--self-play requires --mode arena".into());
    }
    let default_rounds = if options.mode == Mode::Campaign {
        rules.campaign_defaults.default_max_rounds
    } else {
        rules.arena.default_max_rounds
    };
    options.max_rounds = flag_integer(&values, "--rounds", default_rounds, 1, 1000)?;
    options.campaign_max_rounds = flag_integer(
        &values,
        "--rounds",
        rules.campaign_defaults.default_max_rounds,
        1,
        1000,
    )?;
    options.recruit_limit = values
        .contains_key("--recruits")
        .then(|| flag_integer(&values, "--recruits", 0, 0, 20).map(|limit| limit as usize))
        .transpose()?;
    options.armies = selection(&values, "--armies", &options.armies)?;
    options.landscapes = selection(&values, "--landscapes", &options.landscapes)?;
    options.campaigns = selection(&values, "--campaigns", &options.campaigns)?;
    let policy_ids: Vec<_> = crate::ai::Policy::ALL
        .iter()
        .map(|p| p.as_str().to_owned())
        .collect();
    options.policies = crate::ai::Policy::ALL.to_vec();
    if values.contains_key("--policies") {
        options.policies = selection(&values, "--policies", &policy_ids)?
            .into_iter()
            .map(|id| serde_json::from_value(json!(id)).map_err(SimulationError::from))
            .collect::<Result<_, _>>()?;
    }
    let mut participant_ids = policy_ids.clone();
    if options.with.is_some() {
        participant_ids.push("custom".into());
    }
    for (flag, destination, custom) in [
        (
            "--campaign-players",
            &mut options.campaign_players,
            &mut options.campaign_custom_player,
        ),
        (
            "--campaign-enemies",
            &mut options.campaign_enemies,
            &mut options.campaign_custom_enemy,
        ),
    ] {
        if values.contains_key(flag) {
            let selected = selection(&values, flag, &participant_ids)?;
            *custom = selected.iter().any(|id| id == "custom");
            *destination = Some(
                selected
                    .into_iter()
                    .filter(|id| id != "custom")
                    .map(|id| serde_json::from_value(json!(id)).map_err(SimulationError::from))
                    .collect::<Result<_, _>>()?,
            );
        }
    }
    if options.mode != Mode::Arena {
        if options.campaign_players.is_none() {
            if options.with.is_some() {
                options.campaign_custom_player = true;
                options.campaign_players = Some(vec![]);
            } else {
                let players: Vec<_> = options
                    .policies
                    .iter()
                    .copied()
                    .filter(|p| {
                        matches!(p, crate::ai::Policy::Balanced | crate::ai::Policy::Rotated)
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
            .get_or_insert_with(|| vec![crate::ai::Policy::Classic]);
    }
    let diplomacy_ids: Vec<_> = options
        .diplomacy
        .iter()
        .map(|diplomacy| diplomacy.as_str().to_owned())
        .collect();
    options.diplomacy = selection(&values, "--diplomacy", &diplomacy_ids)?
        .into_iter()
        .map(|id| serde_json::from_value(json!(id)).map_err(SimulationError::from))
        .collect::<Result<_, _>>()?;
    let workers = flag_integer(&values, "--workers", default_workers() as u32, 1, 8)? as usize;
    let mut out = values.get("--out").map(PathBuf::from).unwrap_or_else(|| {
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
    let arrow = match values
        .get("--details")
        .map(String::as_str)
        .unwrap_or("jsonl")
    {
        "arrow" => true,
        "jsonl" => false,
        _ => return Err("--details must be arrow or jsonl".into()),
    };
    Ok(CliOptions {
        schedule: options,
        workers,
        out,
        trace: switches.contains("--trace"),
        arrow,
        replay: values.get("--replay").map(PathBuf::from),
        schedule_only: switches.contains("--schedule-only"),
        profile_turns: switches.contains("--profile-turns"),
        no_report: switches.contains("--no-report"),
        baselines,
    })
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
    game.recordings = Some(crate::data::Records::default());
    game.play_blocks(
        request.policy,
        request.blocks,
        request.seed,
        request.recruitment_limit,
        request.goals.as_deref(),
        request.diplomacy,
    )?;
    Ok(json!({"final":game.state(),"records":game.recordings}))
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
    if options.profile_turns {
        report["turnTiming"] = crate::ai::profiling::snapshot().unwrap();
    }
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
    if args == ["--help"] || args == ["-h"] {
        println!("{HELP}");
        return Ok(());
    }
    let mut args = args.to_vec();
    let mut data_dir = None;
    while let Some(index) = args.iter().position(|arg| arg == "--data-dir") {
        if data_dir.is_some() {
            return Err("Duplicate option: --data-dir".into());
        }
        let value = args
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or("Missing value for --data-dir")?
            .clone();
        data_dir = Some(PathBuf::from(value));
        args.drain(index..=index + 1);
    }
    let rules = if let Some(root) = data_dir {
        Rules::load_from(root)?
    } else {
        Rules::load()?
    };
    if args == ["--fingerprint"] {
        println!("{}", fingerprints(&rules));
        return Ok(());
    }
    if args == ["--stdio"] {
        return stdio(&rules);
    }
    if args == ["--phase-stdio"] {
        return phase_stdio(&rules);
    }
    let options = parse(&args, &rules)?;
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
    fn custom_tactic_is_added_to_matrix_and_campaign_roles() {
        let rules = Rules::load().unwrap();
        let mut options = parse(
            &[
                "--with=rotated,-,queued".into(),
                "--mode=arena".into(),
                "--seeds=1".into(),
            ],
            &rules,
        )
        .unwrap()
        .schedule;
        options.policies = vec![
            crate::ai::Policy::Classic,
            crate::ai::Policy::Rush,
            crate::ai::Policy::Balanced,
        ];
        options.armies = vec!["a".into(), "b".into()];
        options.landscapes = vec!["open".into()];
        let fixed = schedule(&options).unwrap();
        assert_eq!(fixed.len(), 16);
        assert_eq!(fixed.len(), options.expected_games().unwrap());
        assert!(fixed.iter().all(|config| matches!(config, MatchConfig::Arena(config) if
            (config.player_blocks.is_some() && config.enemy == crate::ai::Policy::Classic && config.enemy_blocks.is_none())
            || (config.enemy_blocks.is_some() && config.player == crate::ai::Policy::Classic && config.player_blocks.is_none()))));
        options.vs_classic = false;
        let all = schedule(&options).unwrap();
        assert_eq!(all.len(), 128);
        assert_eq!(all.len(), options.expected_games().unwrap());
        assert!(all.iter().any(|config| matches!(config, MatchConfig::Arena(config) if config.player_blocks.is_some() && config.enemy == crate::ai::Policy::Balanced && config.enemy_blocks.is_none())));
        let unique: std::collections::HashSet<_> = all
            .iter()
            .map(|config| serde_json::to_string(config).unwrap())
            .collect();
        assert_eq!(unique.len(), all.len());
        options.self_play = true;
        let self_play = schedule(&options).unwrap();
        assert_eq!(self_play.len(), 32);
        assert!(self_play.iter().all(|config| matches!(config, MatchConfig::Arena(config) if config.player == config.enemy && config.player_blocks == config.enemy_blocks)));
        let campaign = parse(
            &[
                "--with".into(),
                "rotation,-,queued".into(),
                "--mode".into(),
                "campaign".into(),
                "--seeds".into(),
                "1".into(),
                "--campaign-players".into(),
                "balanced".into(),
                "--campaign-enemies".into(),
                "classic,custom".into(),
            ],
            &rules,
        )
        .unwrap()
        .schedule;
        let configs = schedule(&campaign).unwrap();
        assert_eq!(configs.len(), 10);
        assert_eq!(configs.len(), campaign.expected_games().unwrap());
        assert!(configs.iter().all(|config| matches!(config, MatchConfig::Campaign(config) if config.player_blocks.is_none() && config.enemy_blocks.is_none_or(|blocks| blocks.campaign == crate::ai::CampaignBlock::Mission && blocks.recruiter == crate::ai::Recruiter::Rotation))));
        assert!(parse(&["--with=-,-".into()], &rules).is_err());
        assert!(parse(&["--with=unknown,-,queued".into()], &rules).is_err());
        assert!(parse(&["--campaign-enemies=custom".into()], &rules).is_err());
    }

    #[test]
    fn standard_panel_reporting_and_output_options_are_shared_by_all_callers() {
        let rules = Rules::load().unwrap();
        let defaults = parse(&[], &rules).unwrap();
        assert_eq!(defaults.schedule.mode, Mode::Full);
        assert_eq!(defaults.schedule.recruit_limit, None);
        assert_eq!(defaults.schedule.max_rounds, rules.arena.default_max_rounds);
        assert_eq!(
            defaults.schedule.campaign_max_rounds,
            rules.campaign_defaults.default_max_rounds
        );
        let mut one_seed = defaults.schedule.clone();
        one_seed.seeds = 1;
        assert!(
            schedule(&one_seed)
                .unwrap()
                .iter()
                .all(|config| match config {
                    MatchConfig::Arena(config) =>
                        config.max_rounds == rules.arena.default_max_rounds,
                    MatchConfig::Campaign(config) =>
                        config.max_rounds == rules.campaign_defaults.default_max_rounds,
                })
        );
        let extended = parse(&["--rounds".into(), "100".into()], &rules).unwrap();
        assert_eq!(extended.schedule.max_rounds, 100);
        assert_eq!(extended.schedule.campaign_max_rounds, 100);
        let limited = parse(&["--recruits".into(), "1".into()], &rules).unwrap();
        assert_eq!(limited.schedule.recruit_limit, Some(1));
        assert!(defaults.schedule.vs_classic && !defaults.no_report && !defaults.arrow);
        assert_eq!(
            defaults.schedule.campaign_enemies,
            Some(vec![crate::ai::Policy::Classic])
        );
        let raw = parse(
            &["--no-report".into(), "--out".into(), "artifacts/raw".into()],
            &rules,
        )
        .unwrap();
        assert!(raw.no_report);
        assert_eq!(raw.out, PathBuf::from("artifacts/raw/report.json"));
        let all_pairs = parse(
            &[
                "--all-pairs".into(),
                "--out".into(),
                "saved.json".into(),
                "--baseline".into(),
                "first.json".into(),
                "--baseline".into(),
                "second.json".into(),
            ],
            &rules,
        )
        .unwrap();
        assert!(!all_pairs.schedule.vs_classic);
        assert_eq!(all_pairs.out, PathBuf::from("saved.json"));
        assert_eq!(all_pairs.baselines.len(), 2);
        assert!(
            parse(
                &[
                    "--no-report".into(),
                    "--baseline".into(),
                    "first.json".into()
                ],
                &rules
            )
            .is_err()
        );
        assert!(parse(&["--all-pairs".into(), "--vs-classic".into()], &rules).is_err());
        assert!(
            parse(
                &[
                    "--mode".into(),
                    "campaign".into(),
                    "--policies".into(),
                    "battler".into()
                ],
                &rules
            )
            .is_err()
        );
    }

    #[test]
    fn saved_phase_restores_generated_arenas_and_validates_geometry() {
        let rules = Rules::load().unwrap();
        let defaults = parse(&[], &rules).unwrap();
        assert!(!defaults.no_report);
        assert!(parse(&["--no-report".into()], &rules).unwrap().no_report);
        assert!(!defaults.profile_turns && !defaults.schedule.self_play);
        assert!(
            serde_json::to_value(&defaults.schedule)
                .unwrap()
                .get("selfPlay")
                .is_none()
        );
        let timing = parse(
            &[
                "--mode".into(),
                "arena".into(),
                "--self-play".into(),
                "--profile-turns".into(),
            ],
            &rules,
        )
        .unwrap();
        assert!(timing.profile_turns && timing.schedule.self_play);
        let fixed = parse(&["--vs-classic".into()], &rules).unwrap();
        assert!(fixed.schedule.vs_classic);
        assert!(parse(&["--vs-classic".into(), "--self-play".into()], &rules).is_err());
        assert!(
            parse(
                &["--self-play".into(), "--mode".into(), "campaign".into()],
                &rules
            )
            .is_err()
        );
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
