# Hollowbank Extraction opponent and fog qualification

Only the two existing clear-vision Classic-opponent conditions qualified. The first validation used Classic as the standard campaign control. This expansion evaluated Classic, Queued, Full, and Predictor separately for both original layouts, with clear vision, player-only radius-three returning fog, and radius-three returning fog on both sides.

Every case retained the original map, rosters, rewards, capture–rescue–evacuation sequence, and turn-14 deadline. Both forces had zero recruits. The 100-round simulation cap never fired. Across 960 discovery and 360 confirmation games, there were zero harness errors. Fresh confirmation used different seeds on the same maps.

Enemy blocks were Classic / Mission / Classic, Balanced / Mission / Queued, Predictor / Mission / Full, and Predictor / Mission / Queued for Classic, Queued, Full, and Predictor, respectively. Native planner-entry captures verified those block selections. Queued and Predictor had identical terminal results and rounds in all matched cases sampled in this matrix. This is an empirical match under the fixed-roster setup.

## Discovery

Eight matched seeds per policy and cell, 493022001–493022008. An ordinary reference needed at least 6/8 clears and an advantage of at least three clears over Hour 10 to advance.

| Layout | Visibility | Enemy | Hour 8 | Hour 10 | Full | Queued | Classic |
|---|---|---|---:|---:|---:|---:|---:|
| East-road guard | Clear | Classic | 5/8 | 3/8 | 5/8 | 6/8 | 8/8 |
| East-road guard | Clear | Queued | 7/8 | 6/8 | 7/8 | 8/8 | 5/8 |
| East-road guard | Clear | Full | 7/8 | 6/8 | 6/8 | 4/8 | 5/8 |
| East-road guard | Clear | Predictor | 7/8 | 6/8 | 7/8 | 8/8 | 5/8 |
| East-road guard | Player-only fog | Classic | 0/8 | 3/8 | 0/8 | 1/8 | 6/8 |
| East-road guard | Player-only fog | Queued | 6/8 | 3/8 | 0/8 | 1/8 | 3/8 |
| East-road guard | Player-only fog | Full | 4/8 | 3/8 | 0/8 | 1/8 | 1/8 |
| East-road guard | Player-only fog | Predictor | 6/8 | 3/8 | 0/8 | 1/8 | 3/8 |
| East-road guard | Both sides fog | Classic | 2/8 | 2/8 | 0/8 | 0/8 | 2/8 |
| East-road guard | Both sides fog | Queued | 0/8 | 0/8 | 0/8 | 0/8 | 2/8 |
| East-road guard | Both sides fog | Full | 0/8 | 0/8 | 0/8 | 0/8 | 3/8 |
| East-road guard | Both sides fog | Predictor | 0/8 | 0/8 | 0/8 | 0/8 | 2/8 |
| West-bank reserve | Clear | Classic | 4/8 | 0/8 | 6/8 | 3/8 | 7/8 |
| West-bank reserve | Clear | Queued | 2/8 | 2/8 | 3/8 | 6/8 | 3/8 |
| West-bank reserve | Clear | Full | 5/8 | 3/8 | 5/8 | 4/8 | 6/8 |
| West-bank reserve | Clear | Predictor | 2/8 | 2/8 | 3/8 | 6/8 | 3/8 |
| West-bank reserve | Player-only fog | Classic | 1/8 | 2/8 | 1/8 | 2/8 | 3/8 |
| West-bank reserve | Player-only fog | Queued | 2/8 | 1/8 | 2/8 | 0/8 | 5/8 |
| West-bank reserve | Player-only fog | Full | 3/8 | 5/8 | 4/8 | 0/8 | 4/8 |
| West-bank reserve | Player-only fog | Predictor | 2/8 | 1/8 | 2/8 | 0/8 | 5/8 |
| West-bank reserve | Both sides fog | Classic | 1/8 | 0/8 | 1/8 | 1/8 | 3/8 |
| West-bank reserve | Both sides fog | Queued | 0/8 | 0/8 | 1/8 | 1/8 | 0/8 |
| West-bank reserve | Both sides fog | Full | 0/8 | 0/8 | 1/8 | 0/8 | 3/8 |
| West-bank reserve | Both sides fog | Predictor | 0/8 | 0/8 | 1/8 | 1/8 | 0/8 |

## Fresh confirmation

Twenty matched new seeds per policy and cell, 493023001–493023020. The reference was chosen from discovery. Qualification required at least 15/20 reference clears and at least six more clears than Hour 10.

| Layout | Visibility | Enemy | Hour 8 | Hour 10 | Discovery-selected reference | Qualified |
|---|---|---|---:|---:|---|---|
| East-road guard | Clear | Classic | 10/20 | 11/20 | Classic 20/20 | Yes |
| East-road guard | Player-only fog | Classic | 7/20 | 5/20 | Classic 9/20 | No |
| West-bank reserve | Clear | Classic | 14/20 | 5/20 | Classic 17/20 | Hour 10 only |
| West-bank reserve | Clear | Queued | 5/20 | 5/20 | Queued 12/20 | No |
| West-bank reserve | Clear | Full | 9/20 | 6/20 | Classic 7/20 | No |
| West-bank reserve | Clear | Predictor | 5/20 | 5/20 | Queued 12/20 | No |

The east-road guard under player-only fog advanced after Classic cleared 6/8 versus Hour 10 at 3/8. Fresh seeds brought Classic to 9/20 and Hour 10 to 5/20. All 39 defeats in that confirmation cell came from the authored deadline. Under fog on both sides the best ordinary reference reached 3/8, so those cells stopped at discovery. The earlier optional radius-two test looked at a separate tuning variant and also failed screening, with a best reference of Classic at 2/8.

The west-bank reserve with clear vision showed a seven-clear Hour 10 gap against Queued and Predictor, with the selected reference at 12/20 in each case. Against Full the reference reached 7/20 with a one-clear advantage over Hour 10.

## Native visibility and replays

The fog overlay is a radius-three concealment rule added at chapter start. Only the chosen observers lose fog penetration. Native captures restored both faction views and checked that acting planners see the same hostile unit IDs as the corresponding battlefield view. Visibility checks use snapshots with living observers and hostiles. Visible tiles returned to fog for each affected faction. Clear observers had zero fog tiles and all living hostiles visible.

- East-road guard, both sides fog, enemy observer. Living u1 left view between round 2 player and round 2 ai captures.
- West-bank reserve, both sides fog, enemy observer. Living u1 left view between round 2 player and round 2 ai captures.

Ten complete match replays matched winner, terminal round, and outcome text. On east-road guard seed 493023001 versus Classic, Hour 10 and the Classic reference both won in clear vision. Under player-only fog, Hour 10 rescued on round 8 and the reference on 10, after rescues on rounds 7 and 6 in clear vision. Both expired on round 14 under fog. The player-facing rescue goal appeared after capture, while the corresponding tactical target was missing from several fogged phases before the planners approached Mara.

On east-road guard seed 493022002 with both sides fogged, both players captured and rescued. Hour 10 reached neither exit and Classic reached Tavin’s exit. Both expired on round 14. These traces show where the sampled games lost time. The full matrix supplies the qualification counts.

## Artifacts

The exact simulation catalog is in `validated-catalog.tar.gz`. `panel.py` builds it and runs both panels. `final-summary.json` includes all terminal outcomes, seed lists, match settings, native/model hashes, and comparisons. `discovery/` and `confirmation/` contain raw native outcomes and separate protocols. `visibility/` contains engine state captures and faction views. `trace/` contains representative replays and their objective timelines. Nothing was integrated or trained.
