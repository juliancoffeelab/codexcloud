# Purpose and policy roles

Read this for AI policy reviews and when interpreting simulation results.
[Army design](design.md) defines faction identities and unit-role tradeoffs.

Simulations test roster balance and whether simulation players can finish
objectives using the browser's engine rules. They also measure trial execution
cost. **Balanced is the default arena comparison opponent. Classic remains the live
enemy and the campaign competence baseline.** Making it stronger is not the purpose of simulation-player work.

For faction and unit balance, use matching Situational policies with the normal
leader-only arena starts, equal total budgets, recruitment and earned promotions. Inspect each
opposing faction and terrain separately. Collect battlefield
presence and actual contributions (`--unit-usage`, or the relevant presence/value
telemetry) for balance investigations. Lead composition claims with living
unit-turns, then compare recruitment flow with presence and casualties. Purchase
shares can reflect replacement churn; they do not measure the army on the field,
survival or unit strength. Purchase/presence ratios do not identify survival
without cohort evidence. Keep opponents, terrain and policy scopes aligned.
Current presence exports pool opponents: restrict runs to the two relevant
armies, or use an equivalent maintained scoped census, for opponent-specific
composition claims. Do not pair pooled presence with exact-opponent purchases. Isolated advanced-level rosters are
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

Campaign competence concerns **Balanced, Rotated and Situational against Classic**, using
authored rosters and objectives. Classic/Rush campaign failures do not establish
simulation-player competence regressions. Completion, required-actor survival
and progression matter; speed alone does not establish correctness or balance.
A round cap is unresolved. An unsafe forecast is not proof of unavoidable death.

Preserve Classic as the fixed campaign enemy when improving simulation-player
strategies. Compare deliberate behavior changes separately from maintenance
cleanup.
