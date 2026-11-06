# Artwork enhancement implementation

Read this when changing or debugging `prepare.py` or `fill.py`. The operating branches are in [SKILL.md](SKILL.md).

`prepare.py` delegates template construction to the generation helper with the
approved artwork as anchor 1. It infers unit/terrain/prop kind from canonical
paths, with an explicit override for scratch props. For units it reads the source palette's flying metadata
independently of output color selection, then passes the resolved value explicitly.
An explicit pose override takes precedence. Template and extraction internals are
in [generation implementation](../pinkbox-generation/implementation.md).

`fill.py` selects four-neighbor components from the original matrix. Only mapped
source symbols connect a region. It validates source hashes, component sizes and
region overlap before writing anything, and rejects outline, transparent or faction
symbol mappings. Changes are applied to a copied matrix; the original matrix and
palette are preserved in the candidate's `before/` directory. Comparison rendering
uses the cleaning helper's preview functions.

Terrain and prop fill candidates retain the matrix filename and exact source
files under `before/`. For scratch palettes and canonical integration, follow
[floor palettes](../design-terrain/references/surfaces.md#floor-palettes).

## Fill request contract

The fill JSON requires `matrix_sha256`, `palette_sha256` and `regions`. Each region
has `seed: [x,y]`, `mapping: {old: new}`, `expected_pixels`, `material` and `reason`.
Optional `palette_additions` maps new material symbols to RGBA colors. Use a fresh
output directory under `tmp/`. The helper produces the candidate matrix/palette,
an exact `before/` copy, a change record and comparison image. Outline, transparency
and faction-cloth changes use individually reviewed [pixel edits](../pixel-cleaning/SKILL.md#repair).

## Regeneration options

The helper infers the approved matrix kind; use `--kind prop` for isolated
pieces in scratch directories. Unit templates preserve the approved unit's
flying/swimming pose flag by default.
Use `--flying` or `--no-flying` only when changing that pose. Pass `--output-palette`
when its material colors must stay fixed.
