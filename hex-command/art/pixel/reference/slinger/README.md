# Slinger study

The complete workflow is documented in [PIPELINE.md](PIPELINE.md).

A ranged Scout-family concept produced from a generated design rather than a
redrawn silhouette. Existing Scout, Ranger and Archer sprites were supplied at
8× nearest-neighbor enlargement with eleven artwork colors.

The [generated reference](generated-source.png) is 1254×1254. Its lower-right
627×627 slot was converted into a 64×64 intermediate matrix by assigning source
pixels to the supplied palette and selecting the dominant foreground color in
each native pixel block. Magenta background, borders and the caption are removed.
Eight transparent padding pixels were trimmed from each side to produce the
final 48×48 matrix, now stored in `../../units/slinger/`, matching the game's native unit frame. No visible
pixels were cropped.

Material colors and shirt shading are cleaned into connected flat planes;
coordinates and reasons are recorded in `cleanup.json`. The shirt uses ivory
and a contour shadow without interior speckles. No silhouette pixels were added,
removed or moved during cleanup. The result uses ten visible colors.

`palette.json` defines the exact RGBA values; `.` is transparent black. `player.png`
is rendered from the matrix, with alpha values exclusively 0 or 255.

- [Generated design and native conversion](comparison.png)
- [Scout, Ranger and Slinger together](family-preview.png)

Regenerate the native image with the existing project renderer:

```sh
python3 scripts/render-pixel.py art/pixel/units/slinger/player.txt
```

The finished matrices and PNGs live in `art/pixel/units/slinger/` and are packed
into the unit atlas. This folder keeps the generated input and workflow; no
Slinger gameplay definition has been added.
