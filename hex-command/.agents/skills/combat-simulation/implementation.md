# Combat simulation implementation

The native Rust harness owns scheduling, battle execution, campaign transitions
and result recording. The Python runner invokes that harness and then analyzes
the saved report. Reporting uses only the Python standard library.

## Analyzer boundary

All analysis derives from input report outcomes and metadata. The analyzer does
not fetch unit stats, reconstruct artwork or roster coverage, read gameplay
catalogs, look up Git commits or run battles. Recorded source commits and
fingerprints are displayed without resolving them. It works outside a repository
checkout and accepts reports with missing or unavailable source commits.

Arena grouping uses recorded faction, opponent, map and both policies. Every
physical match contributes one result per participant. Mirrored cells describe
the same physical matches; self-play contributes both seats. Round caps remain
in win-rate denominators. Recruitment, promotion and casualty events belong to
their recorded side, including when factions swap seats.

Duplicate match configurations and inconsistent winner/cap records fail analysis.
Baseline comparisons require identical recorded configurations. Arena comparisons
use winners, victory reasons and final-state hashes. Campaign comparisons use
completion and ordered mission results, rounds and final-state hashes, exporting
both mission histories. Timing and journal-file changes do not imply gameplay
changes.

The HTML, Markdown and CSV exports retain every recorded matchup, event and match.
HTML filtering affects only its display. `full-data.zip` includes current exports
and adjacent compact outcome journals when available; it excludes obsolete files
left in an existing output directory. Full gameplay journals remain beside the
input report. `timings.json` records analyzer execution time.

## Native data and recording

A reused native binary defaults to the data directory from its compiled crate,
regardless of its working directory. Pass the intended `--data-dir` when using
another checkout. `ai:combat` supplies the current project's directory by default
and forwards a custom directory without comparing it to Git history.

Keep each run's JSON report, Markdown report, `.outcomes.jsonl`, `.details.jsonl`,
`.partial.json` and optional `.trace.jsonl` together in a fresh ignored artifact
directory. A completed run exits successfully and records `completed == expected`.
On failure, retain partial output and report how many matches completed; worker
errors are not defeats. Details and recovery are documented in the
[recording guide](../../../rust/src/simulation/README.md).
