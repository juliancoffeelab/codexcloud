# Ashbank Relay

A human force starts at Ashbank village on the west side of a river with two fords. Keep the village continuously staffed by any friendly troop for ten complete rounds. Capture the eastern fort by the end of round 7 and own both sites at victory. The Soldier begins wounded and can heal in the village. The Warhammer starts near the lower ford. Unrestricted skeleton reserves enter from the northwest in round 4, behind the fort in round 5, and from the north in round 8.

The event engine advances the hold counter at the start of each eligible player round and clears it when the village becomes unstaffed or changes hands. Victory reads the live counter and current ownership at both sites. Round 13 is the final deadline. Both sides have full vision, the roster is fixed, and all enemies use standard Classic planning. [Scenario files](pressure-catalog/game/campaigns/rl-ashbank-relay-fork-vigil) · [Starting map](render/preview.png)

## Confirmation

Twenty matched seeds from `571239001` through `571239020` followed eight discovery seeds. The opponent stayed Classic and the simulation cap was 100 rounds. Every win ended on round 11.

| Player | Wins | Defeats | Caps | Lost village |
| --- | ---: | ---: | ---: | ---: |
| Classic | 18 | 2 | 0 | 2 |
| Full | 12 | 8 | 0 | 2 |
| Queued | 10 | 10 | 0 | 1 |
| Hour 8 | 1 | 19 | 0 | 14 |
| Hour 10 | 3 | 17 | 0 | 17 |

All 20 Classic and all 20 Hour 10 attempts captured the fort. Classic's two defeats lost the village after capturing it. Hour 10's 17 defeats all lost the village. Seven Classic attempts took enemy damage within two hexes of both objectives in the same phase. In all 18 Classic wins, friendly troops were within two hexes of both objectives when the fort changed hands. Classic's Soldier staffed the village throughout each win and took 12–46 total effective HP damage at the village as it healed. The native replays reproduced all 300 terminals across the three pressure variants and verified both counters and owners at each win. [Confirmation and fingerprints](final-summary.json) · [Per-phase objective trajectories](pressure-confirmation-objective-trajectories.jsonl)

On seed `571239002`, Classic fought at both sites in round 5. Its Warhammer captured the fort in round 7 while the Soldier held the village. The Soldier fell to 8 HP during the round 8 attack and survived through the tenth hold tick. Hour 10 took the fort in round 4, had three troops near it in round 5, and lost the Soldier in the enemy phase of that round. The hold counter reset from 4 to 0 before the village changed hands in round 6. Hour 10 retook the village and ended at 4/10 with the fort still controlled. Hour 8 kept the village on that seed and missed the fort deadline. [Matched replay](matched-example-571239002.json)

## Other variants tested

- The earlier round-5 capture variant had no village losses in 100 matched attempts. Hour 8 went 0/20 and Hour 10 went 9/20 while Full went 11/20. Every defeat came from missing the fort deadline. In the accompanying round-7 variant, Hour 8 and Hour 10 won 14/20 and 17/20, respectively, against Full's 12/20. [Timing tests](confirmation-summary.json)
- The stronger ownership-only variant produced village losses and beat both checkpoints 10/20 to 0/20 with Full. Its other references won at most 4/20. The alternate eastern pressure variant reached 12/20 with both Full and Classic and 13/20 with Hour 10. [Pressure tests](pressure-confirmation-summary.json)
