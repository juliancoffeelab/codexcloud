---
name: pinkbox-generation
description: Generate a new Hex Command pixel unit using three approved matrix anchors and one pinkbox output, then extract and review its 64×64 matrix.
---

# Pinkbox generation

Tell the user when applying this skill. Work from the game root with Python 3
and Pillow. Read `AGENTS.md` and the requested race tree. Use approved matrices
as references. Template geometry, guide flags and extraction behavior are in
[implementation.md](implementation.md).

1. Prepare three occupied anchors and one empty bottom-right output. Keep each
   character at native 64×64 and grounded feet at row 63. Match related tiers'
   body placement; compare mounted units with existing cavalry.
2. Inspect the template and pass its local path to imagegen. Fill only the empty
   box, preserving anchors and captions. Specify anatomy, facing, hands and actual
   weapon grips. Match chunky native pixels, outline thickness and supplied flat
   colors. Exclude antialiasing, gradients, fine texture and ground shadows;
   regenerate style drift.
3. Save the unchanged PNG and exact prompt. Inspect before extraction. Preserve
   existing colors when requested and do not clip or resize anatomy to fit.
4. Extract and use [pixel cleaning](../pixel-cleaning/SKILL.md). Review ground
   contact and material ownership visually. Faction colors belong to intended
   cloth. A staff touching the bottom does not excuse floating feet.
5. Show the clean matrix in the requested faction, otherwise the last used faction
   or enemy, enlarged with nearest-neighbor sampling. Keep diagnostics internal
   unless requested. Use [lineup previews](../pixel-lineup/SKILL.md) for trees.

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
