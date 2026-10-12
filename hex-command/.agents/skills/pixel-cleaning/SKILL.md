---
name: pixel-cleaning
description: Review sprite artifacts with stray and outline detectors, then apply individually justified matrix corrections without redesigning approved units.
---

# Pixel cleaning

Tell the user when applying this skill. Work from the game root with Python 3
and Pillow. Preserve the exact baseline matrix, palette and image in a fresh
`tmp/` directory. Character matrices remain 64×64. Detect means inspect without
editing; stop means stop mutations; revert means restore the named files exactly.
Detector behavior, edit-file schemas and terrain joins are in
[implementation.md](implementation.md).

```sh
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-review
python3 .agents/skills/pixel-cleaning/scripts/apply_edits.py \
  pixel/art/units/humans/scout/sprite.txt tmp/scout-edits.json tmp/scout-candidate
```

Inspect the review images and unmarked source. A full cleanup examines every
surface, including connected defects detectors miss; a narrow fix stays within
its requested scope. Findings are candidates, not instructions to paint them.

- Preserve intentional eyes, highlights, diagonals, limb separators, weapon width
  and negative spaces. Do not bridge every diagonal or fill every notch.
- Infer material ownership from the source and anatomy. Distinguish skin from
  sleeves and pale clothing from fur or armor. Match repairs to the actual material.
- Keep grounded feet or bodies at native row 63; equipment touching it is not
  sufficient. Preserve every painted pixel during alignment. Intentionally suspended
  poses require the explicit flying flag.
- Keep shade clusters coherent. Use the approved human scout as a finish benchmark
  for material planes, without copying its pose or palette.
- For directional terrain, inspect assembled joins at native scale in connected
  networks. Nearby pixels or matching connection names do not establish continuity.

Apply only individually justified matrix edits. Do not batch-paint detector flags
or edit only a PNG. Rerun diagnostics and visually inspect the result. Track
confirmed defects separately from changed pixels; zero flags does not establish
that a sprite is clean. Restore mistaken edits exactly and state remaining uncertainty.

Show a clean nearest-neighbor matrix render in the requested faction, otherwise
last used or enemy. Keep debug colors and overlays internal unless asked. Use
[lineup previews](../pixel-lineup/SKILL.md) for trees. When integration is part of
the task, copy the chosen matrix/palette into its unit folder and rebuild/check art.
