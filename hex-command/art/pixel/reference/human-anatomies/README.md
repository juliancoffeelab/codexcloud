# New human anatomy studies

Generated pink-box PNGs have been removed after matrix approval. References
below to `templates/` describe the historical generation process. For new work,
use `python3 scripts/pink-box.py tmp/new-design.png <anchor1> <anchor2> <anchor3>`
from the project root. Approved `units/*/sprite.txt` matrices are authoritative;
saved raw sources and rebuild recipes remain available for historical inspection.
Do not replay construction recipes as routine artwork verification.

Four complete poses generated using the documented
[Slinger pipeline](../slinger/PIPELINE.md): crossbow infantry, two-handed staff
infantry, a mounted staff/lance rider and a mounted archer.

## Lineups

- [Four cleaned base poses](bases-preview.png)
- [Crossbowman L2 and Arbalester L3](crossbow-preview.png)
- [Hammer and halberd branches, L2/L3](infantry-preview.png)
- [Mounted lance and bow branches, L1/L2/L3](cavalry-preview.png)
- [All twelve tier studies](all-preview.png)

Finished tiers live in [canonical unit sources](../../units/catalog.json), with
`player.txt`, `enemy.txt`, `palette.json`, and transparent player/enemy PNGs. Base
construction poses remain here. Red variants change faction cloth colors while
preserving anatomy and equipment. `cleanup.json` records base material fixes;
`changes.json` records each tier's local edits and palette changes.

The finished artwork is packed into the unit atlas, with no new unit stats or
recruitment registration.

## Frames and extraction

The generated sheets are 1254×1254. Their lower-right 627×627 slots are converted
to a logical 64×64 grid using sixteen explicit visible colors. Background pixels
become fully transparent; no JPEG or interpolated resize is used.

| Base             | Native frame | Crop origin within logical slot |
| ---------------- | ------------ | ------------------------------- |
| Crossbow         | 48×48        | 8, 8                            |
| Two-handed staff | 48×48        | 8, 9                            |
| Mounted lance    | 64×64        | 0, 0                            |
| Mounted bow      | 64×64        | 0, 0                            |

The staff crop is shifted down one logical pixel to preserve both the pole tip
and its bottom end. No visible artwork is cropped. The model drew horses wider
than the requested central 48-pixel area. Cavalry therefore retains the full
64-pixel frame, preserving rider proportions and all four legs. The unit packer
and frame loader retain those native dimensions.

The shared `palette.json` constrains extraction colors. Cleanup merges noisy
material tones, removes skin/gold/gray flecks from the wrong surfaces and creates
flat cloth and coat areas. It preserves the base silhouettes. Weapon variants
edit the free staff ends; existing grips and joints remain in place. Hammers keep
their upright shafts; the broad striking face points down-right across the pole.

## Variants

| Pose                      | Level one    | Level two    | Level three    |
| ------------------------- | ------------ | ------------ | -------------- |
| Crossbow                  | —            | Crossbowman  | Arbalester     |
| Two-handed staff, hammer  | —            | Hammerman    | Warhammer      |
| Two-handed staff, halberd | —            | Halberdier   | Halberd master |
| Mounted lance             | Rider        | Lancer       | Knight         |
| Mounted bow               | Horse archer | Horse bowman | Horse marksman |

Names are working reference labels. Melee progression uses leather, mixed ivory
armor and silver plate covering the torso, arms and legs. Hammer heads are
rectangular, rotated 45° and centered across the shaft; halberds have an axe blade perpendicular
to the pole, an aligned spear tip and an opposing rear spike. The top mounted
archer has a cream hood and a lighter horse; the knight has silver armor and a
white horse. Variants are matrix and
palette edits of the cleaned bases, with no additional image generations.
Remaining leather straps, belts and trim on silver units become gold; boots
remain leather, and wooden weapons retain their wood colors.

## Reproduce

`templates/` contains the four prefilled inputs; `sources/` contains the unchanged
generated images. `create-templates.cjs` rebuilds the inputs from existing game
sprites. Once those sources exist, regenerate extraction, cleanup, tiers,
faction colors, PNGs and all lineups with one command from the project root:

```sh
node art/pixel/reference/human-anatomies/rebuild.cjs
```

To extract one sheet manually:

```sh
node art/pixel/reference/human-anatomies/extract.cjs generated.png output-directory 48 8 8
python3 scripts/render-pixel.py output-directory/player.txt
```

Use `64 0 0` for mounted poses. The extractor saves the uncleaned logical grid,
source crop and extraction metadata, and rejects visible pixels outside the
requested native crop. Inspect material colors, silhouette, hands and equipment
at enlarged native scale before deriving variants.
