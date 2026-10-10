# Skeleton servant progression

For new edits, use [pinkbox enhancement](../../../../.agents/skills/pinkbox-enhancement/SKILL.md) and
[pixel cleaning](../../../../.agents/skills/pixel-cleaning/SKILL.md). This folder records the saved design.

The selected trees live in `../../lineups/skeletons.json`:

- Skeleton branches into Revenant/Dreadguard or Knife servant/Knife shade.
- Caster branches into Staff servant/Staff shade or Scythe servant/Scythe shade.
- Rider, archer and axeman each have one lane.
- The Lich commander progression remains separate.

L1 is grave-worn bone, rusty gear and dull burial cloth. L2 uses colder bone, cold iron and ash cloth. L3 uses bleached bone, blackened equipment and deeper existing cloak shadows. All seven L3 servants have dark graphite skulls; bone bodies and the Lich commanders retain their existing colours. Gold and polished silver no longer signal servant advancement. Faction base/shadow colours remain global; deeper shadows extend existing cloth folds, without new patterns or effects. Existing magic is muted rather than enlarged.

`changes.json` records every palette and matrix edit. No new servant anatomies were generated. Bodies, poses, hands, grips, heads and silhouettes remain unchanged, except the impact caster's oversized weapon head and connector above the grip, replaced by a plain wooden staff. The anatomy hash excludes only that small staff area and normalizes the two faction shades as cloth. Superseded staff-head material cleanup entries were removed from the old recipe.

Approved matrices are authoritative. Do not replay historical generation/tier recipes as a routine refresh. Gameplay stats, registrations and advancement rules are separate from these artwork trees.

```sh
python3 art/pixel/reference/skeleton-progression/validate.py
node art/pixel/reference/faction-expansion/process.cjs boards skeleton
```

The validator checks the chosen tree structure, protected anatomy hashes, faction isolation and wooden staff material. `validation.json` records its result. Both factions were inspected at nearest-neighbour scale.
