C boar, native 48×48, facing screen-right. Coordinates below are zero-based.
The occupied bounds are x2–45, y20–47; hooves end at the common row47 baseline.

- Hide H/S/D: stocky rump, shoulder hump and lowered head. H is the main warm brown; S is the broad upper/front shoulder plane; D is the belly and far-leg shadow. No fur-noise pixels.
- Mane M with O outline: a dark stiff ridge over the back and down the shoulder, y20–33. Tail M/O runs diagonally behind the rump to x2,y42. Neither is faction cloth.
- Ear D/O: a folded vertical wedge at x28–30,y27–30.
- Four legs: near hind leg x4–14 ends in a compact hoof at x4–8,y45–47; far hind leg x14–21 is shadowed and ends at x17–21,y45–47. Near front leg x21–28 ends at x24–28,y45–47; far front leg angles toward x33–37,y45–47. Dark negative gaps separate the four feet.
- Cloth T/B for player and R/r for enemy: one connected diagonal band starts at the loose rear-facing contour x3–11,y26–31, crosses the shoulder through x16–25,y24–38, and narrows around the underside at x28–33,y39–42.
- Eye O/E: a restrained dark surround at x34–37,y33–37 and one light pixel at x35,y35.
- Ivory tusks I/i: two separate curved shapes, near tusk x35–41,y36–44 and far tusk x40–45,y36–44. Dark muzzle cells separate their roots and tips; the muzzle remains brown.

Both variants have identical silhouette and material ownership. Only cloth symbols differ (T→R and B→r). The common palette uses flat opaque RGBA colors and transparent `.`. PNGs are exact renders of the explicit matrices.

Hide remains dark brown: H is RGB (101,61,36), S (117,70,39), D (67,43,29).
The mane is RGB (42,33,26), with a dark (25,22,16) outline. These broad planes
preserve the darker selected reference without dimming the faction scarf or tusks.

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/boar/player.txt art/pixel/units/boar/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.
