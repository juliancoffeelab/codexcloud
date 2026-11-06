# Pixel cleaning implementation

Read this when changing or debugging detectors, review rendering or the guarded
editor. The shared workflow is in [SKILL.md](SKILL.md); subject-specific rules are in
[design-unit](../design-unit/SKILL.md) and [design-terrain](../design-terrain/SKILL.md).

`highlight_colors.py` renders at most two explicit RGB colors per check, comparing the
unchanged sprite with a diagnostic recolor. Distinct selected RGB colors get
distinct replacements absent from the original sprite, preserving shade
boundaries rather than flattening the group into a single highlight. A visible
key and the report record the exact mapping. It matches rendered RGB values across
all nontransparent pixels, regardless of palette symbol, position, neighbors or
ownership. Matching pixels retain their original alpha. Presets are editable color
lists, not material classifiers; presets expand into separate pairwise checks.
Custom `--color` values bypass presets and reject more than two distinct colors.
Reports include source hashes, RGB counts and every matching native coordinate.
Highlights are review evidence for the agent, not automatic edit instructions.

`review.py` combines isolated-pixel and outline detectors for units, terrain and
props, validates unit faction alpha and fixed-material consistency, and creates enlarged
contexts with eight-neighbor material-component information. Optional shade
groups and ownership masks are review inputs, not automatic painting masks.

`check_isolated_pixels.py` compares cardinal neighbors by equal RGBA color;
optional shade groups also permit neighboring symbols from the same group.
`check_broken_outlines.py` inspects outline topology, enclosed holes and supplied
material ownership. `pixel_review_image.py` renders every selected finding beside
the complete original matrix. These standalone detectors also accept terrain
frames, as do the combined reviewer and guarded editor.

## Edit request contract

Use individually reviewed edits with `matrix_sha256`, `palette_sha256` and an
`edits` list. Each edit has native `x`, `y`, `from`, `to`, `material` and `reason`.
Outline/transparency changes require `boundary_reviewed: true`.

`apply_edits.py` validates hashes, native integer coordinates, original symbols,
duplicate coordinates and the required review fields before writing a candidate. It copies the
original palette, matrix and optional ownership mask for before/after inspection.
Candidate matrices retain their original filenames. Baselines remain byte-exact; comparisons render the original and candidate.
For terrain scratch palettes and canonical integration, follow
[floor palettes](../design-terrain/references/surfaces.md#floor-palettes).

The script `scripts/build-pixel-art.py` resolves terrain floor colors from canonical terrain paths
and validates road boundary sections independently of the interior through
`require_road_exits`. That validation checks exits, not complete visual continuity
between assembled tiles.

## Standalone detectors

Use these for focused detector investigations:

```sh
python3 .agents/skills/pixel-cleaning/scripts/check_isolated_pixels.py \
  pixel/art/terrain/road/9.txt --output tmp/road-review/strays.json \
  --images-dir tmp/road-review
python3 .agents/skills/pixel-cleaning/scripts/check_broken_outlines.py \
  pixel/art/terrain/road/9.txt --output tmp/road-review/outlines.json \
  --images-dir tmp/road-review
```
