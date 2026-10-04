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

Build the native adept, bone archer, boar, necromancer, skeleton and wolf pairs into the
game's atlas, including content-hashed source metadata and native 48×48 frames:

```sh
npm run art:build
npm run art:check
```

Both factions use identical component geometry, with only faction cloth and
adept gem colors changing. All unit matrices must remain 48×48. The atlas packs
six columns and two faction rows without scaling any pixel.

The chosen design reference is [C](reference/units-c.png). It records the agreed
appearance; editable matrices define the actual native pixels. Reviews and the
pre-generation component specifications remain in temporary task storage at
`/workspace/artifacts/pixel-C/`.

Skeleton and wolf references are tracked in [skeleton-wolf](reference/skeleton-wolf/README.md).
The selected warrior is B; the selected wolf is A. B's rabid wolf and D's wolf
are retained for possible future use.

The renderer uses only the Python standard library. PNGs use flat colors and
hard transparency. Store native resolution here; any later enlarged exports
must use nearest neighbor. Game atlases belong in `public/assets`.
