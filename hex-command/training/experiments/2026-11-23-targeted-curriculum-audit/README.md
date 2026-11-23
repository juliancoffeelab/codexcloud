# Targeted curriculum audit after hour10

Ran528 fresh-seed learned-planner games:66 configured conditions ×8 seeds,
starting at490000001. Native inference and frozen experiment catalog; no training
or environment changes. Also ran24 matched teacher games on the three persistent
failures. These seeds differ from training and the fixed hourly panel. Layouts
remain the authored training layouts: seed generalization, not unseen-map proof.

| Map | Fixed hour8 | Fixed hour9 | Fixed hour10 | Fresh hour10 |
|---|---:|---:|---:|---:|
| Force the Crossing | 2/2 | 2/2 | 2/2 | 12/16 (75%) |
| Crown Road Relief | 0/1 | 1/1 | 1/1 | 8/8 (100%) |
| Splitpine Dispatch | 2/3 | 1/3 | 0/3 | 0/24 (0%) |
| Echo Gate | 0/1 | 0/1 | 0/1 | 0/8 (0%) |
| Flint Switchback | 1/1 | 1/1 | 1/1 | 7/8 (88%) |
| North of Lantern Road | 0/1 | 0/1 | 0/1 | 0/8 (0%) |
| The Lantern Maze Handoff | 3/3 | 3/3 | 3/3 | 24/24 (100%) |
| Mistfen Patrol | 3/3 | 3/3 | 3/3 | 24/24 (100%) |
| Splitwake Patrol | 3/3 | 3/3 | 3/3 | 24/24 (100%) |
| Lower Gallery Refuge | 0/1 | 0/1 | 0/1 | 0/8 (0%) |
| Tidegate Ferries | 2/4 | 4/4 | 4/4 | 31/32 (97%) |
| Bracken Watch | 4/5 | 2/5 | 2/5 | 26/40 (65%) |
| Siltbar Approach | 0/1 | 1/1 | 0/1 | 6/8 (75%) |
| Herring Tide Road Three | 1/1 | 1/1 | 0/1 | 4/8 (50%) |
| South Ford West Post | 1/1 | 1/1 | 1/1 | 8/8 (100%) |
| Briar Saddle | 1/1 | 1/1 | 1/1 | 7/8 (88%) |
| Whitecap Ford | 3/3 | 3/3 | 3/3 | 22/24 (92%) |
| Slate Gallery | 4/6 | 4/6 | 4/6 | 37/48 (77%) |
| Rally at the Vault | 1/1 | 1/1 | 1/1 | 5/8 (62%) |
| Jawbone Breakthrough | 1/1 | 1/1 | 1/1 | 8/8 (100%) |
| Thorn Ford Breakthrough | 1/1 | 1/1 | 1/1 | 8/8 (100%) |
| Bellmarket Courts | 4/4 | 4/4 | 4/4 | 32/32 (100%) |
| Thorn Verge | 9/9 | 9/9 | 9/9 | 72/72 (100%) |
| Lair Outworks | 0/9 | 3/9 | 2/9 | 18/72 (25%) |

Teacher confirmation on exactly matched fresh cases: Echo Gate Full8/8 versus
learner0/8; North of Lantern Road Queued8/8 versus learner0/8; Lower Gallery
Refuge Full3/8 versus learner0/8. Small samples, especially the one-condition maps.

## Proposed changes, not implemented

Reduce the three fully solved escort maps from three combined base-map budgets
to one combined budget (one-third each). Reduce Bellmarket Courts and Thorn
Verge to one-quarter their current base budget. Retain some rehearsal and
monitor their scores. Existing adaptive downsampling already reduces many of
these; these proposals would alter BASE allocations and multiply with mastery.
No new authored field or sampling implementation is presumed here.

Keep the route failures and Lair Outworks: they remain unsolved. Do not remove
Flint Switchback/Rally at the Vault based on fixed1/1 scores; fresh results7/8 and
5/8, and stochastic training completion estimates are lower. Training samples
actions at temperature0.35; this evaluation is greedy, so their completion
statistics need not match.

Add a candidate named Commander Left Behind: derive a short encounter from
the observed Lantern split-army state. Commander must reach an exit alive,
fragile allies engage a blocking force and a pursuer threatens the rear. Build
variants where accompanying/supporting the army is useful and where wounded
withdrawal is correct. Use existing objective rewards. Require measured failure
by the current learner and success by a rule-based or targeted reference before
adding it to training. This is a proposed design, not a validated new map.

At hour10, three of four previously traced hour9 commander-loss cases still
failed the crossing: seeds480000010 and480000024 capped at100;480000018 suffered
leader_loss on80;480000013 cleared on24. No hour10 action trace was collected
for that check, so no specific repeated movement/idle behavior is inferred.
