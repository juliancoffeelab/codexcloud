# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.
[Army design](design.md) defines faction identities and unit-role tradeoffs.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Balanced is the default arena comparison opponent. Classic remains the live
enemy and the campaign competence baseline.** Making it stronger is not the purpose of simulation-player work.

For faction and unit balance, use matching Situational policies with the normal
leader-only arena starts, equal total budgets, recruitment and earned promotions. Inspect each
opposing faction and terrain separately. Isolated advanced-level rosters are
supplementary diagnostics, not validation of normal progression. Classic can
saturate before a roster is balanced. Pooling recruiters or averaging easy and
hard faction matchups can hide failures; a pooled result alone does not establish
a terrain advantage or a balanced head-to-head matchup. Validate on additional
seeds after tuning, and compare the full matchup matrix and authored campaign
completion against the same baseline conditions.

Aim for roughly 70% Woodland wins in forests and 80% Merfolk wins on water,
with viable armies outside those biomes. Woodland should beat Merfolk on dry
plains. Humans and Skeletons should be roughly equally useful overall across
opponents and maps, particularly under Balanced and Situational. A fair direct
matchup is desirable but not required; never infer overall parity from it alone.
Some stalemates between them are expected. Rotation and Rush are simple baselines,
not the primary targets for recruitment intelligence or numeric balance. Preserve meaningful cavalry use and
lore-specific campaign recruitment pools when tuning numeric balance. Audit
movement above three against the lore; terrain access and movement costs also
contribute to mobility.

The intended player strategies are infantry-only, fair roster use and
situational selection. Compare them against the same fixed enemy under matching
conditions. Fair and situational strategies should improve on the narrow
infantry baseline without relying on repeated selection of one dominant unit.
No acceptable numeric gap is established; report differences by race, map and
opponent.

Situational evaluates combat exchanges, effective HP, routes, movement and
marginal support, and can save for expensive recruits. Campaign recruitment
requirements take priority. Scoring formulas and coefficients belong in
[recruiter evaluation](../../../rust/src/ai/recruiter/evaluation.rs);
[promotion selection](../../../rust/src/ai/recruiter/promotion.rs) uses the same
assessment. Support coverage is an estimate, distinct from observed runtime
support effects.

## Current presets

| Tactic      | Current behavior and limits                                                                             |
| ----------- | ------------------------------------------------------------------------------------------------------- |
| Classic     | Fixed Classic recruitment and Classic planning; fixed enemy, capped at 2 recruits per phase.            |
| Rush        | Frontline recruitment and Queued planning; not strictly infantry-only.                                  |
| Balanced    | Seeded random choice among all underfilled roles, then a random unit within that role; Queued planning. |
| Rotated     | Rotation recruitment and Queued planning, without an additional recruitment cap.                        |
| Situational | Adaptive recruitment and Queued planning, without an additional recruitment cap.                        |

Queued is the default planner for simulation-player presets. Full remains
selectable through independent block composition.
Verify [preset composition](../../../rust/src/ai/presets.rs),
[planner dispatch](../../../rust/src/ai/planner/mod.rs) and actual callers when
explaining behavior; a description or test-only helper is not execution evidence.

Arenas begin with leaders only. Both sides receive the same seeded total budget,
minus their leader's cost, and recruit their opening troops through their policy.
Campaign starting units remain authored.

Rotation fills the least represented role, breaking ties in flank, infantry,
ranged, healer/support order and skipping unavailable roles. Within that role it
fills the least represented family, including expensive mounted units, and waits
when its next recruit is not yet affordable. Authored campaign recruitment pools
remain separate and lore-specific.

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
