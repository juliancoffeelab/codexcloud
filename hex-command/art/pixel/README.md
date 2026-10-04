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

Build the game's pixel assets from the matrices and original artwork:

```sh
npm run art:build
npm run art:check
```

`npm run dev` and `npm run build` regenerate these assets before starting.
The script reads matrices from `art/pixel/units/` and the recorded original crops
in `art/pixel/legacy/`. It writes exactly two public pixel images:

- `public/assets/units-pixel.png`: all units and both factions. Existing matrices
  replace their matching old sprites; every other sprite retains its original pixels.
- `public/assets/terrain-pixel.png`: base terrain, campaign terrain and shallow water.

The generated `src/presentation/pixel-art.ts` records each image's dimensions,
content hash and frame coordinates. The regular game renderer loads these public
assets. Packing copies pixels without resizing; matrix frames retain their full
48×48 canvas and legacy frames retain their fractional crop offsets.

There are currently thirteen matrix-backed frames. The enemy Scout and the other
units without matrices use their original artwork within the same unit atlas.
Terrain has no matrices yet; its original artwork is packed into the terrain atlas.

Paired factions use identical component geometry, with only faction cloth and
adept gem colors changing. All unit matrices must remain 48×48.

The chosen design reference is [C](reference/units-c.png). It records the agreed
appearance; editable matrices define the actual native pixels. Reviews and the
pre-generation component specifications remain in temporary task storage at
`/workspace/artifacts/pixel-C/`.

Skeleton and wolf references are tracked in [skeleton-wolf](reference/skeleton-wolf/README.md).
The selected warrior is B; the selected wolf is A. B's rabid wolf and D's wolf
are retained for possible future use.

The renderer uses only the Python standard library. Matrix PNGs use flat colors and
hard transparency; legacy artwork retains its original colors and alpha. Store native resolution here; any later enlarged exports
must use nearest neighbor. Game atlases belong in `public/assets`.
