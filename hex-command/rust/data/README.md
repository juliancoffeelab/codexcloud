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
