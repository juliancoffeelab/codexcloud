# Faction anatomy expansion

Generated pink-box PNGs have been removed after matrix approval. References
below to `templates/` describe the historical generation process. For new work,
use [the project pinkbox-generation skill](../../../../.agents/skills/pinkbox-generation/SKILL.md)
from the project root. Approved `units/*/sprite.txt` matrices are authoritative;
saved raw sources and rebuild recipes remain available for historical inspection.
Do not replay construction recipes as routine artwork verification.

63 generated anatomies: 38 humanoid/mounted poses and 25 beasts. Each has cleaned
L1–L3 matrices in both faction colors under `art/pixel/units/`. `reuse.cjs` adds
nine L3 variants from existing L2 poses without overwriting those sources.
These are artwork prototypes; game definitions and advancement trees are separate.

## Review

- [Humanoid overview](overview-humanoids.png)
- [Beast overview](overview-beasts.png)
- [Local gallery](gallery.html): all tiers, faction switch, and per-pose links
- [Existing L2 → L3](existing-l3-player.png)
- Each race has `<race>-player.png` and `<race>-enemy.png` tier boards, with
  L1/L2/L3 arranged left to right.
- `grids/<id>-source.png` and `grids/<id>-matrix.png` share **zero-based native
  column/row coordinates**. The source overlay shows the matching original crop;
  the matrix overlay shows the palette-cleaned result. A retained 64px frame keeps
  equipment and mounted/beast anatomy intact instead of clipping it to 48px.

## Pink mask and generation

`manifest.json` records every equipped pose, palette, race vibe and three chosen
canonical references. `prepare.cjs`, `beasts.cjs`, and `templates.cjs` build a
1024×1024 four-quadrant template: three nearest-neighbor enlarged sprites plus
one empty magenta slot. Each quadrant represents 64×64 logical pixels. Foot
soldiers are requested within its central 48px footprint; mounts and beasts may
use the full frame with padding.

Pass the template to imagegen with the corresponding saved prompt. Generate the
**complete anatomy, weapon and gripping hands together**. `sources/` holds the
unchanged returned PNGs and captured prompts. The first five skeleton prompts
were not captured verbatim; their `.recipe.md` files reconstruct the instructions
from the saved manifest and should not be mistaken for exact prompt transcripts.

The fox guard, crocodile and octopus were rerolled for a malformed shield,
unreadable fourth leg, and insufficiently separated arms, respectively. Spider,
crab and basilisk references were also replaced to expose their complete limb
counts; the basilisk uses a direct top view with three feet on each side. The
selected crocodile is seen diagonally from above to expose all four feet. The
selected octopus is frontal with four joined arms on each side. Rejected raw
outputs remain in `rejected/` for comparison and are never atlas inputs.

The crossbow and pickaxe dwarves were regenerated from empty pink slots with
actual dwarf anchors to correct their size. Both use complete 64px native frames
alongside the other dwarves. The pistol goat rider and axe-and-pick infantry are
two additional complete generated poses, with L1–L3 artwork in both factions.
Their current pose-specific material and tier masks live in
[dwarf-renewed.cjs](dwarf-renewed.cjs); the older helmet and metal-apron masks
must not be applied to those new anatomies. Previous crossbow/pickaxe sources,
prompts, templates and extracted studies remain in
[rejected/dwarf-scale-2026-10-08/](rejected/dwarf-scale-2026-10-08/README.md).

## Rebuild saved artwork

Run from the project root:

```sh
npm run art:expansion
```

To rebuild selected base poses, pass IDs, for example
`npm run art:expansion -- skeleton-flanker giant-spider`. The runner stops on a
failed step. It runs extraction, humanoid/animal cleanup, tier construction,
existing L3 recipes, review sheets/grids, and the ordinary atlas build. It does
not rerun image generation. Formatting stays in `npm run format` / `npm run style`.

Extraction removes solid pink, maps to explicit flat RGBA colors, records isolated
debris removal and retains a full frame when the requested crop would lose pixels.
It does not repair anatomy. Material cleanup uses masks for the actual saved
poses; edits are logged in native-slot coordinates and preserve transparency.
Tier recipes preserve limbs, grips and faction markings. Three weakling zombie
families grow through the corresponding thin adult and bulky anatomies; other
families reuse their pose with material and equipment changes. Beast tiers use
natural coat/scale colors, while bone beasts change bone tones. These visual tiers
do not establish costs, stats or game recruitment.

Manual edits belong in matrices and must also be reflected in the owning recipe
before running it again. `components.md` and `changes.json` accompany each unit.
Do not paint only the rendered PNG or use this extractor on an arbitrary sheet.

## Visual review notes

Review covers weapon attachment, hand/arm continuity, tails, visible limb count,
palette ownership, and both faction colors. Four-legged creatures retain all four
legs; spiders have eight, the basilisk six, insects six, and the octopus eight
arms. Dragon/gryphon wings are separate from their walking legs. Black pupils,
mouths, rib gaps and attached claws are intentional anatomical features; random
material-colored texture flecks are removed. Dwarves are deliberately shorter
than humans, elves slender and taller, and weakling zombies shorter or hunched.

The generated poses have more equipment detail than some older sprites. The
overview sheets are intended for art-direction review before gameplay integration,
including deciding which zombie branches and beast tiers to retain.

## Merfolk body and tail renewal

The trident, net-and-spear and mace-and-shield families were regenerated against
Reef guard, Tide hunter and Tidekeeper. They use compact upright bodies and the
old roster's connected curled supporting tail. Tridents are upright too. The
harpoon thrower and existing roster remain unchanged.

The three `templates/merfolk-{trident,netter,maceguard}.png` files retain the actual generation
inputs; current raw outputs live in `sources/`. The previous raw generations,
extracted sources and pose-specific material records are retained under
`rejected/merfolk-tail-2026-10-09/`. Never replay those masks on the new anatomy.
`merfolk-renewed.cjs` owns connected-surface skin cleanup and the new equipment
tiers; selected extraction and tier rebuilds route through that recipe.

```sh
npm run art:expansion -- merfolk-trident merfolk-netter merfolk-maceguard
```

The merfolk faction boards follow the selected trees in `art/pixel/lineups/merfolk.json`:
Harpooner branches into Harpooner or Netter; Reef guard branches into Reef guard
or Two handed trident. Wavemender branches into support and crushing mages; Macer retains one branch. Tidekeeper →
Tide seer → Tide sovereign is the commander line. Each board row shows one L1–L3
path, repeating the shared starter for branching trees. Netter and Two handed
trident prototype L1 artwork remains available as source material rather than
separate starters. The duplicate harpoon-thrower family is archived in
`../merfolk-archive/`. This defines the artwork lineup; gameplay registrations
and balance remain separate.

The Tide sovereign is an independently generated L3 artwork tier. Its template
uses Tidekeeper, Tide seer and Tide sage as anchors. The saved source and native
extraction remain under `sources/` and `extracted/`. It adds regal shell armour,
shield and crown without a skin-colour change. Gameplay currently ends at L2
Tide seer; registering L3 progression and balancing stats is separate from this
artwork grid. Generate only the merfolk boards with:

```sh
node art/pixel/reference/faction-expansion/process.cjs boards merfolk
```

Aquatic infantry tiers and both mage branches are rebuilt from `../merfolk-tier-designs/`; see its README for saved generation templates, source images, extraction and native verification.
