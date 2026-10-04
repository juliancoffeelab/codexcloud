# Native pixel artwork

Keep editable pixel matrices and their native PNGs here. The Scout is currently
48×48 pixels, with its boots on the bottom row. Its approved design follows the
original Scout: cloth shirt and trousers, leather belt, bracers and boots,
skin-colored hands, silver knife, and windblown hair and cloak.

Each character in a `.txt` matrix is one pixel. `.` is transparent. Symbols map
to exact RGBA colors in `palette.json`; uppercase and lowercase are distinct.

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

The renderer uses only the Python standard library. PNGs use flat colors and
hard transparency. Store native resolution here; any later enlarged exports
must use nearest neighbor. Game atlases belong in `public/assets`.
