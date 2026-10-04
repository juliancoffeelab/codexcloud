# Necromancer: selected C reconstruction

The editable source is `player.txt` / `enemy.txt`: exactly 48 rows of 48 symbols.
The flat RGBA `palette.json` defines exact material colors. Rendering needs
only the matrices and palette, not the reference. Coordinates below are 0-based.

| C component                       | Native placement / symbols       | Material and orientation                                           |
| --------------------------------- | -------------------------------- | ------------------------------------------------------------------ |
| Hood and scarf                    | x20–33, y12–26; H/S/T            | Teal cloth, deep hood shadow; enemy h/s/t crimson                  |
| Small severe face                 | x26–33, y16–22; B/b/O            | Warm bone, dark sockets and cavity, turned right, simple jaw       |
| Screen-left flowing cloak         | x5–22, y24–44; H/S/T             | Faction cloth, broad coherent shadow folds, stepped trailing hem   |
| Bent free arm and guard           | x16–23, y25–32; L/l and K/k      | Brown leather guard above warm exposed palm/thumb                  |
| Staff-side forearm/guard and hand | x34–39, y27–31; L/l and K/k      | Small brown wrist guard joins the warm hand gripping wood          |
| Split front robe panels           | x20–25 and x28–31, y33–44; I/i   | Warm ivory cloth separated by faction cloth; gold lower edging G/g |
| Belt and neck clasp               | x22–30, y26–33; L/l, G/g         | Leather belt, restrained gold fittings                             |
| Two compact boots                 | x14–20 and x28–34, y44–47; L/l/O | Brown leather, separated feet; baseline row47                      |
| Staff                             | x37–38, y24–47; W/w              | Straight brown wood, separate from the body and boots              |
| Small skull finial/collar         | x36–42, y15–23; B/b/O, G/g       | Separate warm bone skull, dark sockets, gold collar below          |

Faction matrices have identical geometry and material locations. Only the three
faction cloth symbols swap. Transparent pixels are `.`; all palette alpha values
are 0 or 255. Light comes from upper-left/front on both variants. There are no
glow, background, spell-effect or raster-noise pixels.

Reviewed corrections: jaw cell (30,21) uses bone shadow; finial cell (37,16) uses
dark contour; wrist band (34–35,28–30) uses leather; feet and staff have transparent
baseline separation.

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/necromancer/player.txt art/pixel/units/necromancer/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.

Face gaze follows the user’s menacing-stare reference in an eight-column bone
face (x26–33): paired angular sockets, a two-cell bone bridge, nose at (30,20),
and tooth gaps at (28,21) and (31,21). All three cavities have bone on every
side, including the continuous lower jaw at y22. The extra face column reveals
bone inside the existing hood silhouette; head height, body and staff are unchanged.
