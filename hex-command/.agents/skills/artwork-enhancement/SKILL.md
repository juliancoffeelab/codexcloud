---
name: artwork-enhancement
description: Use to revise approved pixel artwork, including manual edits, recoloring, regeneration, unit upgrades and terrain changes.
---

# Artwork enhancement

For units and portraits, follow [native artwork](../design-unit/SKILL.md#native-artwork).
For terrain/props, follow the [terrain contract](../design-terrain/SKILL.md#terrain-contract).

Work from the game root with Python 3 and Pillow. Save the approved matrix and
palette as an exact baseline in `tmp/`.
Identify the requested shape, pose, equipment, material or composition changes.
Use [design-unit](../design-unit/SKILL.md) or
[design-terrain](../design-terrain/SKILL.md) for subject-specific requirements.
Preserve the shared pixel scale.

## Choose an enhancement route

There are two ways to enhance approved artwork:

- **Manual:** edit individually reviewed native pixels or explicitly selected
  material regions. This preserves approved details for small, controlled
  changes; use the guarded editor for shape/boundary edits and the fill helper
  for material recoloring.
- **Another pinkbox generation:** use approved artwork as the first anchor
  and prompt for the requested changes. This is often safer for substantial
  silhouette, anatomy or equipment changes than reshaping everything by hand.
  It can still damage correct details and always needs extraction, comparison
  with the approved baseline, and another cleanup pass. Generation does not
  make enhancement free or remove the cleanup work.

Both routes finish with [pixel-cleaning](../pixel-cleaning/SKILL.md).

## Manual changes

For palette changes, inspect actual surfaces before recoloring. Keep skin and
cloth colors unless the user asks to change them. Preserve outlines, alpha and
grips, and materials outside the requested change. Recolor base and shadow together
in coherent clusters following overlaps and curves. Identify materials by their
surfaces and anatomy, rather than color alone.

For a guarded material fill, read the [fill request contract](implementation.md#fill-request-contract).
Use a fresh candidate directory under ignored tmp/.

```sh
python3 .agents/skills/artwork-enhancement/scripts/fill.py \
  pixel/art/units/humans/adept/sprite.txt tmp/upgrade/fill.json tmp/upgrade/candidate
```

## Regeneration

For new anatomy, silhouettes, equipment or terrain shapes, use
[pinkbox generation](../pinkbox-generation/SKILL.md). Specify the changes in the
prompt and save the unchanged output. Prepare a separate native 64×64 template
for each tier, with the approved unit first and two style anchors:

```sh
python3 .agents/skills/artwork-enhancement/scripts/prepare.py \
  scout tmp/scout-upgrade/template.png --label 'Scout L2' --style-anchors ranger archer
```

The helper preserves the approved unit's flying/swimming pose flag unless the
requested change requires an override. For kind, pose and palette overrides,
read [regeneration options](implementation.md#regeneration-options).

## Terrain and integration

For terrain floor recoloring, scratch previews or canonical palette integration,
follow [floor palettes](../design-terrain/references/surfaces.md#floor-palettes).
Inspect affected tiles after rebuilding artwork.

Use [pixel cleaning](../pixel-cleaning/SKILL.md) before showing the result and
[lineup previews](../design-unit/SKILL.md#preview-existing-artwork) for progression. Show clean matrix
renders in the requested faction and keep diagnostics internal unless requested.
When integration is part of the task, replace the canonical matrix and palette,
then rebuild and check artwork. For helper changes or fill/template bugs, read
[implementation.md](implementation.md).
