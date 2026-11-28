# Kiln Crossing validation

Kiln Crossing failed confirmation as a targeted curriculum candidate. In the discovery panel, Queued cleared 8/8 in player-only radius-2 fog and 6/8 in both-sides radius-2 fog. Hour10 cleared 5/8 and 3/8. The gaps did not reproduce on 20 new matched seeds per condition. Classic was the active enemy in every confirmation game. Each fog variant used radius 2 and the full-vision control used the same board and roster.

| Visibility | Hour8 | Hour10 | Queued | Full |
|---|---:|---:|---:|---:|
| Player-only fog | 14/20 | 11/20 | 13/20 | 9/20 |
| Both-sides fog | 14/20 | 16/20 | 15/20 | 8/20 |
| Full vision | 9/20 | 9/20 | 9/20 | 2/20 |

No confirmation game reached the harness cap. All 24 hour10 defeats were commander losses across the three visibility conditions. In the two traced player-only fog cases where Queued won and hour10 lost, hour10 withdrew its commander to `(0,7)` on round 2 and lost its remaining army at the commander’s death in round 7. Queued won those seeds on rounds 5 and 6. The traces cover examples of isolation rather than a measured failure to regain visibility. Both holdout checkpoints cleared at least 14/20 in both-sides fog.

The frozen candidate is the 11×8 `rl-kiln-crossing-screen-close-fog`, with Warden, Soldier, Hammerman, Healer and Scout facing Revenant, Skeleton Axeman L2, Skeleton, Deatharcher and Skeleton Impact Caster. Both ranged crews must fall before round 11 while the human leader survives. The renderer uses the same terrain categories and adds road artwork connections only to its preview input. Juniper Works remained at the proposal stage.

- [Qualification, outcome counts, seed list and fingerprints](final-summary.json)
- [Frozen candidate map](catalog/game/campaigns/rl-kiln-crossing-screen-close-fog/map.json) and [preview](render/preview.png)
- [Frozen candidate roster](catalog/game/campaigns/rl-kiln-crossing-screen-close-fog/roster.json) and [objective script](catalog/game/campaigns/rl-kiln-crossing-screen-close-fog/scripts.json)
- [Native confirmation rows](confirmation.jsonl) and [sampled paired phase traces](paired-mechanism-traces.jsonl.gz)
- [Catalog builder](build.py), [native runner](evaluate.py), [trace wrapper](trace.py) and [trace adapter source](trace-adapter.rs)
