# Bone archer — C tracing

48×48 native matrices use row 47 as the boot baseline. The complete C-derived
archer and bow occupy rows 14–47; the lower trailing cloak and ragged waistcloth
retain the selected reference's broad forms. The explicit `player.txt` and `enemy.txt` matrices record the native construction. Both matrices have identical geometry and material ownership. Enemy cloth swaps
`q`, `c`, `C` to crimson `r`, `R`, `h`; the flat palette contains both sets.

| Component                                | Material cells                          | Native location (zero based)         |
| ---------------------------------------- | --------------------------------------- | ------------------------------------ |
| Right-facing skull, jaw and neck         | `B`, `b`, dark `O` cavities             | x20–26, y15–22                       |
| Scarf and trailing screen-left cloak     | `q`, `c`, `C`                           | shoulder y19–26; cloak x4–20, y25–42 |
| Rib groups, spine and pelvis             | `B`, `b`, `O` gaps                      | x17–26, y26–33                       |
| Bent drawing arm and fingers at nock     | ivory `B`, `b`; leather `L`, `l` bracer | x10–24, y21–28                       |
| Extended bow arm and gripping fingers    | ivory `B`, `b`; leather `L`, `l` bracer | x24–38, y26–29                       |
| Belt and central buckle                  | leather `L`, `l`; gold `G`, `g`         | x18–25, y32–33                       |
| Central ragged waistcloth                | faction `q`, `c`, `C`                   | x17–25, y34–42                       |
| Both braced leg chains                   | ivory `B`, `b`                          | x11–20 and x24–32, y33–42            |
| Compact boots                            | leather `L`, `l`                        | x7–14 and x29–36, y41–47             |
| Curved bow                               | wood `W`, `w`, dark outline `O`         | x32–40, y14–42                       |
| Taut bowstring                           | `O`, `o`                                | two diagonals meet drawing fingers   |
| Arrow shaft and right-pointing arrowhead | wood `W`, `w`; silver `S`, `s`          | x24–43, y26–28                       |

Upper-left illumination is expressed through contiguous main/shadow surfaces.
Hands use the same ivory bone palette as the skull, ribs and legs. The arrow shaft occupies one native row, y27 from x25 through x40, at the
middle of the unchanged bow (y14–41, midpoint 27.5). Its silver head uses only
five cells across three rows (y26–28), so it remains a light arrow. The ivory
drawing palm/fingers at x22–24, y25–27 turn downward from the original leather
bracer to the nock at (24,27). The original elbow and returning forearm remain
in their previous locations; the original rib groups and their dark gaps are
preserved below the draw line. The extended ivory arm retains its original
shape, with distinct gripping fingers at x36–37, y28–29 below the shaft. Taut
string lengths connect tip (33,15), nock (24,27), and tip (33,40), with existing
anatomy naturally occluding hidden sections. Both factions share this geometry.
No backdrop, glow, resampling noise, smoothing or
reference RGB samples are retained.

From the project directory, run `python3 scripts/render-pixel.py art/pixel/units/bonearcher/player.txt art/pixel/units/bonearcher/enemy.txt` after editing, or `npm run art:build` to rebuild all native sprites and the game atlas. `npm run art:check` checks saved PNGs and atlas metadata against their matrices.
