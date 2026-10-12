# Pinkbox generation implementation

Read this when changing or debugging `template.py` or `extract.py`. Normal
operation is described in [SKILL.md](SKILL.md).

`template.py` emits a 1024×1168 PNG and a manifest containing source matrix
hashes, palette, guides and the output slot. The fixed 2×2 geometry includes
caption rows and palette swatches; the output rectangle is `[512, 576, 512, 512]`.
Output-palette selection replaces the combined anchor palette rather than
extending it. Faction symbols resolve through `pixel/art/factions.json`.

`extract.py` validates the manifest geometry, faction colors and generated PNG
aspect ratio. It maps the output rectangle into the generated image, classifies
transparent or magenta pixels as background, and quantizes other pixels to the
nearest allowed RGB color. Each native cell uses the most common foreground
symbol when foreground coverage reaches 35%; otherwise it becomes transparent.

Extraction retains the original PNG, manifest, cropped slot, provisional matrix
and metadata before checking empty outputs and ground contact. Grounding checks
opaque pixels on row 63; they cannot distinguish a foot from equipment. A failed
check leaves evidence and omits the accepted `sprite.txt`.
