# Additional twelve-hour full-state PPO continuation

Running, authorized 2026-11-26. Resume the **final hour-12** checkpoint,
not the earlier selected hour 11. Initial actor SHA-256:
`da26cde88dde2918a2f39eab5b8144ea4cfb98c8bdda7f6f61e2f7983c976bf2`.

Start at 43,200.194862209 saved trainer seconds; total target is exactly
86,400.194862209, adding 43,200 seconds. The resumed initial bundle was compared
recursively with the source: actor, critic, reference, both optimizer states,
Torch/NumPy/Python RNG, sampler including credits/mastery, update, episode,
decision and action counters all match exactly. The KL reference stays anchored
to the original hour-10 actor. This is not another policy warm start.

The frozen curriculum, architecture, temperature, sampling and rewards remain
unchanged: 14 ordinary maps 25%, 28 targeted 25%, three broad campaigns 50%,
adaptive inside groups, temperature 0.35, four environments/horizon 64,
arena +1/intermediate chapter +1/finale +2. No imitation, kill or objective-progress
bonuses. Only the frozen CLI's absolute-time budget ceiling and supervisor
termination target were extended. Training equations were not changed.

Original hour 0–12 results remain cached and archived in the immutable parent
experiment. New hour 13–24 evaluations reuse the same fixed 312 heldout,
214 diagnostics and 24 Lantern cases. New checkpoints are evaluated at the first
saved time >= extension origin + additional hour*3600. Only held-out 312 selects,
including all previous candidates (hour 11 remains eligible). Diagnostics and
Lantern never select. No extra confirmation sweep or automatic deployment.

The running local/shared directories retain full state and updated frozen timing
scripts. Recovery uses the same venv and supervisor locks; hourly automation
checks actual per-map exposure and validated local/shared pointers. Original
completion metadata is retained in `phases/twelve-hour-completed`; the current
completion sentinel is valid only at the new target with hour24 evaluated.

Final extension artifacts belong here; do not overwrite the parent archive.
