---
name: design-unit
description: Use to make pixel art for new unit line
---

# Design a unit line

Read the requested race tree in `pixel/art/units/lineups/` and compare approved
units. Establish the base unit, branches and tiers. Progression should read
through equipment, pose and silhouette at the shared battlefield scale.

1. Use [pinkbox-generation](../pinkbox-generation/SKILL.md) to create a base with
   three approved references and `--kind unit`. Specify anatomy, facing, hands,
   grips and equipment. Save the unchanged generated PNG and exact prompt.
2. Use [pixel-cleaning](../pixel-cleaning/SKILL.md) after extraction. Inspect
   brown, ivory, skin and steel separately, including shades and every occurrence.
   Judge ownership by anatomy/surface: brown belongs to leather, wood or hair;
   misplaced brown does not become a gold shadow.
3. Use [pinkbox-enhancement](../pinkbox-enhancement/SKILL.md) for tiers/branches
   based on the approved unit. Choose manual matrix/material edits for controlled
   local changes or another pinkbox generation for substantial shape, equipment
   or pose changes. Generation can handle large changes more reliably than
   extensive hand reshaping, but costs another extraction and cleanup pass and
   can alter correct details. Inspect and clean either route.
4. Preview the whole line with [pixel-lineup](../pixel-lineup/SKILL.md), then
   inspect each unit on the battlefield. Show clean previews in the requested
   faction, otherwise the last used faction or enemy.

Every character/portrait is exactly 64×64 native pixels. Most bodies are under
40 painted pixels tall; compare cavalry and giants with approved peers. Raised
weapons do not establish body height. Never enlarge commanders or resize art to
emphasize tiers.

Grounded bodies contact native row 63 with no transparent rows below. A staff
touching the edge is not body contact. Preserve every pixel during alignment:
translate without cropping/resizing. Hovering/swimming poses require
`"flying": true`; preserve it through upgrades. Faction symbols belong to intended
cloth and retain matching alpha across factions. Use the approved human scout
as a finish benchmark without copying its pose.

When integration is requested, copy chosen `sprite.txt`/`palette.json` files to
unit folders and register the line in its race tree. Run `npm run art:build` and
`npm run art:check`. Artwork does not automatically authorize gameplay stats,
recruitment changes or publication.

A line with one tier uses the same tree structure: a base and an empty `branches`
list. Do not invent promotion tiers or separate actor categories to register it.
An entry can explicitly declare additional `factions` for visual palettes;
`passive` supplies white cloth for neutral actors, including captive villagers.
Ordinary player and enemy frames remain required and all variants retain the same
silhouette. Palette names describe reusable colors, not scenario roles.
