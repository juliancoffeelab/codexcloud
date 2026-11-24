# Retain hour 10 and remove experimental tactical teachers

The user selected the unmodified final hour-10 PPO model for subsequent work.
`selected-baseline.json` records the immutable actor and complete PPO recovery
state. No imitation checkpoint replaces it; no new run or deployment starts.

Removed the production relief/recovery/refuge controllers, their typed catalog
schema/exports, native dispatch and Python selector support. Crown Road Relief,
Briar Saddle and Lower Gallery Refuge now use `pretrain_imitator: "full"` and
`pretrain_enabled: false`. Their PPO maps remain in place. Other teacher and
eligibility settings are unchanged: 15 conditions across 8 targeted logical maps
remain enabled for any future explicitly requested imitation collection.

Historical experiments, frozen runtimes, results and model checkpoints remain
archived. Those snapshots are evidence, not maintained current controllers.

Focused probes establish identical full PPO cases and sampling weights before
and after removal on train/validation/test partitions, disabled Full selectors
for all three affected maps, rejection of old selectors by both Python and the
new native module, and an unchanged hour-10 actor hash. Eligible imitation cases
fall from 600 to 552 for train and 100 to 92 for validation because Lower Gallery
is now disabled. Full verification and the 41-test training-suite logs are
preserved compressed after completion. No extra balance simulations were run:
this changes only removed offline policies and disabled teacher metadata.
