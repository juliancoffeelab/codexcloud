# Force the Crossing

A compact10×10 objective-priority task on Birch Pocket terrain. Seven swordsmen
and a commander face three active impact casters. Reach (8,4) by the end of
turn3; loss executes at turn4. Clear visibility and no recruitment on either
side. The objective is explicitly revealed and available from the start.

This is an urgency task, not prolonged stalling: with a loose deadline all
compared bots eventually finish. The3-turn route is verified in every oracle
run under identical forces, rules, enemy policy and information.

The offline targeted bot holds the commander back, promotes eligible troops,
favors positive expected combat exchanges, and advances along engine-provided
goal paths. It ranks the same legal candidates and public observations supplied
to the learner. Its rules are not added to the learned planner or live bots.

Against Classic enemies, seeds97301–97320: hour9 6/20, oracle20/20; Full8/20.
Independent97401–97420: hour9 3/20, oracle20/20. Combined9/40 versus40/40.
All oracle completions occur on turn3; no enemy orders are frozen or overridden.

Exact schedules, traces, model hash and oracle implementation/specification are
retained in `tmp/rl/hour9-hold-explore/`. Identity-only registration preserves
all tested mechanics. This task qualifies against the final nine-hour policy.
