# Nine-hour PPO from five-minute recurrent imitation

Started 2026-11-23. Live run: `tmp/rl/ppo-from-pretrain-5m-9h-20261123`.
Shared backup: `/workspace/shared/ppo-from-pretrain-5m-9h-20261123`.

This directory records the launch protocol and frozen supervisor/evaluator.
Results are pending; no model has been selected or promoted.

The 24 targeted encounters join the 14 ordinary maps in one arena sampling pool;
there is no reserved targeted allocation. Broad campaigns retain half the base
decision budget. Existing adaptive sampling operates within pools.
Chapter +1 / finale +2 remain; objective-progress and kill rewards are absent.
Only held-out evaluation selects checkpoints; training diagnostics and Lantern
are reported separately. Every hourly comparison uses the same fixed seeds.

The run's frozen Python/native/catalog, workload, checkpoint manifests and shared
backup remain authoritative recovery artifacts. Restart with the existing venv
Python, frozen PYTHONPATH and one-thread BLAS settings, as recorded in protocol.
