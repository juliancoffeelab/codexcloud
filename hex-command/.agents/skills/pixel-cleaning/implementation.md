# Pixel cleaning implementation

Read this when changing or debugging detectors, review rendering or the guarded
editor. Unit and terrain workflows are in [SKILL.md](SKILL.md).

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
