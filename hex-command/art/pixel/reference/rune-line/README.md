# Rune apprentice branches

[Teal progression](lineup-player.png) · [Red progression](lineup-enemy.png).

The cleaned L1 rune apprentice branches into Thundersmith and Stonespell at L2,
with an individual L3 for each. These complete the fifth dwarf tree.
Thundersmith has a sun-gold hand crystal and a crystal-blue hammer, steel helmet, chest plates and bracer at
L2, with cloth sleeves and leather gloves, apron, trousers and boots. L3 upgrades
to full silver armour with gold trim. Stonespell wears a heavy slate hood and
robes; L3 adds carved stone shoulders, simple edging and a crystal-blue rune
tablet. Faction cloth alone swaps teal to red. No green robes or scattered gold
ornaments remain.

The approved image-tool result is retained in `designs/generated-board.png`.
`node art/pixel/reference/rune-line/designs/extract.cjs` samples its two Stonespell
frames to 64×64 matrices with nearest-palette majority voting and transparent
background removal. The saved masks and palette are canonical recipe inputs;
`build.py` copies them to the editable unit sources and derives red faction cloth.
Stonespell has its own hood and equipment silhouette rather than a recoloured
apprentice helmet. Inspect the native boards, not only the generated proposal.

`bases/player.mask` and its palette own the cleaned native L1 design. The saved
original from commit `619b07f` guards its silhouette, facial details, hand and
crystal. Forty explicitly recorded internal seam pixels separate the actual
pointed apron, torso, overskirt, trousers and boots. Six material flecks are
reviewed separately. Shading is consolidated only within each original material.
The crystal keeps its original cyan; its dedicated border is darker cyan with a
small green hue shift, rather than faction cloth or the previous dark green.

Four-way material components are detected from the actual pixels. `parts.mask`
records their exact coverage and `anchors.json` names 28 independently fillable
parts. `roles.json` assigns every visible fragment to garments, fasteners, gloves,
boots, hammer, faction cloth or protected materials. Beard, face, hand, shafts,
crystal and outline retain their colours. No rectangular armour overlays are used.
Every assigned Thundersmith armour pixel uses its complete tier ramp or L3 gold.
L2 lower leather and sleeves are protected from metal fills. The checker verifies
Stonespell against its independent native hood, robe and tablet designs.

The requested orthogonal equal-RGBA speck check dropped from 115 to 42 on L1.
Remaining flags include eyes, diagonal internal seams, thin shaft edges, fasteners
and small partly occluded fragments. Their coordinates remain in
[isolated-pixels.json](isolated-pixels.json); this is a review report, not a claim
that every flagged pixel is an artifact. Dark outline pixels beside transparency
are excluded. Do not remove part boundaries just to reduce the count.

Run from the project root:

```sh
python3 art/pixel/reference/rune-line/build.py
python3 art/pixel/reference/rune-line/build.py --check
python3 art/pixel/reference/rune-line/preview.py
python3 tests/rune_line_test.py
python3 art/pixel/reference/dwarf-upgrades/preview.py
npm run art:build
npm run art:check
npm run style
```

The faction-expansion `tiers dwarf-runesmith` route uses this recipe. The obsolete
single runesmith L2/L3 studies have been removed; previous designs remain in Git.
The lineup is artwork only and does not add gameplay rules.
