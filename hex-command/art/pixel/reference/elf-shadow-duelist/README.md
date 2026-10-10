# Shadow duelist restyle

Only Duelist L2 and L3 change. The approved Two knifer L1 remains unchanged.
Both outfits are modeled after the matching Shadow stalker archer. L2 uses a
moss-green hood/jerkin, leather harness and bracers, ivory lining and leather
boots. L3 adds a darker forest hood/cowl and layered woodland clothing. Only the
two knife blades use metal: neither tier has plate armor or shiny fittings.
The grounded wide stance, two short knives and their grips remain recognizable.

## Current woodland outfit

The `woodland/l2` and `woodland/l3` folders contain the current generation evidence
and guarded corrections. Each used a separate three-reference, one-fill pinkbox
with the current Duelist, matching Shadow stalker and approved L1 Two knifer.
The archer supplies the whole outfit, not just a hood or a palette recolor.

Each folder saves the previous exact canonical matrix/palette, unchanged generated
image and prompt, template geometry, extraction and intermediate matrices/palettes,
three guarded correction logs and final detector report. Cleanup traced local
cloth/leather surfaces, removed quantization flecks and repaired actual outlines
without bridging every diagonal. L2's generated faction-colored lap panel becomes
green tunic lining; faction colors remain on cape and collar only. Unused metal
and artifact shades were removed from the canonical palettes.

Both final matrices have no exposed material borders, enclosed holes or unsupported
outline pixels. Stray/edge previews, both factions and faction debug colors were
inspected; the scripts still flag normal diagonal contours and small clothing
details. L2 changes 672 pixels and L3 changes 730 relative to the hood-only stage.

## Superseded hood-only stage

Each tier used a separate three-reference, one-fill pinkbox: its approved Duelist,
the matching Shadow stalker, and the approved L1 Two knifer. The unchanged generated
image, exact prompt and extraction manifest remain evidence. Extraction introduced
material noise into the body, so only the hood design was translated into simple
matrix surfaces; the approved body was retained instead of denoising it again.

The older top-level tier folders save exact old matrices/palettes, guarded hood and finishing
edits, the intermediate hood matrix/palette, and the final detector report.
L3 adds the Shadow stalker's dark green `D` material before applying its guarded
hood edits. The finishing pass removes one detached outline pixel, restores five
collar/cape boundary pixels and consolidates four hood shadow pixels per tier.
There are 205 net changed pixels per tier. Stray and edge previews were inspected
in both factions and faction debug colours. No exposed material borders,
enclosed transparent holes or unsupported outline pixels remain; the detector
still flags intentional diagonals and small details on the retained body.

Canonical sources are `art/pixel/units/elf-flanker-l2` and `elf-flanker-l3`,
protected by [reviewed elf hashes](../elf-reviewed/sources.json).
Follow the project [enhancement skill](../../../../.agents/skills/pinkbox-enhancement/SKILL.md)
and [cleaning skill](../../../../.agents/skills/pixel-cleaning/SKILL.md) for future changes.
The matrices, rather than saved generation templates, are the next references.
