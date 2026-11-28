# Explicit imitation eligibility

Added the required boolean `pretrain_enabled` independently of each teacher
selector. Sixteen enemy/visibility conditions across nine targeted logical maps
are enabled. All remaining conditions are disabled for automatic imitation.

Both workload preparers retain the full PPO `train` schedule and generate
separate filtered `imitation_train` and `imitation_validation` partitions. The
trainer consumes those partitions; collection rejects disabled/empty requests
before writing. Teacher identity and filtered cases remain in hashed workload
provenance. Imitation weights normalize over eligible conditions; PPO allocation
and adaptive sampling are unchanged. Historical frozen runs are untouched.

The enabled matrix is preserved in `probe.json`. Splitpine enables only Classic
(learner0/8, teacher7/8); its weaker Predictor/Full references stay disabled.
Lair enables player-fog Classic/Predictor and both-fog Classic/Full (learners0–1/8,
teachers6/8), with clear and weaker-reference conditions disabled. Other selected
maps/conditions follow the prior approved table: Echo, North of Lantern Road,
Lower Gallery, Herring Tide Road Three, the two clear Classic Hollowbank starts
and the five configured Ashbank conditions.

Checks:
- Before/after exact case-list and sampling-weight equality for all PPO splits:
  36,648 training,4,756 validation and16,543 test matches.
- Mixed and diverse preparation both produce600 eligible imitation training
  cases and100 imitation validation cases; neither starts collection/training.
- Empty/disabled automatic collection rejected before output creation.
- Native catalog rejects a missing required flag; Python rejects non-booleans.
- One enabled Echo teacher game collected19 decisions with recurrent boundaries,
  zero missing labels, and no optimizer.
- Existing41-test training suite and full repository verification results are
  preserved compressed after completion. No pretraining or PPO run launched.

These checks validate filtering and workload preservation, not policy strength.
Qualification evidence lives in the preceding teacher/encounter experiments.
