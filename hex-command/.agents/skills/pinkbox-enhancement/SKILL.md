---
name: pinkbox-enhancement
description: Generate an enhanced tier, equipment revision or restyle of an existing Hex Command pixel unit through pinkbox, keeping the approved unit as the anatomy reference and delivering cleaned matrix comparisons.
---

# Pinkbox enhancement

Use the pinkbox pipeline for the **same unit**, with the approved matrix as the
primary reference. Read `art/pixel/WORKFLOWS.md` and
`../pinkbox-generation/SKILL.md`. Tell the user when using this skill. All helpers
run from the project root and require Python 3/Pillow. Bundled `prepare.py`
reuses the generation skill's template builder; extraction and cleaning also use
those shared helpers rather than divergent copies.

## User-facing previews

Show a clean nearest-neighbor render of the unit or lineup in one faction colour.
Use the requested faction; otherwise use the most recently requested faction,
or enemy if none is established. Show both factions only when explicitly requested.
Keep debug colours, detector overlays, coordinates, source paths and before/after
boards internal unless the user asks to see diagnostics or a comparison. Still
run and visually inspect required diagnostics yourself. A matrix preview means
the rendered unit from its matrix, not a diagnostic sheet. Lineups may have simple
unit names and tier labels.

## Decide the kind of enhancement

During an established L2/L3 progression pass, a request to move to the next unit
means complete both tiers and show its lineup. Do not stop after L2 unless the
user explicitly narrows the request to one tier.

Identify the approved base from the conversation and actual matrix, not an
unapproved intermediate PNG. Record what may change and what must stay: pose,
anatomy, sex, silhouette, handedness, grips, weapon width, outfit, palette and
faction pieces. A tier upgrade usually preserves anatomy and adds readable
material/equipment progression. Do not add excessive armor, effects or ornament
just because the tier is higher. L1 remains simple and lightly equipped.

If the requested change is only local palette/styling, a direct matrix edit can
preserve anatomy better than regeneration. Say which route you are taking.
For material upgrades, prefer filling existing connected regions over sketching
small armor patches pixel by pixel. Bundled `scripts/fill.py` takes the source
matrix, a JSON specification and a new scratch output directory. The specification
requires source matrix/palette SHA256 hashes and explicit regions with `seed`,
`mapping`, `expected_pixels`, `material` and `reason`. Optional `palette_additions`
supplies new material colors. It uses four-neighbor flood fill on the original
matrix and stops at every material outside the mapping. It preserves outlines,
transparency and faction cloth, saves the exact baseline and pixel change log,
and produces a comparison. Inspect each selected region first and review the
rendered result; this is material painting, not automatic artifact cleanup.
The skeleton servant progression used direct matrix/palette edits; the new Lich
commander used separate generated poses. Both are valid precedents, not blanket
instructions to regenerate or recolor every tier. Consult
`art/pixel/reference/{skeleton-progression,lich-commander}/README.md` as relevant.

## Fast material upgrades: fill, then shade

Prefer direct painting from a cleaned approved L1 when the pose and equipment
shapes already work and the upgrade is mainly leather/green equipment to silver.
This avoids a generation/extraction cycle and its new compression artifacts.
Use pinkbox when the requested silhouette, pose or equipment shapes need redesign.

1. Identify each surface from the reference and matrix. Separate armor from cloth,
   lining, skin, hair, handles and faction pieces. White/ivory cloth stays cloth;
   do not decide material ownership from color alone.
2. Use `scripts/fill.py` with explicit region seeds and guarded counts to recolor
   whole existing armor surfaces. Map the original material's base and shadow to
   the new metal base and shadow together, preserving its existing shading.
3. Add a small, individually reviewed shading pass with the cleaning skill's
   `scripts/apply_edits.py`. Reuse the metal shadow already in the palette. Follow
   the actual lower/right-facing plate curves and overlaps beneath straps or
   cloth; retain a broad light face so the armor reads at native scale. Shade
   coherent clusters rather than scattered single-pixel noise. Do not invent
   shades for outlines, change cloth into armor or alter the silhouette.
4. Render and inspect the result at native scale and nearest-neighbor enlargement.
   Check material separation, then run the stray and edge detectors. Compare new
   findings with the baseline; intentional shading still needs visual review.
   Fix only new actual defects individually. Preserve alpha, outline pixels,
   faction ownership, weapon and grips; record fills and shading separately.
5. Deliver the clean render in one faction colour. Keep diagnostics internal.

The elf blade L2 used filled torso/boot regions followed by twelve reviewed
silver-shadow pixels on its body armor. This demonstrates the technique; those
coordinates are not a reusable mask for another unit. A fill changes a complete
material region; shading recolors pixels within that region without widening it.

## Prepare and generate

```sh
python3 .agents/skills/pinkbox-enhancement/scripts/prepare.py \
  scout tmp/scout-upgrade/template.png --label 'Scout L2' \
  --style-anchors ranger archer
```

Exactly three references are required: the approved unit first, plus exactly
two approved style/material anchors. Never omit the two style anchors. The
layout is always 2×2 with one fill box at bottom right. Without `--size`, the native frame follows the base's
48/64 size. Every template has exactly one empty box. Prepare a separate
template and generation call for each tier or variant. For iterative work, use an approved enhanced matrix as the primary anchor
for the next tier when appropriate. Retain the original base for comparison.

Inspect the template and give it to imagegen. Use the generation skill's pixel,
geometry, palette and faction requirements, plus this explicit prompt contract:

The generation skill's mandatory style and resolution contract applies to every
enhancement and concept variant. Repeat its concrete grid, block scale, outline
and palette constraints in the actual imagegen prompt. Added equipment must use
the same logical pixel size as the approved base; higher tiers add readable shapes,
not finer-resolution texture. Inspect the generated result against the anchors
and regenerate if it drifts before presenting a candidate.

> This is the same unit as anchor 1. Preserve [listed anatomy, pose and grips].
> Change only [requested tier/equipment/style features]. Fill one complete unit
> per designated box. Preserve native proportions and distinct outlines/material
> regions; progression should remain readable at native scale.

For a requested new pose, state that change explicitly rather than pretending
silhouette preservation still applies. Save the unchanged image and prompt.

## Matrix and modest cleanup

Basic cleaning is a required part of delivering an enhanced matrix, not an
optional follow-up. After extraction, inspect the source crop, normal matrix,
faction debug preview, stray image and border/hole images. Use the pixel-cleaning
skill to correct clear quantization artifacts, wrong-material flecks, faction
leakage and genuinely missing outlines or accidental holes in small reviewed
batches. Check all eight neighbors and the local material before each edit.
Preserve intentional highlights, flowers, vine shapes, thin diagonal weapons
and internal limb boundaries. Rerun detectors and visually inspect the final
comparison before showing the matrix. Report exact changed-pixel counts and
remaining uncertainty; zero detector flags is not a delivery requirement.

Extract the single output through `pinkbox-generation/scripts/extract.py`. Run the
**pixel-cleaning** review bundle on each candidate. Correct extraction artifacts,
material leakage and genuine border defects one by one; do not use cleanup as an
excuse to redesign the accepted unit. A technically cleaner sprite can still be
worse if it loses the approved pose, mood, anatomy or weapon readability.

Compare the raw crop, base matrix and candidate matrix at the same native scale.
Show the clean matrix render in one faction colour. Inspect player/enemy and
faction debug checks internally. Describe the
requested enhancement and any actual deviations. Preserve rejected candidates
and exact baselines so a revert restores the named version without guesswork.

Integration follows the generation skill's canonical-source and recipe rules.
Do not replace the approved base while experimenting. A user's prior approval
or explicit integration instruction supplies authorization; do not add a new
permission loop just because this is a skill.
