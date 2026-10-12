# Terrain artwork implementation

Terrain sources live in `pixel/art/terrain/<terrain>/`. `tile.txt` is the default
tile; bridge and ford diagonals use `ne-sw.txt` and `nw-se.txt`. Decorative roads
are complete terrain tiles in `terrain/road/`, with their own grass and winding
paths. The default road tile runs west–east. Other variants use `<mask>.txt`, with
bits 1, 2, 4, 8, 16 and 32 representing e, se, sw, w, nw and ne.

The artwork build reads finished matrices. It does not synthesize roads or modify
source artwork.

## Floor palettes

`pixel/art/terrain/floors.json` defines named `palettes` with `G`/`g` base/shade
pairs and terrain `defaults`. A tile's optional `floor` overrides its default,
including on roads and villages. Floor symbols do not belong in local material
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
