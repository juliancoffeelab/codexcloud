# Pixel cleaning implementation

Read this when changing or debugging detectors, review rendering or the guarded
editor. Unit and terrain workflows are in [SKILL.md](SKILL.md).

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

`review.py` combines the isolated-pixel and outline detectors for unit sprites,
validates faction alpha and fixed-material consistency, and creates enlarged
contexts with eight-neighbor material-component information. Optional shade
groups and ownership masks are review inputs, not automatic painting masks.

`check_isolated_pixels.py` compares cardinal neighbors by equal RGBA color;
optional shade groups also permit neighboring symbols from the same group.
`check_broken_outlines.py` inspects outline topology, enclosed holes and supplied
material ownership. `pixel_review_image.py` renders every selected finding beside
the complete original matrix. These standalone detectors also accept terrain
frames; the combined unit reviewer and editor require `sprite.txt`.

`apply_edits.py` validates matrix/palette hashes, native integer coordinates,
original symbols, duplicate coordinates, material/reason fields and explicit
review of outline/transparency changes before writing a candidate. It copies the
original palette, matrix and optional ownership mask for before/after inspection.

The artwork builder resolves terrain floor colors from canonical terrain paths
and validates road boundary sections independently of the interior through
`require_road_exits`. That validation checks exits, not complete visual continuity
between assembled tiles.
