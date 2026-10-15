# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Balanced is the default arena comparison opponent. Classic remains the live
enemy and the campaign competence baseline.** Making it stronger is not the purpose of simulation-player work.

For faction and unit balance, use matching Situational policies with the normal
starting rosters, budgets, recruitment and earned promotions. Inspect each
opposing faction and terrain separately. Isolated advanced-level rosters are
supplementary diagnostics, not validation of normal progression. Classic can
saturate before a roster is balanced. Pooling recruiters or averaging easy and
hard faction matchups can hide failures; a pooled result alone does not establish
a terrain advantage or a balanced head-to-head matchup. Validate on additional
seeds after tuning, and compare the full matchup matrix and authored campaign
completion against the same baseline conditions.

Aim for roughly 70% Woodland wins in forests and 80% Merfolk wins on water,
with viable armies outside those biomes. Woodland should beat Merfolk on dry
plains. Humans and Skeletons should be near
even on plains; some stalemates between them are expected. Do not weaken their
parity merely to eliminate round caps. Preserve meaningful cavalry use and
lore-specific campaign recruitment pools when tuning numeric balance.

The intended player strategies are infantry-only, fair roster use and
situational selection. Compare them against the same fixed enemy under matching
conditions. Fair and situational strategies should improve on the narrow
infantry baseline without relying on repeated selection of one dominant unit.
No acceptable numeric gap is established; report differences by race, map and
opponent.

Situational is an adaptive recruiter, selectable with `--with=situational,-,queued`.
It scores recruit candidates before affordability by engine combat exchanges,
effective HP, terrain routes and movement, marginal healing and thorn support.
It can save income for a more useful expensive recruit. A soft penalty for repeated families encourages
variety without fixed role quotas. Campaigner recruitment requirements take priority;
reserved lanes remain clear. It adds no recruitment cap.

## Current presets

| Tactic      | Current behavior and limits                                                                  |
| ----------- | -------------------------------------------------------------------------------------------- |
| Classic     | Fixed Classic recruitment and Classic planning; fixed enemy, capped at 2 recruits per phase. |
| Rush        | Frontline recruitment and Queued planning; not strictly infantry-only.                       |
| Balanced    | Role-balanced recruitment with seeded random choices within each role and Queued planning.   |
| Rotated     | Rotation recruitment and Queued planning, without an additional recruitment cap.             |
| Situational | Adaptive recruitment and Queued planning, without an additional recruitment cap.             |

Queued is the default planner for simulation-player presets. Full remains
selectable through independent block composition.
Verify [preset composition](../../../rust/src/ai/presets.rs),
[planner dispatch](../../../rust/src/ai/planner/mod.rs) and actual callers when
explaining behavior; a description or test-only helper is not execution evidence.

Rotation cycles through the least represented roster families, including expensive
mounted units, and waits when its next recruit is not yet affordable. Authored campaign recruitment pools remain separate and lore-specific.

Campaign competence concerns **Balanced, Rotated and Situational against Classic**, using
authored rosters and objectives. Classic/Rush campaign failures do not establish
simulation-player competence regressions. Completion, required-actor survival
and progression matter; speed alone does not establish correctness or balance.
A round cap is unresolved. An unsafe forecast is not proof of unavoidable death.

## Change boundaries

Before AI architecture changes, read [AI contracts](../../../docs/development.md#ai-contracts).
Keep fixed Recruiter/Campaigner/Planner composition, explicit orders, restricted
battle views and one validated planning draft. Only Campaigner interprets
authored mission kinds. Planners may use shared search helpers, but must not
switch to another planner.

Preserve the fixed Classic enemy behavior when changing simulation-player
recruitment strategies. Implementing the intended infantry/fair/
situational strategies is a separate semantic change, not part of a refactor.
