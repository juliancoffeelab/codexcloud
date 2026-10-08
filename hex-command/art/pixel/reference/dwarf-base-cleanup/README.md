# Restored dwarf bases

[Base review grid](lineup-player.png) · [Independent fills](fill-regions.png) ·
[Marked suspect pixels](isolated-pixels.png).

The nine non-rune L1 poses have their equipment and colours restored from before
the rejected armour progression. Their silhouettes and native 64×64 canvases
remain unchanged. This pass cleans the existing designs; it does not advance
them into leather or silver armour. The scout is the palette/linework reference.
The rune family is excluded for the separate Thundersmith / Enchanter idea.

Faces and hands share one skin colour and shadow. Leather, hair and wood share
one brown colour and shadow. Cream fabric, faction cloth and steel have their
own ramps. The original facial features and dark boundaries have been restored,
including the exposed skin fields that the rejected masks had treated as armour.
Only two reviewed, enclosed dark specks in cuirasses are removed. Isolated shades
join a neighbouring shade of the same material; this does not erase a boundary.
`reviewed-specks.json` records 45 specific material/ink corrections across the
poses, including stray skin in sleeves, brown in steel and teal in the pick head. Forearm, apron, trouser
and boot seams separate pieces that had merged into large brown fields. Steel
helmets, weapon heads, shoulder fittings and the axeguard shield retain their
pre-progression materials.

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
connected original skin fields cannot be swallowed by armour. Pixel/PNG equality alone cannot
establish visual quality, so inspect the grid after any source edit.

## Orthogonal speck check

`../../../../scripts/check_isolated_pixels.py` flags an opaque pixel when none of
its left/right/up/down neighbours has the exact same RGBA colour. Diagonals do
not count. Black pixels beside transparent X/Y neighbours are excluded by default;
`--include-exterior-black` restores the unfiltered report. Other shades of the
same material still count as different colours for this check.

The filtered count dropped from **964 to 363** after the speck pass. The unfiltered
count dropped from 1,161 to 559. The remaining suspects are marked in red in the
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
The faction expansion `process.cjs tiers` command rebuilds these bases only.
The rejected L2/L3 study is frozen and its former builder refuses to write art.
New advanced tiers should wait until the base designs are reviewed.

Regenerate all three review boards and the suspect-pixel report without editing sources:

```sh
python3 art/pixel/reference/dwarf-base-cleanup/review.py
```
