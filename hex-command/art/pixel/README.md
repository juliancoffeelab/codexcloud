# Native pixel artwork

Keep editable pixel matrices and their native PNGs here. The Scout is currently
48×48 pixels, with its boots on the bottom row. Its approved design follows the
original Scout: cloth shirt and trousers, leather belt, bracers and boots,
skin-colored hands, silver knife, and windblown hair and cloak.

Reusable imagegen preparation, extraction, tier recipes and preview commands are
documented in [the artwork workflow index](WORKFLOWS.md).

Future faction bodies, poses and equipment are listed in the
[anatomy and weapon references](reference/faction-anatomies.md).

The [faction expansion](reference/faction-expansion/README.md) contains 63 new
anatomies, both faction colors through L3, saved generation masks/prompts,
cleanup recipes, numbered comparison grids and review sheets.

The [chosen zombie lineup](lineups/zombies.json) keeps eight designs active in
`units/`. Unused zombie studies are archived under
[reference/zombies/](reference/zombies/README.md), with the approved branch layout
and review grid. Artwork selection and tier assignments do not define gameplay
stats or advancement rules.

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
`art/pixel/terrain/`. The script writes exactly two public pixel images:

- `public/assets/units-pixel.png`: every canonical unit source in both factions,
  including finished designs awaiting gameplay registration, plus the neutral
  civilian worker portrait.
- `public/assets/terrain-pixel.png`: all 12 terrain types, including the bridge.

The generated `src/presentation/pixel-art.ts` records each image's dimensions,
content hash and frame coordinates. The regular game renderer loads these public
assets. Packing copies pixels without resizing; every frame retains its full
native canvas, separated by a transparent gutter. Foot units use 48×48 canvases;
cavalry uses 64×64. Atlas coordinates are integers, with each frame's own dimensions.

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
adept gem colors changing. Terrain and portraits remain 48×48. Unit frames retain
their native rectangular dimensions; rendering and packing never crop or resize them.

Finished human branch artwork lives in `units/`, alongside the existing game
artwork. [The source catalog](units/catalog.json) records the promoted designs,
their proposed tiers, canvas sizes and construction studies. Every source has
both faction matrices, its palette and native PNGs. Catalog tiers describe the
artwork; gameplay stats and advancement definitions remain separate. Reference
folders retain generated inputs, workflows and comparison boards, rather than
duplicate editable copies of the promoted designs.

The chosen design reference is [C](reference/units-c.png). It records the agreed
appearance; editable matrices define the actual native pixels. Reviews and the
pre-generation component specifications remain in temporary task storage at
`/workspace/artifacts/pixel-C/`.

Skeleton and wolf references are tracked in [skeleton-wolf](reference/skeleton-wolf/README.md).
The selected warrior is B; the selected wolf is A. B's rabid wolf and D's wolf
are retained for possible future use.

Project nicknames are recorded in the [reference dictionary](reference/dictionary.md).

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
