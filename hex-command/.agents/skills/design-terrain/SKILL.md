---
name: design-terrain
description: Use when authoring, revising or integrating terrain artwork, connected geometry, or underground ambience and exploration.
---

# Design terrain

Work from the game root. Follow the [terrain contract](#terrain-contract) and
match the [shared native pixel scale](../design-unit/SKILL.md#native-artwork).
Deliver complete, prebuilt tile frames selected by explicit map data.
Generate reusable tile libraries from tile definitions before constructing maps.
Artwork generators accept no map input. Maps use the shared terrain selector;
an unsupported selection fails instead of generating another tile.

1. Establish the reusable terrain family, floor, connection meanings and
   intended lighting. Compare approved terrain with troops at native scale. For
   ambience-only work, establish covered cells, lights and exploration state.
2. Read the reference for the work being done:
   - [Surfaces](references/surfaces.md) for ground, foreground, foliage or source formats.
   - [Connections](references/connections.md) for directional tiles and crossings.
   - [Castles](references/castles.md) for shared paving, walls, openings and towers.
   - [Caves](references/caves.md) for passages, solid rock, underground ambience and exploration.
   - [Composition](references/composition.md) for layouts, density or shorelines.
   - [Integration](references/integration.md) when adding an ID or installing artwork.
3. For artwork changes, author candidate matrices offline. Use
   [pinkbox-generation](../pinkbox-generation/SKILL.md) for ordinary ground hexes
   or transparent props, and [artwork-enhancement](../artwork-enhancement/SKILL.md)
   for revisions. Review generated or edited candidates with
   [pixel-cleaning](../pixel-cleaning/SKILL.md). Retain original evidence in ignored tmp/.
4. Preview the complete tiles through the
   [shared map renderer](../design-campaign/map-drafting.md) with neighbors and
   troops, using the intended floor and lighting. Inspect every supported join
   and orientation; verify that interior paving, exposed boundaries and shared
   junctions agree with explicit tile data. Inspect empty and occupied structures.
5. Integrate only within the requested scope, following the integration reference.
   Artwork installation, gameplay promotion and publication are distinct actions.

The result is ready when the requested frames exist, their sources pass the
applicable artwork checks, and assembled map views show continuous joins at the
native troop scale. Report unsupported configurations or remaining visual defects.

## Terrain contract

Map post-processing is absolutely forbidden except for weather and atmospheric effects such as fog and lighting. Under no circumstances may this rule be changed, weakened, bypassed or given an exception. Runtime must select complete, prebuilt terrain frames from each tile’s explicit data. It must never assemble structures across the map, inspect neighboring geometry to inject artwork, distribute fragments, or paint terrain features onto other tiles. All terrain geometry and composition must be finished offline and baked into the selected tile frames.

Preserve each finished frame's native canvas, anchor and drawing depth. Read
`require_canvas`, `terrain_canvas` and the approved family/profile constants in
`scripts/build-pixel-art.py` for supported dimensions. Artwork metadata cannot
introduce another canvas profile or family. A profile change requires an explicit
user request and coordinated validator, authoring-tool and documentation changes;
drawing depth never authorizes resizing the logical hex or artwork.

Connected terrain declares geometry explicitly: castles use `walls`, other
connected terrain uses `connections`. Missing or invalid geometry fails validation
and rendering; never infer, default or backfill it. Ordinary connected sources use
`<mask>.txt`; registered road caps use `<exits>-end-<ends>.txt`. Read supported
stems in the builder and variant registries. Implicit orientations, aliases such
as `tile.txt` for a connected mask and unsuffixed connected frames are forbidden.
Unconnected terrain and individual props may use `tile.txt`.
