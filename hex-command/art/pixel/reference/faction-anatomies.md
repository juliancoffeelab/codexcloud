# Faction anatomy and weapon references

The faction artwork list below is approved. Entries are not hypothetical or
optional. Gameplay stats and advancement details still need design. Animals and
beasts are included in the overnight artwork expansion; gameplay integration
remains separate from the artwork.

## Humans — artwork ready

- Sword/shield and spear/shield infantry.
- Two-handed hammer and halberd infantry.
- Bow and crossbow infantry.
- Knife flankers and slingers, including masked and fire variants.
- Mounted lancer and mounted archer.
- Staff mages and healers.
- Banner commander with crystal-sword progression.

Human artwork is finished through L3 in both faction colors and stored as
canonical matrices in `art/pixel/units/`.

## Skeletons

- Sword and shield: exposed ribs, bent knees, shield held forward.
- Bow: existing anatomy, with higher-tier equipment.
- Dual-dagger flanker: lean skeleton, low forward stance, both daggers visibly
  held in its hands, short faction-colored cloak.
- Impact staff caster: blunt stone or bone staff head, free casting hand and
  crushing-force magic.
- Scythe caster / vampiric lich branch: long shaft, curved blade above and beside
  the skull, one hand gripping the shaft and the other casting soul-draining magic.
- Two-handed axe: both hands gripping it, broad blade angled beside the head.
- Skeletal horse and mounted spear wielder: generate horse and rider together.

The skeleton crossbow is dropped. The flanker and impact caster are the immediate
priorities. Generate improved caster head anatomy: skull, jaw, eye sockets and hood
opening together, proportioned against the other skeleton bodies. The scythe
variant needs its own equipped reference, rather than a staff recolor.

Bias skeleton references toward the existing skeleton warrior proportions. The
current lich's oversized head is not a proportion reference; caster skulls should
match the warrior and archer instead.

## Zombies

- Unarmed shambler: hunched torso, reaching hands, torn clothes.
- Weakling wretch: thin, ragged civilian corpse, no armor or weapon.
- Weakling crawler: wasted limbs, planted hands and dragging legs, low silhouette.
- Weakling cleaver fighter: skinny civilian corpse with a small cleaver, no shield.
- Weakling corruption caster: frail body, short ragged robe and a plain branch staff.
- Skeleton-sized walker: thin adult corpse at the skeleton warrior's height.
- Skeleton-sized fighter: same thin adult scale, small cleaver and battered shield.
- Skeleton-sized corruption caster: same thin adult scale, modest staff and robe.
- Armed zombie: upright enough to hold a cleaver and battered shield.
- Ghoul: crouched, long forearms, claws, visibly different from a standing humanoid.
- Plague carrier: swollen body, sack or basket of remains on its back, one free hand.
- Corruption mage.

Zombie bear is retained with the animal expansion below.

## Woodland

The existing lineup is sufficient for now. Fox staff caster and fox healer already
exist; reuse their anatomies for branch development rather than listing them as
missing units.

- Fox spear-and-shield fighter: upright fox body, visible tail, bark or wicker
  shield and a spear held clear of the muzzle.
- Higher-tier wolves: retain the approved basic and dark dire-wolf bodies; develop
  integrated raised back fur and a readable wolf head.
- Higher-tier boars: retain the approved bodies; develop heavier shoulders, larger
  tusks and armor without obscuring the animal anatomy.

Treant and stag are included in the animal and beast artwork expansion.

## Elves

- Spear and narrow shield: lighter armor and slimmer silhouette than human infantry.
- Two-handed curved sword: blade held diagonally, both hands visibly gripping it.
- Bow: upright woodland archer, pointed ears visible.
- Dual knives: low, mobile stance; generate both weapons in their hands.
- Staff caster: flowing clothes, branch staff with a clearly shaped head.
- Deer-mounted archer: distinct mount anatomy from human cavalry.

## Dwarves

- Axe and round shield: short, broad body, substantial beard.
- Two-handed hammer: actual rectangular hammerhead, proper shaft and grip.
- Pickaxe: both hands on the shaft, clearly shaped pickaxe head with a pointed
  working end and a short counterweight.
- Long poleaxe: different stance from the hammer, with the axe blade designed into
  the reference.
- Crossbow: compact stock, broad limbs, held clear of the beard.
- Arquebus: wooden stock against the shoulder, long metal barrel, trigger hand and
  supporting hand beneath the weapon.
- Rune caster: one-handed hammer plus a glowing rune stone in the other hand.
- Goat-mounted hammerman: generate mount, rider, hammer and grip together.

The goat rider carries a hammer, not a lance.

## Merfolk

- Spear and shield: upright torso, curled tail supporting the pose.
- Two-handed trident: aggressive forward reach, weapon clear of the face.
- Harpoon thrower: throwing arm drawn back, other hand holding spare harpoons.
- Net and short spear: net visibly gathered in one hand.
- Staff caster: existing tail anatomy, staff and free casting hand.
- Mace and shield guardian: broader torso, armored tail, large shield suitable for
  shield bash; an alternative branch to spear and shield.

## Animals and beasts

These selections are included in the artwork expansion. Faction assignments remain
open where they have not been decided.

### Land

- Bear
- Lynx
- Stag
- Giant spider
- Giant tortoise
- Giant snake

### Flying

- Giant raven
- Giant owl
- Giant wasp
- Giant bat

### Aquatic and coastal

- Giant crab
- Crocodile
- Shark
- Ray
- Octopus
- Sea serpent: coiled body, raised head, fins; no humanoid torso.

### Undead

- Skeletal hounds
- Corpse vultures
- Skeletal horses
- Zombie bear
- Bone dragon: separately generated skeletal anatomy, including wing bones.

### Other fantasy beasts

- Gryphon: four legs, folded wings, hooked beak; leave room for later wing ornaments
  or a rider.
- Dragon: four legs plus two wings, long neck, readable head and tail.
- Basilisk: six legs, low body, broad reptilian head and crest.
- Treant: thick trunk torso, root feet, branch arms.

## Generation approach

Generate complete equipped poses: the weapon, grip and pose together.
Sword-and-shield, hammer, poleaxe, crossbow, trident and scythe each deserve their
own reference. Tier recolors reuse those poses; changing the weapon family should
not depend on reconstructing it pixel by pixel again.

Use the saved [artwork workflows](../WORKFLOWS.md) for preparation, extraction,
cleanup and canonical source storage. Preserve faction colors and consistent
anatomical proportions and detail across the full unit sheet.
