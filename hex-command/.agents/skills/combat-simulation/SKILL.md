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
5. Match delivery to the task using [reporting](references/reporting.md).
   Maintenance and bug fixes need a concise validation summary, including
   completed/expected runs, relevant failures or caps, and verification of the
   changed behavior. Do not automatically include balance tables or analysis.
   Requested comparisons need their results, denominators and limitations for
   every requested workload. Report files remain internal unless asked for.

For maintenance, complete the requested fix and report its validation; a full
simulation report is not an additional deliverable. For requested comparisons,
a successful run alone is not delivery: report the actual comparison and its
limitations. Missing evidence bounds the claim or requires a new run.
