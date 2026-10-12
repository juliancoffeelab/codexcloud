# Combat simulation implementation

Read this when changing or debugging the native harness, Python runner or analyzer.
Normal operation, result interpretation and recovery are in [usage.md](usage.md).

The Rust harness owns scheduling, battle execution, campaign transitions and
recording. The Python runner invokes it and then analyzes the saved report.
The runner supplies the current project's data directory unless overridden;
it does not compare authored data to Git history. Reporting uses only Python's
standard library.

## Analyzer boundary

All analysis derives from input report outcomes and metadata. The analyzer does
not fetch unit stats, reconstruct artwork or roster coverage, read gameplay
catalogs, look up Git commits or run battles. Recorded source commits and
fingerprints are displayed without resolving them.

Arena grouping uses recorded faction, opponent, map and both policies. Each
physical match contributes one result per participant, including both seats
in self-play. Recruitment, promotion and casualty events belong to their recorded
side. Caps remain in denominators.

Duplicate configurations and inconsistent winner/cap records fail analysis.
Baseline comparisons require identical configurations. Arena comparisons use
winners, victory reasons and final-state hashes. Campaign comparisons use
completion and ordered mission results, rounds and final-state hashes, exporting
both mission histories. Timing and journal-file changes do not imply gameplay
changes.

HTML, Markdown and CSV exports retain all recorded matchups, events and matches.
HTML filtering affects only display. `full-data.zip` uses an explicit export
list and includes adjacent compact outcome journals when available, excluding
obsolete files in an existing output directory. Full gameplay journals remain
beside the input report. `timings.json` records analyzer execution time.
Native aggregate rows own seed-block confidence intervals; the analyzer does
not calculate new intervals.

## Scheduling and execution

Arena scheduling pairs `(race, policy)` entrants across seats and initiatives;
identical entrants avoid duplicate seat permutations. Schedule order is canonical
and independent of worker timing. Campaign scheduling combines authored campaigns,
player/enemy policies and diplomacy settings.

Arena maps use rotationally paired geometry, equal total budgets and paired
recruitment limits. The leader and starter prices are deducted from the starting
budget. Arena holding counters advance only after completed opposing turns;
losing any required objective resets the counter. Campaign execution delegates
objectives, waves and transitions to the ordinary battle rules.

Per-match records contain phase commands, combat damage/retaliation, casualties,
recruitment, promotions and endpoint states. Phase snapshots retain unit identity,
position, HP, XP, action flags and leader mode, plus gold, ownership, objective
progress and campaign state. Casualties include troops recruited and killed
within the same phase. Composition summaries are allocated separately; detailed
summaries borrow full trace arrays from the typed result.

## Journals and checkpoints

The final report is schema 3. Compact outcomes appear in canonical schedule order
and retain byte ranges and SHA-256 digests into `.details.jsonl`. `--trace` also
serializes raw typed results to `.trace.jsonl`.

Completed outcomes append once to `.outcomes.jsonl` as
`{"index": <schedule position>, "outcome": <compact outcome>}` in worker completion
order. `.partial.json` is an atomic progress snapshot marked
`"progressFormat": "indexed-outcomes-jsonl-v1"`, containing status, completed and
expected counts, elapsed time, fingerprint, journal filenames and complete byte
lengths. Progress refreshes every ten completions and on completion or error;
it does not copy accumulated outcomes.

`checkpoint::recover_outcomes` reads complete index records, rejecting duplicates
and out-of-range indices. `records::read_detail` validates byte ranges and checksums
before decoding full details. Comparison summaries omit state hashes while
preserving actual states, array order and missing-versus-null distinctions.

## Full gameplay comparison

`scripts/convert-simulation-dump.py` reads detail journals with an explicit Arrow
schema. `scripts/compare-simulation-dumps.py` uses a one-to-one Polars join on
`(kind, config)` and preserves array order. Missing or duplicate identities fail.
It excludes `stateHash`, `elapsedMs`, `elapsedSeconds` and `detail` at match and
mission level, and records wall time, CPU time and peak memory.

Planning internals are in [AI policy implementation](../../../rust/src/ai/README.md);
content loading is in [authored game data](../../../rust/data/README.md).
