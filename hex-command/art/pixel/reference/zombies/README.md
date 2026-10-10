# Chosen zombie lineup and archived studies

The chosen artwork remains in `../../units/`. Its branch assignments are recorded
in [the lineup](../../lineups/zombies.json), with a [review grid](lineup-player.png).

| L1          | L2           | L3              | Role                |
| ----------- | ------------ | --------------- | ------------------- |
| Thin walker | Thin fighter | Cleaver fighter | Armed melee         |
| Thin walker | Thin caster  | Plague carrier  | Disease and debuffs |
| Thin walker | Shambler     | Ghoul           | Unarmed melee       |

The [Corruption mage commander](../corruption-commander/README.md) has a separate
L1–L3 artwork row. The original mage is preserved as L2; new L1 and L3 use caster
and carrier build influences without copying those units. Gameplay integration remains future work.

Refresh the chosen review grid from canonical sprites with
`node art/pixel/reference/zombies/lineup.cjs` from the project root.

These eight designs are the selected base poses reviewed with the user, including
both faction matrices, palettes and native PNGs. Their later material cleanup is
preserved in [the race cleanup recipes](../cleanup/README.md). The artwork catalog records
their chosen tiers. `changes.json` retains the original L1 construction recipe;
that source recipe tier is separate from the new lineup assignment. Gameplay
integration remains future work.

All other 28 humanoid zombie studies, including the alternate L2/L3 recolors,
are preserved as complete directories in [units/](units/). Their original source
metadata is preserved in [catalog.json](catalog.json). They are excluded from the
active catalog and public pixel atlas. Zombie bears remain in the beast lineup.

Raw generations, prompts, extraction, cleanup and construction recipes remain in
[faction-expansion/](../faction-expansion/). Its historical family sheets include
both active designs and archived studies; they do not describe the chosen tree.
The review gallery labels chosen tiers and reference variants explicitly.

The expansion rebuild resolves every output to its active or archived directory,
and preserves the chosen catalog tiers. Rebuilding the studies does not promote
unused variants back into the active units folder.
