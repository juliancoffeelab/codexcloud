# Pixel artwork workflows

Finished editable sources live in [units/](units/) and are indexed in
[catalog.json](units/catalog.json). Reference folders retain generated inputs,
construction scripts and comparison boards. Run commands from the project root
after `npm install`. Image composition uses `pngjs` and ImageMagick's `magick`;
matrix rendering uses Python's standard library.

The chosen [zombie lineup](lineups/zombies.json) uses the eight reviewed base
sprites with new branch and tier assignments. Unused variants live in
[reference/zombies/](reference/zombies/README.md); the expansion source resolver
keeps rebuild outputs in their active or archived locations and preserves the
chosen catalog metadata. Historical study boards are not advancement trees.

## New anatomy through imagegen

Read the [Slinger pipeline](reference/slinger/PIPELINE.md) first. The image model
designs a complete pose; scripts extract that pose into native pixels. These
scripts do not call imagegen themselves.

1. Run `node art/pixel/reference/human-anatomies/create-templates.cjs` to build
   prefilled four-slot PNGs with three existing style anchors and one empty slot.
   Inputs are saved under `reference/human-anatomies/templates/`.
2. Give a template to the imagegen tool, specifying the complete pose and actual
   hand grips. Request flat supplied palette colors, a magenta background, no
   antialiasing, and consistent proportions. Keep the character inside the
   central 48×48 logical footprint; use the larger 64×64 frame for cavalry.
   Output resolution may differ from the input. Save the unchanged returned PNG.
3. Extract a new result into a scratch directory:

   ```sh
   node art/pixel/reference/human-anatomies/extract.cjs generated.png tmp/new-pose 48 8 8
   python3 scripts/render-pixel.py tmp/new-pose/player.txt
   ```

   Use `64 0 0` for cavalry. Extraction reads the bottom-right quarter, removes
   magenta, quantizes to the supplied palette and rejects foreground outside the
   requested crop. This geometry belongs to the four-slot template; it is not a
   general sprite-sheet detector.

4. Inspect enlarged pixels and material regions. Fix noisy shading and incorrect
   material ownership while preserving anatomy and grips. Inspect against both
   the raw generation and existing units. Extraction is not an anatomy repair.
5. Store the finished matrices, palette and both faction PNGs under `units/`.
   Render from matrices; never patch only the PNG. Rebuild atlases with
   `npm run art:build` after source changes.

The four existing raw results are saved in
[human-anatomies/sources/](reference/human-anatomies/sources/). For those exact
results, `rebuild.cjs` runs extraction, their recorded cleanup, tier edits,
faction variants, rendering and previews. `clean.cjs` and `variants.cjs` contain
pose-specific coordinates; do not apply those corrections to a different pose.

## Tier variants and previews

| Script                                                                           | Purpose                                                                                                                                   |
| -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| [human-magic/create.cjs](reference/human-magic/create.cjs)                       | Builds the white healers, blue storm mage and scarlet archmage from current adept/mage poses. Writes canonical sources and both factions. |
| [human-level3/create-commander.cjs](reference/human-level3/create-commander.cjs) | Builds the crowned Grand Marshal and crystal sword from Marshal.                                                                          |
| [human-anatomies/variants.cjs](reference/human-anatomies/variants.cjs)           | Builds the crossbow, hammer, halberd and cavalry tiers from cleaned base poses.                                                           |
| [human-anatomies/factions.cjs](reference/human-anatomies/factions.cjs)           | Derives red cloth variants for those anatomy studies.                                                                                     |
| [human-lineup/create.cjs](reference/human-lineup/create.cjs)                     | Reads canonical PNGs and builds the complete teal/red branch boards; does not edit source art.                                            |
| [human-anatomies/preview.cjs](reference/human-anatomies/preview.cjs)             | General labeled contact sheet from a JSON list of `{label, image}`, with nearest-neighbor enlargement and a checkerboard.                 |
| [render-pixel.py](../../scripts/render-pixel.py)                                 | Renders matrices and packs native frames into the public atlases.                                                                         |
| [pixel-source-path.cjs](../../scripts/pixel-source-path.cjs)                     | Resolves construction-study IDs to canonical unit directories using the catalog.                                                          |

Construction scripts replace the outputs they own, including canonical matrices.
They are recipes for the saved designs. For later manual edits, edit the matrix
and render it; update its construction recipe before using that recipe again.

The [reviewed material-cleanup recipes](reference/cleanup/README.md) preserve the
race-specific color and shading corrections across construction rebuilds. The
matrix renderer replays their guarded player/enemy and palette edits before
rendering; check mode verifies they are present. Update the owning cleanup recipe
alongside future edits to corrected pixels or palette colors.

The regenerated dwarf crossbow/pickaxe, pistol goat rider and axe-and-pick
infantry use the saved pose masks in
[dwarf-renewed.cjs](reference/faction-expansion/dwarf-renewed.cjs). Their pink
templates use actual dwarf scale anchors. Keep those masks paired with their new
source generations; the older metal-apron rectangles belong to different poses.

The nine non-rune dwarf families now use the
[reviewed armour progression](reference/dwarf-progression/README.md), derived from
the cleaned native inputs and full material ownership masks saved there.
`process.cjs tiers` delegates these families to its `build.py` recipe.
L1 armour is leather, L2 mixes complete metal pieces with leather, and L3 covers
all wearer armour in silver. The rune family remains on the earlier recipe.
Update the saved input or ownership mask for later edits, then rebuild and run
`python3 art/pixel/reference/dwarf-progression/build.py --check` to verify coverage.

Regenerate the full boards without changing artwork:

```sh
node art/pixel/reference/human-lineup/create.cjs
```

Create a custom board:

```sh
node art/pixel/reference/human-anatomies/preview.cjs entries.json tmp/board.png 3 8
```
