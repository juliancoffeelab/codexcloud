# greatboar

White-coated veteran of boar. Preserve the approved compact right-facing
head/eye, two connected ivory tusks and four grounded hooves. The charcoal `M/O`
shoulder mane grows to y18 at x18–28 without changing the snout or limbs. Brown
leather `L/l` reinforces the existing faction harness along its right edge,
y31–41. A small connected gold `G/g` clasp sits at x21–23,y33–34. Leather and
gold are equipment, not hide texture. Both factions share anatomy and material
boundaries; illumination remains upper left.

See [the progression specifications](../../../../design/progression-art.md).

C boar, native 48×48, facing screen-right. Coordinates below are zero-based.
The occupied bounds are x2–45, y20–47; hooves end at the common row47 baseline.

- Hide H/S/D: stocky rump, shoulder hump and lowered head. H is the main pale hide; S is the broad ivory upper/front shoulder plane; D is the warm grey belly and far-leg shadow. No fur-noise pixels.
- Mane M with O outline: a dark stiff ridge over the back and down the shoulder, y20–33. Tail M/O runs diagonally behind the rump to x2,y42. Neither is faction cloth.
- Ear D/O: a folded vertical wedge at x28–30,y27–30.
- Four legs: near hind leg x4–14 ends in a compact hoof at x4–8,y45–47; far hind leg x14–21 is shadowed and ends at x17–21,y45–47. Near front leg x21–28 ends at x24–28,y45–47; far front leg angles toward x33–37,y45–47. Dark negative gaps separate the four feet.
- Cloth T/B for player and R/r for enemy: one connected diagonal band starts at the loose rear-facing contour x3–11,y26–31, crosses the shoulder through x16–25,y24–38, and narrows around the underside at x28–33,y39–42.
- Eye O/E: a restrained dark surround at x34–37,y33–37 and one light pixel at x35,y35.
- Ivory tusks I/i: two separate curved shapes, near tusk x35–41,y36–44 and far tusk x40–45,y36–44. Dark muzzle cells separate their roots and tips; the muzzle shares the pale hide palette.

Both variants have identical silhouette and material ownership. Only cloth symbols differ (T→R and B→r). The common palette uses flat opaque RGBA colors and transparent `.`. PNGs are exact renders of the explicit matrices.

The selected veteran has pale hide: H is RGB (191,182,161), S (220,212,190),
and D (122,116,102). The mane is dark warm grey, RGB (65,60,52), with a
(32,29,24) outline. The level-one boar keeps its approved deep brown coat.
Faction cloth, leather reinforcement, gold clasp and ivory tusks keep their
existing colors. These broad material planes introduce no extra texture.

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/boar/player.txt art/pixel/units/boar/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.
