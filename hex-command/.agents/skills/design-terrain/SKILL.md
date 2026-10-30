---
name: design-terrain
description: Use to make pixel art for new terrain
---

# Design terrain

Read [terrain artwork](../../../docs/terrain-artwork.md). Compare approved
terrain at native scale and in a map. Use exactly 64×64 native canvases with a
centered pointy-top regular hex, at the troops' pixel scale. Match the chunky
shapes, flat material colors and lighting.

## Separate ground and foreground

Design two layers, rather than a monolithic illustration for every tile:

- **Background:** the ground surface: grass, soil, rock or water. Shared `G`/`g`
  colors come from `pixel/art/terrain/floors.json`. Floor changes should not
  recolor trees, structures or stones.
- **Foreground:** grass tufts, trees, roads, rocks, crossing stones, villages and
  structures. Use separate material symbols and reusable transparent 64×64
  pieces. These are composed over ground, with procedural variation suited to
  their complexity; not every object needs the same amount of randomization.

The existing implementation is partly separated: forest, hill and plains bases
still include ground decoration; ford bases include water/ripples; props are
placed through `layouts.json`. Roads and villages use finished directional
frames, some with ground baked into their source. Respect these formats while
designing new pieces toward separate layers. Do not claim all foreground already
has a procedural generator.

## Generate and clean source pieces

1. Decide the ground, foreground pieces, density and connections. Select related
   terrain references; unit artwork can help judge scale.
2. Use [pinkbox-generation](../pinkbox-generation/SKILL.md) with `--kind terrain`
   for a ground hex or complete constrained frame, and `--kind prop` for an
   isolated foreground piece. Terrain fits the hex; props stay transparent
   around the object. Neither uses character grounding or faction symbols.
3. Save the raw generation and prompt. Extract native matrices, then use
   [pixel-cleaning](../pixel-cleaning/SKILL.md). Inspect rocks, foliage, roofs,
   ground and their shades separately, as well as the whole composition.
4. Use [pinkbox-enhancement](../pinkbox-enhancement/SKILL.md) for controlled
   palette/shape revisions. Regeneration requires cleanup again.

## Procedural foreground composition

Compose foreground variation procedurally from approved pieces and legal
placements, at different levels of complexity:

- Grass and small stones: sparse, inexpensive variation with open walking space.
- Forests and hills: several pieces, clustered densities, varied silhouettes and
  coherent depth ordering. Mountains must read differently from hill stones.
- Fords: irregular stepping stones that communicate the crossing direction.
- Roads: constrained connected paths with correct exits.
- Villages/structures: fewer constrained arrangements with sensible entrances and
  approaches, rather than scattering building fragments.

Keep variation deterministic from map/tile identity. Currently the build emits
approved `layouts.json` arrangements and runtime selects among them; it does not
randomly place objects every frame or synthesize road artwork. Procedural
authoring may generate candidate layouts; persist and validate accepted
arrangements. More complex generators are separate implementation work when
needed by the requested terrain.

Place pieces at native integer coordinates without mirroring/resizing; keep
painted pixels inside the hex and draw overlaps in depth order. Preview several
densities and seeds, retaining intentional clearings. Do not cut up a finished
building or mountain illustration to manufacture variation.

## Validate and integrate

Preview pieces, assembled tiles and neighbors: repeated ranges/forests,
coastlines, forks and crossings. Road exits must meet with matching position,
width and normal tangent. Matching connection names and schema checks alone do
not establish visual continuity. Check with troops and the game's night
treatment; keep scene-level night shading out of source material colors.

When integration is requested, add matrices/palettes under
`pixel/art/terrain/<terrain>/`, props under `terrain/props/<terrain>/<piece>/`,
and approved placements in `layouts.json`. Canonical ground colors stay in
`floors.json`; remove resolved scratch `G`/`g` from local palettes. Run
`npm run art:build` and `npm run art:check`, then inspect assembled maps.
Gameplay terrain types, movement rules, map promotion and publication follow
their corresponding task scope.
