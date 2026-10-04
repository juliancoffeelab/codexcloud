# Necromancer: editable native sprite

`player.txt` and `enemy.txt` are the complete 48×48 source matrices. Their local
`palette.json` defines flat RGBA material colors. The PNGs and game atlas are
generated from these sources; edit the matrices rather than the PNGs.

Coordinates are zero-based. Both factions have identical geometry. H/S/T are
teal cloth; h/s/t are the corresponding crimson cloth. Light comes from the
upper left. There are no gradients, glow or antialiased pixels.

| Component               | Placement / symbols              | Material                                                            |
| ----------------------- | -------------------------------- | ------------------------------------------------------------------- |
| Hood and scarf          | x15–34, y8–25; H/S/T             | Faction cloth with a shaded lower fold above the shoulder           |
| Face                    | x22–32, y13–22; B/b/E/N          | Warm bone, near-black eye sockets and tooth gaps, brown nose cavity |
| Hood opening            | E                                | Darkness inside the hood, distinct from cloth shadow H              |
| Flowing cloak           | x5–22, y24–44; H/S/T             | Broad cloth folds and a stepped trailing hem                        |
| Free arm and hand       | x16–23, y25–32; L/l, K/k         | Leather wrist guard and warm exposed skin                           |
| Staff-side arm and hand | x34–39, y27–31; L/l, K/k         | Leather wrist guard and hand gripping wood                          |
| Split robe panels       | x20–25 and x28–31, y33–44; I/i   | Ivory cloth, separated by faction cloth, with gold edging           |
| Belt and clasp          | x22–30, y26–33; L/l, G/g         | Leather and gold                                                    |
| Boots                   | x14–20 and x28–34, y44–47; L/l/O | Leather, with separated feet on row 47                              |
| Staff                   | x37–38, y24–47; W/w              | Brown wood                                                          |
| Staff skull and collar  | x36–42, y15–23; B/b/O, G/g       | Bone and gold                                                       |

The approved isolated skull is saved as the 11×10 study
`art/pixel/parts/necromancer-head/head.txt`. Its O symbol maps to E in the complete
unit palette, preserving the same RGBA color without changing the body's O
outline color. It is placed at (22, 13); its final extra bottom row was removed.
The trimmed hood study is saved in `art/pixel/parts/necromancer-upper/` and is
placed at (11, 6). These studies retain the working components; the complete unit
matrices remain the authoritative game sources.

Run `npm run art:build` to regenerate sprites and the atlas. `npm run art:check`
verifies that saved PNGs and atlas metadata match their source matrices.
