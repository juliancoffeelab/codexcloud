# Scree Relay validation

**Use the blocked-road version as the new route challenge. Keep the bypass-threat version as its evaluation control.** Moving a single dreadguard changes which route Full chooses. The learned checkpoint at hour10 consistently completes the control and consistently fails the challenge.

Both fixtures are 18×11 surface maps. The captain starts at (4,4), the ranger at (8,4), and the exit is (16,4). A connector near (6,5) leads to the bypass along row 7. The dreadguard starts at (10,4) in the challenge and at (10,7) in the control. Win by bringing a living captain to the exit by the end of turn 7. The enemy is active Classic, both sides have clear sight, and recruitment is disabled. Player Campaigner and Recruiter are Mission and Situational throughout. The main selection uses the normal Full planner as its reference. [Map and captain paths](scree-relay.png) · [Exact fixture and configuration paths](final-summary.json)

## Results

| Version | Cohort | Hour8 | Hour10 | Full |
| --- | --- | ---: | ---: | ---: |
| Blocked road | Discovery | 0/8 | 0/8 | 8/8 |
| Bypass-threat control | Discovery | 0/8 | 8/8 | 8/8 |
| Blocked road | Fresh confirmation | 0/20 | 0/20 | 20/20 |
| Bypass-threat control | Fresh confirmation | 0/20 | 20/20 | 20/20 |

Discovery ran seeds 56180101–56180108. Confirmation ran seeds 56180201–56180220. Every failed game was an authored defeat. The 100-turn simulation cap was never reached. The blocked road produced twenty deadline defeats for each learner. In the control, hour8 produced nineteen deadline defeats and one commander loss. [Discovery](v8/discovery/results.jsonl) · [Confirmation](v8/confirmation/results.jsonl) · [Frozen configs and model digests](v8/confirmation/protocol.json)

On the blocked road, Full sent the captain toward the connector on turn 1 in all twenty cases, took the bypass, and arrived on turn 5. Hour8 and hour10 both sent the captain west to (2,4) on turn 1 in all twenty cases. Neither learner moved the captain onto the bypass before the deadline. Hour10 still had its ranger alive in all twenty cases, so the measured failure is captain route choice and progress even when the ranger survives. In the control, Full and hour10 both used the direct road in every case. Full also kept the ranger alive in all twenty control wins. Hour10 lost the ranger in every control win. All 120 detailed replays matched the frozen evaluator winner and terminal round. [Phase records and compact action traces](v8/confirmation/traces)

## Iterations and limits

The screened iterations with a separate ranger relay objective did not give a sustained reference advantage. The validated fixture uses the existing named-captain exit objective and restages the route decision with the threat already present. It measures a response to a visible threat and does not test remembering a prior enemy-free phase. Fog, mirrored maps, and training transfer are unmeasured.

The optional two-enemy version v6 adds pursuit pressure on the bypass. In its fresh blocked-road panel, hour8 and hour10 went 0/20, Full and Queued each went 20/20 and Classic went 16/20. Hour10 already took the bypass in all five detailed traces but lost its captain each time. Both learner results in v6's blocked-road panel were twenty commander losses. Treat v6 as a separate stress test for continuing under pursuit, with v8 preferred for route choice. [v6 confirmation](v6/confirmation/results.jsonl) · [v6 phase records](v6/confirmation/traces)

Experiments used disposable copies of the frozen native evaluator catalog. The authored playable catalog, sampling mix, reward, and checkpoints have no changes. [Fixture builder](build.py) · [Matched evaluator](evaluate.py) · [Phase replay script](trace.py)
