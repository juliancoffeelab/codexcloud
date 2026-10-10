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

## Generation, enhancement and cleaning

The project skills own the current workflows and bundled scripts:

- [Pinkbox generation](../../.agents/skills/pinkbox-generation/SKILL.md): one
  2×2 template with three matrix references and one fill box, current faction
  palette and manifest-based extraction.
- [Pinkbox enhancement](../../.agents/skills/pinkbox-enhancement/SKILL.md): use
  an approved unit as the anatomy reference for tiers, equipment or restyling.
- [Pixel cleaning](../../.agents/skills/pixel-cleaning/SKILL.md): detector images,
  eight-neighbor context, material decisions, guarded edits and visual verification.

Generated templates and candidates belong in ignored `tmp/`. Approved matrices
are the template source; do not keep redundant approved generation templates.
Show a clean render of the extracted matrix in one faction colour, following the
project skills. Inspect both factions and debug previews internally; show
diagnostics or comparisons only when requested.

The [Slinger pipeline](reference/slinger/PIPELINE.md) records a historical
four-slot extraction. Its saved inputs and pose-specific construction recipes
remain evidence, not the default pipeline for new designs. The four raw anatomy
results are saved in [human-anatomies/sources/](reference/human-anatomies/sources/).
For those exact results, `rebuild.cjs` replays their extraction, recorded cleanup,
tier edits and previews; do not apply its coordinates to another pose.

## Tier variants and previews

The [reviewed elf sources](reference/elf-reviewed/README.md) live in canonical
unit matrices and are protected from the superseded expansion tier masks.
This includes the separate jumper row, reviewed blade upgrades and shadow riders.

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

## Outline and stray review

Use the [pixel-cleaning skill](../../.agents/skills/pixel-cleaning/SKILL.md).
Its bundled scripts include both detectors, marked images for each rule,
21×21 native-pixel neighborhoods and guarded per-pixel edits. Findings are
review candidates; only material and contour inspection establishes a defect.
See the skill's [failure examples](../../.agents/skills/pixel-cleaning/references/failure-examples.md)
for the elf cloak leaks, leg separator, hair ring and diagonal-outline mistakes.
