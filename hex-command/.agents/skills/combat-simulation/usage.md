# Combat simulation usage

Run commands from the `hex-command/` project root. The agent workflow is in
[SKILL.md](SKILL.md). For analyzer/harness changes or debugging, read
[implementation.md](implementation.md).

## Run the role-appropriate full panel

```sh
# Build if needed, run all arena policies/maps and Mixed/Priority campaigns vs Classic,
# then automatically generate a readable analysis.
npm run ai:full -- --out artifacts/current

# Focused campaign comparison, retaining both campaign players and the fixed enemy.
npm run ai:combat -- --mode campaign --campaign-players mixed,priority \
  --campaign-enemies classic --seeds 30 --out artifacts/campaign-current

# Inspect a schedule without running it.
npm run ai:schedule -- --mode full --seeds 10 \
  --policies classic,rush,mixed,priority,battler \
  --campaign-players mixed,priority --campaign-enemies classic
```

`ai:full` builds the native harness when needed and runs the combined full
panel. `ai:combat` also builds when needed and automatically runs the analyzer
after successful simulation. Do not include Classic or Rush as campaign players
when measuring campaign competence; they remain available in the arena matrix.
Use `npm run ai:simulate -- --help` for current options and defaults, and
`ai:schedule` to inspect selected configurations and counts.

A reused native binary loads data from its compiled crate by default. When using
another checkout, pass its `--data-dir` explicitly. `ai:combat` selects this
project's data by default and forwards a custom directory.

`ai:rust:build` compiles for the current CPU using LLVM profile-guided
optimization. Its training workload covers every existing policy, landscape and
campaign at seeds 17–18, with 30-round caps. It caches the compiler profile by
source, catalog, toolchain and CPU; unchanged builds reuse it. Training results
are not used as simulation answers. Build time is separate from command timing.
Wasm builds use their ordinary toolchain and are unaffected by native flags.

Frozen references `classicold`, `rushold` and `mixedold` contain the complete
engine and catalog from `b1f553de69c0de8a340088c0a9ed42b2149fb27e`. They retain
the system allocator, thin LTO and eight codegen units. Builds verify the vendored
file checksums and never consult a moving Git branch. Compare all eight arena
policies without switching branches:

```sh
npm run ai:simulate -- --data-dir rust/legacy/data --mode arena --self-play --profile-turns --policies classic,rush,mixed,battler,priority,classicold,rushold,mixedold --seeds 10 --seed 1 --rounds 30 --recruits 1 --workers 1 --out artifacts/policy-comparison/report.json
```

This runs 1,280 games per policy. One timed turn is one side's completed own phase;
`meanDecisionMicrosPerTurn` sums chooser CPU and divides by actual completed
phases. The frozen engine completes its original execution/JSONL recording path
before the current host imports its complete gameplay into Arrow. Reported old
chooser/phase CPU excludes this import; whole command wall time includes both
recording passes. Serial per-game transport and overlapping transport altered
allocator reuse and failed to reproduce reference CPU costs.

Old references support complete native arena and campaign runs with old
opponents. Cross-version opponents and live/WASM planner continuations are
rejected. Use `--self-play` when combining old and current policies in an arena
report. Use only old policies for campaign comparisons. The frozen data directory
keeps the comparison stable after another PR changes the live catalog. Timings
vary between runs; compare against a fresh frozen-binary run as well as historical
measurements. Full gameplay, phase counts and decision counts must match exactly.

Full gameplay is saved directly to `.details.arrow` by default. Each outcome's
checksummed byte range contains one standard Arrow IPC stream. Troop snapshots
use typed columns, ownership snapshots store owner codes against chapter tile
identities, and ordered phase columns reference those dictionaries. JSON headers
retain complete commands, combat records, objectives, campaign data and endpoints.
No rounds or gameplay fields are omitted. Use `--details jsonl` for legacy output.
The analyzer reads the compact `report.json` with either format.

## Interpret the panels

Arena entrants pair a race with a policy. Opponents are scheduled in both seats
and both initiatives; identical entrants run both initiatives once. Each round
contains both phases. Maps and total starting budgets are paired; leaders and
starters are charged at their unit prices before the remaining gold is available
for recruitment.

Arena victory comes from defeating the opposing leader or holding all central
objectives for the landscape's configured duration. Losing an objective resets
the hold. This territory rule belongs to the simulation arena. Campaign runs use
authored rosters, maps, objectives and waves, earn carryover and recall survivors,
and receive no retries. Their seeds vary AI tie breaks rather than authored maps.

Count unresolved round caps in arena win-rate denominators. Mirrored faction rows
describe the same physical matches; self-play contributes both seats. Native
confidence intervals resample complete seed blocks, preserving map/seat/initiative
correlations. They measure these scenarios and policies, rather than optimal or
human play. More seeds reduce sampling uncertainty, rather than policy weakness.

Campaign competence is measured with Mixed/Priority as player and Classic as
the fixed enemy. Compare those policies on matched campaign, seed, diplomacy,
round limit and recruit limit. Campaign players use authored rosters, maps,
objectives and waves, earn carryover and recall survivors, and receive no retries.
Campaign seeds vary AI tie breaks rather than authored maps. Report completions
and denominators, then stage outcomes and goal progress; keep round caps,
defeats, and completions distinct. Classic/Rush player campaign results are not
competence evidence.

## Read the automatic report internally and present results in chat

```sh
cat artifacts/current/analytics/findings.md
cat artifacts/current/analytics/summary.md
```

The full runner has already analyzed a successful batch. The findings open with
coverage, compact race-by-map win tables against Classic for each player policy,
and campaign completion tables. Wins include caps in their denominator; opponent
races are pooled and identical race/policy self-play is excluded. Stage diagnostics
and the existing detailed matchup matrices follow. Summarize both arenas and
campaigns for a full run. Present the compact tables and important findings
directly in chat. The Markdown reports and CSV exports are internal records;
users in cloud environments cannot open workspace paths. Do not require the user
to open a file or replace the answer with local file links. Deliver files only
when explicitly requested, using a mechanism the user can access. CSV exports
contain every recorded table, including arena comparisons and campaign/stage/failure
diagnostics. With
`--profile-turns`, readable reports include chooser and phase CPU by policy. A matched `--baseline` adds campaign completion changes. New
campaign journals also retain explicit orders and planning reports; the analyzer
reports unmet-duty reasons and checks protected actors for retaliation damage.
Duty counts describe observations per phase, not independent failed attempts.
Use CSV exports only when a table-shaped view is useful.

When explicitly asked to analyze a saved run or compare saved reports, invoke
the maintained analyzer directly:

```sh
npm run ai:analyze -- artifacts/current/report.json --out artifacts/current/analytics
npm run ai:analyze -- artifacts/current/report.json \
  --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

When a report lacks a finding needed to answer the question, first add that
finding to the maintained analyzer and its tests. Then analyze the existing
saved run again. Do not replace this step with an ad hoc scratch script or
manual parsing of journal/CSV/JSON files. Inspect a targeted phase journal or
the implementation only after the report identifies a specific unexplained
match, stage or failure pattern. See [implementation.md](implementation.md) for
the report's evidence boundary and supported diagnostics.

For arena balance, use the equal-policy and map-policy views to compare other
factions with matching policies. The matchup table retains every opponent and
policy pairing, including self-play; all-policies aggregates mix policies.

The analyzer uses saved outcomes and the run's checksummed gameplay journal;
it does not rerun battles. It can run outside a Git checkout. Baseline reports
must contain exactly the same match configurations; missing or duplicate
configurations fail analysis. Full gameplay remains in the adjacent detail
journal; report outputs and compact outcomes remain in their run directory.
The analyzer also accepts a journal compressed as its original filename plus
`.gz`; offsets and checksums continue to refer to the decompressed bytes.

## Progress and recovery

Keep a batch's report, Markdown output, `.outcomes.jsonl`, `.details.jsonl`,
`.partial.json` and optional `.trace.jsonl` together. Progress is in `.partial.json`;
a complete run exits successfully with `completed == expected`. Preserve partial
files after a failure and report the completed count; worker errors are not defeats.

`.outcomes.jsonl` records completed results in worker completion order. To recover
them, read complete newline-terminated records and place each `outcome` at its
zero-based schedule `index`; ignore an interrupted trailing line. Reject duplicate
or out-of-range indices and leave missing positions unresolved. The progress
snapshot may lag completed journal records. Recovered outcomes retain checksummed
references to their full details. Recovery exposes completed work; it does not
automatically resume or rerun the batch.

## Replay and benchmarks

Replay reruns the recorded matches and requires the original engine/content
fingerprint. It validates journal references before execution:

```sh
npm run ai:simulate -- --replay artifacts/current/report.json
```

`npm run ai:benchmark -- --out artifacts/baseline.json` runs a representative
sample. Use `--limit 12` for a smaller sample, or `--compare artifacts/baseline.json`
to check identical complete-result hashes after an optimization.

## Compare full saved gameplay

For exact optimization checks, compare complete reports and their adjacent journals:

```sh
uv run --with pyarrow scripts/compare-simulation-reports.py artifacts/baseline/report.json artifacts/current/report.json --out artifacts/current/equality.json
```

This validates journal checksums and every gameplay field, including endpoint
hashes, unknown fields, missing/null distinctions and numeric representations.
Only object key order and explicitly listed run metadata (source commit/fingerprint,
elapsed times, journal format and journal locations) may differ. Arrays retain their order.
Use identical workloads and CPU quotas; build separately and time the complete
native command. A single-digit target is satisfied only when measured.

Export complete Arrow gameplay back to JSONL, in canonical schedule order:

```sh
uv run --with pyarrow scripts/export-simulation-details.py artifacts/current/report.json artifacts/current/export.jsonl
```

For columnar analysis, convert each detail dump once and compare the Arrow files. These
commands compare recorded gameplay without rerunning matches:

```sh
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

AI behavior is described in the [policy reference](../../../rust/src/ai/README.md).
