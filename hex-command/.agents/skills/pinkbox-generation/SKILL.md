---
name: pinkbox-generation
description: Use to generate or regenerate native pixel artwork from reference matrices using a pinkbox template.
---

# Pinkbox generation

This is the shared generation step for units, terrain tiles and foreground props.
Use [design-unit](../design-unit/SKILL.md) or
[design-terrain](../design-terrain/SKILL.md) for subject-specific design and
integration. Work from the game root with Python 3 and Pillow. For helper changes
or extraction bugs, read [implementation.md](implementation.md).

1. Choose three approved matrix references for style, scale and materials. Unit
   IDs and explicit matrix paths are accepted. Select `--kind unit`, `terrain`
   or `prop`; every output is native 64×64. Existing callers default to `unit`.
2. Prepare and inspect the 2×2 template: three occupied anchors and one empty
   bottom-right output, enlarged eight times. Pass its local path to imagegen.
   Fill only the output, preserving references, captions and aspect ratio.
   Specify the shapes and supplied flat colors. Match chunky pixels and the
   references' outline treatment; exclude antialiasing, gradients and fine texture.
3. Save the unchanged generated PNG and exact prompt under ignored `tmp/`.
   Inspect before extraction. Do not fix layout drift by silently cropping or
   resizing the subject.
4. Extract through the saved manifest. Units retain faction colors and grounding
   rules; terrain fits the centered pointy-top hex; props use transparent 64×64
   canvases without unit grounding. Failed extraction preserves evidence.
5. Run [pixel-cleaning](../pixel-cleaning/SKILL.md) and inspect the complete
   native and enlarged matrix renders. Generation produces a candidate, not a
   finished asset. Show the clean result and return to the design skill for
   composition and integration. Keep diagnostics internal unless requested.

```sh
python3 .agents/skills/pinkbox-generation/scripts/template.py tmp/new-art/template.png \
  --kind terrain \
  --anchors pixel/art/terrain/plains/tile.txt pixel/art/terrain/hill/tile.txt pixel/art/terrain/forest/tile.txt \
  --label 'New ground tile'
python3 .agents/skills/pinkbox-generation/scripts/extract.py \
  tmp/new-art/generated.png tmp/new-art/template.json tmp/new-art/extracted
```

Unit results are `unit-1/sprite.txt`; terrain and prop results are
`terrain-1/tile.txt` and `prop-1/tile.txt`, with palettes alongside them. Clean
standalone props before judging their placement in an assembled terrain hex.

Use `--output-palette` to preserve reference material colors rather than letting
style anchors widen the output palette. This selects output colors, not anchor
backgrounds. Canonical terrain references render `G`/`g` transparent because no
floor is selected; inspect the sheet for other ground decorations before generation.
Terrain scratch palettes carry resolved
floor colors for preview; canonical ground shades remain in
`pixel/art/terrain/floors.json`, as described in
[terrain artwork](../../../docs/terrain-artwork.md).

Unit-only options are `--flying` / `--no-flying`, `--body-guide X Y W H` and
`--staff-guide X Y W H`. Grounded body guides end at row 63. Guides are removable
magenta marks, never extraction crops. A preserved output palette carries its
flying flag unless overridden. The unit design skill explains anatomy, ground
contact and faction-material ownership.

For an object derived from an existing prop, measure the reference's painted
bounds and use `--bounding-box X Y W H` to constrain its painted bounds.
This is a hard extraction check: overflow is rejected with evidence, never
trimmed or resized. A ruined version retains the original building's scale.
