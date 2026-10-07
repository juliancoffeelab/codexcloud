# Human magic branches

[L2/L3 healers and mages, both factions](preview.png).

- Healer L2: white robes and pale green crystal.
- High healer L3: white robes, broad gold stole and gold sun staff.
- Storm mage L2: dark blue robes and a lightning staff.
- Archmage L3: scarlet robes, gold hood and shoulder trim, larger blue crystal.

The finished sources preserve the current adept/mage anatomy, skin, grips, wooden shaft
and leather boots. All matrices are 48×48 with transparent backgrounds. They are
stored in `art/pixel/units/{healer,high-healer,storm-mage,archmage}/` and packed
into the unit atlas. Game unit definitions and existing playable art are unchanged.

Faction color occupies the hood and trailing cloak. Player symbols `F/f/Z` use
teal; enemy `R/r/Q` use red with dark shading. Body clothing, scarf, central sash,
belts and leather sleeve strips use the unit's white, blue or scarlet scheme;
boots remain leather. These separate material symbols allow more faction
palettes without recoloring the body or spell effects.

Each study contains both matrices, a shared palette, transparent PNGs and a
record of edits against its existing base. Regenerate with:

```sh
node art/pixel/reference/human-magic/create.cjs
```
