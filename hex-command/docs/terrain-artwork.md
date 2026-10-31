# Terrain artwork implementation

Terrain sources live in `pixel/art/terrain/<terrain>/`. `tile.txt` is the default
tile; bridge and ford diagonals use `ne-sw.txt` and `nw-se.txt`. Decorative roads
are complete terrain tiles in `terrain/road/`, with their own grass and winding
paths. The default road tile runs west–east. Other variants use `<mask>.txt`, with
bits 1, 2, 4, 8, 16 and 32 representing e, se, sw, w, nw and ne.

Village sources are explicitly numbered combinations of buildings and road
connections: `0.txt` has no road connections and `9.txt` has the east–west road.
There is no generic village `tile.txt` or unsuffixed village frame.

The artwork build reads finished matrices. It does not synthesize roads or modify
source artwork.

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
stem (`tile`, `nw-se`, `ne-sw`); use the arrangement matching the selected frame.
Ford stones suggest the crossing direction through uneven steps, varied outer
stones and scattered smaller stones, without fixed endpoints.

Select arrangements deterministically from map and tile identity. Preserve native
size and lighting; rendering does not mirror, resize or randomly reposition props.

Landing and ferry use separate terrain IDs. Both have west-facing `tile.txt` and
east-facing `east.txt` water bases with corresponding jetty layouts. Their
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
