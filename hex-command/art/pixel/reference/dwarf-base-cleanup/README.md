# Restored dwarf bases

[Base review grid](lineup-player.png) · [Independent fills](fill-regions.png) ·
[Marked suspect pixels](isolated-pixels.png).

The nine non-rune starter/reference poses have their equipment and colours restored from before
the rejected armour progression. Their silhouettes and native 64×64 canvases
remain unchanged. This pass cleans the existing designs; it does not advance
them into leather or silver armour. The scout is the palette/linework reference.
Four are active L1 starters; five are saved as
[branch pose references](../dwarf-branch-references/README.md). The rune family is
owned by the separate [rune-line recipe](../rune-line/README.md), which cleans
the apprentice and derives Thundersmith / Stonespell.

Faces and hands share one skin colour and shadow. Leather, hair and wood share
one brown colour and shadow. Cream fabric, faction cloth and steel have their
own ramps. The original facial features and dark boundaries have been restored,
including the exposed skin fields that the rejected masks had treated as armour.
The crossbowman and arquebusier each have one added eye toward their weapon,
recorded explicitly in `reviewed-specks.json`; their original eyes remain.
Only two reviewed, enclosed dark specks in cuirasses are removed. Isolated shades
join a neighbouring shade of the same material; this does not erase a boundary.
`reviewed-specks.json` records 67 specific material/ink corrections across the
poses, including stray skin in sleeves, brown in steel and teal in the pick head. Forearm, apron, trouser
and boot seams separate pieces that had merged into large brown fields. Steel
helmets, weapon heads, shoulder fittings and the axeguard shield retain their
pre-progression materials.
The later 22 skin-fleck corrections cover shirt edges, cuffs, trouser cloth,
one cuirass speck and one beard speck with adjacent cream/brown materials.
Their exact pixels become gold inlays in L3; faces and grips remain skin.
Another 31 requested forearm pixels on the hammerer and goat hammerer are covered
with cloth/leather sleeves and guards, then armour in advanced tiers.

## Source and fills

`bases/*/player.mask` is the authoritative cleaned native matrix, using the same
symbols as a normal player matrix. Its extension keeps the renderer from treating
it as a second canonical image. Each base also has a compact palette.
`build.py` creates both faction matrices, PNGs and a pixel change record against
the saved pre-progression input in `../dwarf-progression/bases/`.

Fill regions are detected from the **actual cleaned pixels**. The detector groups
shades by their base colour, finds edges between those colours and the dark
outline, then fills each region using four-way adjacency. Shading does not split
a surface. Thin diagonal shafts and partly occluded features can have several
visible islands; all islands have saved seeds. `parts.json` records these seeds,
counts and bounds; `parts.mask` records their exact coverage. These are detected
regions, not hand-assigned armour rectangles. `anchors.json` names 161 physical
part seeds across the nine poses. Every named field must fill independently: a
cuirass/apron, beard/torso, stock/armour or trouser/boot merge fails the check.
Cream mount fur and cream clothing share a ramp but remain in their visible regions.

The fill review paints every detected region with one flat colour, keeping dark
outlines and transparency. `--check` compares region coverage as well as the exact
canonical matrices, palettes and PNGs. It rejects a changed material boundary or
a gap that joins previously separated parts. Original facial/contour pixels are
protected, with explicit guarded exceptions for the two interior ink specks;
connected original skin fields cannot be swallowed by armour, except for the
explicitly recorded garment flecks. The two reviewed
eye additions are single facial pixels below the helmet, with skin on either
side and beneath. Pixel/PNG equality alone cannot
establish visual quality, so inspect the grid after any source edit.

## Orthogonal speck check

[The pixel-cleaning skill’s bundled stray detector](../../../../.agents/skills/pixel-cleaning/SKILL.md) flags an opaque pixel when none of
its left/right/up/down neighbours has the exact same RGBA colour. Diagonals do
not count. Black pixels beside transparent X/Y neighbours are excluded by default;
`--include-exterior-black` restores the unfiltered report. Other shades of the
same material still count as different colours for this check.

The filtered count dropped from **964 to 363** after the speck pass. The unfiltered
count dropped from 1,161 to 559. Adding the two eyes subsequently brings the
counts to 365 filtered / 561 unfiltered. Covering the 22 skin flecks brings the
counts to 354 filtered / 550 unfiltered. The requested forearm coverage brings
the current counts to **352 filtered / 548 unfiltered**. The remaining suspects are marked in red in the
[review sheet](isolated-pixels.png), with native coordinates and colours in
[isolated-pixels.json](isolated-pixels.json). The counts are recorded in
[check-summary.json](check-summary.json). Eyes, internal diagonal boundaries,
thin shafts and small fittings can be valid flags. These pixels remain visible
for review; the report does not assert that every remaining flag is an artifact.
Deleting a flagged boundary to lower the count can join two actual parts.

## Rebuild

Run from the project root:

```sh
python3 art/pixel/reference/dwarf-base-cleanup/build.py
python3 art/pixel/reference/dwarf-base-cleanup/build.py --check
python3 tests/dwarf_base_parts_test.py
python3 tests/isolated_pixels_test.py
npm run art:build
npm run art:check
npm run style
```

After a reviewed source edit, update its palette and explicitly record the new
regions with `build.py dwarf-axeguard --record-parts`, then inspect and check them.
The faction expansion `process.cjs tiers` command rebuilds these sources and their
[whole-surface branch upgrades](../dwarf-upgrades/README.md).
The rejected L2/L3 study and its obsolete builder have been removed.
The saved original `dwarf-progression/bases/` inputs remain required for these checks.

Regenerate all three review boards and the suspect-pixel report without editing sources:

```sh
python3 art/pixel/reference/dwarf-base-cleanup/review.py
```
