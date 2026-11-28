# Nine-hour PPO from five-minute recurrent imitation

Completed 2026-11-23: 32400.713 trainer seconds, 2464 updates, 9494 completed
episodes, 630712 decisions. Fixed gameplay evaluation completed for all ten
checkpoints (hour 0–9), 5040 games total. No model was deployed or promoted.

| Fixed evaluation | Initialization | Hour 7 | Hour 8 | Hour 9 |
|---|---:|---:|---:|---:|
| Held-out balanced score |24.53%|21.02%|23.20%|20.45%|
| Ordinary training arenas |28/84|36/84|33/84|29/84|
| Targeted equal-map score |55.58%|78.75%|69.31%|78.06%|
| Broad training campaign clears |3/18|1/18|6/18|4/18|
| Lantern Road clears |4/24|0/24|6/24|1/24|
| Lantern turn caps |10/24|10/24|4/24|13/24|

The prescribed held-out reliability selector selected hour 5 (25.38% balanced
score); it allows worst-stratum scores within 2.5 percentage points of the best,
then prefers overall score and fewer caps. Training diagnostics and Lantern were
excluded. Hour 8 is retained as a separate interesting campaign candidate, not
the automatic winner. No independent final confirmation was run.

## Allocation failure and correction

The initial sampler starved ordinary arenas: 860 decisions in six episodes,
ending at 12m35s; nine of fourteen maps never ran. Ordinary base mass was 18.42%
of total decisions, but measured exposure was about 0.175% near hour 7. The
combined arena pool still received 50%, hiding the failure.

The user authorized pausing and continuing with a correction at 26041.891 trainer
seconds (7h14m). Hierarchical map/condition allocation and persistent credits
replaced flattened template allocation with repeated debt resets. Policy, critic,
optimizers, reference, RNG and mastery statistics were retained. Existing lifetime
exposure remains recorded; new credits allocate prospectively. Corrected training
therefore lasted only about 1h46m. Per-map post-fix counts are in final-summary.json.
All fourteen ordinary maps received training after correction. Before/after
checkpoints are not evidence for an uninterrupted nine hours on the intended mix.
The rebound at hour 8 did not persist at hour 9; this is not an isolated causal
comparison of the sampler fix.

## Evidence and recovery

- `plots/final-gameplay.png`: fixed gameplay comparison with allocation boundary.
- `plots/final-training.png`: live losses, entropy, KL and rolling episode returns.
- `plots/final-campaign-chapters.png`: conditional chapter clear rates; missing
  points mean no entrants, not zero success.
- `evaluation/`: gzip raw game rows and reports for each hourly panel.
- `recovery/hour-00` through `hour-09`: gzip native planner/resume plus original
  hash manifests. Decompress names to restore bundles.
- `catalog.tar.gz`, `python.tar.gz`, `workload.json.gz`: frozen authored data,
  corrected training source and exact cases.
- `rl/metrics.jsonl.gz`, `sampler-status.json.gz`: complete training evidence.
- `allocation-fix/`: reproduction, migration and live verification evidence.
- `archive-manifest.json`: artifact hashes.

The full live run and native module remain at
`tmp/rl/ppo-from-pretrain-5m-9h-20261123`, with verified shared backup at
`/workspace/shared/ppo-from-pretrain-5m-9h-20261123`. Native hash, source revisions
and actual checkpoint times are recorded. The hourly monitor is disabled.

Evaluation uses 312 held-out cases, 168 training diagnostics and 24 Lantern cases
per checkpoint. Training-map diagnostics have small per-map samples and one
recruiter representative per condition. Held-out comparison is for this fixed
opponent/seed panel, not a matched V2/rulebot comparison. Selecting among repeated
fixed-panel evaluations introduces selection bias.
