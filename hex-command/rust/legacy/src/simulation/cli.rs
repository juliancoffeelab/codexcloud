//! Native schedule, bounded scoped worker pool, reporting and replay.
use super::{
    checkpoint::{IndexedOutcome, Progress},
    error::SimulationError,
    matches,
    records::{DetailRecord, ResultJournal, read_detail, sha256},
    report,
    schedule::{Mode, ScheduleOptions, schedule},
    types::{MatchConfig, MatchResult},
};
use crate::data::Rules;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Instant,
};

const HELP: &str = "Usage: hex-command-sim [options]\n  --mode arena|campaign|full      default arena\n  --seeds N --seed N              count (default 2) and first seed (default 1)\n  --seed-start N                  alias for --seed\n  --policies classic,rush,mixed   complete policies to compare\n  --armies human,woodland,undead,coastal\n  --landscapes open,wooded,crossings,islands\n  --campaigns lantern-road,green-accord,ashen-oath,borrowed-tide\n  --rounds N --recruits N         bounded rounds (default 30), recruits/phase (default 1)\n  --diplomacy pragmatic,restrained campaign behavior settings\n  --workers N                    scoped thread pool, CPU quota default, maximum 8\n  --out PATH                     JSON report and readable Markdown report\n  --trace                        also retain raw match results\n  --replay PATH                  verify saved report details and results (strict fingerprint)\n  --data-dir PATH                authored JSON catalog root (HEX_COMMAND_DATA_ROOT also works)\n  --schedule-only                emit configurations without running any matches\n  --stdio                        compatibility JSON request/result line protocol\n  --fingerprint                  compiled content and engine SHA256 fingerprints\nArena entrants have both seats and both initiatives; self-play has both initiatives.\nCampaigns begin at chapter one, earn carryover and receive no retries. Caps remain unresolved.";
#[derive(Debug)]
struct CliOptions {
    schedule: ScheduleOptions,
    workers: usize,
    out: PathBuf,
    trace: bool,
    replay: Option<PathBuf>,
    schedule_only: bool,
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
    let mut values = BTreeMap::new();
    let mut switches = HashSet::new();
    let mut index = 0;
    const VALUE_FLAGS: &[&str] = &[
        "--mode",
        "--seeds",
        "--seed",
        "--seed-start",
        "--policies",
        "--armies",
        "--landscapes",
        "--campaigns",
        "--rounds",
        "--recruits",
        "--diplomacy",
        "--workers",
        "--out",
        "--replay",
        "--engine",
    ];
    while index < args.len() {
        let flag = &args[index];
        if ["--trace", "--schedule-only"].contains(&flag.as_str()) {
            switches.insert(flag.clone());
        } else if VALUE_FLAGS.contains(&flag.as_str()) {
            index += 1;
            let value = args
                .get(index)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("Missing value for {flag}"))?;
            if values.insert(flag.clone(), value.clone()).is_some() {
                return Err(format!("Duplicate option: {flag}").into());
            }
        } else {
            return Err(format!("Unknown option: {flag}").into());
        }
        index += 1;
    }
    if values
        .get("--engine")
        .is_some_and(|engine| engine != "rust")
    {
        return Err("Native harness supports --engine rust".into());
    }
    if values.contains_key("--seed") && values.contains_key("--seed-start") {
        return Err("Specify one of --seed or --seed-start".into());
    }
    if switches.contains("--schedule-only") && values.contains_key("--replay") {
        return Err("--schedule-only cannot be combined with --replay".into());
    }
    let mut options = ScheduleOptions::defaults(rules)?;
    options.mode = match values.get("--mode").map(String::as_str).unwrap_or("arena") {
        "arena" => Mode::Arena,
        "campaign" => Mode::Campaign,
        "full" => Mode::Full,
        _ => return Err("Unknown simulation mode; use arena, campaign or full".into()),
    };
    options.first_seed = flag_integer(
        &values,
        if values.contains_key("--seed-start") {
            "--seed-start"
        } else {
            "--seed"
        },
        1,
        0,
        2147483647,
    )?;
    options.seeds = flag_integer(&values, "--seeds", 2, 1, 10000)?;
    let default_rounds = if options.mode == Mode::Campaign {
        rules.campaign_defaults.default_max_rounds
    } else {
        rules.arena.default_max_rounds
    };
    let default_recruits = if options.mode == Mode::Campaign {
        rules.campaign_defaults.default_recruit_limit
    } else {
        rules.arena.default_recruit_limit
    };
    options.max_rounds = flag_integer(&values, "--rounds", default_rounds, 1, 1000)?;
    options.recruit_limit =
        flag_integer(&values, "--recruits", default_recruits as u32, 0, 20)? as usize;
    options.armies = selection(&values, "--armies", &options.armies)?;
    options.landscapes = selection(&values, "--landscapes", &options.landscapes)?;
    options.campaigns = selection(&values, "--campaigns", &options.campaigns)?;
    let policy_ids: Vec<_> = options
        .policies
        .iter()
        .map(|policy| policy.as_str().to_owned())
        .collect();
    options.policies = selection(&values, "--policies", &policy_ids)?
        .into_iter()
        .map(|id| serde_json::from_value(json!(id)).map_err(SimulationError::from))
        .collect::<Result<_, _>>()?;
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
    let out = PathBuf::from(
        values
            .get("--out")
            .map(String::as_str)
            .unwrap_or("artifacts/ai-simulation.json"),
    );
    if out.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err("--out must end in .json".into());
    }
    Ok(CliOptions {
        schedule: options,
        workers,
        out,
        trace: switches.contains("--trace"),
        replay: values.get("--replay").map(PathBuf::from),
        schedule_only: switches.contains("--schedule-only"),
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
fn execute_schedule<F>(
    rules: &Rules,
    configs: &[MatchConfig],
    workers: usize,
    append: F,
) -> Result<(), SimulationError>
where
    F: FnMut(usize, MatchResult, f64) -> Result<(), SimulationError>,
{
    // Catalog/configuration are immutable shared borrows. Each game and planning workspace
    // stays entirely within its worker, with no locks or shared ownership on the turn path.
    execute_jobs(
        configs,
        workers,
        &|config| matches::run(rules, config.clone()),
        append,
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
fn run_batch(
    rules: &Rules,
    options: CliOptions,
    fingerprint: &str,
    reference: Option<&[MatchConfig]>,
) -> Result<(), SimulationError> {
    let configs = match reference {
        Some(configs) => configs.to_vec(),
        None => schedule(&options.schedule)?,
    };
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
    let mut journal = ResultJournal::create(&sibling(&output, ".details.jsonl")?)?;
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
        |index, result, elapsed| {
            let summary = report::borrowed_summary(&result, true)?;
            let reference = journal.append(&summary)?;
            if let Some(trace) = &mut trace {
                trace.append(&result)?;
            }
            let compact = report::compact_summary(&summary, reference, elapsed)?;
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
    // The host owns aggregate reporting. Keep every completed compact result
    // for transport after the original execution/recording loop has finished.
    if reference.is_some() {
        write_json(&output, &json!({"outcomes":outcomes}), false)?;
        return Ok(());
    }
    let report = report::build_report(
        &options.schedule,
        fingerprint,
        &source_commit,
        start.elapsed().as_secs_f64(),
        outcomes,
        trace_files,
    )?;
    write_json(&output, &report, true)?;
    fs::write(sibling(&output, ".md")?, report::markdown(&report))?;
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
    println!(
        "{}",
        json!({"output":output,"games":completed,"elapsedSeconds":report["elapsedSeconds"],"rows":report["rows"]})
    );
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
    let options = parse(&args, &rules)?;
    let fingerprint = report_fingerprint(&rules)?;
    if let Some(path) = &options.replay {
        return replay(&rules, path, &fingerprint);
    }
    run_batch(&rules, options, &fingerprint, None)
}

/// Transport supplies the schedule; execution and recording retain the frozen CLI path.
pub(crate) fn reference_batch(
    rules: &Rules,
    configs: &[MatchConfig],
    workers: usize,
    out: PathBuf,
) -> Result<(), SimulationError> {
    let mut schedule = ScheduleOptions::defaults(rules)?;
    schedule.mode = Mode::Full;
    schedule.first_seed = configs
        .iter()
        .map(|c| match c {
            MatchConfig::Arena(c) => c.seed,
            MatchConfig::Campaign(c) => c.seed,
        })
        .min()
        .unwrap_or(1);
    run_batch(
        rules,
        CliOptions {
            schedule,
            workers,
            out,
            trace: false,
            replay: None,
            schedule_only: false,
        },
        &report_fingerprint(rules)?,
        Some(configs),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
