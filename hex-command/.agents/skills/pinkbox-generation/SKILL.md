---
name: pinkbox-generation
description: Generate new Hex Command pixel units with one unit per pink box, extract shared matrices, and review faction ownership. Use for new anatomy or poses; use pinkbox-enhancement for an existing unit's tier or equipment change.
---

# Pinkbox generation

Work from the Hex Command project root. Read `art/pixel/WORKFLOWS.md` and the
relevant race's approved matrices and lineup. Tell the user when using this skill.
The matrix is the reference, not an old generated template. Helpers require
Python 3 and Pillow; repository renderer/detector modules remain authoritative.
Scripts prepare and extract images; image generation uses the available imagegen
tool, not a shell command.

## User-facing previews

Show a clean nearest-neighbor render of the unit or lineup in one faction colour.
Use the requested faction; otherwise use the most recently requested faction,
or enemy if none is established. Show both factions only when explicitly requested.
Keep debug colours, detector overlays, coordinates, source paths and before/after
boards internal unless the user asks to see diagnostics or a comparison. Still
run and visually inspect required diagnostics yourself. A matrix preview means
the rendered unit from its matrix, not a diagnostic sheet. Lineups may have simple
unit names and tier labels.

## Prepare

Resolve requested units, native size, anatomy, equipment, grips and style anchors
from the conversation. Each template is a **2×2 grid: exactly three occupied reference boxes and one
empty output box at bottom right**. One-reference or two-reference templates
are invalid, including for tier enhancements and concept variants. A requested three-unit
lineup or three variants needs three separate templates and generation calls. Use 48×48 for normal infantry unless the
approved family uses 64×64; cavalry uses 64×64. Do not cram multiple tiers into
one box. Use current approved matrices as anchors, preferably simple palettes.

```sh
python3 .agents/skills/pinkbox-generation/scripts/template.py tmp/new-unit/template.png \
  --anchors scout ranger archer --label 'New unit' --size 48
```

Supply exactly one output label. The helper requires exactly three anchors and always makes a 2×2 pinkbox
with the single empty box at bottom right. Captions sit outside extraction regions and swatches below. It writes a geometry/palette/hash manifest
beside the PNG. It accepts unit IDs or paths to shared `sprite.txt` matrices.
If two anchors use the same symbol for different colors, the manifest assigns a
new symbol. Those symbols need material review after extraction; they are not
proof of semantic ownership. Avoid unnecessarily combining large palettes.

Inspect the actual template before invoking imagegen. Supply its local path.
Prompt requirements:

- Preserve reference boxes, layout, captions, canvas aspect ratio and palette.
- Fill only the single empty box with one complete unit.
- Specify pose, hands, actual grips, weapon placement, proportions and facing.
- Native block pixels, flat supplied colors, no antialiasing, gradients, ground
  shadow or texture noise. Keep meaningful parts separated by clear outlines.
- Pure magenta background. The frame is a logical 64×64 grid; for 48×48 output
  keep all artwork within the central 48×48, leaving eight pixels of padding.
- Use the supplied player faction base/shadow only on designated cloth, never
  skin, hair, bone, weapons, outlines or a whole shield. Identify faction pieces
  explicitly; a shield stripe is only included when requested.

Save the unchanged returned PNG and exact prompt. Inspect the result before
extraction; imagegen may resize the canvas or change layout. Never infer success
from its description alone.

### Mandatory style and resolution contract

Every imagegen prompt must explicitly require the approved anchors' pixel style
and logical resolution. Specify the native grid (48×48 or 64×64), its placement
inside the output box, and the template's block scale (512/64 = 8 image pixels
per logical pixel). If the returned canvas is resized, the logical grid remains
unchanged: do not let imagegen add finer detail to use the extra image pixels.
Require the same chunky blocks, outline thickness, flat material regions and
limited supplied palette as the anchors. Forbid subpixel marks, antialiasing,
gradients, fine texture, extra shading colors and detailed illustration rendered
to look vaguely pixelated. Preserve all occupied reference boxes unchanged and
fill only the single empty box.

Visually compare the returned unit with the anchors before presenting it as a
valid candidate. Reject and regenerate style or resolution drift; quantization
and cleanup must not conceal a generation that ignored this contract. These
requirements apply to concept variants too, even when the user defers matrices.

## Extract and review

```sh
python3 .agents/skills/pinkbox-generation/scripts/extract.py \
  tmp/new-unit/generated.png tmp/new-unit/template.json tmp/new-unit/extracted
python3 .agents/skills/pixel-cleaning/scripts/review.py \
  tmp/new-unit/extracted/unit-1/sprite.txt tmp/new-unit/review-1
```

Repeat generation, extraction and review for each separately requested variant. The extractor uses manifest fractions,
current supplied colors and a 35% block foreground threshold. It saves raw PNG,
crops, the full 64×64 extraction and extraction reports. It rejects empty boxes,
changed aspect ratio and any visible pixels outside the requested native crop.
It never discards perimeter rows to force a fit. If geometry changed, inspect
and record explicit corrected geometry or regenerate. If anatomy overflows,
regenerate or choose a justified larger frame; never silently clip/rescale limbs.

Faction assignment by quantization is provisional. Inspect normal player/enemy
and debug matrix previews: collar versus face, sleeve versus hand, cloak versus
feet, shield stripe versus shield body, outlines versus cloth. Only `^`/`*` may
change across factions; fixed materials and alpha must match exactly. The unit
palette must exclude `^`/`*`; those resolve through `art/pixel/factions.json`.
Resolve merged outline symbols to the intended outline material before relying
on the detector's default `O` classification.

Use **pixel-cleaning** for review and individually reasoned corrections. Show
clean native nearest-neighbor matrix renders every time. Compare with the
approved family internally; show that comparison only when requested. Do not substitute the generated artwork for the matrix preview.

When the chosen candidate is authorized for integration, store its shared matrix,
material palette, raw input, prompt, extraction manifest and guarded corrections
in the appropriate canonical/reference locations. Update the owning recipe so a
rebuild preserves corrections. Gameplay trees/stats change only within requested
scope. After canonical edits, rebuild artwork and run `npm run art:check`.
