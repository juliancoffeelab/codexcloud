---
name: pixel-cleaning
description: Use to inspect, clean or repair native pixel artwork, including units, terrain and foreground props.
---

# Pixel cleaning

This is the shared cleanup step after generation, enhancement or manual edits.
Use [design-unit](../design-unit/SKILL.md) and
[design-terrain](../design-terrain/SKILL.md) for subject-specific requirements.
Work from the game root with Python 3 and Pillow. Read
[implementation.md](implementation.md) before changing the helpers.

## Inspect

Detection requests are read-only. Before edits, preserve the exact baseline
matrix, palette and rendered image in a fresh ignored `tmp/` directory. Keep
source matrix dimensions unchanged. For units, follow the
[native artwork rules](../design-unit/SKILL.md#native-artwork); for terrain, follow
[its frame contract](../design-terrain/SKILL.md#terrain-contract). The current cleanup
helpers accept ordinary 64×64 matrices. For complete padded
terrain frames, retain the declared canvas, anchor and depth and inspect the full
frame through the artwork builder/map renderer. Report that guarded cleanup helpers
do not yet support this branch; extend those shared helpers within an authorized
implementation task before using them on padded frames.
Inspect at native scale and enlarged with nearest neighbor.

Start by isolating material colors. Highlight every occurrence of each selected
RGB color, including misplaced pixels. Do not filter matches by location,
neighbors, symbol or presumed ownership. Each check accepts at most two source
colors with distinct replacements. Presets expand into pairs; explicit `--color`
values work for any subject. Check coordinates, counts, hashes and source/shade
colors in the report. The agent judges which pixels are wrong.

```sh
python3 .agents/skills/pixel-cleaning/scripts/highlight_colors.py \
  pixel/art/terrain/hill/tile.txt --color '#1e511a' --color '#476922' \
  --output tmp/hill-colors
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  pixel/art/terrain/hill/tile.txt tmp/hill-review
```

The combined reviewer accepts unit, terrain and prop matrices. It provides
isolated-pixel and outline candidates, material neighborhoods and full previews.
Unit previews also compare player, enemy and debug faction colors. For standalone detectors or detector-specific options, consult
[detector commands](implementation.md#standalone-detectors).

## Repair

Judge findings against the intended surface. Flags are candidates, not painting
instructions. Inspect connected wrong-material patches too; zero flags does not
establish clean artwork. Preserve intentional diagonals, details and negative
spaces. Terrain does not automatically need a black outline.

For confirmed edits, follow the
[guarded edit request contract](implementation.md#edit-request-contract).

```sh
python3 .agents/skills/pixel-cleaning/scripts/apply_edits.py \
  pixel/art/terrain/hill/tile.txt tmp/hill-edits.json tmp/hill-candidate
```

The helper writes a fresh candidate and byte-exact baseline for comparison.
For terrain scratch palette integration, follow
[floor palettes](../design-terrain/references/surfaces.md#floor-palettes).

Rerun diagnostics and inspect the full result. Correct only confirmed defects,
restore mistaken edits exactly and state remaining uncertainty. Do not modify
only a PNG, resize/crop artwork to hide defects, or bulk-apply detector flags.
Shape or palette-wide changes belong in
[artwork-enhancement](../artwork-enhancement/SKILL.md).

## Finish

Review unit anatomy, material ownership and body ground contact with the unit
design skill. Review terrain/props in assembled neighborhoods, including joins,
floor variants and densities, with the terrain design skill. A clean isolated
piece can still compose badly.

Show clean matrix renders; retain diagnostic views under ignored `tmp/`. When
integration is requested, update canonical matrices/palettes and run
`npm run art:build` and `npm run art:check`. These invoke the artwork-builder
script; the builder is not a skill. Stay within the requested integration scope.
