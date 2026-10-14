# Combat simulation implementation

Read when changing the harness or analyzer. Usage is in [SKILL.md](SKILL.md);
policy intent is in [purpose.md](purpose.md).

## Component responsibilities

- [CLI](../../../rust/src/simulation/cli.rs): arguments, batch execution,
  reporting and replay.
- [Schedule](../../../rust/src/simulation/schedule.rs): deterministic
  configurations and expected match counts.
- [Matches](../../../rust/src/simulation/matches.rs): execution through the
  shared game engine and selected AI components.
- [Native report](../../../rust/src/simulation/report.rs): aggregate results.
- [Analyzer](scripts/report-combat.py): findings and exports from saved runs.

Keep CLI options, defaults, encoding schemas and profiling details in their
owning code rather than duplicating them here.

## Stable contracts

Simulations use the game engine; the analyzer reads recorded outcomes rather
than rerunning battles. Preserve selected AI component composition.
Scheduling is independent of worker completion order. Mirrored participant
results describe the same physical match, not independent evidence.
Recorded failures establish outcomes, not tactical counterfactuals or root causes.

## Journals and recovery

[Records](../../../rust/src/simulation/records.rs) and
[record types](../../../rust/src/simulation/types.rs) own lossless gameplay
records and validation. [Checkpoint recovery](../../../rust/src/simulation/checkpoint.rs)
owns incomplete-run recovery. Preserve completed raw results when reporting fails.
[Exact comparison](../../../scripts/compare-simulation-reports.py) owns the
rules for comparing saved gameplay.
