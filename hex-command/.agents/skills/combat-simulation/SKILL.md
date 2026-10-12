---
name: combat-simulation
description: Run requested Hex Command combat simulations and analyze saved results, including matched balance comparisons and readable reports.
---

# Combat simulation

Tell the user when applying this skill. Run commands from the `hex-command/`
project root. Use the existing simulation and analysis scripts; do not duplicate
their scheduling or statistics. See the [AI guide](../../../docs/ai-simulation.md)
for options and the [Rust toolchain skill](../rust-toolchain/SKILL.md) for builds.

## Scope and roster coverage

Run battles only when the user requests simulation. Creating or updating this
skill, analyzing saved data, and ordinary verification do not authorize another
batch. Honor simulation scope already authorized in the conversation without
asking again. For an unspecified combat simulation, use the default arena batch
with two seeds; include campaigns only when requested. `ai:full` runs both arenas
and campaigns with ten seeds.

Before running, inspect `rust/data/simulation/rosters.json` and `arena.json`.
Report the selected races, policies, maps, seeds, round cap and roster coverage
before presenting balance conclusions. Arena rosters have explicit leaders,
starting screens and recruitment pools. Follow `advancesTo` in
`rust/data/game/units/` to establish which promotions are reachable. Units outside
those paths are excluded even if they have gameplay stats or pixel artwork.
Pixel lineup trees are artwork catalogs, not simulation recruitment lists.
Campaigns use their own authored rosters.

Use the user's selected seeds, filters and limits. Otherwise retain CLI defaults,
including CPU-quota-aware workers. If workload size needs checking, enumerate
the selected options with `npm run ai:schedule -- ...`; this builds the runner
but executes no matches. Do not expand rosters or change balance to run a report.

## Run and preserve results

Choose a new ignored artifact directory for each run, preserving previous results.
The npm command builds the release runner using the project's toolchain wrapper.
Node dependencies must be installed; use `npm ci` if needed. Reporting uses Python 3 and its standard library.
Record which gameplay data and engine the batch uses. A reused native binary loads
data from its compiled crate location, regardless of the working directory. When
running it against another checkout, always pass that checkout's `--data-dir`.
The `ai:combat` wrapper supplies the current project's data directory explicitly
unless given a custom `--data-dir`.

```sh
simulation_run=artifacts/combat-$(date -u +%Y%m%dT%H%M%SZ)
npm run ai:simulate -- --mode arena --seeds 2 --out "$simulation_run/report.json"
npm run ai:analyze -- "$simulation_run/report.json" --out "$simulation_run/analytics"
```

Adapt the first command to the authorized scope. Keep the report, Markdown report,
`.outcomes.jsonl`, `.details.jsonl`, `.partial.json` and optional `.trace.jsonl`
together. Detailed gameplay can be much larger than the compact report; avoid
printing or loading entire journals to summarize results.

For long runs, report progress from the runner or `.partial.json`. A successful
run exits with zero and marks progress `complete` with `completed == expected`.
On failure, preserve partial output, report the error and count of completed
matches, and label any analysis as partial. Never count worker errors as defeats.
Recovery details are in [the recording guide](../../../rust/src/simulation/README.md).

## Analyze existing results and compare

For saved data, run `ai:analyze`: all analysis comes from the recorded reports.
It does not read gameplay catalogs, artwork or Git history, and never runs battles.
Source commits and fingerprints are displayed as recorded metadata only; analysis
works without those commits or a repository checkout.
For an authorized new batch, `npm run ai:combat -- --seeds 10 --out artifacts/current`
runs the native simulation and immediately generates the complete report.
The reporter and runner live in this skill's `scripts/` directory.
Always show all recorded factions. Never substitute one opponent or selected AI
policies for a request for full data. Include every map/policy-pair matrix, unit
recruitment, promotions, casualties, per-match results and baseline changes.

```sh
npm run ai:analyze -- artifacts/current/report.json \
  --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

Baseline comparisons require identical match configurations; mismatches fail.
Campaign comparisons include completion, reached chapters, chapter winners,
rounds and final-state hashes, with both sets of mission records exported.
Use the report's recorded source commit and fingerprint when describing historical
data. Report only the factions, units and configurations recorded in the inputs;
the analyzer does not reconstruct roster coverage or unit stats.

## Report findings

Link `analytics/index.html`, `analytics/summary.md` and `analytics/full-data.zip`.
The full report includes every matchup; the HTML filters never change the CSV exports.
The archive includes current and baseline compact outcome journals. Link original
`.details.jsonl` journals too when the user asks for all raw gameplay.
Retain every CSV and `timings.json`.
State the recorded source commit, actual completed match count, seed range,
selected scope and any coverage limitations established before the run. Mention caps and
the limited evidence from small seed panels.

For race balance, lead with `race_equal_policy.csv`, which compares different
races using the same policy. Use `race_map_equal_policy.csv` for terrain effects and
`matchups.csv` for every opponent and policy pairing. `race_all_policies.csv`
mixes policies. Self-play cells count both seats. Label those
denominators accurately. A roster can perform differently under different AI.
For a specific policy, use `race_map_policy.csv`: it keeps each policy separate
and excludes self-play. Show every race and map, then the full opponent matrices.

Keep round-capped games in win-rate denominators and report them as unresolved.
Separate leader and territory victories, and simulation timing from build time
when useful. Campaign stage completion counts only attempts reaching that stage.
Treat the results as measurements of these rosters, maps and policies, not a claim
about optimal play or every unit in a race. Do not commit generated artifacts,
publish, or change gameplay as part of running or analyzing simulations.
