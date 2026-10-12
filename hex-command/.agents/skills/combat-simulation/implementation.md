# Combat simulation implementation

The native Rust harness owns scheduling, battle execution, campaign transitions
and result recording. The Python runner invokes that harness and then analyzes
the saved report. Reporting uses only the Python standard library.

See [usage.md](usage.md) for commands and [SKILL.md](SKILL.md) for the agent workflow.

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

## Arena coverage and victory

Maps are rotationally paired radius-four hexagons: open, wooded, river crossings
and islands. Islands has two home islands and three separate objective islands:
the central fort and the northern/southern villages. Bridges link the home islands
to the fort; shallow crossings reach the village islands and give alternative
approaches to the fort, including the hex east of the northern village and its
southern counterpart. Open channels give merfolk routes around those crossings.
Seeds vary paired terrain, equal total budgets and small policy tie breakers. Water arenas
retain the river identity for classic's routing. Each side gets its selected race's leader,
two melee starters and its own recruitment pool. Human, woodland, undead and coastal
rosters can face each other; same-race baselines remain included. Campaign pools
are authored independently of arena pools. Each side receives the same total
starting budget: the authored `startingBudget` of 36 gold plus a paired seed-dependent
0, 3 or 6 gold. Its leader and two starters are charged at their unit prices, and
the remainder is spendable cash. Positions and per-phase recruit limits are paired,
while race-specific stats differ.

An entrant is `(race, policy)`. Every unordered entrant pair is scheduled in both
seats and both initiatives; identical entrants run both initiatives once. Four races
and three policies give 12 entrants, 288 games per terrain, **1,152 games per seed**
across the four terrains. Each round contains both phases regardless of initiative.

Victory means defeating the opposing leader, or owning the central fort and both
central villages through four opposing turns, or six on islands to allow ground
troops time to reach the objectives. Only completed opposing turns count; losing
any of the three objectives resets the hold. Optional ruins record visits without
resources. Territory victory belongs to this testing scenario, not ordinary battles.

Campaigns use the actual authored player and enemy rosters, maps, objectives and
waves. They begin in chapter one, earn carryover and recall up to four eligible
survivors using the same ranking. No retries. Four campaigns, three player policies,
three enemy policies and two diplomacy settings give **72 attempts per seed**.
Campaign seeds vary policy tie breaks, not authored maps; deterministic classic
choices can repeat across seeds.

## Recorded data and limits

Reports retain every configuration, outcome, reason, unresolved cap,
phase command, combat damage/retaliation, casualty, recruit, promotion and initial/
final state. Phase records include beginning/end rosters with identities, positions,
HP, XP, action flags and leader mode; gold, ownership, goal progress and full campaign
state retain mediation escalation and named deaths. Casualties include troops
recruited and killed during the same phase. Arena goal ownership/holding and optional
visits are recorded. Endpoint hashes and a SHA-256 source fingerprint permit exact replay.

Aggregates compare race/policy entrants, ordered race matchups and terrain; self-play
reports seat and initiative diagnostics. Campaign aggregates retain campaign,
player/enemy policies, diplomacy setting, completion, defeat, unresolved caps,
escalation and chapter reach. Recruitment, losses and promotions are totaled by
race/policy entrant. Loss totals include starting and promoted troops, so they are
not purchase survival percentages.

Bootstrap intervals resample complete seed blocks, preserving seat/initiative/map
correlations. They describe these scenarios and policies, not optimal play or human
difficulty. More seeds reduce sampling uncertainty, not policy weakness. Partial
checkpoints retain completed configurations; worker failures are errors, not defeats.

The seed-block intervals belong to the native report's aggregate rows. The
Python analyzer exports outcome counts and win rates; it does not calculate new
confidence intervals.

## Full gameplay comparison

`scripts/convert-simulation-dump.py` reads complete detail journals using an
explicit Arrow schema. Supply every journal for a sharded report. The comparator
uses a one-to-one Polars join on `(kind, config)` and preserves array order within
matches. Missing or duplicate identities fail comparison. It excludes `stateHash`,
`elapsedMs`, `elapsedSeconds` and `detail` at match and mission level, and records
wall time, CPU time and peak memory. Convert each dump once and reuse its Arrow file.

See [AI policy implementation](../../../rust/src/ai/README.md) for planning
behavior and [authored game data](../../../rust/data/README.md) for current stats.
