# Hollowbank Extraction

A one-stage, 18×12 field encounter. Capture the enemy-owned crossing post at (7,4). Once secured, approach Mara's stockade at (13,2) with any friendly unit to free her. This transfers her from a passive prisoner faction to the player. Both actors must survive and occupy their exits at the same time before turn 14. Mara goes southwest to (2,9) and Commander Tavin southeast to (16,9). The map supports two fords and a dry southern bypass. Native winning routes used all three crossings.

The east-road guard starts its Bonearcher on the eastern bank. The west-bank reserve starts the archer nearer the stockade and spawns one Skeleton on the western bank when Mara joins the player. Classic commands every hostile under ordinary combat rules. The reserve took actions in all 100 confirmed west-bank games.

## Fresh confirmation

20 matched seeds per planner and variant, 493021001–493021020. Both sides have clear vision. The five planners use the same authored objectives, fixed rosters, Classic opposition, unrestricted diplomacy, zero recruits, and a simulation cap of 100 rounds. Hour 8 and Hour 10 replace the same planner slot with their frozen model. All 300 matches reached an in-game result. The fresh seeds share the discovery layout.

| Fixture | Hour 8 | Hour 10 | Full | Queued | Classic | Decision |
|---|---:|---:|---:|---:|---:|---|
| East-road guard | 8/20 | 8/20 | 14/20 | 18/20 | 18/20 | Validated |
| West-bank reserve | 12/20 | 6/20 | 10/20 | 12/20 | 16/20 | Validated for Hour 10 |
| Both banks reserve | 12/20 | 13/20 | 10/20 | 16/20 | 14/20 | Discovery gap shrank on fresh seeds |

Classic won 11 seeds that Hour 10 lost and Hour 10 won one seed that Classic lost on each accepted fixture. The player-only radius-two fog variant stopped in discovery after Classic won 2/8 and both checkpoints won 0/8. Both the harder initial roster and the easier switchback settings were screened out in the 560 discovery games.

## Objective handoffs

200 phase replays of the two accepted conditions matched the frozen evaluation's winner, terminal round and outcome text. Each victory ended with both living player actors on their assigned tiles. The logs recorded the post's player ownership at capture, Mara's faction change at rescue, and the exit objectives becoming visible after rescue. Native planner target IDs were checked against the visible objective sets.

Each cell shows clears among entrants to that phase. Reaching an exit is a live requirement. A unit can get there early and remains exposed until both are safe.

| Fixture | Planner | Capture | Rescue | Both exits |
|---|---|---:|---:|---:|
| East-road guard | Hour 8 | 20/20 | 20/20 | 8/20 |
| East-road guard | Hour 10 | 20/20 | 17/20 | 8/17 |
| East-road guard | Full | 20/20 | 20/20 | 14/20 |
| East-road guard | Queued | 20/20 | 20/20 | 18/20 |
| East-road guard | Classic | 20/20 | 20/20 | 18/20 |
| West-bank reserve | Hour 8 | 20/20 | 20/20 | 12/20 |
| West-bank reserve | Hour 10 | 20/20 | 20/20 | 6/20 |
| West-bank reserve | Full | 20/20 | 20/20 | 10/20 |
| West-bank reserve | Queued | 20/20 | 20/20 | 12/20 |
| West-bank reserve | Classic | 20/20 | 20/20 | 16/20 |

On the east-road guard, three Hour 10 games stalled at rescue. Nine other losses reached evacuation and then hit the deadline. Five of those got only Mara to an exit, three got only Tavin there, and one reached neither. Classic and Queued reached Tavin's exit every time. Each of their two defeats expired before Mara's arrival.

On the west-bank reserve, Hour 10 freed Mara every time. Mara reached her exit in 18/20 and Tavin in 7/20. There were twelve deadline defeats and two Mara deaths. Classic got Mara to her exit in all 20 games. Its four defeats expired before Tavin's arrival.

- East-road guard, seed 493021006. Hour 10 freed Mara on round 5 and got her out on 11. With all hostiles gone, Tavin stayed on the captured post from round 6 through the deadline. Classic freed Mara on 6 and brought both actors to their exits on 11.
- East-road guard, seed 493021001. Both policies captured on round 2. Hour 10 received the rescue target after capture and failed to reach Mara even after the last hostile was gone by round 7. Classic rescued her on 8 and won on 13.
- West-bank reserve, seed 493021011. Hour 10 freed Mara on round 5. She reached her exit with 6 HP on 11 while Tavin stayed at the post. The newly spawned Skeleton killed her on 12. Classic killed it, got Mara to her exit on 11, and won on 13.

## Reproduction

`validated-catalog.tar.gz` contains the exact catalog used for confirmation and includes the rejected both-banks condition for replay. `fixtures/` contains just the two accepted stage packages. `final-summary.json` records protocol paths, frozen model and native hashes, paired seed outcomes, and per-stage trace metrics. `discovery/`, `discovery-2/`, and `confirmation/` hold each original seed list and raw native outcomes. `confirmation/traces-crossfire-late-guard-crossfire-nearside/` holds the verified phase replays, the probe metadata and the extracted objective progress. Render inputs and previews are under `renders/`.

From the game root in a shell that sourced `/workspace/.setup/env.sh`, run `build.py catalog-final crossfire-nearside crossfire-banks crossfire-late-guard` to recreate the catalog when absent. Run `evaluate.py catalog-final confirmation 493021001 20 --variants crossfire-nearside,crossfire-banks,crossfire-late-guard --players classic,hour10,full,queued,hour8` in the same script directory to recreate the original confirmation directory when absent. `trace.py confirmation --variants crossfire-late-guard,crossfire-nearside --players classic,queued,full,hour10,hour8 --seeds 20` recreates the phase traces from the preserved probe and CLI. All scripts and authored data are isolated from the playable catalog and training pipeline.
