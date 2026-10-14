---
name: pinkbox-generation
description: Use when generating new Hex Command pixel unit artwork or regenerating a unit's silhouette, anatomy, pose or equipment through image generation. Build a pinkbox template from three approved matrix anchors, generate one output, then extract and review its native 64×64 matrix before presentation or integration.
---

# Pinkbox generation

Work from the game root with Python 3 and Pillow. Read the requested race tree
and use approved matrices as references. For helper changes or extraction bugs,
read [implementation.md](implementation.md).

1. Prepare a 2×2 template with three occupied anchors and one empty bottom-right
   output. Each character is native 64×64, enlarged eight times in the template.
   Keep grounded feet at row 63. Match related tiers' body placement; compare
   mounted units with existing cavalry.
2. Inspect the template and pass its local path to imagegen. Fill only the empty
   box, preserving anchors and captions. Specify anatomy, facing, hands and actual
   weapon grips. Match chunky native pixels, outline thickness and supplied flat
   colors. Exclude antialiasing, gradients, fine texture and ground shadows;
   regenerate style drift.
3. Save the unchanged PNG and exact prompt. Inspect before extraction; retain
   the full anatomy and native pixel scale. Failed extraction retains evidence
   for inspection and regeneration.
4. Extract and use [pixel cleaning](../pixel-cleaning/SKILL.md). Review ground
   contact and material ownership visually. Faction colors belong to intended
   cloth. A staff touching the bottom does not excuse floating feet.
5. Show the clean matrix in the requested faction, otherwise the last used faction
   or enemy, enlarged with nearest-neighbor sampling. Keep diagnostics internal
   unless requested. Use [lineup previews](../pixel-lineup/SKILL.md) for trees.

When keeping an existing palette, pass `--output-palette` to the template helper;
style anchors then cannot introduce additional output colors. Intentionally
suspended poses use `--flying`; an output palette carries its existing flying
flag unless explicitly overridden with `--flying` or `--no-flying`.

For placement guides, use `--body-guide X Y W H` and, for raised equipment,
`--staff-guide X Y W H`. Grounded body guides end at the canvas bottom. Guides
are removable magenta marks, not extraction crops; reuse them across related tiers.

```sh
python3 .agents/skills/pinkbox-generation/scripts/template.py tmp/new-unit/template.png \
  --anchors scout ranger archer --label 'New unit'
python3 .agents/skills/pinkbox-generation/scripts/extract.py \
  tmp/new-unit/generated.png tmp/new-unit/template.json tmp/new-unit/extracted
```

Keep raw images, prompts and candidates in ignored `tmp/`. When integration is
part of the task, copy the chosen matrix and palette to the unit folder, register
its lineup ID, and run `npm run art:build` and `npm run art:check`. Gameplay changes
follow the user's requested scope.
