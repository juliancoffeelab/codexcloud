# Dwarf and elf commander progression

Canonical matrices live in `art/pixel/units/`. The crowned L1 sources were
approved in [commander candidates](../commander-candidates/README.md).

- Forge lord → Forge lord → Forge king: leather smith apron; silver harness,
  bracer and upper greaves at L2; gilded crown/bracer and narrow shield-rim trim
  at L3. Wooden shield, silver anvil and forging hammer remain readable.
- Banner captain → Banner lord → Banner sovereign: leather and green equipment;
  shaded silver breastplate at L2; shaded gold breastplate/shoulder plate, taller
  leaf crown and narrow gold pennant piping at L3. Ivory cloth, spear and grip
  retain their materials.

Both lines are separate commanders in their race lineup, keeping five ordinary
starting roots. Gameplay registrations, statistics and abilities are unchanged.

Guarded fill and finish records accompany exact raw baselines (`.matrix` and
`.palette` files preserve original hash bytes). Restore those into a new scratch
folder as `sprite.txt` / `palette.json` when deliberately replaying an edit.
Do not replay a recipe merely to generate a preview. Canonical sources are
authoritative; `sources.json` guards their promoted hashes.

Use the project [enhancement skill](../../../../.agents/skills/pinkbox-enhancement/SKILL.md)
and [cleaning skill](../../../../.agents/skills/pixel-cleaning/SKILL.md). Both
detectors and faction previews were inspected. Findings inherited from the
accepted L1 are not claimed to be universally false positives or fully removed.

Run `python3 art/pixel/reference/commander-progression/check.py`,
`npm run art:build`, `npm run art:check` and `npm run style`.
