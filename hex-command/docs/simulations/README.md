# AI tournament results

The run stopped at the user’s request. These tables retain only complete matched seed blocks; incomplete blocks are preserved separately and excluded from comparisons.

**7236 retained runs:** 1944 preferred-recruitment arena games (seeds [1, 2, 3, 4, 5, 6, 7, 8, 9]), 5184 crossed arena games (seeds [1, 2, 4, 5, 7, 8]), and 108 campaign runs (seeds 1–3).

Frozen game/policy source: `578fae2`. All three batches use the same source fingerprint. No language models play these games.

The [side-bias and campaign investigation](investigation.md) identifies and fixes
classic's directional choices and an enemy-first arena round-clock bug, and
documents the subsequent shared leader-restraint rule. These tables remain
measurements of the original source; replay their archives on `578fae2`, not the
current implementation.

## Preferred recruitment

| Bot   | Opponent | Wins | Losses | Unresolved | Games |
| ----- | -------- | ---: | -----: | ---------: | ----: |
| mixed | rush     |  353 |     76 |          3 |   432 |
| mixed | classic  |  359 |     73 |          0 |   432 |
| rush  | classic  |  260 |    172 |          0 |   432 |

## Recruitment effect: same planner on both sides

| Planner | Mixed recruitment wins | Melee recruitment wins | Unresolved | Games | Net-win margin, 95% seed-block interval |
| ------- | ---------------------: | ---------------------: | ---------: | ----: | --------------------------------------- |
| classic |                    238 |                     50 |          0 |   288 | 0.653 [0.604, 0.708]                    |
| rush    |                    238 |                     45 |          5 |   288 | 0.670 [0.604, 0.747]                    |
| mixed   |                    236 |                     46 |          6 |   288 | 0.660 [0.583, 0.736]                    |

Mixed recruitment wins more often within each of the three planners. The advantage survives with the planner held constant. Holding recruitment fixed, rush edges the layered mixed planner in this sample; the large preferred-recruitment gap is primarily a recruitment effect. Recruitment rules do not force identical counts: affordability, captured income and casualties can still change each army.

## Recruitment effect by roster

| Roster   | Planner | Mixed wins | Melee wins | Unresolved | Games |
| -------- | ------- | ---------: | ---------: | ---------: | ----: |
| human    | classic |         69 |          3 |          0 |    72 |
| human    | rush    |         65 |          6 |          1 |    72 |
| human    | mixed   |         61 |         10 |          1 |    72 |
| woodland | classic |         47 |         25 |          0 |    72 |
| woodland | rush    |         49 |         21 |          2 |    72 |
| woodland | mixed   |         54 |         14 |          4 |    72 |
| undead   | classic |         71 |          1 |          0 |    72 |
| undead   | rush    |         70 |          0 |          2 |    72 |
| undead   | mixed   |         67 |          5 |          0 |    72 |
| coastal  | classic |         51 |         21 |          0 |    72 |
| coastal  | rush    |         54 |         18 |          0 |    72 |
| coastal  | mixed   |         54 |         17 |          1 |    72 |

## Planner effect: same recruitment rule

| Recruitment | Planner | Opponent planner | Wins | Losses | Unresolved | Games |
| ----------- | ------- | ---------------- | ---: | -----: | ---------: | ----: |
| rush        | mixed   | rush             |  127 |    142 |         19 |   288 |
| rush        | mixed   | classic          |  249 |     37 |          2 |   288 |
| rush        | rush    | classic          |  240 |     48 |          0 |   288 |
| mixed       | mixed   | rush             |  135 |    148 |          5 |   288 |
| mixed       | mixed   | classic          |  198 |     90 |          0 |   288 |
| mixed       | rush    | classic          |  173 |    114 |          1 |   288 |

## Campaign completion

| Campaign      | Completed | Defeated | Unresolved | Runs |
| ------------- | --------: | -------: | ---------: | ---: |
| lantern-road  |         6 |       12 |          9 |   27 |
| green-accord  |         0 |       21 |          6 |   27 |
| ashen-oath    |        11 |       13 |          3 |   27 |
| borrowed-tide |        13 |       11 |          3 |   27 |

Woodland has no complete campaign run under these policies. Some players stall at the two defended logging camps; runs that reach the vigil lose Lark. Earlier Luna woodland successes remain recorded, so this does not establish that the campaign is impossible or that these bots play it adequately. Arena composition results do not close campaign balance.

## Method and artifacts

Each arena matchup uses matched terrain/resources/starting troops, both seats and both initiatives. Required fort/village control and optional ruins are real scenario targets; real engine rules govern every command. Campaigns retain scripted waves, real objectives, earned carryover and no retries. Caps remain unresolved.

Intervals resample whole complete seed blocks 1,000 times using Python `random.Random(1)` for this stopped-run summary. They are descriptive, unadjusted intervals on `(wins − losses) / all games`, not an optimal-play or human-difficulty claim. The full factorial table shows every bot pairing; per-unit purchase/loss totals are in JSON. A unit never bought by a recruiter is not meaningfully assessed by its win rate.

Classic self-play can show board-direction bias. Counterbalancing includes both seats and initiatives; more seeds do not remove systematic policy/scenario bias. Baseline and crossed panels have different retained seed lists and should not be treated as identical paired samples.

- [Preferred tournament table](preferred.md) and [complete outcomes](preferred.json.gz).
- [Crossed tournament table](factorial.md) and [complete outcomes](factorial.json.gz).
- [Campaign records](campaigns.md) and [complete outcomes](campaigns.json.gz).
- [Excluded partial blocks](interrupted-blocks.json.gz).
- [Policy and simulation guide](../ai-simulation.md).

Verification: all 20 lightweight test files pass in 9.19 seconds; strict checks, formatting and production build pass. Sixteen worker-generated arena outcomes replayed exactly in a single process, and five actual campaign outcomes covering completion, defeat and caps replayed with identical final-state hashes. The game keeps its existing enemy; these new policies are testing tools.

To reproduce the raw completed-block panels, run preferred mode with `--seed 1 --seeds 9`; run factorial mode separately with `--seed 1 --seeds 2`, `--seed 4 --seeds 2` and `--seed 7 --seeds 2`; run campaign mode with `--seed 1 --seeds 3`. Default rounds are 30 and recruitment limit one. Worker count affects execution speed, not outcomes. `--replay docs/simulations/factorial.json.gz` verifies retained individual outcomes against the recorded source fingerprint.
