# Combat balance

This records the initial pass. See [the coastal follow-up](coastal-balance.md) for
current macer/healer stats and the subsequent complete Mixed-policy comparison.

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

Final validation used untouched seeds **61–110**: **6,400 physical arena
matches** per version, including self-play controls. The baseline is `697f66c`;
the selected gameplay source is `844a407`. Every scheduled match finished without
a worker error. Each cross-faction pair/map has **200 games**, and each faction/map
total below has **600 games** against the other three factions. Cells show
**win percentage (wins/losses/round caps)**. Caps stay in the denominator.

| Faction | open | wooded | crossings | islands |
| --- | --- | --- | --- | --- |
| human | 63.8% (383/215/2) | 52.0% (312/268/20) | 55.5% (333/262/5) | 50.7% (304/277/19) |
| woodland | 46.0% (276/322/2) | 72.8% (437/156/7) | 43.2% (259/334/7) | 20.2% (121/462/17) |
| undead | 63.2% (379/218/3) | 48.3% (290/298/12) | 49.7% (298/295/7) | 44.3% (266/305/29) |
| coastal | 26.3% (158/441/1) | 23.2% (139/456/5) | 50.0% (300/299/1) | 78.8% (473/120/7) |

Human versus undead, on exactly the same seeds and match configurations:

| Map | Before | After |
| --- | --- | --- |
| open | 28.5% (57/143/0) | 43.5% (87/111/2) |
| wooded | 25.5% (51/143/6) | 53.0% (106/83/11) |
| crossings | 22.0% (44/154/2) | 56.5% (113/85/2) |
| islands | 15.5% (31/147/22) | 52.0% (104/83/13) |

Woodland retains a clear forest advantage and no longer dominates plains.
Merfolk retains a clear island advantage. Humans versus undead lies between
43.5% and 56.5% human wins on every map. Generalist factions win more often
on plains because merfolk performs poorly there; these totals do not imply
that every cross-faction pairing should have a 50% result. Human-versus-undead
island games still have 13 caps out of 200; those outcomes are recorded as caps.

The independent validation supports the intended Mixed-policy targets. It does
not establish equal balance under every AI policy or optimal human play.

Complete validation data:

- [Searchable report](../artifacts/balance-final-mixed/analytics/index.html)
- [Every matrix and unit-event count](../artifacts/balance-final-mixed/analytics/summary.md)
- [Complete CSVs, catalogs and compact outcomes](../artifacts/balance-final-mixed/analytics/full-data.zip)
- [Every faction/opponent/map result](../artifacts/balance-final-mixed/analytics/matchups.csv)
- [Every paired match change](../artifacts/balance-final-mixed/analytics/match_changes.csv)
- [Raw current gameplay](../artifacts/balance-final-mixed/report.details.jsonl)
- [Raw baseline gameplay](../artifacts/balance-final-mixed-before/report.details.jsonl)

The complete **11,520-match** ten-seed arena panel includes all four factions,
all four maps, all three policies, every policy pairing, both seats and both
initiatives. It is compared with the saved pre-fix `697f66c` panel using identical
match configurations. Policies stay separate in every matchup table:

- [All-policy searchable report](../artifacts/balance-final-all-policies/analytics/index.html)
- [All-policy complete tables](../artifacts/balance-final-all-policies/analytics/summary.md)
- [All-policy complete data archive](../artifacts/balance-final-all-policies/analytics/full-data.zip)
- [All-policy raw gameplay](../artifacts/balance-final-all-policies/report.details.jsonl)

The all-policy panel also exposes a remaining Rush-policy imbalance. Human wins
against undead are 87.5% on plains (35/2/3), 57.5% in woods (23/6/11), 72.5% at
crossings (29/4/7), and 47.5% on islands (19/6/15), each out of 40 games. Rush
does not recruit the new ranged caster counters. These results are not evidence
that the Mixed-policy balance also fixes Rush. Classic human-versus-undead wins
are 50%, 50%, 55%, and 50% respectively on the same ten-seed panel.

## Every Mixed-policy opponent matchup

The diagonal contains self-play controls and counts both participants; mirrored
cross-faction cells describe the same physical matches from the opposite side.

### open

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 50.0% (100/100/0) | 66.5% (133/67/0) | 43.5% (87/111/2) | 81.5% (163/37/0) |
| woodland | 33.5% (67/133/0) | 48.0% (96/96/8) | 51.5% (103/96/1) | 53.0% (106/93/1) |
| undead | 55.5% (111/87/2) | 48.0% (96/103/1) | 50.0% (100/100/0) | 86.0% (172/28/0) |
| coastal | 18.5% (37/163/0) | 46.5% (93/106/1) | 14.0% (28/172/0) | 50.0% (100/100/0) |

### wooded

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 48.0% (96/96/8) | 32.5% (65/129/6) | 53.0% (106/83/11) | 70.5% (141/56/3) |
| woodland | 64.5% (129/65/6) | 48.5% (97/97/6) | 80.0% (160/40/0) | 74.0% (148/51/1) |
| undead | 41.5% (83/106/11) | 20.0% (40/160/0) | 47.0% (94/94/12) | 83.5% (167/32/1) |
| coastal | 28.0% (56/141/3) | 25.5% (51/148/1) | 16.0% (32/167/1) | 49.5% (99/99/2) |

### crossings

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 50.0% (100/100/0) | 61.5% (123/74/3) | 56.5% (113/85/2) | 48.5% (97/103/0) |
| woodland | 37.0% (74/123/3) | 47.0% (94/94/12) | 56.5% (113/83/4) | 36.0% (72/128/0) |
| undead | 42.5% (85/113/2) | 41.5% (83/113/4) | 49.0% (98/98/4) | 65.0% (130/69/1) |
| coastal | 51.5% (103/97/0) | 64.0% (128/72/0) | 34.5% (69/130/1) | 49.5% (99/99/2) |

### islands

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 47.0% (94/94/12) | 87.0% (174/23/3) | 52.0% (104/83/13) | 13.0% (26/171/3) |
| woodland | 11.5% (23/174/3) | 43.0% (86/86/28) | 28.5% (57/130/13) | 20.5% (41/158/1) |
| undead | 41.5% (83/104/13) | 65.0% (130/57/13) | 42.0% (84/84/32) | 26.5% (53/144/3) |
| coastal | 85.5% (171/26/3) | 79.0% (158/41/1) | 72.0% (144/53/3) | 50.0% (100/100/0) |


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

## Verification

`npm run verify` passed all nine groups: style, content, WASM bindings, UI, types,
Rust, artwork, browser tests and production build. The browser test command took
9.90 seconds, within the unchanged 20-second budget. Both final runs and baselines
completed their entire schedules. Native catalog/engine fingerprints, every
matchup denominator, paired match count, unit-event total and ZIP archive were
checked. CSV recruitment, promotion and casualty totals agree with the native
composition counters for every faction, policy and unit.
