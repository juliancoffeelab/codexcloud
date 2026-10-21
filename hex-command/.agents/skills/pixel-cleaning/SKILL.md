---
name: pixel-cleaning
description: Use for inspecting or repairing Hex Command sprite and terrain pixels, outlines, shading, alignment or joins.
---

# Pixel cleaning

Work from the game root with Python 3 and Pillow. Detection requests use only
read-only commands. Before editing, preserve the exact baseline matrix, palette
and image in a fresh `tmp/` directory. Use the branch matching the source.

## Unit sprites

Start with color isolation: inspect brown, ivory, skin and steel in separate views.
The script highlights **every occurrence** of the selected RGB colors, including
pixels outside their intended material. No spatial or ownership filtering is
allowed. The agent judges which highlighted pixels are misplaced, edits those
pixels individually, then reruns the views. Brown belongs to leather, wood or
hair; do not reinterpret misplaced brown as a gold shadow.

```sh
python3 .agents/skills/pixel-cleaning/scripts/highlight_colors.py \
  pixel/art/units/humans/scout/sprite.txt --output tmp/scout-colors
# Or select exact colors rather than a preset (quote shell # characters):
python3 .agents/skills/pixel-cleaning/scripts/highlight_colors.py \
  pixel/art/units/humans/scout/sprite.txt --color '#6f482b' --color '#40281c' \
  --output tmp/scout-browns
```

Each check accepts **at most two source colors**. Presets expand into separate
pairwise original/recolored comparisons; do not recolor a whole multi-color
family at once. Each matching source color gets a **different high-contrast
replacement**, with a visible color key: do not
flatten base and shadow colors into one highlight. All occurrences of a source
color receive the same replacement, like a faction-palette preview.
Presets are explicit RGB
lists in `references/highlight-colors.json`, not complete material definitions.
Check `report.json` for selected colors, all matching coordinates and the source's
RGB counts; add missing colors explicitly. Inspect shade colors as well as bases.
Use existing player/enemy/debug previews for faction-cloth ownership.

```sh
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-review
python3 .agents/skills/pixel-cleaning/scripts/apply_edits.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-edits.json tmp/scout-candidate
```

The edit JSON requires `matrix_sha256`, `palette_sha256` and `edits`. Use the source
hashes from the review's `report.json` (`sha256` and `palette_sha256`). Each edit
has native `x`, `y`, `from`, `to`, `material` and `reason`. Outline or transparency
changes also require `boundary_reviewed: true`. Apply to a fresh `tmp/` directory;
the helper returns `sprite.txt`, its palette, a comparison image and an exact
`before/` copy. Rerun review on that candidate before integration.

Character matrices remain 64×64. Keep grounded feet or bodies at native row 63;
equipment touching it is insufficient. Preserve every painted pixel during
alignment. Intentionally suspended poses retain their explicit flying flag.
Use the approved human scout as a finish benchmark for material planes,
without copying its pose or palette. Its detector count is not a cleanup target;
preserve the unit's established shapes and readable shading.

## Terrain matrices

For `tile.txt` and directional frames, use the standalone detectors, which accept
terrain sources and resolve their shared floor colors. The unit review/edit
helpers above accept only `sprite.txt`.

```sh
python3 .agents/skills/pixel-cleaning/scripts/check_isolated_pixels.py \
  pixel/art/terrain/road/tile.txt --output tmp/road-review/strays.json \
  --images-dir tmp/road-review
python3 .agents/skills/pixel-cleaning/scripts/check_broken_outlines.py \
  pixel/art/terrain/road/tile.txt --output tmp/road-review/outlines.json \
  --images-dir tmp/road-review
```

Review each affected directional frame and connected forks, crossroads and curves.
Road exits must meet at the same position and width, with a normal tangent through
the hex boundary. Assess the joins at native scale; matching connection names
alone do not establish continuity. Treat outline findings as candidates: terrain
surfaces do not necessarily use character-style black borders.

For requested source fixes, back up the matrix and its palette, then edit only
individually reviewed native matrix pixels. Keep shared floor colors in
`floors.json`, as described in [terrain artwork](../../../docs/terrain-artwork.md).
Rerun the detectors and `npm run art:build` / `npm run art:check`, and inspect the
affected tile networks. Preserve the original frame filenames during edits.

## Review and present

Inspect the review images and unmarked source. A full cleanup examines every
surface, including connected defects detectors miss; a narrow fix stays within
its requested scope. Findings are candidates, not instructions to paint them.

- Preserve intentional eyes, highlights, diagonals, limb separators, weapon width
  and negative spaces. Do not bridge every diagonal or fill every notch.
- Infer material ownership from the source and anatomy. Distinguish skin from
  sleeves and pale clothing from fur or armor. Match repairs to the actual material.
- Keep shade clusters coherent.

Apply only individually justified matrix edits. Do not batch-paint detector flags
or edit only a PNG. Rerun diagnostics and visually inspect the result. Track
confirmed defects separately from changed pixels; zero flags does not establish
that a sprite is clean. Restore mistaken edits exactly and state remaining uncertainty.

Show a clean nearest-neighbor matrix render in the requested faction, otherwise
last used or enemy for units. Keep debug colors and overlays internal unless asked. Use
[lineup previews](../pixel-lineup/SKILL.md) for trees. When integration is part of
the task, copy the chosen unit matrix/palette into its unit folder and rebuild/check
art. For detector or guarded-editor changes and debugging, read
[implementation.md](implementation.md).
