# Pinkbox enhancement implementation

Read this when changing or debugging `prepare.py` or `fill.py`. Input contracts
and operating branches are in [SKILL.md](SKILL.md).

`prepare.py` delegates template construction to the generation helper with the
approved unit as anchor 1. It reads the source palette's flying metadata
independently of output color selection, then passes the resolved value explicitly.
An explicit pose override takes precedence. Template and extraction internals are
in [generation implementation](../pinkbox-generation/implementation.md).

`fill.py` selects four-neighbor components from the original matrix. Only mapped
source symbols connect a region. It validates source hashes, component sizes and
region overlap before writing anything, and rejects outline, transparent or faction
symbol mappings. Changes are applied to a copied matrix; the original matrix and
palette are preserved in the candidate's `before/` directory. Comparison rendering
uses the cleaning helper's preview functions.
