---
name: combat-simulation
description: Use for AI policy work, simulation harness or analyzer changes, combat simulations, roster balance or campaign competence analysis.
---

# Combat simulation

- Keep Classic's decision flow, escort evaluation, and route-selection heuristics
  owned by Classic. Queued/Full search heuristics must not be applied to Classic
  through shared preparation or mode flags. Share mechanical facts and explicit
  order/command handling rather than implicit heuristic weights.

Run combat simulations when needed to explore or validate balance without waiting
for separate permission. Experiments stay separate from the playable catalog;
use ignored data copies and `--data-dir`. Requests to prototype, render, simulate
or evaluate content do not authorize promoting it into the playable catalog.

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
   for build problems. For maintenance and bug fixes, use a matching baseline to
   check for unintended outcome changes. Investigate differences and distinguish
   expected effects of the fix from regressions; disclose missing comparisons.
4. Read the printed findings. If output is truncated, consult the run’s
   analytics/findings.md and summary.md. Separate observations from proposed
   causes, and use recorded terminal reasons, goal progress and casualties.
5. Match delivery to the task using [reporting](references/reporting.md).
   For maintenance and bug fixes, keep unchanged simulation results internal and
   briefly confirm validation passed. Report meaningful differences, regressions
   or validation gaps; give counts and details only when they help assess those
   findings. Do not automatically include balance tables or analysis.
   Requested comparisons need their results, denominators and limitations for
   every requested workload. Report files remain internal unless asked for.

For maintenance, complete the requested fix and report its validation; a full
simulation report is not an additional deliverable. For requested comparisons,
a successful run alone is not delivery: report the actual comparison and its
limitations. Missing evidence bounds the claim or requires a new run.
