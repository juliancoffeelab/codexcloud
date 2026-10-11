# Combat balance

This pass targets Mixed versus Mixed army battles across all four implemented
factions and all four arena maps. Woodland should retain its forest advantage,
merfolk its island advantage, and humans versus undead should be roughly even
on each map. Arena pools, budget (36), recruitment limit (one per phase) and
round cap (30) remain unchanged. Each faction pairing uses both seats and both
initiatives. Round-capped matches stay in every win-rate denominator.

The skeletal mage and merfolk macer lines are fully recruitable, promotable and
included in campaign pools. Skeletal mages have 14 HP, cost 8, move 3 and use
10-damage ranged Impact; their promotion has 18 HP and 11-damage ranged Impact.
Merfolk macers have 20 HP, cost 8, move 4 and use 7-damage melee Impact; their
promotion has 26 HP and 9-damage Impact. Reef guards have a 4-damage Impact tail.

The subsequent balance adjustments are:

| Unit | Before | After |
| --- | --- | --- |
| Human adept | 16 HP, Spark 6 | 17 HP, Spark 7 |
| Stone mage | 22 HP, Spark 8 | 23 HP, Spark 9 |
| Scout | Knife 4 | Knife 5 |
| Ranger | Knife 6 | Knife 7 |
| Wolf and dire wolf | Movement 5 | Movement 4 |

All other stats for these units are unchanged. The human impact caster becomes
worth recruiting against skeletons, while scouts contribute enough damage to
support it. Wolves keep their racial forest movement and protection bonuses.

## Tuning evidence

Seeds 1–10 were used for initial trials, then 11–60 for expanded tuning. The
selected candidate's human-versus-undead win rates on seeds 11–60 were 52% on
plains, 54% in woods, 59.5% at crossings and 53.5% on islands. Woodland won 46.8%
on plains and 72.2% in woods; merfolk won 75.2% on islands. Each pair/map has
200 games, and each faction/map total has 600 games against the other factions.

Rejected changes remain in the ignored experiment artifacts: stronger human
casters made humans dominate undead on islands; reducing human bow damage
severely harmed their other matchups. Blunt melee buffs were also removed after
the scout changes closed the gap. The complete valid tuning reports, recorded
unit stats and all faction/map totals are indexed in
`artifacts/balance-tuning/experiments.csv`. Every run retains its outcomes and
detailed gameplay journal. An incorrectly selected duplicate baseline was marked
invalid and excluded; valid baselines were checked against the native fingerprint
with the historical data directory explicitly selected.

## Final evidence

The final candidate is checked on untouched seeds 61–110, followed by the complete
ten-seed arena panel for all three policies. The final results and artifact links
are added here when those runs finish.

The report covers every implemented faction, opponent, map and policy pair,
including self-play. Artwork-only lineups are listed separately with their actual
simulation coverage; unimplemented artwork does not produce fabricated results.
Riders have gameplay stats but are outside these arena recruitment pools. This
pass measures the authored armies and AI policies rather than optimal player play.

## Reproduce and inspect

Use the [combat simulation skill](../.agents/skills/combat-simulation/SKILL.md).
The one-command workflow runs the simulation and complete analyzer together:

```sh
npm run ai:combat -- --policies mixed --seed 61 --seeds 50 --out artifacts/mixed-run
npm run ai:combat -- --seeds 10 --out artifacts/all-policies-run
```

Add `--baseline PATH/TO/report.json` for paired comparisons with an identical
schedule. `analytics/index.html` is searchable; `analytics/summary.md` contains
every matrix and unit-event row; `analytics/full-data.zip` contains the complete
CSV exports, unit catalogs and compact outcome journals. Full gameplay remains
in the original `.details.jsonl` files. `race_map_policy.csv` keeps each policy
separate; its faction/map totals exclude self-play. `matchups.csv` contains all
opponents, including self-play, and keeps both policies separate.
