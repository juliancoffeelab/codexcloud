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

Distinguish the backdrop, foreground and foliage of a terrain tile. These are
visual roles, not a requirement for three separate files:

- **Background:** the ground surface: grass, soil, rock or water. Shared `G`/`g`
  colors come from `pixel/art/terrain/floors.json`. Floor changes should not
  recolor trees, structures or stones.
- **Foreground:** grass tufts, trees, roads, rocks, crossing stones, villages and
  structures. Use separate material symbols and reusable transparent 64×64
  pieces. These are composed over ground, with procedural variation suited to
  their complexity; not every object needs the same amount of randomization.

Within that artwork, distinguish the main foreground feature (buildings,
mountains, crossing stones) from foliage and small surface decoration (grass,
shrubs, wave marks). Sea tiles have the water backdrop and wave decoration, with
no main foreground object. Mountains have the rock backdrop and mountain
foreground, generally without foliage. Most other tiles have both a main feature
and surrounding foliage. Plains have ground decoration without a main structure.
Changing a backdrop palette does not supply missing foliage. A burned building
does not imply that the whole hex has burned ground or no vegetation.

Check the complete tile on an actual map through the shared map renderer:
backdrop joins with neighbors, foreground scale and placement, and foliage density
must all fit the surroundings. An isolated prop or matching ground color alone
is not enough. Use the same lighting as the intended map when judging the result.

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

Do not manufacture shoreline ground by slicing a finished hex with an arbitrary
coordinate cutoff. This produces grass scraps and geometric wedges, not an
authored shore. Prefer an existing whole ground tile with foreground pieces;
design new shoreline artwork deliberately only when needed. Fitting inside the
hex is necessary but does not establish visual quality. Inspect actual joins on
both banks, not just isolated pieces or one cropped bank.

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

## Add terrain to the game

Follow existing terrain implementations before adding a new representation.
Bridge and ford demonstrate directional frames; plains, forest, hill and ford
demonstrate foreground layouts. A new terrain uses the existing tile `terrain`
identifier. Do not add a parallel artwork selector to tiles or extend the save
format just to select its sprite.

1. Register its stable ID, name, movement cost, passability, description and any
   protection in `rust/data/game/terrain.json`. Add the ID to `Terrain` and
   `as_str()` in `rust/src/data/ids.rs` and `TerrainType` in `src/game/types.ts`.
   Check race/unit `terrainModifiers` if the terrain needs special treatment;
   do not inherit another terrain's behavior merely because its artwork is similar.
2. Add the native `tile.txt` and material `palette.json` under
   `pixel/art/terrain/<id>/`. Shared `G`/`g` ground colors come from the map tile's
   explicit `floor`; without one they render transparent. Keep foreground pieces
   in `pixel/art/terrain/props/<id>/<piece>/` and compose them in `layouts.json`.
   Every painted piece must fit inside its hex. The build bakes complete frames;
   runtime does not draw props outside those frames.
3. If directional frames are needed, follow bridge/ford: add source matrices,
   register their allowed stems in `scripts/build-pixel-art.py`, and select them
   in `src/presentation/terrain-connections.ts`. Match connection validation in
   `rust/src/data/state.rs`. Connection edges must have a stated meaning: for
   landing/ferry, the single `e` or `w` connection identifies the landward bank;
   `east.txt` faces east and `tile.txt` faces west. Key directional layouts by the
   same source stems. Extend the existing generic directional checks as needed.
4. Add the terrain's inspection description in `src/ui/terrain-view.ts` and
   preview color in `src/ui/geometry.ts`. Existing movement, ownership, healing
   and recruitment behavior must be checked explicitly; a new ID does not
   automatically acquire the special behavior of villages or keeps.
5. Add it to the developer terrain showcase. Keep campaign/map experiments in
   ignored draft data until promotion is requested. Maps select the terrain with
   `terrain`, its ground with `floor`, and supported orientation with `connections`.
6. Run `npm run art:build`, `npm run art:check`, `npm run content:check`,
   `npm run check` and the shared `npm test` as appropriate to the changed code.
   Use the Rust toolchain skill for Rust/WASM checks. Inspect standalone hexes,
   all supported orientations and neighboring tiles through the normal terrain
   renderer. Preserve original evidence; do not substitute map-canvas overlays
   for an implementation or add save/reload checks for artwork-only changes.
