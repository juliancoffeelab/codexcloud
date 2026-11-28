# One-hour continuation from final hour9

Completed 2026-11-23. Same environments, corrected sampler, rewards, fixed
temperature, architecture and frozen-reference policy as the nine-hour experiment.
No new model was promoted and no further run started.

| Fixed evaluation | Hour8 | Hour9 | Hour10 |
|---|---:|---:|---:|
| Held-out balanced score |23.20%|20.45%|23.30%|
| Ordinary training arenas |33/84|29/84|39/84|
| Targeted equal-map score |69.31%|78.06%|67.87%|
| Broad training campaigns |6/18|4/18|4/18|
| Lantern clears |6/24|1/24|4/24|
| Lantern caps |4/24|13/24|7/24|

Lantern chapter clears / entrants: hour8 13/24→6/13→6/6; hour9
3/24→2/3→1/2; hour10 10/24→9/10→4/9. Thus the opener and second chapter
recovered versus hour9, while the finale now loses five of nine entrants.
Chapter comparisons are conditional on reaching each chapter with the resulting
army, not independent chapter tests.

The extension delivered 3600.20 saved training seconds, 273 optimizer updates,
1217 completed episodes and69888 decisions. All14 ordinary maps received
893–1306 decisions during the extension. Exact per-map counts are in
`monitor/extension-final-training-verification.json`. State retained policy,
critic, optimizers, frozen reference, mastery/allocation credits, RNG and next seed.
The automatic held-out-only selector still chose hour5; this does not choose the
next experiment, which remains a user decision.

This is evidence against a monotonic collapse: the extra hour improved ordinary
arena results and recovered some Lantern performance, while targeted results
fell and broad campaign completion remained4/18. It does not establish a stable
upward trend or isolate a cause. Hour8 still has more campaign clears on these
small panels. No architecture, reward or environment change was tested.

## Evidence

- `plots/gameplay-through-hour10.png`: fixed gameplay curves.
- `plots/targeted-through-hour10.png`: eight targeted families.
- `plots/chapters-through-hour10.png`: chapter-conditional campaign curves.
- `plots/extension-training.png`: losses, entropy and KL during the extra hour.
- `comparison.json`, `chapter-results.json`: exact aggregated results.
- Three compressed raw hour10 reports:504 new evaluation cases, identical to
  earlier checkpoints. Small per-map samples; no independent confirmation or
  matched V2/rulebot comparison was added.
- `recovery/hour-10/`: final planner/resume gzip files and hash manifest.
- `metrics.jsonl.gz`, `sampler-status.json.gz`: extension training evidence.
- `extension-checkpoint-index.json`: saved intervening checkpoints; full bundles
  remain locally and at the verified shared backup.
- `finished.json`, protocol, current supervisor/evaluator and archive manifest.

The same live root is `tmp/rl/ppo-from-pretrain-5m-9h-20261123`, shared backup
`/workspace/shared/ppo-from-pretrain-5m-9h-20261123`. The immutable parent archive
retains the original nine-hour results. Allocation was faulty until7h14m;
only the final1h46m of that run plus this hour used the corrected sampler.
The extension monitor is disabled.
