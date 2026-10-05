# Normal and Low-poly roster additions

The original artwork is retained. Necromancer and Lich artwork assignments were
swapped in both styles and both factions following playtest feedback: the simpler
gold-edged robed caster is the Necromancer, and the original caster with skull
ornaments and heavier armor is the Lich. Pixel artwork keeps its own assignments.
These additions cover the twenty-one missing unit types, both factions, plus a
neutral worker portrait. Generated reference sheets live under `reference/`;
the individual, transparent sprites under `normal/` and `low-poly/` are the
source assets. The packer combines complete sprite canvases with gutters and
records their coordinates; it does not guess atlas cell boundaries at runtime.

Each style has `units/<type>/player.png` and `enemy.png` for all 34 unit types,
and `portraits/worker/portrait.png`. Build with `npm run art:build` and check with
`npm run art:check`. Both commands handle pixel matrices and these source sprites.
`scripts/pack-styled-art.ts` losslessly packs each style into its single
`public/assets/units-<style>.png` and writes `src/presentation/styled-art.ts`.
The loader extracts only the recorded whole-frame rectangles into standalone
images before publishing them to the UI. No rank aliases or pixel fallbacks are
used in Normal or Low-poly.

The reference imports isolate connected transparent silhouettes. Their recorded
bounds are in `reference/layout.json`. A low-alpha cutoff removes faint generator
background residue before the standalone source sprites are saved. Colors are
not sampled from JPEGs, quantized or interpolated during atlas packing.

All figures face screen right in a three-quarter game view, lit from above left.
Humanoid bodies have consistent height; wolves and boars are naturally shorter.
Full weapons, tails, cloaks and fins fit inside their sprite canvas. Merfolk have
a connected fish tail and fins, not boots or human legs. Faction cloth is teal
for player and crimson for enemy, while skin, bone, fur, steel and gold keep their
material colors. Normal uses painted fantasy shading; Low-poly uses flat,
faceted triangular planes. Neither style uses pixel sprites.

## Components and progression

| Unit          | Components and materials                                                                                                                                                           |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Swordsman     | Human skin hands and face, leather cap/vest/shield rim/boots, ivory cloth, faction scarf, wood shield, steel sword. Less armor than Heavy infantry.                                |
| Spearman      | Same beginner equipment, wood spear with steel point, wood shield. Less armor than Shield warden.                                                                                  |
| Marksman      | Hooded human, reinforced leather shoulders/chest, faction cloak, ivory cloth, long wood bow and attached shoulder quiver.                                                          |
| Ranger        | Human with charcoal clothes, faction cloak, leather belt/boots/bracers, two knives visibly held in separate hands.                                                                 |
| Stone mage    | Human, gold-edged faction hood/mantle, ivory robe, leather belt/boots, wood staff with a centered small crystal in a forked gold crown.                                            |
| Marshal       | Human in silver plate armor, gold helmet crest and shoulder borders, ivory/faction tabard, steel sword, rich faction standard.                                                     |
| Revenant      | Visible bone ribs/hands/legs, jagged dark iron face-guard crown with open sockets and exposed teeth, shoulder/shin plates, steel sword, faction shield/cape.                       |
| Death archer  | Bone skull/ribs/hands/legs, steel shoulders/wrists, leather boots/belt, faction cape, reinforced wood bow, shoulder quiver, thin centered arrow.                                   |
| Necromancer   | Compact menacing bone skull and hands, square dark sockets, small nose opening, dark hood cavity, gold-edged faction cloak, ivory split robe, metal chest guard, wood skull staff. |
| Dire wolf     | Natural wolf anatomy, four connected legs, ears, attached muzzle/fangs, near-black fur with connected raised shoulder/back mane, faction harness.                                  |
| Great boar    | Four connected hooves, stocky boar, pale ivory/gray fur, dark bristled spine, two attached curved tusks, small angry eye, faction harness.                                         |
| Briarweaver   | Orange fox head/ears/tail, ivory muzzle/paws/tail tip, leaf/vine shoulder mantle, faction cloak with gold edge, leather belt/boots, attached leafy branch staff.                   |
| Grove warden  | Human, antler crown, faction hood/cloak, connected green leaf shoulder mantle, ivory sleeves/robe, leather belt/boots, attached branching leaf staff.                              |
| Reef guard    | Blue-green scaled head/hands/tail/fins, ivory shell armor/shield, leather belt/cuffs, faction mantle, wood spear/trident with steel tips.                                          |
| Trident guard | Violet scales, silver breastplate/shoulders, larger ivory shell kite shield with gold rim, broad steel trident, faction mantle.                                                    |
| Harpooner     | Blue-green scales/tail/fins, light ivory cloth, leather bracers/belt, faction mantle, wood harpoon with steel barb.                                                                |
| Tide hunter   | Violet scales, dark leather armor, faction mantle, double-barbed harpoon held in hand, two attached spare shafts behind shoulder.                                                  |
| Wavemender    | Blue-green scales/tail/fins, ivory robe, faction hood/mantle, leather belt, wood staff with a small pale pearl/crystal.                                                            |
| Tide sage     | Violet scales, flowing gold-edged faction mantle, ivory robe panels, larger round pearl centered in gold staff cradle.                                                             |
| Tidekeeper    | Blue-green scales/tail/fins, shell armor, faction mantle, leather belt, trident staff.                                                                                             |
| Tide seer     | Violet scales, gold reef crown, ceremonial ivory/gold shoulders and stole, faction cloak with gold edge, trident staff with tied streamer.                                         |
| Worker        | Neutral human villager, brown/ivory work clothes, empty hands and leather boots, no faction cloth, weapon or armor.                                                                |

Reference sheet ordering is six columns by four rows, left to right:

1. Swordsman, Spearman, Marksman, Ranger, Stone mage, Marshal.
2. Revenant, Death archer, Necromancer, Dire wolf, Great boar, Briarweaver.
3. Grove warden, Reef guard, Trident guard, Harpooner, Tide hunter, Wavemender.
4. Tide sage, Tidekeeper, Tide seer, Worker, empty, empty.
