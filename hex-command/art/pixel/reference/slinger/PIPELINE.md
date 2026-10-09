# Enlarged reference → native matrix

This is the process used for the Slinger, including its limitations. The image
model designs a complete unit; the extraction preserves that design instead of
reconstructing the body from polygons.

## 1. Prefill the input

Use `python3 scripts/pink-box.py tmp/new-design.png scout ranger archer`
from the project root, substituting the three approved anchor IDs. The tool
reads current matrices and faction colours; generated templates are disposable
and stay in ignored `tmp/`. Saved raw imagegen outputs and extraction recipes
remain historical evidence, not the source of truth for approved units.

Create a 1024×1024 PNG divided into four 512×512 slots. Fill three slots with
existing 48×48 unit PNGs, enlarged exactly 8× using nearest-neighbor sampling.
Each sprite is centered in a logical 64×64 slot, with eight native pixels of
padding on each side. Leave the fourth slot for the new unit. Include palette
swatches and label the new slot. The Slinger anchors were Scout, Ranger and
Archer. The background is solid magenta, #FF00FF, for later removal.

The Slinger supplied eleven visible colors from the existing Scout palette:
leather, teal cloth, skin, ivory cloth and gray stone/steel, plus their shadows.
Anatomy studies requiring armor and horses use a slightly larger explicit
palette. A smaller palette limits color noise but does not guarantee anatomy.

For new designs, render the anchors from current shared matrices and take the
two faction swatches from `art/pixel/factions.json`, using the player palette.
Use exactly those two shades on faction cloth; keep bone, skin, leather, metal,
weapon effects and outlines in separate material colours. Older templates and
saved extraction palettes may contain obsolete faction ramps; refresh their
swatches and extraction palette together before generating a new design.
Do not change historical raw extractions merely to refresh a new template.

Keep magenta reserved for the removable background. The debug faction palette
also contains magenta, so never use debug faction colours in generation inputs
or raw extraction: the background classifier would remove faction pixels.

## 2. Generate one complete new unit

Give the image model the prefilled PNG. Ask it to preserve the anchors, fill
only the empty slot, match the pixel size and character proportions, and use
flat supplied colors without antialiasing, gradients or ground shadows. Specify
the pose, actual hand grips and weapon placement. Keep the whole unit inside
the central 48×48 area, leaving the outer padding empty.

The original Slinger response was 1254×1254 rather than 1024×1024. Therefore
extraction uses fractions of the returned dimensions, not assumed input pixels.
Save the raw generated image before any extraction or cleanup.

## 3. Cut out and quantize

Crop the bottom-right quarter of the returned image. For the Slinger this was
`x=627, y=627, width=627, height=627`.

Classify transparent or magenta-dominant source pixels as background. Map every
remaining pixel to the nearest supplied RGB color using squared RGB distance.
Divide the cropped slot into 64×64 logical blocks. In each block, use the most
frequent foreground palette color when foreground covers at least 35% of the
block; otherwise use transparent `.`. Exclude the caption area and perimeter.
Do not interpolate or use a JPEG intermediate.

Trim the eight logical padding pixels on each side only after confirming that
all visible artwork fits inside the central 48×48 area. If it does not fit,
inspect the crop or regenerate; do not silently crop off limbs or rescale the
character to make it fit. Store the result as a symbolic matrix and a palette
whose alpha values are exclusively 0 or 255. The `.` entry is transparent black.

Raw single-sided extractions may retain `player.txt` as generation evidence.
Before promoting a design into `units/`, classify faction cloth by material
ownership and write one `sprite.txt`: `^` is the faction base and `*` its shadow.
Keep those symbols out of the unit's material `palette.json`; the renderer reads
their RGBA colours from `factions.json`. Do not identify faction pixels solely
by RGB similarity: sleeves and cloak folds need faction channels, while exposed
hands, feet, bone and dark outer outlines retain their material symbols.
If a saved recipe writes a temporary player/enemy pair, `normalize_pair` in
`scripts/pixel_sources.py` consolidates it during rendering. New final sources
must not retain separate faction matrices.

## 4. Inspect material regions

Render at native resolution, then enlarge with nearest-neighbor sampling. Look
at each material region, not just isolated pixels or colors absent from the
palette. A legal ivory pixel in a leather belt can still be an artifact.

Check skin, sleeves, tunic, hair, leather, weapon and silhouette separately.
Remove wrong-material flecks; use broad flat areas and connected contour
shadows. Preserve intentional eyes, buckles, weapon highlights and small gaps.
Inspect feet, elbows, hands and weapon grips against the generated source.
Record manual changes with native coordinates, old/new symbols and reasons.

The Slinger needed fourteen initial material corrections, nineteen shirt-shading
corrections and five subsequent skin/belt corrections. Its silhouette was
preserved throughout. These corrections are recorded in `cleanup.json`.

## 5. Render and compare

Render PNGs from matrices with `scripts/render-pixel.py`; never patch only the
PNG. Show the generated crop beside the matrix, and the new unit beside the
existing family at the same native scale. Inspect actual transparency on a
checkerboard. The Slinger uses ten visible colors after cleanup.

Render both player and enemy previews from the same `sprite.txt`, then inspect
the debug preview separately after extraction. Check sleeves against hands,
cloak hems against feet, collars against faces, and cloth folds against trim.
Only `^`/`*` pixels may change between sides; all other material pixels and
transparency must match. Never bake debug colours into the gameplay atlas.
Record corrections in the owning rebuild recipe or reviewed cleanup recipe so
that regeneration preserves them. Run `npm run art:check` after rebuilding the
atlas.

For tier variants, start from the cleaned base matrix. Change palette entries
for broad material changes, and edit only the small local regions needed for
masks, armor, weapon heads or effects. Preserve the base grip and joints. Render
and inspect each variant again; recoloring alone can obscure limbs or produce
unreadable equipment.

The finished Slinger source lives in `art/pixel/units/slinger/`, with no gameplay
registration or stats. This folder retains the generated input and workflow.
