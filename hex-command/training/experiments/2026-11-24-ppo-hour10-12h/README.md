# Twelve-hour PPO continuation from hour 10

Completed 2026-11-24: **43,200.359 saved trainer seconds, 3,301 updates,
11,370 completed training episodes and 845,056 decisions**. All 13 hourly panels
finished: 7,150 native evaluation matches (13 × [312 + 214 + 24]).

The held-out-only selector chose **new-run hour 11**, at 39,619.576 trainer
seconds. This is a saved experimental candidate, not a promoted/deployed model.
Hour 0 denotes the original prior experiment's final hour-10 actor.

## Results on identical fixed cases

| Measurement | Start (hour 0) | Selected hour 11 | Final hour 12 |
|---|---:|---:|---:|
| Held-out balanced score | 23.30% | 28.41% | 24.62% |
| Held-out arena wins | 101/264 | 117/264 | 119/264 |
| Held-out campaign clears | 4/48 | 6/48 | 2/48 |
| Held-out turn-limit caps | 34 | 27 | 21 |
| Training diagnostic score | 45.56% | 54.16% | 55.07% |
| Ordinary diagnostic wins | 15/28 | 19/28 | 16/28 |
| Targeted diagnostic wins | 103/150 | 116/150 | 115/150 |
| Broad campaign diagnostic clears | 5/36 | 7/36 | 11/36 |
| Lantern full clears | 4/24 | 4/24 | 1/24 |

Balanced scores use the frozen evaluator's weights; they are not raw wins / all
matches. The held-out panel assigns equal total mass to arenas and campaigns.
The diagnostic score weights logical maps, rather than pooling all physical
matches. The final diagnostic increase did **not** select the checkpoint.

## Opponent strata (held-out)

| Stratum | Start | Selected hour 11 | Final hour 12 |
|---|---:|---:|---:|
| arena/classic | 50/88 | 63/88 | 59/88 |
| arena/full | 24/88 | 25/88 | 28/88 |
| arena/predictor | 27/88 | 29/88 | 32/88 |
| campaign/classic | 1/16 | 1/16 | 1/16 |
| campaign/full | 2/16 | 2/16 | 1/16 |
| campaign/predictor | 1/16 | 3/16 | 0/16 |

The arena gain at hour 11 is concentrated against Classic (50→63/88); Full
24→25/88 and Predictor 27→29/88 improve much less. Held-out campaigns gain two
clears against Predictor; Classic and Full totals match the baseline. Hour 12
retains arena gains but loses four of hour 11's six held-out campaign clears.
These are observed outcome changes, not evidence of a particular cause.

## Broad training campaigns, by visibility

Authored = clear; fog = **player-only fog**, opponent unrestricted. Each cell is
full campaign clears / starts. Opponent presets are pooled across the frozen
Classic/Predictor/Full panel; diplomacy is unrestricted.

| Campaign / visibility | Start | Hour 11 | Hour 12 |
|---|---:|---:|---:|
| rl-splitwake-crossing / authored | 1/6 | 1/6 | 4/6 |
| rl-splitwake-crossing / fog | 2/6 | 1/6 | 0/6 |
| rl-underground-escape / authored | 0/6 | 0/6 | 0/6 |
| rl-underground-escape / fog | 0/6 | 0/6 | 0/6 |
| rl-woodland-defense / authored | 1/6 | 5/6 | 6/6 |
| rl-woodland-defense / fog | 1/6 | 0/6 | 1/6 |

Underground full completion remains unresolved at the selected and final
checkpoints. Woodland's improvement is mainly clear: 1→5→6/6, while fog is
1→0→1/6. Splitwake clear improves at hour 12, but fog falls 2→1→0/6. Pooling
visibility conceals these different outcomes.

Lantern's conditional chapter funnel is 10/24 → 9/10 → 4/9 at the start,
13/24 → 6/13 → 4/6 at hour 11, and 15/24 → 4/15 → 1/4 at hour 12.
The opening improves while later conversion regresses at the final checkpoint.
Later chapters receive different survivors/carryover, and have small denominators;
these are natural campaign funnels, not independent fresh chapter starts.

## Controls, allocation and recovery

Initial actor SHA-256:
`e8dcc209f5cc420a616cfe6648551d19a8b476b235b1f59d67a108042eb2ebee`.
Frozen source: `60537725f26b968b5d8eb3597e931a06b3efb0d2`.
No imitation or architecture changes. Existing policy warm start initializes fresh
critic, optimizers, sampler and RNG; the reference is the original hour-10 actor.
Actor updates begin after the existing 24-episode critic warmup. Recovery bundles
contain actor, critic, reference, both optimizers, sampler/mastery and RNG states.
The hour-00 bundle is the original source checkpoint supplied to the warm start;
its old optimizers were not loaded at launch.

Temperature 0.35, actor LR 5e-6, critic LR 1e-3, four native environments,
horizon 64, reference KL coefficient 0.02. Arena win +1; genuine intermediate
campaign chapters +1 and finale +2, existing failure/cap penalties. No kill or
objective-progress bonuses. All frozen controls are in `protocol.json` and
`rl/parameters.json`.

Actual final decision shares: **24.9989% ordinary /25.0008% targeted /50.0002%
broad campaigns**. All 14 ordinary, 28 targeted and 3 broad logical maps received
positive incremental work at every hourly monitor check. Adaptive sampling stayed
inside those groups; no reserved targeted allocation beyond its authored 25%.
No faulty-allocation phase occurred in this run. The earlier experiment's
allocation bug is not part of these 12 hours.

There were no trainer/evaluator restarts or observed training outages. Launch
included a successful supervisor reattachment test. One snapshot attempt at
06:44 UTC logged FileNotFoundError without the missing path; subsequent snapshots
recovered automatically. Final local/shared pointers and every archived hourly
checkpoint bundle passed hash validation. No failed snapshot was accepted as a
valid checkpoint.

## Coverage and limits

Each hour uses held-out 312, training diagnostic 214, Lantern 24. See
`coverage.json` for exact seed ranges, limits and recruitment caps; raw configs
retain factions, opponent blocks, initiative, map and visibility conditions.
Broad fog affects the learner only. Targeted clear/player-fog/both-fog/sight
conditions retain their explicit frozen map effects and faction settings.

Selection uses only held-out 312, baseline eligible, worst-stratum tolerance
0.025 then weighted score and fewer caps. Diagnostics and Lantern never choose
the winner. No extra confirmation sweep was run, as instructed. There is one
training seed, repeated fixed panels and selection across 13 checkpoints: small
campaign counts and checkpoint selection prevent a strong generalization claim.
The diagnostic maps are training maps with separate seeds; their gains do not
prove transfer to new layouts. Lower value loss or higher training return alone
is not evidence of stronger gameplay. Adaptive training return curves also mix
changing difficulty.

## Saved evidence and curves

- `evaluation/hourly-summary.json`, `selection.json`, `comparison-and-chapters.json`:
  all hourly scores, opponent strata and exact chapter entrants/clears.
- `evaluation/hour-*/monitor/*.json.gz` and `*.jsonl.gz`: all raw native outcomes.
- `plots/gameplay-over-time.png`: fixed panel scores.
- `plots/arena-per-map.png`, `targeted-per-map.png`, `campaign-per-map.png`:
  fixed-seed per-map outcomes.
- `plots/broad-campaign-chapters.png`, `lantern-focused-curves.png`: conditional
  chapter success curves; counts retained in evaluation JSON.
- `plots/training-curves.png`, `training-return-by-pool.png`,
  `actual-allocation.png`: optimization, live returns and actual exposure.
- `rl/metrics.jsonl.gz`, `sampler-status.json.gz`, `allocation-history.jsonl.gz`,
  monitor records and logs: training and recovery evidence.
- `recovery/hour-00` through `hour-12`: compressed native actors and PPO states,
  with actual times and SHA-256 in `hourly-checkpoints.json`.
- `recovery/frozen-runtime-inputs.tar.gz`: exact Python/native/catalog inputs.
  Recreate the Python 3.12 runtime using `runtime-versions.json`; do not rebuild
  against a later global catalog. `scripts` retains evaluation, plotting,
  supervision and archival code. No further training is authorized by this archive.

Local run: `tmp/rl/ppo-hour10-continuation-12h-20261124`.
Shared recovery: `/workspace/shared/ppo-hour10-continuation-12h-20261124`.
