# Pinkbox enhancement implementation

## Guarded fills

The fill JSON requires `matrix_sha256`, `palette_sha256` and `regions`. Each region
has `seed: [x,y]`, `mapping: {old: new}`, `expected_pixels`, `material` and `reason`.
Optional `palette_additions` maps new symbols to RGBA colors.

Four-neighbor fills stop at unmapped materials. The helper rejects stale hashes,
changed counts, overlapping regions and outline, faction or transparent fills.
Individually reviewed shading uses the cleaning skill's
[edit contract](../pixel-cleaning/implementation.md).

## Shape templates

The approved unit is anchor 1, followed by exactly two style anchors and one empty
output. Each tier uses its own 64×64 template. `--output-palette` locks existing
materials when required. Generation and extraction use the
[pinkbox contract](../pinkbox-generation/implementation.md).

## Floor recoloring

Terrain floor formats are described in
[terrain artwork](../../../docs/terrain-artwork.md). A palette change reuses the
matrix. Render previews through `read_palette(matrix, floor=...)` and
`rgba_pixels(matrix, floor=...)` to resolve the selected floor colors.
