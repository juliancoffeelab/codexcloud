---
name: combat-simulation
description: Run role-appropriate Hex Command combat simulations and read the maintained analyzer's complete human-readable reports before diagnosing balance.
---

# Combat simulation

Work from the `hex-command/` project root. Read [usage.md](usage.md) for
running and reviewing batches. Read [implementation.md](implementation.md) when
changing or debugging the runner or analyzer. Use the [Rust toolchain
skill](../rust-toolchain/SKILL.md) for build problems.

## Choose the right panel

Follow the user's requested races, maps, policies, seeds and limits. When they
do not specify scope, strongly prefer the complete role-appropriate panel over
an isolated duel or a hand-picked subset. The default full panel covers all five current
arena tactics and every map, plus campaign competence for the simulation-player
policies Mixed and Priority against the fixed Classic enemy. Campaign runs use
authored rosters and objectives. Do not treat Classic or Rush player campaign
failures as competence regressions; they are not the campaign simulation
players. Classic remains the fixed campaign opponent and all requested policies
remain in the arena panel.

Use recorded report metadata and composition summaries to establish coverage.
Inspect `rust/data/simulation/rosters.json` and `arena.json` only when preparing
a changed panel or resolving participation information missing from the report.
Campaigns use their authored rosters. Artwork lineups do not establish participation. Use `ai:schedule` to inspect the exact configurations
and counts when useful. State the panel and completed count before drawing
conclusions.

## Run and read the report first

A request to run the report means run fresh simulations and generate their report,
including after analyzer changes. Use the full runner below. Use only the analyzer
when the user explicitly asks to analyze or regenerate a particular saved run;
available results from an earlier review do not replace a requested simulation.

Use the full runner, which saves the native run and automatically analyzes it:

```sh
npm run ai:full -- --out artifacts/current
```

This builds the native harness when needed, runs the default complete panel,
and writes readable findings and full exports under `artifacts/current/analytics/`.
For an explicitly requested campaign-only panel, keep Mixed and Priority
as players and Classic as enemy:

```sh
npm run ai:combat -- --mode campaign --campaign-players mixed,priority \
  --campaign-enemies classic --seeds 10 --out artifacts/current
```

Both entry points build the native binary when needed. Use `ai:full` for the
standard panel and `ai:combat` to pass explicit scope or limits. Preserve each run
in a fresh ignored artifact directory, record its engine/data fingerprint, and
include an explicit `--data-dir` when simulating another checkout. The runner
defaults to the current catalog.

The runner prints the readable findings, campaign and stage tables, and arena
matrices automatically. Use that returned report first; do not issue extra reads
when it already answers the question. Open `analytics/findings.md` and
`analytics/summary.md` when the returned output was truncated or more detail is
needed. The report gives
campaign completion and stage outcomes, failure categories and recorded
reasons, named losses, objective progress, idle/repeated tails, and arena
comparisons. Complete tables are available as CSV exports. Do not start
by manually parsing outcomes, CSV/JSON, or gameplay journals for routine
questions that the report already answers. Preserve partial output on failure
and report completed matches; an incomplete run is not a complete panel.

If the user's question needs evidence the maintained report does not emit,
improve that analyzer and its focused tests, then regenerate the report from
the saved runs. Do not substitute an ad hoc scratch analyzer, manual raw-data
reading, or source-code speculation for a report gap. Once the report
establishes a precise unexplained case, inspect that case's phase journal and
relevant code to test a narrow causal explanation. Keep any trace extraction
targeted to the identified match and phase.

## Interpret results

For arena balance, show all recorded factions and requested maps and policy
pairings. Include recruitment, promotions, casualties and per-match outcomes;
retain caps in denominators and distinguish self-play from independent matches.
Use the report's equal-policy and map-policy tables to compare like policies,
and its matchup table for every opponent pairing. Label sample sizes and
unresolved results.

For campaign competence, compare Mixed and Priority against the same fixed
Classic enemy under matching campaigns, seeds, diplomacy, round limits and
recruit limits. Report completion rates with denominators, stage-level win/cap
outcomes, objective progress and terminal reasons. Separate a goal that never
progressed from a loss after partial progress, and distinguish defeat from a
round cap. Do not infer a causal bug from aggregate rates alone: use recorded
phase evidence and then targeted code inspection for the specific pattern.

Link `analytics/findings.md` and `analytics/summary.md`, along with the saved
run location and its metadata.
Retain raw journals and exports in ignored `artifacts/`; do not commit generated
simulation data.
