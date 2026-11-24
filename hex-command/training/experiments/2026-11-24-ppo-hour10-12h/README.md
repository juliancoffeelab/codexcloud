# Twelve-hour PPO continuation from hour 10

Status: running. The requested budget is 43,200 saved trainer seconds from the
unmodified final hour-10 actor; no imitation and no architecture change.
Initial native model SHA-256:
`e8dcc209f5cc420a616cfe6648551d19a8b476b235b1f59d67a108042eb2ebee`.

The current curriculum differs from the preceding experiment, so this uses the
existing PPO **policy warm-start** path: fresh actor/critic optimizers, critic,
reference anchored to hour 10, sampler and RNG at launch. Recovery thereafter
restores the complete saved state. Actor updates began after the existing
24-episode critic warmup. Temperature 0.35, actor LR 5e-6, critic LR 1e-3,
four native environments and horizon 64 are unchanged controls.

Current decision allocation is 25% ordinary arenas /25% targeted /50% broad
campaigns, with schema 3 adaptive sampling only within each group. There are 14
ordinary maps, 28 targeted logical maps and 3 broad campaigns. Rewards remain
arena+1, campaign intermediate chapter+1/finale+2, existing failure/cap penalties;
no kill or objective-progress bonuses. Teacher eligibility flags do not remove
maps from PPO.

Fixed panels at new-run hours 0–12 contain 312 held-out selection cases,
214 training diagnostics and 24 authored Lantern showcase cases. Only the 312
held-out cases choose a checkpoint using the existing reliability criterion;
baseline remains eligible. No automatic promotion or extra confirmation sweep.
Actual checkpoint times and per-map observed exposure are retained.

The launch check verified the starting actor exactly, finite live updates,
schema 3 sampler state, 14/14 ordinary maps receiving actual work, validated local
and shared checkpoint pointers, and successful supervisor reattachment without
restarting/duplicating the trainer or evaluator. Hourly monitoring is enabled;
healthy checks are silent. The monitor reports final results after training and
all 13 panels finish, then disables itself.

Local run:
`tmp/rl/ppo-hour10-continuation-12h-20261124`.
Shared recovery:
`/workspace/shared/ppo-hour10-continuation-12h-20261124`.
Frozen code/catalog/native/workload and model state live in those directories.
`launch` preserves the protocol, source identities, controls and verified launch
snapshot. This file will be updated with completed results; startup evidence is
not evidence of policy improvement.
