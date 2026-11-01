# Terrain artwork implementation

Terrain sources live in `pixel/art/terrain/<terrain>/`. Unconnected terrain uses
`tile.txt`. Every connected terrain frame uses its explicit `<mask>.txt`, with
bits 1, 2, 4, 8, 16 and 32 representing e, se, sw, w, nw and ne. Special filename
aliases and default masks are forbidden. Connected map tiles must supply
`connections`; an explicit empty array selects mask 0 where supported. Missing
or invalid connections fail validation and rendering.

Signposts use mask 9. Roads use masks 1–63; village masks have up to three bits, including 0.
Bridge and ford masks are 9, 18 and 36; landing and ferry masks are 1 and 8.
Castles declare an explicit `walls` array rather than `connections`.
Their numbered sources encode wall directions in six bits. `variants.json` registers supported readable configurations.
Every tile includes complete towers at its own boundary endpoints; there are no
neighbor-sharing codes. The enclosed bell has one frame and higher authored z.
Castle courtyard is registered gameplay terrain: movement cost 1,
all incoming damage reduced by 1, no income, healing or recruitment.

The artwork build reads finished matrices. It does not synthesize roads or modify
source artwork.

## Complete castle tiles and overflow

The offline fort generator reads map-independent definitions from
`pixel/art/terrain/castle-library.json` and writes finished matrices containing
paving, walls and complete endpoint towers. It reads
readable version-three definitions without a map. Maps select existing geometry
through an explicit `walls` array. Neither map authoring nor runtime
creates missing artwork or inspects neighbors to distribute tower pieces.
The artwork build checks finished registries against the reusable definitions.

Every terrain image contains its intended ground. The shared painter draws images
only, without a polygon underfill or missing-image substitute. Ordinary native
hex edges must cover the 56×48 lattice without transparent raster gaps.

The logical hex remains 64×64 at the normal native scale. Explicit `canvas.json`
metadata supplies padded image dimensions, anchor and drawing depth. The build
enforces a finite native canvas contract: only `castle-wall` and
`castle-bell` may use 80×96, anchored at (40,48); all other artwork stays 64×64.
Artwork metadata cannot introduce another canvas profile. Changing the profile
requires an explicitly requested contract change across validation, authoring
tools and documentation, rather than enlarging a sprite to fit. Depth may be
any integer (courtyards use 1 and the bell keep uses 2) and never changes native scale. Padding preserves
projected height without resizing artwork. Canvas and SVG render the same
finished images in depth/row/column order. No terrain geometry, fragments or
neighbor-dependent overlays are assembled at runtime. Fog and lighting remain
the only permitted map post-processing.

## Floor palettes

`pixel/art/terrain/floors.json` defines named `palettes` with `G`/`g` base/shade
pairs. A tile's optional `floor` explicitly selects its ground, including on roads
and villages. Without a floor, `G`/`g` are transparent, including in artwork
references; there is no terrain-based default. Floor symbols do not belong in local material
palettes. Tree foliage and grass tufts use other symbols and retain their colors
when the floor changes. Different floor palettes reuse the same matrix.

## Props and layouts

Forest, hill and plains base matrices contain the floor and ground decoration.
Ford bases contain water and ripples; crossing stones are props. Reusable pieces
live in `terrain/props/<terrain>/<piece>/tile.txt` with material palettes, on
transparent native 64×64 canvases.

`<terrain>/layouts.json` defines integer placements drawn in depth order. Each
painted piece fits within the hex. Directional layouts are keyed by source frame
stem (`9`, `18`, `36` for crossings); use the arrangement matching the selected frame.
Ford stones suggest the crossing direction through uneven steps, varied outer
stones and scattered smaller stones, without fixed endpoints.

Select arrangements deterministically from map and tile identity. Preserve native
size and lighting; rendering does not mirror, resize or randomly reposition props.

Landing and ferry use separate terrain IDs. Both have west-facing `8.txt` and
east-facing `1.txt` water bases with corresponding jetty layouts. Their
single `connections` entry (`w` or `e`) indicates the landward bank. Ferry adds a
boat alongside the jetty within the same hex; it does not transport units.
Select a water `floor`; the jetty reaches the landward hex edge and connects to
the neighboring bank, without cutting grass fragments into the water tile.
House uses a ground base and one intact house prop, without village income or
healing. These use the same build, frame selection and map rendering as the
other terrains; there is no per-tile artwork selector.

Mountain and cave share the `mountain-rock` floor and native mountain foreground.
Mountain has three approved massif layouts and is impassable. Cave adds a
separate dark entrance prop to that foreground and permits entry at movement
cost 2. Scenario encounter data determines cave occupants and rewards; the
terrain itself does not spawn wildlife or grant items.

Burned-village artwork composes two burned-home foreground pieces over the ordinary
`green-plain` floor. The ruined home is bounded by the intact house's painted
footprint. Standalone house terrain remains the ferryman's home; map drafts use
village terrain for settlements.
