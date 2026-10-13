---
name: combat-simulation
description: Run role-appropriate Hex Command combat simulations and read the maintained analyzer's complete human-readable reports before diagnosing balance.
---

# Combat simulation

Work from the `hex-command/` project root. Read [usage.md](usage.md) for
running and reviewing batches. Read [implementation.md](implementation.md) when
changing or debugging the runner or analyzer. Use the [Rust toolchain
skill](../rust-toolchain/SKILL.md) for build problems.

## Architecture and change boundaries

Read [IDEA.md](../../../IDEA.md) before changing AI architecture. It defines fixed
Recruiter/Campaigner/Planner composition, explicit orders, restricted battle
views and a shared validated planning draft. Only Campaigner interprets authored
mission kinds. A selected planner may call shared search helpers; it must not
switch to a different planner.

Structural changes must preserve the existing Rotation, Frontline and Balanced
recruitment semantics unless the user explicitly requests a recruitment change.
The infantry/fair/situational strategies in the design are a separate semantic
migration; do not implement them implicitly during a module refactor.

## Purpose and policy roles

These simulations test roster balance and whether simulation players can finish
objectives using the same engine rules as the browser. They also measure the
cost of running those trials. Classic is the live enemy and fixed comparison
opponent; making it stronger is not the purpose of simulation-player work.

The intended player strategies are infantry-only, fair roster use and situational
selection. Compare
their success against the same fixed enemy under matching conditions. Fair roster
use and situational selection should improve on the narrow infantry baseline,
without a large advantage from repeatedly choosing one unit. No numeric threshold
for the acceptable gap is established; report differences by map and opponent.

Current presets are imperfect proxies for those intended roles. Rush recruits
frontline units rather than strictly infantry; Mixed balances role shares rather
than guaranteeing fair use of every unit. Priority and Battler reuse Mixed
recruitment and experiment with planning; they are not separate situational
recruiters. Battler is arena-only. Check [preset composition](../../../rust/src/ai/presets.rs)
and the relevant implementation when explaining current policy behavior. State
those limits when interpreting strategy comparisons. Documentation descriptions
are not evidence that a planner branch executes in production; inspect dispatch
and callers, including whether a helper is test-only.

Campaign completion, required-actor survival and progression establish whether
these players are useful for balance trials. Faster simulations must preserve
those behaviors; speed alone does not establish correctness or balance. A round
cap is an unresolved attempt, not a victory, and an unsafe forecast is not proof
that an actor's death was unavoidable.

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

The runner prints coverage, compact race-by-map and campaign tables, then stage
diagnostics and detailed arena matrices automatically. Use that returned report
first; do not issue extra reads when it already answers the question. Read
`analytics/findings.md` and `analytics/summary.md` internally when the returned
output was truncated or more detail is needed. The report gives
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

A full report must summarize both arenas and campaigns. Follow the report's
opening order: completed coverage, short race-by-map results against Classic,
then campaign completion, followed by the important detailed findings. Summarize
both sections even when one contains more detail. For a scoped request, report
that scope and say when a requested comparison is not recorded.

Keep player policies separate in race-by-map comparisons. Name the fixed enemy,
show wins/all games and caps, and state when opponent races or diplomacy modes
are pooled. Arena completion means a decisive result for either side; it is not
the race's win rate. Campaign completion means finishing the whole campaign;
stage victories use only attempts that reached that stage. Do not mix these rates.

Connect notable gaps to recorded recruitment, promotions, casualties, goal progress
and terminal reasons. Distinguish an observed failure from its proposed cause.
Only diagnose a causal bug after checking the relevant recorded phases and code.
If a baseline exists, require matching panel conditions before calling a change
a regression. Without a baseline, describe current results rather than a change.

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

Present the results directly in chat: completed coverage, compact race-by-map
and campaign tables, then the important findings and limitations. Users working
in cloud environments cannot open workspace files. Keep report files, metadata,
raw journals and exports as internal records in ignored `artifacts/`; do not
use local file links as the answer or require the user to open a file. Provide
files only when explicitly requested and through a delivery mechanism the user
can access. Do not commit generated simulation data.
