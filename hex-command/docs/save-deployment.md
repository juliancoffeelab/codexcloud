# Reinforcement identity in saved campaigns

New saves identify deployed reinforcement waves by an authored, mission-local
name and record `campaignDeploymentFormat: "named-waves-v1"`. Runtime spawning
and player observations use that name. Reordering the definition cannot change
which wave a save has already resolved.

Untagged saves from published `3956f5c` and `7a30040` import numeric markers
through each wave's explicit historical `legacyIndex`. Revision-one Lantern
village and Ashen finale saves already contained the subsequently delayed
troops; their new waves are marked deployed without replacing wounded troops,
changing gold or allocating new IDs. Revision-two saves retain pending waves.
Subsequent saves write named markers. Both published layouts have independent
preservation and repeated-load probes.

The untagged intermediate draft `1acc1dd` was never published to Sites. It used
a different Ashen numeric order and the same revision as an older published
layout. Those two formats cannot be distinguished reliably from the saved
metadata. Its traces remain exactly replayable in their frozen checkout, but
importing that ambiguous draft is not a supported migration. A draft save may
need a new chapter attempt; do not claim universal compatibility or infer wave
identity from surviving troop health.
