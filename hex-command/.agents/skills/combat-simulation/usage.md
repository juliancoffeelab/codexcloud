# Combat simulation usage

Run commands from the `hex-command/` project root. The agent workflow is in
[SKILL.md](SKILL.md). For analyzer/harness changes or debugging, read
[implementation.md](implementation.md).

## Run a panel

```sh
# Enumerate the full schedule without running matches.
npm run ai:schedule -- --mode full --seeds 10

# Simulate a complete Mixed-policy panel and analyze it.
npm run ai:combat -- --policies mixed --seed 61 --seeds 50 --out artifacts/current

# Restrict races, policies and maps.
npm run ai:combat -- --armies human,coastal --policies classic,mixed --landscapes crossings --seeds 2 --out artifacts/coast-panel

# Native arena or campaign output without the Python analysis step.
npm run ai:simulate -- --mode arena --seeds 2 --out artifacts/arena.json
npm run ai:simulate -- --mode campaign --seeds 5 --out artifacts/campaigns.json
```

Use `npm run ai:simulate -- --help` for current options and defaults, and
`ai:schedule` to inspect the selected configurations and counts. `ai:full` runs
the combined arena/campaign batch configured in `package.json`.

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

## Interpret the panel

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

Count unresolved round caps in win-rate denominators. Mirrored faction rows
describe the same physical matches; self-play contributes both seats. Native
confidence intervals resample complete seed blocks, preserving map/seat/initiative
correlations. They measure these scenarios and policies, rather than optimal or
human play. More seeds reduce sampling uncertainty, rather than policy weakness.

## Analyze saved reports

```sh
npm run ai:analyze -- artifacts/current/report.json --out artifacts/current/analytics
npm run ai:analyze -- artifacts/current/report.json --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

Open `index.html` for searchable tables, `summary.md` for complete matrices, or
CSV exports for every row. For race balance, use `race_equal_policy.csv` and
`race_map_policy.csv` to compare other factions with matching policies.
`matchups.csv` retains every opponent and policy pairing, including self-play;
`race_all_policies.csv` mixes policies.

The analyzer uses only saved outcomes and metadata. It can run outside a Git
checkout. Baseline reports must contain exactly the same match configurations;
missing or duplicate configurations fail analysis. Full gameplay is in the
adjacent `.details.jsonl` journals, while `full-data.zip` contains the exported
tables and available compact outcome journals.

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
