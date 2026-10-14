---
name: pixel-cleaning
description: Use when inspecting, diagnosing, cleaning or repairing Hex Command unit sprites or terrain matrices, including generated artwork, stray pixels, broken outlines, material shading, ground alignment and terrain joins. Keep detection requests read-only; apply only individually reviewed native matrix edits when fixes are requested, preserving approved artwork and the 64×64 canvas.
---

# Pixel cleaning

Work from the game root with Python 3 and Pillow. Detection requests use only
read-only commands. Before editing, preserve the exact baseline matrix, palette
and image in a fresh `tmp/` directory. Use the branch matching the source.

## Unit sprites

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
without copying its pose or palette.

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
