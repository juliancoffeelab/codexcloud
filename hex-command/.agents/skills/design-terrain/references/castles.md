# Multi-hex structures

Read before authoring or repairing shared paving, walls, gates or tower junctions.
The project’s [permanent terrain contract](../../../../AGENTS.md#permanent-prohibition-on-map-post-processing) applies throughout.

Define reusable tiles and what each connection means before laying out a map.
Maps select existing tiles through the shared terrain selector, as roads and
fords do. Review the whole assembled structure, empty and occupied, at
the troop and neighboring terrain scale. Interior ground, boundary walls, gates
and towers must make being inside visually unambiguous. Shared edges continue the
interior; exposed edges carry the boundary. Gates deliberately interrupt it.
Shared surfaces must preserve texture phase under the hex grid's neighbor
translations. Prove a plain patch across all three adjacency axes before emitting
the configuration family; restarting an arbitrary brick grid per tile breaks
joins. Each tile draws only its own exposed wall edges. Their endpoints are
fixed hex corners; do not infer or copy neighboring wall sections from the local
connection mask. Wall caps, walks and shaded faces extend inward into the owning
hex, so the complete cross-section fits without borrowing pixels from another
tile. Towers are constructed offline in shared coordinates, partitioned between
adjoining castle tiles in ground coordinates, then projected into complete
images. Map authors select an existing variant's explicit connection and corner
data; neither map authoring nor runtime creates missing artwork.
Plain ground tiles remain unchanged. No neighboring tile receives injected
fragments at runtime. Explicit padding and drawing depth preserve the height
of the baked portions where they project outside the logical hex.
Paving covers the tile independently of its wall mask, including an enclosed
keep inside a courtyard.
Review assembled corners with different neighboring wall configurations as well
as isolated frames. Match the renderer's native 56×48 lattice (28-pixel half-column)
so connecting pixels preserve their raster phase.

The offline [fort generator](../scripts/generate-fort.py) reads the explicit,
map-independent `pixel/art/terrain/castle-library.json` definitions and produces
finished tile candidates. Its inputs are tile definitions, materials and paving
colors. It accepts no map or layout, writes no map data, and never runs inside
the artwork build or runtime. For example:

```sh
python3 .agents/skills/design-terrain/scripts/generate-fort.py tmp/fort-candidate \
  --palette pixel/art/terrain/castle-wall/palette.json \
  --paving-floor castle-paving \
  --catalog pixel/art/terrain/castle-library.json
```

The catalog defines a finite supported set, not every theoretical corner
combination. Each generated family registry must match those definitions, and
the artwork build rejects missing or extra finished frames. A new reusable tile
is an explicit library-authoring change; drawing a new map cannot extend the
library. Unsupported combinations remain errors in the shared terrain selector.
