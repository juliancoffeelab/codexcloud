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

Convert each detail dump once, then compare the resulting Arrow files. These
commands compare recorded gameplay without rerunning matches:

```sh
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

AI behavior is described in the [policy reference](../../../rust/src/ai/README.md).
