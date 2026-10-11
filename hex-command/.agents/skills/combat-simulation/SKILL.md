---
name: combat-simulation
description: Run requested Hex Command combat simulations and analyze saved results, including roster coverage, matched balance comparisons, and readable reports.
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
Node dependencies must be installed; use `npm ci` if needed. Analysis requires
`uv`, which installs the Python script's pinned dependencies.

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

For a request about saved data, run only the analyzer. It reads the compact outcome
journal, not gameplay details, and never builds or runs battles. The analyzer and
shared journal schema are bundled in this skill's `scripts/` directory; the
`ai:analyze` npm command invokes the bundled analyzer.

```sh
npm run ai:analyze -- artifacts/current/report.json \
  --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

Baseline comparisons require identical match configurations; mismatches fail.
Check roster and rules differences as well: a change in recruitment coverage is
part of the experiment, not evidence of a particular weapon or stat's effect.
Use the report's recorded source commit and fingerprint when describing historical
data. Do not infer its roster or rules from the current checkout.

## Report findings

Link `analytics/summary.md` and retain its CSV breakdowns and `timings.json`.
State the recorded source commit, actual completed match count, seed range,
selected scope and roster omissions relevant to the question. Mention caps and
the limited evidence from small seed panels.

For race balance, lead with `race_equal_policy.csv`, which compares different
races using the same policy. Use `race_map.csv` for terrain effects and
`race_matchups.csv` for particular opponents and policies. Overall `race.csv`
mixes policies; policy tables include both seats of same-race matches. Label those
denominators accurately. A roster can perform differently under different AI.

Keep round-capped games in win-rate denominators and report them as unresolved.
Separate leader and territory victories, and simulation timing from build time
when useful. Campaign stage completion counts only attempts reaching that stage.
Treat the results as measurements of these rosters, maps and policies, not a claim
about optimal play or every unit in a race. Do not commit generated artifacts,
publish, or change gameplay as part of running or analyzing simulations.
