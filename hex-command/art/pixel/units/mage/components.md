# mage

Veteran of adept. Body, face, hands, boots, staff shaft and crystal keep their
approved locations. Gold `G/g` follows the hood and shoulder mantle from
x26,y13 to x12,y31; connected gold shoulder borders sit above the ivory sleeves.
Wider connected gold hems cover x20–24,y41–42 and x29–31,y39–42. The staff
crown is symmetric about x39: matching arms at x37/x41 rise one row
above the original fitting. The six-cell gem remains x38–40,y20–21, blue `J/j`
for player and crimson `K/k` for enemy. The fitting grows; the crystal does not.

See [the progression specifications](../../../../design/progression-art.md).

# Adept C component mapping

The final C design is directly re-traced from source at about 10 source pixels per native cell. Its visible 39×36-cell footprint occupies x4–42,y12–47 on a 48×48 canvas. Coordinates below are zero-based. Both boot soles end on row47, matching the other standing mage. Raster glow and backdrop are removed. The rear boot placement is corrected from the reference to connect anatomically beneath the robe/hip rather than extend the leather side panel.

| Component                     | Native cells / material symbols              | Shape and ownership                                                                                                                                                                                                                       |
| ----------------------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hood and scarf                | x21–33,y12–31; T/t/o (enemy R/r/q)           | Hood turns right around dark cavity. Upper/left cloth faces lit, underside shadowed; scarf connects jaw to chest.                                                                                                                         |
| Face                          | x27–30,y17–23; S                             | 17 warm skin cells form a concealed rightward profile. No extra facial detail.                                                                                                                                                            |
| Trailing cloak                | x4–18,y23–45; T/t/o                          | Broad folds descend screen-left behind bent arm and leg. Main surface follows diagonal; underside/right folds darker.                                                                                                                     |
| Ivory shoulder/sleeve patches | x18–24,y25–30 and x32–35,y27–31; I/i         | Warm cloth attached to torso and forearms, distinct from gold edging.                                                                                                                                                                     |
| Bent left arm and hand        | x15–21,y29–36; L/l and S                     | Brown guard connects shoulder to an 11-cell stepped warm palm beside belt.                                                                                                                                                                |
| Staff grip                    | x37–40,y29–31; S                             | 10 skin cells form palm with inward thumb. Connected leather forearm; hand covers shaft at grip.                                                                                                                                          |
| Belt and torso                | x25–30,y27–32; L/l                           | Broad leather waist separates shoulders from split garment.                                                                                                                                                                               |
| Chest clasp                   | x28–29,y27–29; G/g                           | Small gold ornament at scarf/belt junction.                                                                                                                                                                                               |
| Split front panels            | x20–27 and x30–31,y32–42; I/i with G/g hem   | Left panel falls diagonally left; right compact. Faction cloth fills middle; separated hem and legs retain negative gaps.                                                                                                                 |
| Boots                         | x19–24 and x28–34,y43–47; L/l with O contour | Rear boot sits directly beneath the ivory/gold robe hem, with a short shin, dark sole and three-cell gap to the other boot. The adjacent leather side panel ends in a shaded hem at y40; rear cloak folds show below it at x15–18,y41–42. |
| Staff                         | x38–40,y24–45; W/w with O contour            | Straight wooden shaft runs behind hand; transparent space separates it from coat and boots.                                                                                                                                               |
| Fitting and gem               | x37–41,y19–23; G/g and J/j (enemy K/k)       | Setting, dark cap and six-cell blue/crimson gem are symmetric about x39, the center of the full three-cell shaft silhouette x38–40. The gem occupies x38–40 on two rows; the outer fitting has widths3/5/5/5/3.                           |

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/adept/player.txt art/pixel/units/adept/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.
