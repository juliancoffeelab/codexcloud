# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Classic is the live enemy and the default policy/campaign comparison
opponent.** Making it stronger is not the purpose of simulation-player work.

For faction and unit balance, use matching Situational policies and inspect each
opposing faction, terrain and rank separately. Classic can saturate before a
roster is balanced. Pooling recruiters or averaging easy and hard faction
matchups can hide failures; a pooled result alone does not establish a terrain
advantage or a balanced head-to-head matchup. Validate on additional seeds after
tuning, and retain campaign checks against Classic with authored recruitment pools.

The intended player strategies are infantry-only, fair roster use and
situational selection. Compare them against the same fixed enemy under matching
conditions. Fair and situational strategies should improve on the narrow
infantry baseline without relying on repeated selection of one dominant unit.
No acceptable numeric gap is established; report differences by race, map and
opponent.

Situational is an adaptive recruiter, selectable with `--with=situational,-,queued`.
It scores affordable recruits by enemy matchups, terrain routes to attack positions,
survivability, local healing demand and adjacent thorn support. A soft penalty for repeated families encourages
variety without fixed role quotas. Campaigner recruitment requirements take priority;
reserved lanes remain clear. It adds no recruitment cap.

## Current presets

| Tactic      | Current behavior and limits                                                              |
| ----------- | ---------------------------------------------------------------------------------------- |
| Classic     | Rotation recruitment and Classic planning; fixed enemy, capped at 2 recruits per phase.  |
| Rush        | Frontline recruitment and Queued planning; not strictly infantry-only.                   |
| Balanced    | Role-balanced recruitment and Queued planning; repeated families receive a soft penalty. |
| Rotated     | Rotation recruitment and Queued planning, without an additional recruitment cap.         |
| Situational | Adaptive recruitment and Queued planning, without an additional recruitment cap.         |

Queued is the default planner for simulation-player presets. Full remains
selectable through independent block composition.
Verify [preset composition](../../../rust/src/ai/presets.rs),
[planner dispatch](../../../rust/src/ai/planner/mod.rs) and actual callers when
explaining behavior; a description or test-only helper is not execution evidence.

Rotation traverses the full explicit arena roster, including expensive mounted
units. Authored campaign recruitment pools remain separate and lore-specific.

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

Preserve Rotation, Frontline and Balanced recruitment semantics unless a
recruitment change is requested. Implementing the intended infantry/fair/
situational strategies is a separate semantic change, not part of a refactor.
