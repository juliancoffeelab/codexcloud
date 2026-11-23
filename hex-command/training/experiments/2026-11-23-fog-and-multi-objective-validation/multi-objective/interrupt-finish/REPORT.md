# Marlow Cut

Marlow Cut uses normal arena victory rules. The enemy begins in control of the watch post at `(4,2)` with a public six-round hold score and zero rounds counted. Either side can win by holding the post for six rounds or defeating the opposing commander. Both sides have clear visibility and no recruits. The same player forces face live Classic opponents throughout. [Rules and provenance](final-summary.json)

The same battlefield and forces are used in both starts. The 18 HP enemy commander begins beyond the western switchback at `(1,17)` in Interrupt. This puts the post on the way to the commander. In Finish, the commander starts across the southern basin at `(16,8)`, within reach before the post matters. [Interrupt preview](renders/interrupt/preview.png) · [Finish preview](renders/finish/preview.png)

| Paired start | Queued | Full | Classic | Hour 8 | Hour 10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Interrupt | 20/20 | 19/20 | 20/20 | 20/20 | 20/20 |
| Finish | 20/20 | 20/20 | 20/20 | 13/20 | 12/20 |

Confirmation used fresh matched seeds 661135201–220. All 200 games ended under normal victory rules. In the discovery cohort of eight earlier seeds per start, both learners won all eight Interrupt games and five Finish games each. Queued won eight of eight in each. [Discovery](discovery-2/summary.json) · [Confirmation protocol](confirmation/protocol.json) · [Raw games](confirmation/games.jsonl)

- **Interrupt.** Queued's knight took the western approach in all 20 opening turns. The company captured the post on round 3 in 19 games and round 4 in one. It won by territory in every game. Both learners also captured the post in every game, and Hour 10 won nine by defeating the enemy commander afterward.
- **Finish.** Queued's knight advanced east in all 20 opening turns and attacked the enemy commander on round 2 in every game. Queued won 19 games on round 2 and one on round 3, with the enemy still controlling the post. Both learners sent their knight to `(7,4)` toward the post in all 20 openings, just as they did in Interrupt. They took the post on round 2 in every Finish game and never won by defeating the commander. The enemy recaptured the post and completed its hold in each of the seven Hour 8 defeats and eight Hour 10 defeats.
- **Matched witness, seed 661135202.** Queued moved the knight west to `(7,3)` for Interrupt and took the post on round 3. In Finish it moved east to `(14,9)`, attacked the commander on round 2 and finished on round 3. Hour 10 moved west to `(7,4)` in both starts. Its knight captured the post on round 2 in Finish. The bone archer killed the knight on round 5 and reclaimed the keep on round 7. The enemy completed its hold on round 13. [Matched actions](confirmation/representative-actions.json)

Observations collected from the frozen native binding showed the marked enemy objective and six turns remaining at `(4,2)`, then five and four as the enemy accrued rounds. After Queued captured the keep in the representative Interrupt sample, the next player phase showed the clock reset to six with enemy ownership cleared. All 200 detailed engine replays matched their native whole-game winner, reason and terminal round. [Observation evidence](confirmation/observation-evidence.json) · [Phase records](confirmation/traces/raw-phases.jsonl.gz)

Finish still permits a post-first win. The observed gap concerns recognizing the earlier and safer route. Commander placement also changes who can contest the post later, and the experiment keeps the six-round clock fixed. [Fixture builder](build.py) · [Evaluator](evaluate.py) · [Replay adapter](probe/src/main.rs) · [Renderer inputs](renders)
