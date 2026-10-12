# Pinkbox generation implementation

The template has exactly three occupied anchors and one empty bottom-right output
in a 2×2 layout. Each logical character is 64×64; the generated output box is
512×512, with each logical pixel occupying 8×8 image pixels. Image-generation
resizing must not introduce finer detail.

`--body-guide X Y W H` places removable magenta body guides. Reuse the same guide
and native row-63 baseline for related tiers; grounded guides end at the canvas
bottom. `--staff-guide X Y W H` gives raised equipment separate space. Guides
describe placement and do not define an extraction crop.

`--flying` declares an intentionally suspended pose. Extraction records that
artwork-only flag in `palette.json`; `--output-palette` inherits it unless
`--no-flying` overrides it. An output palette preserves the chosen unit's colors;
anchor palettes do not widen it.

Keep the original PNG and prompt as evidence when extracting. A grounded output that fails to
touch row 63 is retained for inspection and regeneration, rather than silently
shifted. Existing approved sprites can be aligned by translating every painted
pixel without cropping. Equipment at the bottom does not establish foot contact.

Faction symbols `^`/`*` resolve through `pixel/art/factions.json`, outside the
material palette. They belong to faction cloth, not skin, hair, bone, outlines or
weapons. Integrated sources are `sprite.txt` and `palette.json` in
`pixel/art/units/<race>/<unit-id>/`, referenced by the race lineup tree. Raw images,
prompts and edit history stay in ignored `tmp/`; the matrix is the build input.
