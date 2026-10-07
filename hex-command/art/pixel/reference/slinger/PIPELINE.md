# Enlarged reference → native matrix

This is the process used for the Slinger, including its limitations. The image
model designs a complete unit; the extraction preserves that design instead of
reconstructing the body from polygons.

## 1. Prefill the input

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

For tier variants, start from the cleaned base matrix. Change palette entries
for broad material changes, and edit only the small local regions needed for
masks, armor, weapon heads or effects. Preserve the base grip and joints. Render
and inspect each variant again; recoloring alone can obscure limbs or produce
unreadable equipment.

The finished Slinger source lives in `art/pixel/units/slinger/`, with no gameplay
registration or stats. This folder retains the generated input and workflow.
