---
name: design-unit
description: Use when creating, revising or integrating unit artwork, promotion branches or shared sprite scale, or previewing and comparing faction lineups.
---

# Unit artwork and lineups

Work from the game root with Python 3 and Pillow. Read the requested tree under
`pixel/art/units/lineups/` and choose only the requested branch:

- **New base:** establish its role, branches and tiers against approved peers.
  Use [pinkbox-generation](../pinkbox-generation/SKILL.md) with three approved
  references and `--kind unit`, then [pixel-cleaning](../pixel-cleaning/SKILL.md).
  Specify anatomy, facing, hands, grips and equipment; retain the exact prompt
  and unchanged generated PNG.
- **Existing artwork or branch:** start from the approved matrix and palette;
  use [artwork-enhancement](../artwork-enhancement/SKILL.md) for the requested
  edits, recoloring or regeneration. Preserve unrelated branches and progression.
- **Preview or comparison:** use [Preview existing artwork](#preview-existing-artwork).
  Render current matrices directly; generation, artwork builds, cleanup and source
  edits are outside this branch.
- **Integrate approved artwork:** use [Integration](#integration), without creating
  a replacement base or regenerating accepted artwork.

## Native artwork

Character/unit sprites and portraits are permanently 64×64 native pixels. Keep
that canvas through generation, extraction, cleaning and integration; never crop
or resize artwork. Terrain, troops and commanders share one native pixel scale:
no per-unit scaling to emphasize leaders.

Native row 63 is the ground line. Grounded bodies stand on it with no transparent
rows underneath; align by translation within the canvas, preserving every painted
pixel. Intentionally suspended hovering or swimming poses declare `"flying": true`
in `palette.json`; absent or false means grounded. Preserve the pose through upgrades.

Most bodies should be below 40 painted pixels tall; cavalry and giants may be
slightly taller. Compare mounted units with mounted peers. Raised weapons or
staffs do not establish body height or anatomical foot contact.

## Design and review

Progression should read through equipment, pose and silhouette. Inspect brown,
ivory, skin and steel separately, including shades and every occurrence. Judge
material ownership by anatomy and surface: brown belongs to leather, wood or hair;
misplaced brown does not become a gold shadow. Faction symbols belong to intended
cloth and retain matching alpha across palettes. Use the approved human scout as
a finish benchmark without copying its pose.

For new or revised artwork, inspect each unit at battlefield scale and preview the
complete progression. Finish with clean renders and explicitly state remaining
visual defects. Apply integration only when it is part of the request.

## Preview existing artwork

A lineup is a faction. Render the current progression tree without rebuilding
artwork or replaying recipes:

```sh
python3 .agents/skills/design-unit/scripts/preview.py --list
python3 .agents/skills/design-unit/scripts/preview.py \
  --race humans --layout tree --side enemy --output tmp/humans-lineup
```

Prefer the tree layout: show each shared L1 once with its L2/L3 branches and
connectors, with commander tiers before ordinary families. Use `--layout grid`
for row-per-branch comparisons. Show the requested faction, otherwise the last
used faction or enemy; show both only when requested. Keep complete native frames
at one shared integer nearest-neighbor preview scale, without shifting baselines
or adding padding. Use `--help` for display options.

Tree pagination keeps a family and its connectors together, so a family can exceed
the requested row limit. Hiding names/boxes leaves arrows visible. Open and show
the generated PNGs; retain JSON/hash reports internally unless requested. Outputs
stay in ignored `tmp/`; this branch never writes matrices, palettes or game assets.

## Integration

Copy approved `sprite.txt`/`palette.json` files to unit folders and register the
line in its race tree. A one-tier line has a base and empty `branches`; do not
invent tiers or actor categories to register it. Additional `factions` select
visual palettes; `passive` is white. Player/enemy frames remain required, and all
variants retain the same silhouette. Palette names are colors, not scenario roles.

Run `dune build --root=. @artwork` and `python3 -m unittest discover -s scripts -p 'test_pixel_*.py'`, then inspect the delivered line
at battlefield scale. Completion requires the requested registrations, valid
sources and matching clean previews. Artwork integration alone does not authorize
gameplay stats, recruitment changes or publication.
