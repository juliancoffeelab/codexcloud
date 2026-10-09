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

## Shared faction sources

Every finished unit and paired archived study has one `sprite.txt` matrix and a
flat material `palette.json`. `^` is faction base and `*` is faction shadow;
[factions.json](factions.json) defines those two RGBA colours for each side.
Older three-to-five-shade faction ramps are clamped to these two shades.
Skin, coral, metal, leather and other fixed material pixels retain their colours.
Do not add per-unit `^`/`*` palette entries or maintain separate faction matrices.

`npm run art:build` reads the shared matrix directly and bakes both sides into the
existing atlas frames. Individual faction PNGs are optional ignored previews:

```sh
python3 scripts/render-pixel.py --previews
python3 scripts/render-pixel.py --atlas --debug-faction tmp/faction-debug
python3 scripts/render-pixel.py art/pixel/units/reefguard/sprite.txt
```

The debug command renders the faction base in neon magenta and shadow in neon
lime; fixed materials stay normal. Inspect these for colour leaking into skin,
weapons or armour. Debug images never enter the gameplay atlas.

Saved construction recipes use [pixel-fs.cjs](../../scripts/pixel-fs.cjs) and
[pixel_sources.py](../../scripts/pixel_sources.py) to read their older paired
inputs virtually. When a recipe writes a temporary pair, the matrix renderer
consolidates it into `sprite.txt` before rendering. Check mode rejects unconverted
pairs. New recipes should write a shared matrix directly. Raw, single-sided
extractions keep their original matrices and palettes as generation evidence.

## New anatomy through imagegen

Read the [Slinger pipeline](reference/slinger/PIPELINE.md) first. The image model
designs a complete pose; scripts extract that pose into native pixels. These
scripts do not call imagegen themselves.

1. Build a disposable pink-box reference directly from three approved matrices:

   ```sh
   python3 scripts/pink-box.py tmp/lich-template.png skeleton revenant lich
   ```

   Choose suitable anchors for the new design. The tool renders current player
   colours from `sprite.txt`, writes palette swatches and a companion JSON, and
   leaves the fourth slot empty. Outputs belong in ignored `tmp/`; approved
   designs do not need committed generation templates. Historical template
   scripts are retained for recipe archaeology, not the new-design workflow.

2. Give a template to the imagegen tool, specifying the complete pose and actual
   hand grips. Request flat supplied palette colors, a magenta background, no
   antialiasing, and consistent proportions. Keep the character inside the
   central 48×48 logical footprint; use the larger 64×64 frame for cavalry.
   Output resolution may differ from the input. Save the unchanged returned PNG.
   For new templates, refresh faction swatches and the extraction palette from
   the player shades in `factions.json`. Keep magenta exclusively as background;
   the debug faction palette is for review after extraction, never generation.
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
5. Store the finished shared matrix and material palette under `units/`.
   Render from matrices; never patch only the PNG. Rebuild atlases with
   `npm run art:build` after source changes.
   Assign `^`/`*` by cloth ownership, including sleeves and cloak folds, while
   preserving exposed hands, feet and outer outlines as fixed materials.
   Review both factions and a separate debug preview before accepting the design.

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

The normal artwork build reads approved matrices directly. Historical
construction scripts replace the outputs they own, including canonical matrices.
Run them only when deliberately reconstructing a saved generation; never as a
routine refresh or to prepare a new pink-box reference.
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

The nine non-rune dwarf bases now use the
[base cleanup recipe](reference/dwarf-base-cleanup/README.md). Their pre-progression
equipment is restored; the saved native matrices use flat material colours,
simple shading and cleaned boundaries, with fill regions detected from the art.
Five L1 starters are recorded in [dwarves.json](lineups/dwarves.json), including
the rune apprentice tree. Extra poses are saved as
[branch references](reference/dwarf-branch-references/README.md).
`process.cjs tiers` rebuilds cleaned sources and the sixteen
[whole-surface branch upgrades](reference/dwarf-upgrades/README.md), including
red enemy cloth and L3 gold trim/inlays. The separate
[rune-line recipe](reference/rune-line/README.md) owns the cleaned apprentice and
Thundersmith / Stonespell L2–L3, completing ten branches across five trees. The rejected intermediate dwarf tiers and obsolete builder have been removed;
[original inputs](reference/dwarf-progression/README.md) remain required for rebuild guards.
Update the owning base matrix and its detected region records for later edits,
then rebuild and run
`python3 art/pixel/reference/dwarf-base-cleanup/build.py --check`.
For branch upgrades also run `python3 art/pixel/reference/dwarf-upgrades/build.py --check`
and `python3 art/pixel/reference/rune-line/build.py --check`.
Render the actual five trees with `python3 art/pixel/reference/dwarf-upgrades/preview.py`.

Regenerate the full boards without changing artwork:

```sh
node art/pixel/reference/human-lineup/create.cjs
```

Create a custom board:

```sh
node art/pixel/reference/human-anatomies/preview.cjs entries.json tmp/board.png 3 8
```
