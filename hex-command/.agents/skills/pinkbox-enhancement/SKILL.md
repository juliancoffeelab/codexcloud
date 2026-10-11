---
name: pinkbox-enhancement
description: Revise an approved Hex Command unit or tier through guarded material recoloring, or pinkbox when its shapes need redesign.
---

# Pinkbox enhancement

Tell the user when applying this skill. Work from the game root with Python 3
and Pillow. Keep the exact approved matrix/palette as the baseline in `tmp/`.
Identify what may change from the request: anatomy, pose, grips, equipment,
materials and faction cloth. Higher tiers need readable progression at the
same pixel scale, not extra texture or arbitrary ornament.

For palette-only changes, paint existing parts directly. Inspect the actual
surfaces, preserving outlines, alpha, skin, cloth and grips. Recolor base and
shadow together; shade coherent clusters following overlaps and surface curves.
Do not mistake pale clothing for armor or fur based on its color.

Terrain floors use `pixel/art/terrain/floors.json`: tune each terrain's `G`/`g`
land or `W`/`w` water pair there, keeping those symbols out of local material
palettes. Tree foliage and grass tufts can share other green symbols; leave
them alone when tuning the floor. Render through the atlas packer's
`read_palette`/`rgba_pixels` so previews resolve the shared floor colors too.

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/fill.py \
  pixel/art/units/humans/adept/sprite.txt tmp/upgrade/fill.json tmp/upgrade/candidate
```

The fill specification requires `matrix_sha256`, `palette_sha256`, and `regions`:
each region has `seed: [x,y]`, `mapping: {old: new}`, `expected_pixels`, `material`
and `reason`. Optional `palette_additions` maps new symbols to RGBA colors.
Four-neighbor fills stop at unmapped materials; stale hashes, changed counts,
overlapping regions and outline/faction/transparent fills are rejected. Follow
with individually reviewed shading through the cleaning skill's edit helper.

For new anatomy, silhouettes or equipment shapes, use
[pinkbox generation](../pinkbox-generation/SKILL.md) and its pixel-style contract:

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/prepare.py \
  scout tmp/scout-upgrade/template.png --label 'Scout L2' --style-anchors ranger archer
```

The approved unit is anchor 1, plus exactly two style anchors and one empty
output. Use a separate 64×64 template for each tier. `--output-palette` locks
existing materials when required. Specify the intended changes explicitly in
the prompt; save the unchanged output and prompt.

Use [pixel cleaning](../pixel-cleaning/SKILL.md) before showing the result and
[pixel lineup](../pixel-lineup/SKILL.md) for progression. Keep diagnostics internal;
show clean matrix renders in one faction. For authorized integration, replace
the canonical matrix/palette and run `npm run art:build` / `npm run art:check`.
Do not add a confirmation loop when approval is already established.
