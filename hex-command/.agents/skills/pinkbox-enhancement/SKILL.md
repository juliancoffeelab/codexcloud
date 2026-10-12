---
name: pinkbox-enhancement
description: Revise approved Hex Command unit artwork through material recoloring or shape redesign; recolor terrain floors through their shared palettes.
---

# Pinkbox enhancement

Work from the game root with Python 3 and Pillow. Save the approved matrix and
palette as an exact baseline in `tmp/`.
Identify the requested changes to anatomy, pose, grips, equipment, materials and
faction cloth. Preserve the shared pixel scale; progression needs readable shapes.

For palette changes, inspect actual surfaces before recoloring. Keep skin and
cloth colors unless the user asks to change them. Preserve outlines, alpha and
grips, and materials outside the requested change. Recolor base and shadow together
in coherent clusters following overlaps and curves. Identify materials by their
surfaces and anatomy, rather than color alone.

The fill JSON requires `matrix_sha256`, `palette_sha256` and `regions`. Each region
has `seed: [x,y]`, `mapping: {old: new}`, `expected_pixels`, `material` and `reason`.
Optional `palette_additions` maps new material symbols to RGBA colors. Use a fresh
output directory under `tmp/`. The helper produces the candidate matrix/palette,
an exact `before/` copy, a change record and comparison image. Outline, transparency
and faction-cloth changes use individually reviewed [pixel edits](../pixel-cleaning/SKILL.md#unit-sprites).

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/fill.py \
  pixel/art/units/humans/adept/sprite.txt tmp/upgrade/fill.json tmp/upgrade/candidate
```

For new anatomy, silhouettes or equipment shapes, use
[pinkbox generation](../pinkbox-generation/SKILL.md). Specify the changes in the
prompt and save the unchanged output. Prepare a separate native 64×64 template
for each tier, with the approved unit first and two style anchors:

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/prepare.py \
  scout tmp/scout-upgrade/template.png --label 'Scout L2' --style-anchors ranger archer
```

The template preserves the approved unit's flying/swimming pose flag by default.
Use `--flying` or `--no-flying` only when changing that pose. Pass `--output-palette`
when its material colors must stay fixed.

For terrain floor recoloring, change the selected shared palette or a tile's
floor selection as described in [terrain artwork](../../../docs/terrain-artwork.md).
The matrix stays unchanged; inspect the affected tiles after rebuilding artwork.

Use [pixel cleaning](../pixel-cleaning/SKILL.md) before showing the result and
[lineup previews](../pixel-lineup/SKILL.md) for progression. Show clean matrix
renders in the requested faction and keep diagnostics internal unless requested.
When integration is part of the task, replace the canonical matrix and palette,
then rebuild and check artwork. For helper changes or fill/template bugs, read
[implementation.md](implementation.md).
