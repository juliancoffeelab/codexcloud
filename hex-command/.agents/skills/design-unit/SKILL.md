---
name: design-unit
description: Use when designing or integrating a unit artwork line, promotion branches or shared sprite scale.
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
3. Use [artwork-enhancement](../artwork-enhancement/SKILL.md) for tiers/branches
   based on the approved unit. Choose manual matrix/material edits for controlled
   local changes or another pinkbox generation for substantial shape, equipment
   or pose changes. Generation can handle large changes more reliably than
   extensive hand reshaping, but costs another extraction and cleanup pass and
   can alter correct details. Inspect and clean either route.
4. Preview the whole line with [pixel-lineup](../pixel-lineup/SKILL.md), then
   inspect each unit on the battlefield. Show clean previews in the requested
   faction, otherwise the last used faction or enemy.

Apply the project’s native canvas and grounding rules in
[battlefield contract](../battlefield-contract/SKILL.md). Compare bodies with approved peers at battlefield
scale; raised equipment does not establish body height. Judge body contact
anatomically—a staff touching the ground line does not establish standing feet.
Preserve declared hovering/swimming poses through upgrades. Faction symbols belong to intended
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
