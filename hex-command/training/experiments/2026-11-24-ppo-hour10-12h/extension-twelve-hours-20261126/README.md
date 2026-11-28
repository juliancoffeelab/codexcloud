# Completed additional twelve-hour PPO continuation

Completed 2026-11-26. Full-state resume from final hour 12 added approximately
43,200.779 saved trainer seconds, 2,426 updates, 7,876 completed training episodes
and 620,844 decisions. Final cumulative trainer time: 86,400.974 seconds,
5,727 updates, 19,246 episodes, 1,465,900 decisions.

All twelve new hourly panels 13–24 completed: **6,600 additional native matches**.
All 25 checkpoints 0–24 retain the same fixed panels (13,750 total matches including
previously completed results, which were reused without rerunning or alteration).

**Held-out-only selection chose hour 13**, at 46,916.639 trainer seconds, roughly
one additional hour after extension start. No model was promoted or deployed.
The final weights are not the selected candidate.

## Matched fixed-panel results

| Measurement | Prior selected 11 | Extension start 12 | New selected 13 | Final 24 |
|---|---:|---:|---:|---:|
| Balanced held-out score | 28.41% | 24.62% | 28.98% | 23.20% |
| Held-out arena wins | 117/264 | 119/264 | 120/264 | 117/264 |
| Held-out campaign clears | 6/48 | 2/48 | 6/48 | 1/48 |
| Held-out turn-limit caps | 27 | 21 | 24 | 24 |
| Training diagnostic weighted score | 54.16% | 55.07% | 52.11% | 51.78% |
| Ordinary diagnostic wins | 19/28 | 16/28 | 16/28 | 13/28 |
| Targeted diagnostic wins | 116/150 | 115/150 | 117/150 | 121/150 |
| Broad diagnostic campaign clears | 7/36 | 11/36 | 8/36 | 10/36 |
| Lantern clears | 4/24 | 1/24 | 3/24 | 0/24 |

Balanced held-out score gives equal total weight to arenas and campaigns; it is
not raw wins/312. Diagnostics weight logical maps rather than physical matches.
Diagnostics and Lantern never entered selection. The selector retains the
previous worst-stratum tolerance 0.025, then weighted score and fewer caps;
all previous candidates remained eligible.

## What changed

- The best balanced result moved only 28.41%→28.98% (prior selected 11→selected 13).
  Both clear 6/48 held-out campaigns. Hour 13 spreads these as 2/16 against each
  Classic/Full/Predictor, while hour 11 was 1/16,2/16,3/16.
- Held-out arena wins briefly peaked at 132/264 at hour 14 (50%), but finished
  117/264, below the extension start 119/264. There was no steady gain per hour.
- Targeted diagnostics improve 115→121/150 at the final checkpoint. Slate Gallery
  accounts for 8→11/12; Flint Switchback 0→2/2. Ashbank and Mistfen each gain one,
  while near-side Hollowbank loses one. Lair remains 9/18. These maps have unequal,
  often small, numbers of selected conditions/seeds.
- Lower Gallery Refuge and Echo Gate stay 0/2 at every checkpoint. Splitpine
  Dispatch ends 0/6. Additional unchanged training did not resolve those cases.
- Lantern is the clearest retention regression at the final checkpoint:
  start 12 funnel 15/24→4/15→1/4; selected 13 funnel 9/24→5/9→3/5;
  final 24 funnel 3/24→2/3→0/2, with 17/24 turn-limit caps. Chapter rates are conditional
  on entrants and use natural campaign carryover, not independent fresh starts.

These observations show uneven transfer and unstable checkpoints. They do not
identify a causal limit in the architecture, KL coefficient, rewards or curriculum.
No interventions or extra training were performed to establish such a cause.

## Held-out opponent strata

| Stratum | Prior 11 | Start 12 | Selected 13 | Final 24 |
|---|---:|---:|---:|---:|
| arena/classic | 63/88 | 59/88 | 59/88 | 58/88 |
| arena/full | 25/88 | 28/88 | 31/88 | 32/88 |
| arena/predictor | 29/88 | 32/88 | 30/88 | 27/88 |
| campaign/classic | 1/16 | 1/16 | 2/16 | 0/16 |
| campaign/full | 2/16 | 1/16 | 2/16 | 1/16 |
| campaign/predictor | 3/16 | 0/16 | 2/16 | 0/16 |

## Broad diagnostic campaigns by visibility

Authored is clear. Fog affects the learner only; the enemy is unrestricted.
Each cell is full campaign clears/starts, pooling the frozen opponent presets
(Classic/Predictor/Full). Diplomacy is unrestricted. Six cases per visibility
condition are insufficient to establish a reliable ordering.

| Campaign / visibility | Prior 11 | Start 12 | Selected 13 | Final 24 |
|---|---:|---:|---:|---:|
| rl-splitwake-crossing / authored | 1/6 | 4/6 | 0/6 | 1/6 |
| rl-splitwake-crossing / fog | 1/6 | 0/6 | 1/6 | 1/6 |
| rl-underground-escape / authored | 0/6 | 0/6 | 2/6 | 1/6 |
| rl-underground-escape / fog | 0/6 | 0/6 | 0/6 | 0/6 |
| rl-woodland-defense / authored | 5/6 | 6/6 | 5/6 | 5/6 |
| rl-woodland-defense / fog | 0/6 | 1/6 | 0/6 | 2/6 |

## Per-map diagnostics

All entries below use identical cases at each checkpoint. Opponent/visibility
conditions are pooled within each logical map; raw reports and chapter JSON retain
those dimensions. Ordinary arenas have only two diagnostic cases each. Training
maps with separate seeds measure retention on those layouts, not new-map transfer.

### Ordinary arenas

| Map | Prior 11 | Start 12 | Selected 13 | Final 24 |
|---|---:|---:|---:|---:|
| Crown Gap | 2/2 | 1/2 | 2/2 | 1/2 |
| Jawbone Crossing | 2/2 | 1/2 | 2/2 | 1/2 |
| Sump Teeth | 0/2 | 0/2 | 1/2 | 0/2 |
| Last Lantern | 2/2 | 2/2 | 1/2 | 2/2 |
| Twin Fords | 2/2 | 0/2 | 1/2 | 1/2 |
| Splitpine Clearing | 1/2 | 2/2 | 2/2 | 2/2 |
| Switchback Saltworks | 1/2 | 0/2 | 0/2 | 0/2 |
| Cinderfault | 2/2 | 1/2 | 1/2 | 1/2 |
| Frosthook Gate | 0/2 | 0/2 | 0/2 | 0/2 |
| Birch Pocket | 2/2 | 2/2 | 2/2 | 1/2 |
| Quarry Ring | 2/2 | 2/2 | 2/2 | 2/2 |
| Fordstep | 2/2 | 2/2 | 1/2 | 1/2 |
| Sable Sound | 1/2 | 2/2 | 1/2 | 1/2 |
| Three Wells Trail | 0/2 | 1/2 | 0/2 | 0/2 |

### Targeted maps

| Map | Prior 11 | Start 12 | Selected 13 | Final 24 |
|---|---:|---:|---:|---:|
| Marlow Cut Pass Finish | 4/4 | 4/4 | 4/4 | 4/4 |
| Ashbank Relay | 3/10 | 2/10 | 3/10 | 3/10 |
| Bellmarket Courts | 8/8 | 8/8 | 8/8 | 8/8 |
| Jawbone Breakthrough | 2/2 | 2/2 | 2/2 | 2/2 |
| Thorn Ford Breakthrough | 2/2 | 2/2 | 2/2 | 2/2 |
| Briar Saddle | 2/2 | 2/2 | 2/2 | 2/2 |
| Splitpine Dispatch | 0/6 | 0/6 | 0/6 | 0/6 |
| Crown Road Relief | 2/2 | 2/2 | 2/2 | 2/2 |
| Bracken Watch | 10/10 | 10/10 | 10/10 | 10/10 |
| Tidegate Ferries | 8/8 | 8/8 | 8/8 | 8/8 |
| Lower Gallery Refuge | 0/2 | 0/2 | 0/2 | 0/2 |
| The Lantern Maze Handoff | 6/6 | 6/6 | 6/6 | 6/6 |
| Mistfen Patrol | 6/6 | 5/6 | 6/6 | 6/6 |
| Splitwake Patrol | 6/6 | 6/6 | 6/6 | 6/6 |
| Hollowbank Extraction (late guard) | 0/2 | 2/2 | 1/2 | 2/2 |
| Siltbar Approach | 2/2 | 2/2 | 2/2 | 2/2 |
| South Ford West Post | 2/2 | 2/2 | 2/2 | 2/2 |
| Lair Outworks | 9/18 | 9/18 | 9/18 | 9/18 |
| Hollowbank Extraction (near side) | 1/2 | 2/2 | 2/2 | 1/2 |
| Herring Tide Road Three | 2/2 | 2/2 | 2/2 | 2/2 |
| Echo Gate | 0/2 | 0/2 | 0/2 | 0/2 |
| Slate Gallery | 9/12 | 8/12 | 9/12 | 11/12 |
| North of Lantern Road | 2/2 | 2/2 | 2/2 | 2/2 |
| Flint Switchback | 1/2 | 0/2 | 0/2 | 2/2 |
| Thorn Verge | 18/18 | 18/18 | 18/18 | 18/18 |
| Force the Crossing | 4/4 | 4/4 | 4/4 | 4/4 |
| Rally at the Vault | 1/2 | 1/2 | 1/2 | 1/2 |
| Whitecap Ford | 6/6 | 6/6 | 6/6 | 6/6 |

## Frozen controls and operational verification

Initial final-hour 12 actor:
`da26cde88dde2918a2f39eab5b8144ea4cfb98c8bdda7f6f61e2f7983c976bf2`.
The resumed actor, critic, reference, both optimizers, RNG, sampler/mastery/credits
and counters matched the source checkpoint exactly. The reference stayed equal
to the original hour10 actor throughout. No new critic warmup or imitation.

No architecture, training equations, native engine, catalog, reward or sampler
changes. Only the absolute trainer-time CLI ceiling and supervisor target were
extended, and twelve evaluation slots added. Frozen source 6053772 plus the
archived timing patch identifies the code. Temperature 0.35, four environments,
horizon 64, reference KL coefficient 0.02, arena win +1, intermediate chapter +1,
finale +2, existing failure/cap penalties, no kill/objective-progress bonuses.

Whole-extension actual decision shares: **25.0176% ordinary /25.0151% targeted
/49.9673% broad campaigns**. All 45 logical maps, including all 14 ordinary maps,
received additional decisions. Every full hourly exposure check had positive
work on all maps. The final roughly 29-minute partial check has no additional
Lantern Maze decisions; it had work earlier in the extension. This partial
interval is recorded rather than presented as another full-hour starvation test.

No trainer/evaluator restarts or observed training outages. A launch-only
supervisor reattachment test preserved both child processes. The sole snapshot
error is the historical Nov24 FileNotFoundError; no new extension snapshot errors
were logged. Current local/shared pointers and every archived hourly bundle
passed hash validation. Final saved PPO state retains all recovery components.

## Coverage and evidence

Each hour: 312 heldout selection cases, 214 training diagnostics, 24 Lantern.
Exact unchanged seeds, limits and recruitment controls are in the parent
`coverage.json` and raw row configs. Broad fog is player-only; targeted clear,
player-fog,both-fog and sight conditions retain their explicit authored settings.
All 25 panels were checked against the same configuration multiset and the exact
actor hash for that checkpoint. No extra confirmation sweep was requested or run.
One training seed, small campaign samples and selection across 25 checkpoints
limit generalization and make the selected score optimistic.

- `evaluation/hourly-summary.json`, `selection.json`, `comparison-and-chapters.json`
  and `per-map-and-chapter-curves.json`: full 0–24 comparison series.
- `evaluation/hour-13` through `hour-24`: only extension raw native outcomes,
  compressed JSON and JSONL. Original 0–12 reports remain in the parent archive.
- `plots`: full 0–24 fixed evaluations, per-map and conditional chapter curves,
  optimization losses/entropy/reference KL, live training returns and actual
  allocation. Adaptive live returns mix changing difficulty and are not fixed tests.
- `recovery/hour-12` through `hour-24`: starting actor/state plus every new hourly
  actor/full PPO state. `hourly-checkpoints.json` records actual times and hashes.
  Prior selected 11 remains in the immutable parent `recovery/hour-11`.
- `rl/metrics.jsonl.gz`, trainer/evaluator/supervisor logs and
  `allocation-history.jsonl.gz`: extension-only suffixes, verified against parent
  prefixes. Final sampler, monitor records and integrity checks retain exposure.
- Parent `recovery/frozen-runtime-inputs.tar.gz` plus this extension's
  `launch`/`scripts` timing patches and runtime-versions.json restore the exact
  runtime. Do not rebuild against later global authored data.

The parent experiment's manifest was rechecked and all original artifacts remain
unchanged. No next run, deployment or promotion is authorized by this archive.
