---
name: pinkbox-generation
description: Generate a new Hex Command pixel unit using three approved matrix anchors and one pinkbox output, then extract and review its 64×64 matrix.
---

# Pinkbox generation

Work from the game root with Python 3 and Pillow. Read `AGENTS.md` and the
requested race tree in `pixel/art/units/lineups/`. Use current approved matrices
as references. Tell the user when applying this skill.

1. Prepare exactly three occupied anchors and one empty bottom-right output in
   a 2×2 template. Each character is exactly 64×64; most bodies stay below 40
   painted pixels tall. For related tiers, use the same `--body-guide X Y W H`
   and foot baseline at native row 63; grounded body guides must end at the
   canvas bottom. `--staff-guide X Y W H` gives raised equipment separate
   space. Guides are removable magenta, never an extraction crop. Compare cavalry
   with existing mounted units. Use `--flying` only for an intentionally
   suspended pose; extraction records `"flying": true` in its `palette.json`.
   `--output-palette` inherits that flag unless overridden with `--no-flying`.
2. Inspect the template, then give its local path to imagegen. Fill only the
   empty box with one complete unit; preserve anchors, layout and captions.
   Specify anatomy, facing, hands and actual weapon grips. Require chunky native
   pixels, the anchors' outline thickness, flat supplied colors and magenta
   background: no antialiasing, gradients, fine texture or ground shadow.
   Each logical pixel occupies 8×8 image pixels in the 512×512 output box;
   resizing by imagegen must not introduce finer detail. Regenerate style drift.
3. Save the unchanged generated PNG and exact prompt. Inspect before extracting.
   Do not clip or resize anatomy to fit. Use `--output-palette` when preserving
   an existing unit's colors; other anchors must not widen that output palette.
4. Extract, then use [pixel cleaning](../pixel-cleaning/SKILL.md).
   Grounded extraction must touch row 63; failure retains evidence for inspection
   and regeneration, without silently shifting the generated artwork. When
   aligning an approved existing sprite, translate it without losing any painted
   pixel. A staff touching the bottom does not excuse feet floating above it.
   Review material ownership visually: faction symbols `^`/`*` belong only to intended cloth,
   never skin, hair, bone, outlines or weapons. They resolve through
   `pixel/art/factions.json`, outside each material palette.
5. Show the rendered matrix in the requested faction (otherwise the last used
   faction, or enemy), enlarged with nearest-neighbor sampling. Keep diagnostics
   internal unless requested. Use [pixel lineup](../pixel-lineup/SKILL.md) for trees.

```sh
python3 .agents/skills/pinkbox-generation/scripts/template.py tmp/new-unit/template.png \
  --anchors scout ranger archer --label 'New unit'
python3 .agents/skills/pinkbox-generation/scripts/extract.py \
  tmp/new-unit/generated.png tmp/new-unit/template.json tmp/new-unit/extracted
```

Helpers write fresh scratch files under `tmp/`. For authorized integration,
copy the chosen `sprite.txt` and material `palette.json` into
`pixel/art/units/<race>/<unit-id>/` and register the ID in the race tree. Keep
raw images, prompts and edit history in ignored `tmp/`, outside merge contents.
No replay recipe is needed: the matrix is the
source. Run `npm run art:build` and `npm run art:check`. Gameplay changes require
scope from the user.
