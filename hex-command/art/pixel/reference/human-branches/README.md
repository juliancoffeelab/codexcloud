# Human branch studies

Thirty 64×64 matrix sprites: ten existing humans and twenty new branch concepts.
Each character in `player.txt` is one native pixel. `palette.json` supplies exact
RGBA values; `.` is transparent black. Visible pixels have alpha 255.

The ten existing designs use their original 48×48 game matrices, padded with
eight transparent pixels on every side. New designs are transcribed from
[the generated reference](generated-source.png), with its background, borders,
blur and incidental colors removed.

Each row has two paths: **1 → 2 → 3** and **1 → 4 → 5**.

| Row | 1: L1     | 2: L2 A  | 3: L3 A     | 4: L2 B     | 5: L3 B      |
| --- | --------- | -------- | ----------- | ----------- | ------------ |
| A   | Swordsman | Soldier  | Champion    | Duelist     | Blademaster  |
| B   | Spearman  | Warden   | Bulwark     | Pikeman     | Halberdier   |
| C   | Scout     | Ranger   | Shadowblade | Bow Scout   | Pathfinder   |
| D   | Archer    | Marksman | Longbowman  | Crossbowman | Arbalester   |
| E   | Adept     | Mage     | Archmage    | Healer      | High Healer  |
| F   | Rider     | Cavalier | Knight      | Outrider    | Horse Archer |

[Preview](preview.png) shows the native sprites enlarged 4× using nearest-neighbor
scaling over a checkerboard. [Native sheet](sheet-native.png) retains transparency.
Each labeled directory contains its matrix, palette and matching native PNG.
`layout.json` records the original reference crop positions and labels.

These are artwork studies. Game roster definitions and gameplay assets retain
their current approved units.

Render a changed matrix with the existing project renderer, for example:

```sh
python3 scripts/render-pixel.py art/pixel/reference/human-branches/A3-champion-l3/player.txt
```
