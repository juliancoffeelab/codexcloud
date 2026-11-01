---
name: design-terrain
description: Use when authoring, revising or integrating terrain artwork, connected variants, or shared tile geometry.
---

# Design terrain

Work from the game root. Apply the native scale and canvas rules in
[AGENTS.md](../../../AGENTS.md), including its
[permanent terrain contract](../../../AGENTS.md#permanent-prohibition-on-map-post-processing).
Deliver complete, prebuilt tile frames selected by explicit map data.
Generate reusable tile libraries from tile definitions before constructing maps.
Artwork generators accept no map input. Maps use the shared terrain selector;
an unsupported selection fails instead of generating another tile.

Runtime map painting belongs entirely in the map rendering worker, including
terrain, rivers, lighting and fog. It returns one completed background bitmap;
the UI only swaps it beneath live units and interaction overlays. Do not add
main-thread painting, separate atmospheric canvases or fallback renderers.
Missing worker/image APIs and rendering failures must surface as explicit errors.
This changes where complete tile frames are painted, never the prohibition on
runtime terrain assembly. Offline draft renderers remain independent tools.

Native canvas dimensions are a finite, enforced contract, not a convenience
setting in artwork metadata. Ordinary terrain is exactly 64×64. Only
`castle-wall`, `castle-gate` and `castle-bell` use the approved 80×96 canvas,
anchored at (40,48), without resizing the 64×64 logical hex or its pixels.
Do not enlarge canvases, move anchors, or add another size/family to make artwork
fit. A different profile requires an explicit user request and a coordinated
change to the validator, authoring tools and documented contract. Drawing depth
may be any authored integer; it does not authorize larger artwork.

1. Establish the reusable terrain family, floor, connection meanings and
   intended lighting. Compare approved terrain with troops at native scale.
2. Read the reference for the work being done:
   - [Surfaces](references/surfaces.md) for ground, foreground, foliage or source formats.
   - [Connections](references/connections.md) for directional tiles and crossings.
   - [Castles](references/castles.md) for shared paving, walls, gates and towers.
   - [Composition](references/composition.md) for layouts, density or shorelines.
   - [Integration](references/integration.md) when adding an ID or installing artwork.
3. Author candidate matrices offline. Use
   [pinkbox-generation](../pinkbox-generation/SKILL.md) for ordinary ground hexes
   or transparent props, and [pinkbox-enhancement](../pinkbox-enhancement/SKILL.md)
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
Source-format details and floor ownership are in
[terrain artwork](../../../docs/terrain-artwork.md).
