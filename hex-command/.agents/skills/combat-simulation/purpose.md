# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Classic is the live enemy and fixed comparison opponent.** Making it
stronger is not the purpose of simulation-player work.

The intended player strategies are infantry-only, fair roster use and
situational selection. Compare them against the same fixed enemy under matching
conditions. Fair and situational strategies should improve on the narrow
infantry baseline without relying on repeated selection of one dominant unit.
No acceptable numeric gap is established; report differences by race, map and
opponent.

## Current presets

| Tactic   | Current behavior and limits                                                         |
| -------- | ----------------------------------------------------------------------------------- |
| Classic  | Rotation recruitment and Classic planning; fixed enemy.                             |
| Rush     | Frontline recruitment and Full planning; not strictly infantry-only.                |
| Mixed    | Role-balanced recruitment and Full planning; not guaranteed fair use of every unit. |
| Priority | Mixed recruitment and Queued planning with mission orders.                          |
| Battler  | Mixed recruitment and Queued planning with arena orders; arena-only.                |

Priority and Battler test planning, not separate situational recruiters.
Verify [preset composition](../../../rust/src/ai/presets.rs),
[planner dispatch](../../../rust/src/ai/planner/mod.rs) and actual callers when
explaining behavior; a description or test-only helper is not execution evidence.

Campaign competence concerns **Mixed and Priority against Classic**, using
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
