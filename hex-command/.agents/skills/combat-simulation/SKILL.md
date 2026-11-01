---
name: combat-simulation
description: Use for AI policy work, combat simulations, roster balance or campaign competence analysis.
---

# Combat simulation

Work from the game root. For policy or balance work, read
[purpose](purpose.md) for comparison goals and [army design](design.md) for
faction roles. For coefficient tuning, read [recruitment](recruitment.md).

1. Select the requested workload and a matching baseline when making a regression
   or improvement claim. Use [running](references/running.md) for schedules,
   custom tactics, catalog copies and saved-run analysis.
2. Collect evidence adequate for the claim. Balance work requires --unit-usage or
   equivalent presence and contribution telemetry. Read
   [metrics](references/metrics.md) before interpreting unit composition, damage,
   support or casualty value; use [replays](references/replays.md) for interventions.
3. Run the selected scope. After changing the harness, analyzer or AI behavior,
   run fresh simulations of the affected workload; use the full panel when the
   affected scope is unclear. Use [rust-toolchain](../rust-toolchain/SKILL.md)
   for build problems.
4. Read the printed findings. If output is truncated, consult the run’s
   analytics/findings.md and summary.md. Separate observations from proposed
   causes, and use recorded terminal reasons, goal progress and casualties.
5. Present the results using [reporting](references/reporting.md), covering every
   requested workload. State completed/expected coverage and limits, including
   failures, caps and missing telemetry. Report files remain internal unless asked for.

The task is complete when the requested comparisons are reported with their
actual denominators, evidence scope and limitations. A successful test or run
alone is not delivery. Missing evidence bounds the claim or requires a new run.
