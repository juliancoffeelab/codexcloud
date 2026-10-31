# Pinkbox generation implementation

Read this when changing or debugging `template.py` or `extract.py`. Normal
operation is described in [SKILL.md](SKILL.md).

`template.py` emits a 1024×1168 PNG and a manifest containing source matrix
hashes, palette, guides and the output slot. The fixed 2×2 geometry includes
caption rows and palette swatches; the output rectangle is `[512, 576, 512, 512]`.
Output-palette selection replaces the combined anchor palette rather than
extending it. Unit faction symbols resolve through `pixel/art/factions.json`. Terrain and
prop outputs use material palettes without faction ownership. Anchors can be
any native 64×64 matrix; canonical terrain references leave shared floor symbols
transparent when no floor is explicitly selected. The optional slot `kind` is `unit`,
`terrain` or `prop`; older manifests default to `unit`. Unit-only options are
body/staff guides and flying metadata.

`extract.py` validates the manifest geometry, unit faction colors and generated PNG
aspect ratio. It maps the output rectangle into the generated image, classifies
transparent or magenta pixels as background, and quantizes other pixels to the
nearest allowed RGB color. Each native cell uses the most common foreground
symbol when foreground coverage reaches 35%; otherwise it becomes transparent.

Extraction retains the original PNG, manifest, cropped slot, provisional matrix
and metadata before checking empty outputs and subject-specific boundaries. Grounding checks
opaque pixels on row 63; they cannot distinguish a foot from equipment. A failed
check leaves evidence and omits the accepted matrix. Units produce
`unit-1/sprite.txt`; terrain and props produce `terrain-1/tile.txt` and
`prop-1/tile.txt`. Terrain extraction rejects opaque pixels outside the centered
pointy-top hex; it never silently clips them. Props are transparent pieces whose
placement inside the hex is validated during composition. Neither terrain nor
props inherit unit grounding or flying metadata.

`--object-guide` records a hard native footprint in the slot's guides. Extraction
checks every foreground pixel against it after retaining provisional evidence;
overflow writes `footprint-overflow.json` and prevents accepting the matrix.
