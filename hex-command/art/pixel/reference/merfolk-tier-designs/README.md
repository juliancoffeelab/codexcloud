# Aquatic L2–L3 armour and mage branches

Generated pink-box PNGs have been removed after matrix approval. References
below to `templates/` describe the historical generation process. For new work,
use [the project pinkbox-generation skill](../../../../.agents/skills/pinkbox-generation/SKILL.md)
from the project root. Approved `units/*/sprite.txt` matrices are authoritative;
saved raw sources and rebuild recipes remain available for historical inspection.
Do not replay construction recipes as routine artwork verification.

The top row of each saved pink template contains the previous L2/L3 sprites.
Imagegen fills the lower-left L2 and lower-right L3 with aquatic material designs.
Original returned images are retained in `sources/`; extraction preserves their
native grid rather than rescaling their anatomy.

Infantry wears fitted orange coral cuirasses, attached shoulder guards and
articulated abdominal protection over leather. L2 uses small coral shoulders and
restrained lower coverage. L3 has broader shell-faced shoulders and fuller
abdominal bands. Shield branches advance from wood with coral framing to carved
ivory shell faces. Trident geometry remains the previously reviewed design.
Macer heads are faceted river stone; the L1 conversion is owned by
`../merfolk-materials/build.cjs`.

Wavemender branches into support (Tide sage → Tide oracle) and crushing
(Tide crusher → Tide breaker). Support uses connected shell collars, flowing
cloth and shell-and-pearl staff heads. Crushing uses coral shoulders, shell
clasps and heavy river-stone pressure staffs. These are artwork roles; this
recipe does not assign combat statistics or spells.

Rebuild extraction from the project root, for each family and level:

```sh
node art/pixel/reference/merfolk-tier-designs/extract.cjs \
  art/pixel/reference/merfolk-tier-designs/sources/reefguard.png \
  art/pixel/reference/merfolk-tier-designs/extracted/reefguard-l2 48 8 8 0
```

The final argument selects lower-left (0/L2) or lower-right (1/L3).
Reefguard and trident use `48 8 8`; macer, crushing and support use `64 0 0`.
The additional canvas retains taller stone/staff heads without clipping or
changing body scale. Palette quantization excludes reserved red faction symbols.

```sh
node art/pixel/reference/merfolk-tier-designs/build.cjs
node art/pixel/reference/faction-expansion/process.cjs boards merfolk
npm run art:build
```

`build.cjs` records the native skin cleanup, preserved trident heads and reduced
L2 two-handed lower armour. Canonical sources are editable matrices and palettes
under `art/pixel/units/`. Prior matrices and obsolete cleanup ownership remain in
`baseline/`; old coordinate recipes must not be applied to these new poses.
The expansion rebuild runs this recipe after its older tier and reuse recipes.

Review both faction boards at native pixel scale. Check fitted chest/shoulder
joins, shield and hand occlusion, continuous tails and physical staff collars.
The reviewed ten sprites each form one connected foreground component; larger
L3 shoulders, abdominal coverage and staff heads distinguish the tiers.

## Restrained mage revision

Support and crushing now use `sources/{support,crushing}-revised.png` and
`extracted/{support,crushing}-revised-l{2,3}`. Both use `64 0 0` extraction.
New L2 designs bridge the plain Wavemender and the previous L2 artwork. Previous
L2 bodies are preserved exactly at L3 from `revisions/`, with only the smaller
staff head transferred from the revised generation. The old extravagant L3
sources remain saved as rejected design history rather than active sprites.

## Light hunter armour

Harpoon hunter and Netter L2 use leather vests/harnesses and wrist guards with
exposed upper arms. L3 adds a small fitted shell shoulder cap and forearm brace
over leather, rather than the infantry coral cuirass and belly plates. Saved
`hunter.png` and `netter.png` generations retain weapons and mesh grip references.
Extract each lower slot with `64 0 0`, then rebuild using `build.cjs`.

Harpoon hunter revision: current `hunter.png` uses plain leather chest panels and small fitted shell guards, replacing the crossed-strap and large shell-cap draft archived in `revisions/hunter-rejected/`. The current template is `templates/hunter-redo.png`. Rebuild this branch alone with `node art/pixel/reference/merfolk-tier-designs/build.cjs hunter`.

Hunter and Netter tier promotion: the previous L3 sprites are retained exactly
as the new L2 in `revisions/{hunter,netter}-promoted/`. New mildly embellished L3
designs come from the bottom-right slot of `sources/{hunter,netter}-fancier.png`.
Extract only that slot with `64 0 0 1`; the bottom-left slot is deliberately empty.

## Commander family

Commander L1 is a young lieutenant: fitted leather, modest coral shoulders,
shell trim, wooden shield and restrained coral-and-shell circlet. L2 and L3
advance to coral cuirasses and shell shields, with ivory officer sashes and
short tabards distinguishing command from infantry. Crown size and body scale
stay restrained. Each `sources/commander-l{1,2,3}.png` contributes its bottom-right
slot; extract with `64 0 0 1`. Rebuild with `build.cjs commander`. Prior commander
geometry and obsolete outline corrections remain in `baseline/`.
