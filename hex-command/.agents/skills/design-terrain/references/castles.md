# Multi-hex structures

Read before authoring or repairing paving, walls or towers. The
[permanent terrain contract](../../../../AGENTS.md#permanent-prohibition-on-map-post-processing)
applies throughout.

Define reusable tiles before laying out a map. Castle tile data declares
`walls` as an explicit edge-direction array, alongside its `terrain` and `floor`.
An opening is an omitted wall edge; there is no dedicated gate tile or aperture
field. Empty wall arrays are explicit. Ordinary connected terrains continue using
`connections`; castles must not carry that field, `gates`, or `wallCorners`.

Each tile contains its own paving, declared walls and complete towers at
boundary endpoints. Edge i has endpoints i and i+1 in the clockwise corner order
NE, SE, S, SW, NW, N. Every tower uses the same fixed native corner position.
Never partition towers between neighbors, move them according to neighboring
walls, or generate a different frame because of surrounding tiles. Coincident
complete tower pixels may overdraw identically. An enclosed keep owns all six
complete towers and uses higher authored z than courtyard tiles. Bake paving,
then the lower half of the bell prop, then walls and towers, then
the upper half of the prop. The 64×64 prop splits at row 32 without moving pixels.
Both halves stay inside one finished tile; this offline composition does not
change its z or create runtime layers.

The shared painter draws only complete images in z order. It never draws a ground
polygon beneath them; ground belongs in the source artwork. Runtime and offline
map rendering use the same painter. Verify order independence by reversing and
shuffling tiles within each z level while retaining the keep’s higher z.

Frames remain 80×96, anchored at (40,48), around the 64×64 logical hex. Do not
resize artwork or extend this canvas. Paving phase uses the native 56×48 lattice,
so ground and walls continue under all three neighbor translations. Inspect empty
and occupied layouts, inward corners, three-way joins and an enclosed keep.

The offline [generator](../scripts/generate-fort.py) reads the map-independent
`pixel/art/terrain/castle-library.json` version-three definitions and produces finished
matrices. It accepts no map, never reads neighboring tiles and never runs inside
the artwork build or runtime:

```sh
python3 .agents/skills/design-terrain/scripts/generate-fort.py tmp/fort-candidate \
  --palette pixel/art/terrain/castle-wall/palette.json \
  --paving-floor castle-paving \
  --catalog pixel/art/terrain/castle-library.json
```

The source filename is the six-bit wall-direction mask. Authors supply readable
directions, not that filename. The registered library includes all 64 wall
configurations and one enclosed bell keep.
Unsupported combinations fail rather than generating map-specific artwork.
