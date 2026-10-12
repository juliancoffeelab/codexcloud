---
name: pinkbox-enhancement
description: Revise an approved Hex Command unit or tier through guarded material recoloring, or pinkbox when its shapes need redesign.
---

# Pinkbox enhancement

Tell the user when applying this skill. Work from the game root with Python 3
and Pillow. Save the approved matrix and palette as an exact baseline in `tmp/`.
Identify the requested changes to anatomy, pose, grips, equipment, materials and
faction cloth. Preserve the shared pixel scale; progression needs readable shapes.

For palette changes, inspect actual surfaces before recoloring. Preserve outlines,
alpha, skin, cloth and grips. Recolor base and shadow together in coherent clusters
following overlaps and curves. Do not infer material from color alone. Fill-file
schemas, guards and terrain-floor handling are in [implementation.md](implementation.md).

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/fill.py \
  pixel/art/units/humans/adept/sprite.txt tmp/upgrade/fill.json tmp/upgrade/candidate
```

For new anatomy, silhouettes or equipment shapes, use
[pinkbox generation](../pinkbox-generation/SKILL.md). Specify the changes in the
prompt and save the unchanged output. Prepare a separate native 64×64 template
for each tier:

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/prepare.py \
  scout tmp/scout-upgrade/template.png --label 'Scout L2' --style-anchors ranger archer
```

Use [pixel cleaning](../pixel-cleaning/SKILL.md) before showing the result and
[lineup previews](../pixel-lineup/SKILL.md) for progression. Show clean matrix
renders in the requested faction and keep diagnostics internal unless requested.
When integration is part of the task, replace the canonical matrix and palette,
then rebuild and check artwork. Do not repeat confirmation already established.
