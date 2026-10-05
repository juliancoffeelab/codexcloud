# Native pixel artwork

Keep editable pixel matrices and their native PNGs here. The Scout is currently
48×48 pixels, with its boots on the bottom row. Its approved design follows the
original Scout: cloth shirt and trousers, leather belt, bracers and boots,
skin-colored hands, silver knife, and windblown hair and cloak.

Each character in a `.txt` matrix is one pixel. `.` is transparent. Symbols map
to exact RGBA colors in that unit's `palette.json`; uppercase and lowercase are
distinct. Scout uses the shared palette in this directory. Other units keep
their material symbols and palettes beside their matrices; see `components.md`
in each unit directory for component locations and material ownership.

The shared Scout palette uses these symbols:

| Symbols       | Material                               |
| ------------- | -------------------------------------- |
| `B`, `b`      | Leather and hair                       |
| `T`, `t`      | Teal cloth                             |
| `S`           | Skin                                   |
| `C`, `c`, `q` | Off-white cloth, including deep shadow |
| `W`, `w`, `g` | Silver blade                           |

Edit the matrix, then regenerate its PNG from the project directory:

```sh
python3 scripts/render-pixel.py art/pixel/units/scout/player.txt
```

Check that all saved PNGs match their matrices:

```sh
python3 scripts/render-pixel.py --check
```

Build the game's pixel assets from the matrices:

```sh
npm run art:build
npm run art:check
```

`npm run dev` and `npm run build` regenerate these assets before starting.
The script reads matrices and palettes from `art/pixel/units/` and
`art/pixel/terrain/`. Original sheets in `art/pixel/legacy/` are references for
editing. The script writes exactly two public pixel images:

- `public/assets/units-pixel.png`: all 34 unit ranks, both factions (68 frames), plus
  the neutral civilian worker portrait.
- `public/assets/terrain-pixel.png`: all 12 terrain types, including the bridge.

The generated `src/presentation/pixel-art.ts` records each image's dimensions,
content hash and frame coordinates. The regular game renderer loads these public
assets. Packing copies pixels without resizing; every frame retains its full
48×48 canvas, separated by a transparent gutter. Atlas coordinates are integers.

During artwork loading, the game extracts each frame into its own native-size
PNG image, with smoothing disabled for pixels. These images are cached and
published together when ready. Every renderer receives only a complete frame
image. The full atlas is never inserted into the page or SVG, so viewport size,
letterboxing or overflow cannot expose neighboring frames. Normal and low-poly
artwork use the same extraction boundary, preserving their existing source crops.

Terrain palettes use `G` for their ground base, or `W` for water-only backgrounds.
The generated metadata includes that exact colour for the battlefield's vector
hex beneath each sprite. Transparent pixels along the stepped raster silhouette
therefore expose matching ground, without a second grey or dark jagged border.

Each unit directory has `player.txt`, `enemy.txt`, the rendered PNGs and
`components.md`. Each terrain directory has `tile.txt`, its PNG, palette and
component description. Keep changes in these sources; rebuild the public atlases
with the command above. Civilian artwork is in `portraits/`; its `portrait.txt`
and palette render a portrait without defining a recruit or battlefield unit.
There is no raster fallback in the pixel build.

Paired factions use identical component geometry, with only faction cloth and
adept gem colors changing. All unit and terrain matrices must remain 48×48.

The chosen design reference is [C](reference/units-c.png). It records the agreed
appearance; editable matrices define the actual native pixels. Reviews and the
pre-generation component specifications remain in temporary task storage at
`/workspace/artifacts/pixel-C/`.

Skeleton and wolf references are tracked in [skeleton-wolf](reference/skeleton-wolf/README.md).
The selected warrior is B; the selected wolf is A. B's rabid wolf and D's wolf
are retained for possible future use.

The renderer uses only the Python standard library. Matrix PNGs use flat colors and
hard transparency. Store native resolution here; enlarged exports must use nearest
neighbor. Game atlases belong in `public/assets`.

Palette membership alone does not establish that a pixel is correct. Check each
material against the reference and neighboring components: ivory in a teal cloak,
gold outside its trim, or skin in a fur paw is a transcription error even when the
palette contains that color. Keep broad shaded planes connected; single pixels
need a visible purpose such as an eye, fang or blade tip. Inspect the sprite on
both dark and green ground. Source-to-atlas checks establish faithful rendering
of the matrices; visual reference checks establish the artwork itself.
