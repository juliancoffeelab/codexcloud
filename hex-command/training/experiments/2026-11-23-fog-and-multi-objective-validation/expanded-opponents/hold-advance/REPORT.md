# Ashbank Relay under returning fog

Classic was the original campaign baseline because it is the live campaign enemy. This extension holds the map, rosters, waves and objectives fixed and repeats the comparison against four enemy configurations. Classic uses the Classic recruiter and planner. Queued uses the Balanced recruiter and Queued planner. Full uses the Predictor recruiter and Full planner. Predictor uses the Predictor recruiter and Queued planner. All use the Mission campaigner and have zero recruits.

The clear fixture is byte-identical to the earlier stage. Fog uses the authored three-hex returning concealment on all 187 tiles. In player-only fog the enemy penetrates it. In both-side fog both factions must reveal tiles locally. Native replay found 95 initially hidden tiles and four hidden enemy units from the player viewpoint. For the enemy under both-side fog, 97 tiles and five opposing units started hidden. The probe also found initially visible tiles hidden again at the end for surviving fog-limited factions. Every sampled opponent configuration moved and attacked in every visibility setting. [Native visibility and replay audit](audit-summary.json)

## Results

The discovery matrix used eight matched seeds per cell and all five players, for 480 games. Cells advanced when a rule-based player won at least six and led Hour 10 by at least three. Ten cells advanced to 20 fresh seeds for their selected reference and both checkpoints, for 600 further games. Confirmation required at least 15 reference wins and a six-win gap over Hour 10. No game reached the cap. [Discovery and confirmation matrices](matrices.md)

| Visibility | Enemy | Selected reference | Reference | Hour 8 | Hour 10 | Confirmation |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Clear | Classic | Classic | 14/20 | 3/20 | 8/20 | Missed reference minimum |
| Clear | Queued | Classic | 16/20 | 1/20 | 4/20 | Passed |
| Clear | Full | Queued | 19/20 | 5/20 | 10/20 | Passed |
| Clear | Predictor | Classic | 16/20 | 1/20 | 4/20 | Passed |
| Player-only fog | Queued | Full | 12/20 | 7/20 | 1/20 | Missed reference minimum |
| Player-only fog | Full | Queued | 15/20 | 6/20 | 9/20 | Passed |
| Player-only fog | Predictor | Full | 12/20 | 7/20 | 1/20 | Missed reference minimum |
| Both-side fog | Queued | Queued | 20/20 | 0/20 | 0/20 | Passed |
| Both-side fog | Full | Full | 18/20 | 8/20 | 1/20 | Passed |
| Both-side fog | Predictor | Queued | 20/20 | 0/20 | 0/20 | Passed |

Classic under either fog setting peaked at five reference wins out of eight in discovery. The two settings stayed at screening. Clear Classic retained a six-win gap against Hour 10 on confirmation, while the reference fell to 14 wins compared with 18 in the earlier confirmation.

Queued and Predictor opposition shared a planner and produced the same discovery and confirmation win counts. Per-seed successes agreed in all 120 discovery pairs and 180 confirmation pairs. Two pairs in each panel differed in terminal round or defeat reason. [Paired terminal differences](audit-summary.json)

Against Queued in both-side fog on seed `571241001`, all three players took the fort by round 7. The Queued player completed the ten-round hold. Hour 8 and Hour 10 both lost the village during the game and ended at 3/10 with the fort owned. On the same seed against Full, Full and both checkpoints captured the fort on round 3. The reference completed the hold. Hour 8 and Hour 10 lost village ownership and ended at 6/10 and 4/10. These are sampled examples of hold failures after a successful advance. All 72 sampled replays reproduced the original terminal outcomes. [Replay trajectories](replay-evidence.json) · [Exact schedules, results and fingerprints](final-summary.json)
