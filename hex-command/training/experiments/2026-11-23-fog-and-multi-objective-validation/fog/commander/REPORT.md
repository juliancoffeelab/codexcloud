# Commander Left Behind under returning fog

Player-only fog qualifies for all three commander variants. Both-sides fog qualifies for the wounded version with a healer. Clear conditions remain qualified for all three. Every game used a fixed Classic enemy, the original forces and objectives, and zero recruitment. The healthy and HP-only wounded versions kept their turn-22 deadlines. The wounded version with a healer kept its turn-13 deadline. [Prior clear validation](../../new-env-validation/commander/REPORT.md) · [Catalog and fingerprints](final-summary.json)

| Variant | Visibility | Hour 8 | Hour 10 | Classic | Full | Queued | Qualifying reference |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Healthy | Clear | 0/20 | 0/20 | 18/20 | 19/20 | 18/20 | Full |
| Healthy | Player-only fog | 0/20 | 4/20 | 0/20 | 0/20 | 18/20 | Queued |
| Healthy | Both-sides fog | 5/20 | 4/20 | 2/20 | 2/20 | 0/20 | — |
| Wounded | Clear | 0/20 | 0/20 | 16/20 | 1/20 | 12/20 | Classic |
| Wounded | Player-only fog | 0/20 | 0/20 | 0/20 | 0/20 | 12/20 | Queued |
| Wounded | Both-sides fog | 5/20 | 0/20 | 0/20 | 1/20 | 0/20 | — |
| Wounded with healer | Clear | 0/20 | 3/20 | 20/20 | 15/20 | 20/20 | Classic |
| Wounded with healer | Player-only fog | 0/20 | 0/20 | 0/20 | 4/20 | 15/20 | Queued |
| Wounded with healer | Both-sides fog | 0/20 | 0/20 | 0/20 | 10/20 | 3/20 | Full |

Discovery seeds 491240001–08 and confirmation seeds 491241001–20 were disjoint. Each seed was reused across the five player policies and three visibility scopes. All 900 confirmation games finished with an authored result. A qualifying cell required at least 10 reference wins and eight more wins than hour 10. Reference planners were selected from earlier clear results and fog discovery. The eight-seed discovery reference for the wounded healer won 2/8 under player-only fog and 3/8 under both-sides fog, so the subsequent 15/20 and 10/20 estimates have higher sampling coverage. [Discovery scores](discovery-summary.json) · [Confirmation scores](confirmation-summary.json) · [Raw confirmation](confirmation.jsonl)

## Runtime visibility

The exact catalog uses `fog-reveal-3` on all 204 tiles of the fog maps. The authored objective route adds no events. The player and enemy have separate vision groups, and the selected scope explicitly sets each group's fog penetration. A native probe checked the resolved tile overlays, group assignments and engine-calculated faction presentation at the initial phase boundary and at each sampled game's conclusion. [Scope audit and probe fingerprints](final-summary.json) · [Probe source](probe/src/main.rs)

| Scope | Player penetrates fog | Enemy penetrates fog | Initially hidden tiles for player | Initially hidden tiles for enemy |
| --- | --- | --- | ---: | ---: |
| Clear | Yes | Yes | 0 | 0 |
| Player-only fog | No | Yes | 130 healthy or wounded, 122 with healer | 0 |
| Both-sides fog | No | No | 130 healthy or wounded, 122 with healer | 139 |

The probe also observed previously visible tiles become fogged again while each restricted faction still had surviving units. The unrestricted enemy in the player-only runs initially saw every unit and tile. In the fogged groups the initial presentation omitted actual units as well as tiles. [Exact initial and terminal presentations](trace-rules.jsonl.gz)

## Representative decisions

- **Healthy, player-only, seed 491241001.** Queued reverses off the ford after turn 5, takes the southern crossing with two allies, and secures both objectives on turn 12 at full HP. Hour 10 backs north after turn 4, has no surviving allies by turn 7, and loses Nera at the northern forest. Across the cell, 13 of hour 10's 16 defeats are commander losses. [Phase evidence](action-evidence.json)
- **Wounded with healer, player-only, seed 491241003.** Queued heals Nera to 15 HP by turn 4 and moves her along the southern crossing with three surviving allies. She exits on turn 10. Hour 10 reaches (6,5) at 7 HP with advancing allies by turn 2, remains there through turn 3, and loses Nera. All 20 hour-10 defeats in this cell are commander losses. [Phase evidence](action-evidence.json)
- **Wounded with healer, both-sides, seed 491241002.** Full retreats west on turns 5–7 and then crosses the south with two surviving allies, healing Nera to 25 HP before the turn-12 exit. Hour 10 recovers to 11 HP and dies before clearing the crossing. Full's 10/20 puts this condition at the qualification threshold. [Phase evidence](action-evidence.json)

The 42 selected action replays match the frozen evaluator's terminal round and winner. Each sampled success ends with the commander alive at the exit and friendly ownership of the eastern post. These are comparisons on a single layout and one enemy policy. [Raw hour-10 replays](trace-hour10.jsonl.gz) · [Raw reference replays](trace-rules.jsonl.gz)
