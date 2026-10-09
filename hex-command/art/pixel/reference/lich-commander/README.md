# Lich commander lineup

Grave Adept → Lich → Lich Sovereign. These are artwork designs; gameplay registrations remain separate. Each tier was generated independently in the empty bottom-right box of a four-slot reference. The other three boxes supplied current Skeleton, Revenant and Corruption Mage anchors. The earlier three-character concept is inspiration only, not an extraction input.

Approved sources live in `../../units/lich-commander-l{1,2,3}/sprite.txt`. Each uses a 64×64 native canvas because the generated staff/feet extended beyond the requested central 48×48 area. No anatomy was cropped or scaled to force the smaller frame.

`sources/` retains unchanged generated images. `prompts.json` records the tier design instructions. The generation additionally specified the fourth-box geometry, native pixel style and exact faction/material palette. `extracted-l*.json` and `extraction-l*.json` preserve the initial extraction and its dimensions. `cleanup.json` records coordinate corrections for green pixels incorrectly assigned to gold, cloth or bone. Green effects remain confined to the staff heads.

Normal builds read the approved matrices. To inspect an extraction again without replacing approved artwork:

```sh
node art/pixel/reference/lich-commander/extract.cjs art/pixel/reference/lich-commander/sources/l1.png tmp/lich-review/l1
python3 art/pixel/reference/lich-commander/validate.py
```

Validation compares both faction renders pixel by pixel, checks material palette ownership and identical alpha, and verifies every correction preserves the extracted silhouette. `validation.json` records the results. Only `^`/`*` change between factions; bone, gold, wood, effects and outlines stay fixed. Player/enemy previews were also reviewed visually at native nearest-neighbour scale for hands, sleeves, skulls, feet and cloak boundaries.
