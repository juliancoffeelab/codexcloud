# Artwork handoff for the next session

Generated pink-box PNGs have been removed after matrix approval. References
below to `templates/` describe the historical generation process. For new work,
use [the project pinkbox-generation skill](../../../../.agents/skills/pinkbox-generation/SKILL.md)
from the project root. Approved `units/*/sprite.txt` matrices are authoritative;
saved raw sources and rebuild recipes remain available for historical inspection.
Do not replay construction recipes as routine artwork verification.

Branch: `art/human-unit-trees`. Pull request: [#10](https://github.com/juliancoffeelab/codexcloud/pull/10).

## Subsequent zombie lineup selection

The user selected Thin walker as L1, branching into Thin fighter → Cleaver fighter,
Thin caster → Plague carrier, and Shambler → Ghoul at L2/L3. The Corruption mage is
the separate commander. Those eight reviewed base designs remain active; the
other 28 humanoid zombie variants are archived in
[../zombies/](../zombies/README.md). See the [lineup](../../lineups/zombies.json)
for current artwork tiers. The expansion counts and family sheets below describe
the original generation studies, not the current active zombie roster.

## What is saved

Subsequent dwarf work regenerated the crossbow and pickaxe using dwarf-sized pink
templates, added a pistol goat rider and axe-and-pick infantry, and refined the
helmets and fitted torso armour across the dwarf roster. The current manifest has
63 generated base poses (38 humanoid/mounted, 25 beasts). The original expansion
counts below are retained as historical context. New dwarf pose masks live in
`dwarf-renewed.cjs`; reviewed material overlays live in `../cleanup/dwarf.json`.

The previous commit contains 25 approved human designs through L3 in both faction
colors. This expansion adds 61 generated base poses, each with L1–L3 artwork, plus
nine L3 variants built from existing L2 units: **192 additional designs / 384
faction sprites**. All finished matrices, palettes, component notes, change logs
and native PNGs live in `art/pixel/units/`; the catalog registers the artwork and
the public unit atlas includes it. Existing catalog entries were preserved.

New base poses comprise:

- Skeletons: dual daggers, blunt impact staff, scythe caster, two-handed axe,
  skeletal mounted spearman. Caster skulls use warrior proportions rather than
  the oversized old lich head. There is no skeleton crossbow.
- Zombies: shambler, cleaver/shield, ghoul, plague carrier, corruption mage,
  four weaklings and three thin adult skeleton-sized alternatives.
- Elves: spear/shield, two-handed curved sword, bow, dual knives, branch staff,
  deer-mounted archer.
- Dwarves: axe/shield, hammer, pickaxe, poleaxe, crossbow, arquebus, rune caster,
  goat-mounted **hammerman**, not lancer.
- Woodland: fox spear/shield; existing wolf, boar and caster poses receive L3
  variants via `reuse.cjs`.
- Merfolk: two-handed trident, net/spear, mace/shield, and a new drawn-back harpoon
  thrower with two spare harpoons. Existing guard, hunter and sage receive L3.
- Beasts: bear, lynx, stag, giant spider/tortoise/snake/raven/owl/wasp/bat/crab,
  crocodile, shark, ray, octopus, sea serpent, skeletal hound/horse, corpse
  vulture, zombie bear, gryphon, dragon, basilisk, treant and bone dragon.

These anatomies were requested by the user, but the newly generated designs have
not all received their individual art-direction approval. They are **artwork
prototypes**, not new playable unit definitions. No new stats, recruitment,
advancement rules, campaign content or simulations were added. No deployment or
merge was performed as part of this handoff.

## Where to look first

Read project `AGENTS.md`, `art/pixel/WORKFLOWS.md`, then [this workflow's README](README.md).

- [Humanoid overview](overview-humanoids.png): 38 numbered base poses.
- [Beast overview](overview-beasts.png): 25 numbered base poses.
- [Local review gallery](gallery.html): L1–L3, teal/red selector, raw generations,
  component notes, and numbered source/matrix overlays. Open locally or serve
  the project directory; GitHub displays its source rather than running it.
- `<race>-player.png` / `<race>-enemy.png`: one family per row, L1/L2/L3 left to right.
- [Existing L2 → L3](existing-l3-player.png), also available for the red faction.
- `manifest.json`: exact pose list, race direction, three selected anchors,
  palettes, frame sizes and completed status.
- `art/pixel/reference/faction-anatomies.md`: requested anatomy/weapon list.

`art/pixel/reference/human-lineup/full.png` and `full-red.png` show the prior
approved human trees. `art/pixel/reference/human-branches/` preserves an earlier
30-sprite exploratory transcription sheet. It is historical reference material,
**not the canonical or approved human lineup**; do not copy it over the units.

## How the artwork was made

Three real canonical sprites are enlarged without smoothing into three quadrants
of an opaque pink template; the fourth quadrant is empty. The whole equipped pose
is generated together, including anatomy, weapon and hands. Each quadrant has a
64×64 logical pixel budget; foot poses are requested within the central 48px.
Returned image dimensions vary (often 1254×1254), so extraction uses the actual
bottom-right quadrant dimensions, not a fixed image size.

Saved unchanged imagegen outputs are in `sources/`; templates are in `templates/`.
Most exact prompts are saved beside the source. The first five skeleton prompts
were not captured verbatim: their `.recipe.md` files are explicitly reconstructed
instructions, not transcripts. `rejected/` preserves failed reference attempts.

`process.cjs` extracts and quantizes to explicit flat RGBA palettes. `clean.cjs`
and `clean-beasts.cjs` assign actual pose materials and remove texture artifacts.
Their masks belong to the **specific saved pose**, not arbitrary future rerolls.
`upgrades.cjs` adds local equipment changes; `reuse.cjs` extends existing L2 poses.
Three weakling zombie branches use thin-adult and bulky source anatomies for
their higher levels; the other variants largely reuse geometry with equipment
and color changes. Most beast tiers are natural coat/scale recolors, not bigger
anatomies. This distinction should be considered during future art review.

Never replace only a PNG. Edit canonical matrices and reflect the edit in its
owning recipe before rebuilding, or the recipe will overwrite that manual edit.
The new sources may retain 64px frames when a 48px crop would clip limbs or weapons.
Paired numbered grids share **zero-based native matrix coordinates**, accounting
for any crop origin. Keep alpha hard (0 or 255), intentional pupils/mouths/rib
gaps, attached claws and coherent shadows. Random white/brown/gray pixels count
as stray too; checking only green or off-palette pixels is insufficient.

The fox shield was rerolled with a solid face, connected rim and boss. Crocodile,
octopus, spider, crab and basilisk were rerolled for limb readability. The final
basilisk uses a direct top view with three exposed feet on each side. Review
both numbered source and matrix overlays if revisiting these designs.

## Commands and verification

From `hex-command/`:

```sh
npm run art:expansion
# Or selected saved poses:
npm run art:expansion -- skeleton-flanker giant-spider
```

This rebuilds from saved outputs without calling imagegen: extraction, semantic
cleanup, tiers, existing L3 recipes, review sheets/grids and ordinary atlas packing.
For a matrix-only edit, use the renderer and `npm run art:build`; do not rerun a
construction recipe until that recipe includes the edit.

Completed verification before handoff: `npm run art:build`, `npm run art:check`,
`npm run style`, and all **12** tests in `tests/art.test.ts`. An additional audit
checked all 192 added designs for valid palette symbols, hard alpha and shared
faction geometry; all 122 source/matrix overlays exist. No fresh independent
reviewer was used for this expansion; visual review was performed in this session.
Build caches and scratch logs in `tmp/` are not required to resume the task.

## Resume scope

Start with the saved overview/tier sheets and the user's next feedback. Further
art review should focus on anatomy, consistent detail against older units, and
whether each tier is distinct enough. Do not assume every generated variant is
approved just because it exists. Gameplay integration is a separate next task;
wait for the user's direction before editing definitions or running simulations.
