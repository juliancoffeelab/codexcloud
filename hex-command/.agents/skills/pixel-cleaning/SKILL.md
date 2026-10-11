---
name: pixel-cleaning
description: Review sprite artifacts with stray and outline detectors, then apply individually justified matrix corrections without redesigning approved units.
---

# Pixel cleaning

Tell the user when applying this skill. Run from the game root with Python 3
and Pillow. Character matrices are always 64×64. Save the exact baseline matrix,
palette and source image; use a fresh `tmp/` directory for each candidate.
Detect means inspect without editing; stop means stop mutations; revert means
restore the exact named files, not reconstruct them.

```sh
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-review
python3 .agents/skills/pixel-cleaning/scripts/apply_edits.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-edits.json tmp/scout-candidate
```

The review bundle produces faction previews, every detector finding and enlarged
eight-neighbor contexts. Inspect these images and the unmarked sprite. For full
cleanup, review every surface and candidate, including connected wrong-material
patches detectors miss. For a narrow fix, stay within that scope.

- The stray detector uses cardinal equal-color neighbors; intentional eyes,
  highlights and diagonal details can be flagged. Optional material shade groups
  are review inputs, not proof that a shade belongs at a particular location.
- Edge findings are candidates. Preserve intentional diagonal outlines, limb
  separators, weapon width and negative spaces. A hair notch may need an exterior
  black pixel removed rather than its hole filled. Do not bridge every diagonal.
- Infer ownership from the source image and local anatomy, not palette names.
  A brown leak in cloth needs the local cloth color, not a black seam. Preserve
  cream clothing on foxes and dark exposed paws; distinguish skin from sleeves.
- Grounded feet/body rest on native row 63. A staff reaching that row is not
  enough if the feet hover. Preserve all painted pixels during alignment; only
  palettes with `"flying": true` allow intentionally suspended poses.
- Keep shade clusters coherent and surfaces connected. Use the
  current `pixel/art/units/humans/scout/sprite.txt` as a finish benchmark for
  deliberate material planes, not a pose or palette to copy. Avoid shade noise.

The edit JSON requires `matrix_sha256`, `palette_sha256`, and `edits`. Each edit
has native `x`, `y`, `from`, `to`, `material` and `reason`; outline/transparency
changes also require `boundary_reviewed: true`. Apply only individually reviewed
changes. Hash guards reject stale inputs and the helper preserves the baseline.
Never batch-paint detector flags or edit only a PNG.

Rerun and visually inspect current diagnostics after corrections. Track confirmed
defects separately from changed pixels; zero flags is not the goal. Restore
mistaken edits exactly. State unresolved uncertainty instead of claiming a
sprite is clean based on counts alone. Show a clean nearest-neighbor matrix
render in the requested faction, otherwise last used or enemy. Keep debug colors,
coordinates and overlays internal unless asked. Use
[pixel lineup](../pixel-lineup/SKILL.md) for tree previews.

For authorized integration, copy the candidate matrix/palette to its canonical
unit folder and rebuild/check art. Matrices are authoritative; no owning recipe
or cleanup replay is required.
