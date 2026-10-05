# Woodland guards and the third comparisons

The transcripts in [results.json](results.json) were replayed by frozen `208c655`.
That version adds objective-defense behavior to the woodland camp guards and
Lantern village guards. Coastal comparisons on the same version are still running.

| Run | Camp chapter | Vigil | Crossing | Campaign complete | Casualties |
| --- | --- | --- | --- | --- | --- |
| Independent Luna, mixed recruits | Win, 22 turns | Loss at turn 2 twice | Unreached | No | 4 in chapter one, 3 across failed vigil attempts |
| Independent Luna, boar recruits | Win, 19 turns | Win, 4 turns | Win, 8 turns | Yes | 7 |
| Root developer, mixed route study | Win, 9 turns | Win, 4 turns | Win, 5 turns | Yes | 2 wolves |

The independent mono-recruitment policy performed better. This prompted a new
balance pass: boars inherit their race's piercing vulnerability; wolves cost 5
and thorncallers 7, so the starting 12 gold affords both roles. Both policies will
be tested again, with the same read-only enemy-turn forecast available.

The developer study establishes a legal, efficient mixed route, including ranged
pressure, scouting, melee screens, promoted companions and Impact support at the
crossing. It preserves Dain and has no losses after chapter one. It is **not an
independent or unbiased comparison**: the developer knows the implementation and
had read earlier failures. It must not replace the failed independent mixed run.

No failed attempt or invalid action was erased. Earlier successful actions from an
invalid batch remain in the trial, as they do in ordinary play. All enemy responses
and final states were reconstructed exactly by the live engine replay.
