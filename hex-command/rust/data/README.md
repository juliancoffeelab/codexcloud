# Authored content

Edit game rules, units, scenarios and campaigns as authored content. Validate
changes with `npm run content:check`; use combat simulations to assess balance.
Keep authored content and generated output separate.

A lineup is a faction. Enabled factions should have their complete progression
available in gameplay. Campaign recruitment remains a deliberate, lore-specific
selection from that roster.

Follow [army design](../../.agents/skills/combat-simulation/design.md) for faction
identities, unit roles and equipment tradeoffs. Keep campaign rosters, budgets
and objectives consistent with their stories. New units do not automatically
belong in existing campaigns.

Use descriptive names and stable identifiers. Keep descriptions focused on role
and character, avoiding duplicated numeric stats. Preserve intended progression
and campaign order when editing content. Check references, placements and
starting health through content validation.

See the [development guide](../../docs/development.md) for verification and the
[combat-simulation skill](../../.agents/skills/combat-simulation/SKILL.md) for
balance evaluation.

## Strike rolls

Each attack rolls `count` independent integer dice. A strike's raw damage is
uniform from `damage - spread` through `damage + spread`, inclusive. `spread`
defaults to zero for fixed attacks and must be between zero and `damage`.
Current physical ranged attacks use `spread = damage`; melee attacks use roughly
one quarter of base damage, and controlled magical projectiles use a spread of
one. These are authored values, so individual weapons can vary.

Apply armour and terrain to each raw roll and clamp each result to zero. Zero
raw rolls still receive the normal signed modifiers. The attacker strikes first,
then the defender replies with its best legal weapon of the same category.
Alternate individual strikes, skipping a side that has exhausted its weapon's
count, and stop immediately when either unit dies. After each strike, drain heals
from actual HP removed, rounded down and capped at the wielder's maximum HP.
Defensive thorns then reflect that initiating melee strike, even when its roll
causes no damage or kills the defender. Unexecuted strikes neither roll nor reflect.

Symmetric raw rolls preserve the unmodified mean. Clamping at zero increases the
mean relative to fixed damage against strong armour or cover. Previews and AI
use an exact alternating exchange forecast, including interrupted strikes, drain,
reflection, kill odds and conditional retaliation;
protected actors use conservative incoming damage. Combat RNG state is saved,
and inspecting or planning an attack does not consume live rolls. Policies
reassess after each resolved attack rather than assuming a whole turn's kills.

Simulation runs write `damage-tables.csv` beside their report. It contains every
weapon's exact total range, expectation and kill odds for combined incoming
modifiers of 0, -2, -4 and +1, without HP caps. Combat records retain individual
raw rolls, modified rolls and signed mitigation stages for actual exchanges.
