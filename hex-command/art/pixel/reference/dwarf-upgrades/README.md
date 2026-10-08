# Dwarf branch upgrades

[Teal trees](tree-player.png) · [Red trees](tree-enemy.png).

The actual lineup is in [dwarves.json](../../lineups/dwarves.json): five L1
starters. All five trees have two L2 branches apiece, each with its own L3.
The [rune-line recipe](../rune-line/README.md) owns Thundersmith and Stonespell. Axe + pick, hammerer, poleaxe, arquebusier and pistol
rider are [pose references](../dwarf-branch-references/README.md), not extra L1
starters. The miner does not have a separate pickaxe L2/L3 branch.

Branches inherit their cleaned starter's native 64×64 scale and material palette.
Equipment and grips come from the corresponding cleaned pose reference. L2 fills
complete helmet, cuirass, sleeves and bracer regions with steel, leaving leather
lower armour. L3 fills all assigned armour and the shield face with silver, then
adds gold hems, cuff trim and stitching in the faction cloth. Teal cloth becomes
red for enemies; silver and gold remain shared. Rune branches have their own
complete armour/robe ramps and a darker cyan crystal border in both factions.

`regions.json` assigns every **detected four-way fill component** from the cleaned
sources to helmet, upper armour, lower armour, shield, faction cloth or protected
material. The exact source SHA prevents replay against a changed face or seam.
Named armour anchors must belong to armour; faces, hands, beard, weapon wood and
steel, goat fur, horns and tack retain their own materials. No rectangular armour
overlays are used. Source outlines, eyes and transparency are preserved.

The follow-up covers 22 reviewed stray skin pixels in sleeve cuffs, shirt edges,
trouser cloth, one cuirass speck and one beard speck. Their L1 replacements are
recorded in the base cleanup's `reviewed-specks.json`. L2 uses the neighbouring
garment/armour material. `gold-inlays.json` requests gold at those exact pixels in
L3, including a small beard ornament. These explicit inlays may be single gold
rivets; ordinary gold stitching must remain contiguous. Real face and hand fields
remain visible.

The hammerer and mounted hammerer also have 31 forearm pixels covered with cloth
or leather in their base poses, then steel/silver in both advanced tiers. Their
gripping hands remain skin. These requested sleeves/guards are recorded as
`arm-covering` edits, separate from the stray-pixel gold inlays.

`coverage.json` records complete coverage of assigned regions and gold counts.
The checker rejects leftover skin/leather inside silver pieces, recoloured
protected fields, changes to eyes/contours, missing gold inlays and stale region
assignments. Coverage checks verify the assignment; visual inspection is still
needed to judge whether a small fragment was assigned to the right physical part.

## Rebuild

Run from the project root:

```sh
python3 art/pixel/reference/dwarf-base-cleanup/build.py
python3 art/pixel/reference/dwarf-upgrades/build.py
python3 art/pixel/reference/rune-line/build.py
python3 art/pixel/reference/dwarf-upgrades/preview.py
python3 art/pixel/reference/dwarf-upgrades/build.py --check
python3 tests/dwarf_upgrades_test.py
python3 tests/rune_line_test.py
npm run art:build
npm run art:check
npm run style
```

The faction expansion `process.cjs tiers` routes these dwarf jobs through the
cleaned sources and this upgrade recipe. Passing a starter ID rebuilds its two
branches. The older rejected [dwarf progression](../dwarf-progression/README.md)
has its own archived tier outputs and remains a read-only historical study.
