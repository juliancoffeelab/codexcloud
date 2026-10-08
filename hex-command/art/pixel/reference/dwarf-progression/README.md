# Dwarf armour progression

[Teal branch grid](lineup-player.png) · [Red branch grid](lineup-enemy.png).
Each row reads L1 → L2 → L3. Shared starters repeat for their two branches.
The [chosen trees](../../lineups/dwarves.json) contain axe/shield, miner,
crossbow and goat-rider starters, with eight physical branches. The rune line is
reserved for the separate Thundersmith / Enchanter design; its existing sources,
palettes and recipes are unchanged.

The primary style reference is the cleaner human
[shield infantry progression](../infantry-progression/preview.png).
These are artwork advancement trees; gameplay definitions are unchanged.
The extra base poses and unselected pickaxe tiers remain construction studies.

- L1: leather helmet, shoulders, arm guards, torso, apron, leggings and boots.
- L2: complete iron helmet, shoulder plates, cuirass and arm guards over leather
  apron, leggings and boots. A plate covers every pixel of its material region,
  including any extraction fleck that was previously skin or wood coloured.
- L3: all wearer armour, including apron, leggings and boots, becomes silver.
  The axeguard shield also becomes silver while retaining its separate boss/rim.

Faces, hands, beards, wooden shafts, weapon steel, faction cloth, goat coats and
mount tack remain their own materials. Natural mount/tack browns are separate
from rider armour. No gold or leather flecks remain inside a silver armour piece.

## Native source and ownership

The nine cleaned L1 input matrices and palettes in `bases/` were captured from
commit `b559200283b3279239a3225cfd2d20617e777d74`. `sources.json` records six
additional, exact material corrections at steel heads and hand edges. Each
`materials.mask` classifies every native pixel. Helmet, torso, shoulders, arm
guards, lower apron, leggings and boots have separate ownership; genuine skin,
hair, wood and mount pixels are protected. These masks describe the exact saved
poses, including their occlusions, rather than inferred colour rectangles.

`build.py` derives all three tiers directly from those cleaned inputs. Armour
gets dedicated leather or metal symbols; weapons and hair retain their original
palette. Shading follows complete pieces instead of replacing only pixels that
already happen to have the expected material colour. Transparency and outline
positions stay identical across all tiers.

The faction expansion `process.cjs tiers` command delegates these nine poses to
this recipe. Their older per-tier material cleanup records no longer own the
canonical outputs. The three rune records still use the earlier workflow.
Raw imagegen/extraction studies remain saved in `../faction-expansion/`.
For a different anatomy, capture and review a new native input and ownership mask;
do not apply these masks to another generated pose.

For future edits, update the owning input or mask here and rebuild its outputs.
PNG-only edits and edits to canonical matrices alone will be replaced by a rebuild.

## Rebuild and coverage check

Run from the project root:

```sh
python3 art/pixel/reference/dwarf-progression/build.py
python3 art/pixel/reference/dwarf-progression/build.py --check
python3 tests/dwarf_progression_test.py
npm run art:build
npm run art:check
npm run style
```

`--check` verifies all 27 tier designs, both faction matrices, palettes and PNGs.
Every intended armour pixel must use the complete material ramp for its tier;
skin, leather or wood contamination is rejected. Protected pixels and their
colours, silhouettes and outlines must be unchanged. `coverage.json` records the
verified pixel counts for every part of every tier.

Regenerate the branch boards without changing sources:

```sh
node art/pixel/reference/human-anatomies/preview.cjs art/pixel/reference/dwarf-progression/lineup-player.json art/pixel/reference/dwarf-progression/lineup-player.png 3 4
node art/pixel/reference/human-anatomies/preview.cjs art/pixel/reference/dwarf-progression/lineup-enemy.json art/pixel/reference/dwarf-progression/lineup-enemy.png 3 4
```
