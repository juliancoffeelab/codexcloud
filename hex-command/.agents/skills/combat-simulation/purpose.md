# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.
[Army design](design.md) defines faction identities and unit-role tradeoffs.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Balanced is the default arena comparison opponent. Classic remains the live
enemy and the campaign competence baseline.** Making it stronger is not the purpose of simulation-player work.

Use [recorded metrics](references/metrics.md) for evidence definitions and scope
limitations, and [reporting](references/reporting.md) for output conventions.
Isolated advanced-level rosters are supplementary diagnostics; validate normal
progression on held-out seeds against matching baseline conditions.

Use the [army-design goals](design.md#terrain-and-overall-strength) for biome
targets, faction parity, cavalry roles and lore-specific recruitment limits.

Use Rotation and Rush as simple baselines. Focus recruitment-intelligence and
numeric-balance evaluation on Balanced, Situational and Predictor.

The intended player strategies are infantry-only, fair roster use and
adaptive selection. Situational evaluates observed enemies; Predictor additionally
anticipates legal future recruits. Both use distance-free combat and support
valuation, with movement rewarded through village capture. Balanced represents fair roster use. Both adaptive recruiters should
maximize winning strength, including repeated selection of a successful unit.
Do not weaken either adaptive recruiter or impose roster diversity to conceal a strong unit.
Concentrated field presence is balance evidence to investigate, not a reason to
reject a winning recruitment strategy. Compare strategies against the same fixed
enemy under matching conditions and validate on separate seeds. No acceptable
numeric gap is established; report differences by race, map and opponent.

Campaign competence concerns **Balanced, Rotated, Situational and Predictor against Classic**, using
authored rosters and objectives. Classic/Rush campaign failures do not establish
simulation-player competence regressions. Completion, required-actor survival
and progression matter; speed alone does not establish correctness or balance.
A round cap is unresolved. An unsafe forecast is not proof of unavoidable death.

Preserve Classic as the fixed campaign enemy when improving simulation-player
strategies. Compare deliberate behavior changes separately from maintenance
cleanup.
