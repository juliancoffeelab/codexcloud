# Multi-objective curriculum wiring

Configured, not trained. Source implementation: `9fdaf429f8ba21a64a1c049e8823971a695d1ea3`.

## Decision allocation

Ordinary arenas25%, targeted encounters25%, broad campaigns50%. All three retain fixed budgets with adaptive weighting only inside each group. There are14 ordinary maps,28 targeted logical maps and3 broad campaigns. Ashbank visibility variants share a logical map; Marlow challenge and control share a logical map.

Map base weights: each of the three escort maps1/3; Bellmarket Courts and Thorn Verge1/4; all others1. New Marlow control conditionWeight0.25 versus1 for Finish. These multiply mastery weights and normalize within each pool. Existing80%-threshold,10%-floor, EMA and warmup controls remain.

Actual prospective allocation over3000 variable-length synthetic episodes, with a midstream mastery reweight and retained credits:
- arena: 25.000810% (15441 decisions).
- targeted: 24.991095% (15435 decisions).
- campaign: 50.008096% (30886 decisions).

All45 logical maps received decisions. This is a sampler probe, not PPO training.

## New authored conditions

| Physical encounter | Enemy | Teacher recruiter / Campaigner / planner | Base share of all decisions |
|---|---|---|---:|
|rl-marlow-cut-pass-finish|classic|balanced / mission / queued|0.816327%|
|rl-marlow-cut-pass-interrupt|classic|balanced / mission / queued|0.204082%|
|rl-ashbank-relay-fork-vigil|predictor|classic / mission / classic|0.204082%|
|rl-ashbank-relay-fork-vigil|Full|balanced / mission / queued|0.204082%|
|rl-ashbank-relay-fork-vigil-both-fog|predictor|balanced / mission / queued|0.204082%|
|rl-ashbank-relay-fork-vigil-both-fog|Full|predictor / mission / full|0.204082%|
|rl-ashbank-relay-fork-vigil-player-fog|Full|balanced / mission / queued|0.204082%|
|rl-hollowbank-extraction-crossfire-late-guard|classic|classic / mission / classic|1.020408%|
|rl-hollowbank-extraction-crossfire-nearside|classic|classic / mission / classic|1.020408%|

The eight challenge combinations preserve the validated fixed rosters, zero recruits, initiative, clear/player-only-fog/both-fog scope, objective rules and deadlines. The ninth condition is the previously validated distant-commander rehearsal control. Native collection with explicit teacher blocks matched direct configured-reference collection exactly for all three compositions;178 decisions total, zero missing labels. All38 imported authored JSON documents match the evaluated fixtures semantically.

## Rewards and evaluation

Authored reward profiles make victory+1 for ordinary arenas and all single-stage targeted encounters, even when stored as campaigns. Genuine multi-chapter campaign intermediate clears pay+1 and finale+2. Existing defeat, escort/protected/commander-loss and cap penalties remain. Kill and objective-progress bonuses remain off. Profiles, base map weights and teachers are covered by workload provenance. Episode logs identify their sampling group, logical map and reward profile explicitly.

Fixed evaluation does not adapt or inherit training map reductions/pool proportions. Held-out ordinary-arena and campaign families retain equal aggregate mass; changing curriculum allocation does not change the checkpoint-selection goalposts.

## Checks

Training Python suite:41 tests, one optional dependency skip. Native gameplay check:18 completed matches, two fresh validation seeds for each configured start; no caps. These are integration checks, not a new strength estimate. Existing20-seed qualification evidence remains in `../2026-11-23-fog-and-multi-objective-validation/`. `make full-verify` passed:272 frontend tests and all Rust/workspace/script checks. Full output and the41-test training-suite output are archived compressed alongside `checks.json`. No pretraining, PPO, checkpoint promotion, new monitor or deployment launched. Historical frozen run catalogs remain immutable.
