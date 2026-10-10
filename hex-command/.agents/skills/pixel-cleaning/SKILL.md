---
name: pixel-cleaning
description: Inspect and clean Hex Command sprite matrices using stray/edge detectors, eight-neighbor context and visual verification. Use for extraction artifacts, wrong-material pixels, holes and torn outlines; never automatically paint detector findings.
---

# Pixel cleaning

Tell the user when using this skill. Work from the project root; read
`art/pixel/WORKFLOWS.md` and [failure examples](references/failure-examples.md).
Helpers require Python 3/Pillow and use the existing repository detectors.
**Detect**, **fix**, **revert** and **stop** are different instructions. Detect
means run the scripts AND inspect/analyze their images, without editing. Stop
means stop mutations immediately. Revert means restore the exact requested
checkpoint, not an approximate reconstruction or an older generation.

## User-facing previews

Show a clean nearest-neighbor render of the unit or lineup in one faction colour.
Use the requested faction; otherwise use the most recently requested faction,
or enemy if none is established. Show both factions only when explicitly requested.
Keep debug colours, detector overlays, coordinates, source paths and before/after
boards internal unless the user asks to see diagnostics or a comparison. Still
run and visually inspect required diagnostics yourself. A matrix preview means
the rendered unit from its matrix, not a diagnostic sheet. Lineups may have simple
unit names and tier labels.

## Establish the scope and baseline

Work one unit at a time, restricted to the requested tier. Save the exact matrix,
palette and render before edits; preserve any approved pose and styling. Use
versioned scratch directories. Never replay a historical recipe just to render:
some recipes mutate matrices. These bundled previews call read-only rendering
functions directly. Canonical matrices stay untouched while comparing candidates.

Locate and open the original generated image or unchanged extraction crop before
cleaning. Use the catalog's study entry, the owning reference directory, saved
source paths and generation prompt to identify the correct reference. Compare
it with the approved matrix to understand garment panels, lining, straps, hands,
grips and weapon geometry. Record the reference path alongside the edit log.
Recheck the corresponding source area before changing an ambiguous material or
contour; detector flags and palette names cannot establish what a part represents.
For example, the flanker's white shape is cream lining inside its split tunic.

The approved matrix and later user decisions remain authoritative when they
deliberately differ from the generation. The reference explains design intent;
do not restore its compression noise or overwrite approved revisions. If no
reference exists, state that limitation and preserve ambiguous parts while
continuing clear corrections elsewhere.

Read the local palette. Identify cloth, skin, hair, leather, wood, metal and
outline by actual regions, not just symbol names or nearest RGB. A valid palette
color can still be the wrong material. Shade policies are per palette: the elf
policy groups leather/wood/cloth, rejects extra skin/metal shades, and provides
no outline shade ramp. Do not blindly apply elf symbol meanings to other races.

## Detect, then look

```sh
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  path/to/sprite.txt tmp/unit-review/pass-1 \
  --policy .agents/skills/pixel-cleaning/references/elf-shades.json
```

Omit policy if it does not match the unit. The read-only bundle writes:

- Strict stray image; separate material-policy image when requested.
- Separate images for every edge rule, avoiding overlapping colored flags.
- Player, enemy and debug matrix previews.
- `report.json`: every flag, matrix/palette hashes, coordinates, all eight
  neighbors, same-material neighbor count and eight-connected component size.
- Paginated enlarged **21×21 pixel** neighborhoods centered on every flagged
  coordinate. `--points 17,19 17,20` focuses neighborhoods on specific native
  coordinates without suppressing the full detection images/report.

Open the images with the image-viewing tool. Read matrix coordinates for uncertain
spots and inspect the unmarked render too. Images are evidence only after you
actually look at them. Do not confuse generated detector images with the user's
marked screenshot; align its arrows with native coordinates first.

The strict detector uses cardinal exact-color adjacency. It can flag intentional
diagonals, eyes, buckles and small highlights; it can miss same-colored artifact
clusters and wrong-material patches with a matching neighbor. The bundle adds
8-neighbor context, not a claim that these ambiguities are solved automatically.
The edge detector checks exposed material, enclosed transparent holes, black
fragments/endpoints, unsupported black, junctions and diagonal gaps. A diagonal
pair without a cardinal bridge is often normal pixel art. Its flag is **not** an
instruction to fill the two orthogonal cells. Inspect every applicable rule and
every user-marked example before declaring a defect resolved.

## Decide each correction

For each candidate ask: What surface is this? Which region should continue here?
What do cardinal AND diagonal neighbors imply? Is the pixel intentional anatomy,
a grip, shading, an internal seam or an extraction artifact? What happens to
adjacent pixels if I change it? Inspect about twenty pixels of context when needed.

Choose the actual correction:

- Blend wrong-material or noisy shade pixels into the correct local material.
  Skin inside an ivory sleeve becomes ivory; ivory in a neck becomes skin.
  Brown in a cloak becomes the appropriate faction cloth shade, not black.
- Adjust neighboring pixels too when a compression artifact distorts a cluster.
  Every changed pixel still gets its own reason; do not flood-fill the sprite.
- Restore a genuinely missing boundary using the intended outline. Preserve
  internal leg separation, actual hand edges and thin diagonal weapon contours.
- Trim unsupported exterior black when it falsely encloses transparency. Do not
  fill an invented hole with hair or more outline merely to satisfy the checker.
- Realign a torn contour when the local anatomy calls for it. Do not thicken every
  diagonal to XY connectivity, erase a limb edge, fatten a spear, or recolor a
  weapon when the requested fix is its outline.
- Leave intentional pixels unchanged and explain their function when relevant.
  A sash is a sash; a leather fleck far from leather is not justified by its color.

No bulk conversion of findings to outline, majority-color smoothing, automatic
bridges, global palette flattening or color-distance cleanup. A user may request
broader denoising, but material ownership still needs individual visual reasoning.

For a tier material upgrade on an already cleaned matrix, use the enhancement
skill's [fill-then-shade route](../pinkbox-enhancement/SKILL.md#fast-material-upgrades-fill-then-shade)
to preserve geometry and avoid generating fresh extraction noise. Still inspect
the new shading and run both detectors; material painting does not require
re-cleaning every intentional pixel in the approved baseline.

## Apply a small reviewed batch

Write an explicit JSON edit list using the hashes from `report.json`:

```json
{
  "matrix_sha256": "HASH_FROM_REPORT",
  "palette_sha256": "HASH_FROM_REPORT",
  "edits": [
    {
      "x": 17,
      "y": 19,
      "from": "H",
      "to": "I",
      "material": "ivory sleeve",
      "reason": "Skin-colored extraction artifact inside the ivory cuff"
    }
  ]
}
```

Outline or transparency edits also need `"boundary_reviewed": true` in that
individual entry after you inspect the contour. This is a script guard, not a
request for user permission. Coordinates are zero-based native pixels.

```sh
python3 .agents/skills/pixel-cleaning/scripts/apply_edits.py \
  path/to/sprite.txt tmp/edits.json tmp/unit-review/candidate-1
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  tmp/unit-review/candidate-1/sprite.txt tmp/unit-review/pass-2
```

The edit helper rejects stale hashes/symbols, duplicate coordinates, undefined
colors, missing reasons and unreviewed boundary changes. It creates a new
candidate, exact baseline, change log and before/after faction comparison. It
never overwrites the input. Palette edits require a separately explained,
reviewed change; this helper only edits matrix pixels.

## Verify and report accurately

Open the new comparison, each relevant detector image and edited neighborhoods.
Compare edited regions with the original reference and approved baseline again.
Verify material continuity, anatomy, grip, thin weapons, diagonal contour and
transparency. Confirm every user-marked defect against the render. Rerun both
detectors after each batch; fewer flags or zero flags alone prove nothing.
If every remaining candidate is judged intentional and the user has authorized
independent review, obtain it before calling the sprite clean. Do not delegate
without authorization. Otherwise state remaining uncertainty instead of claiming
completion. Do not keep modifying unmarked areas after a narrow fix.

Show the clean native matrix render in one faction colour. Report exact changed-pixel count and what each
change accomplished. Count corrected **defects** separately from changed
**pixels**: one edge may take several pixels, and painting nearby black does not
fix a marked stray. If three strays were shown, confirm all three, not merely one.

Promote the chosen matrix only within authorized scope, update its owning recipe
or guarded cleanup record, and rebuild/check art after canonical integration.
Keep the exact approved baseline for reversions. Never edit only a PNG.
