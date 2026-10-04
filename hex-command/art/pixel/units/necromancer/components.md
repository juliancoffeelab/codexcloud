# Necromancer: editable native sprite

`player.txt` and `enemy.txt` are the complete 48×48 source matrices. Their local
`palette.json` defines flat RGBA material colors. The PNGs and game atlas are
generated from these sources; edit the matrices rather than the PNGs.

Coordinates are zero-based. Both factions have identical geometry. H/S/T are
teal cloth; h/s/t are the corresponding crimson cloth. Light comes from the
upper left. There are no gradients, glow or antialiased pixels.

| Component               | Placement / symbols              | Material                                                                  |
| ----------------------- | -------------------------------- | ------------------------------------------------------------------------- |
| Hood and scarf          | x18–34, y12–25; H/S/T            | Faction cloth with a shaded lower fold above the shoulder                 |
| Face                    | x25–32, y15–22; B/b/E/N          | Warm bone, near-black eye sockets, faint tooth shadows, brown nose cavity |
| Hood opening            | E                                | Darkness inside the hood, distinct from cloth shadow H                    |
| Flowing cloak           | x5–22, y24–44; H/S/T             | Broad cloth folds and a stepped trailing hem                              |
| Free arm and hand       | x16–23, y25–32; L/l, B/b         | Leather wrist guard and bone hand                                         |
| Staff-side arm and hand | x34–39, y27–31; L/l, B/b         | Leather wrist guard and bone hand gripping wood                           |
| Split robe panels       | x20–25 and x28–31, y33–44; I/i   | Ivory cloth, separated by faction cloth, with gold edging                 |
| Belt and clasp          | x22–30, y26–33; L/l, G/g         | Leather and gold                                                          |
| Boots                   | x14–20 and x28–34, y44–47; L/l/O | Leather, with separated feet on row 47                                    |
| Staff                   | x37–38, y24–47; W/w              | Brown wood                                                                |
| Staff skull and collar  | x36–42, y15–23; B/b/O, G/g       | Bone and gold                                                             |

The compact isolated skull is saved as the 8×8 study
`art/pixel/parts/necromancer-head/head.txt`. Its O symbol maps to E in the complete
unit palette, preserving the same RGBA color without changing the body's O
outline color. It is placed at (25, 15). The near eye is a 3×2 socket at
x26–28/y18–19; the far eye is 2×2 at x31–32/y18–19. The nose cavity
is N at (30, 20). Two faint b tooth shadows at (27, 21) and (29, 21)
replace the former black gaps, preserving an almost continuous ivory jaw.
The trimmed hood study is saved in `art/pixel/parts/necromancer-upper/` and is
placed at (11, 6). These studies retain the working components; the complete unit
matrices remain the authoritative game sources.

Run `npm run art:build` to regenerate sprites and the atlas. `npm run art:check`
verifies that saved PNGs and atlas metadata match their source matrices.
