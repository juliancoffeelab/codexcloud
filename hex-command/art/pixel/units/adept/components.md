# Adept C component mapping

The final C design is directly re-traced from source at about 10 source pixels per native cell. Its visible 39×36-cell footprint occupies x4–42,y12–47 on a 48×48 canvas. Coordinates below are zero-based. Both boot soles end on row47, matching the other standing mage. Raster glow and backdrop are removed.

| Component                     | Native cells / material symbols              | Shape and ownership                                                                                                       |
| ----------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Hood and scarf                | x21–33,y12–31; T/t/o (enemy R/r/q)           | Hood turns right around dark cavity. Upper/left cloth faces lit, underside shadowed; scarf connects jaw to chest.         |
| Face                          | x27–30,y17–23; S                             | 17 warm skin cells form a concealed rightward profile. No extra facial detail.                                            |
| Trailing cloak                | x4–15,y23–45; T/t/o                          | Broad folds descend screen-left behind bent arm and leg. Main surface follows diagonal; underside/right folds darker.     |
| Ivory shoulder/sleeve patches | x18–24,y25–30 and x32–35,y27–31; I/i         | Warm cloth attached to torso and forearms, distinct from gold edging.                                                     |
| Bent left arm and hand        | x15–21,y29–36; L/l and S                     | Brown guard connects shoulder to an 11-cell stepped warm palm beside belt.                                                |
| Staff grip                    | x37–40,y29–31; S                             | 10 skin cells form palm with inward thumb. Connected leather forearm; hand covers shaft at grip.                          |
| Belt and torso                | x25–30,y27–32; L/l                           | Broad leather waist separates shoulders from split garment.                                                               |
| Chest clasp                   | x28–29,y27–29; G/g                           | Small gold ornament at scarf/belt junction.                                                                               |
| Split front panels            | x20–27 and x30–31,y32–42; I/i with G/g hem   | Left panel falls diagonally left; right compact. Faction cloth fills middle; separated hem and legs retain negative gaps. |
| Boots                         | x15–20 and x28–34,y43–47; L/l with O contour | Two compact leather boots, dark soles and clear transparent gap.                                                          |
| Staff                         | x38–40,y24–45; W/w with O contour            | Straight wooden shaft runs behind hand; transparent space separates it from coat and boots.                               |
| Fitting and gem               | x37–42,y19–23; G/g and J/j (enemy K/k)       | Small gold rim encloses a six-cell blue/crimson gem beneath dark cap.                                                     |

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/adept/player.txt art/pixel/units/adept/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.
