# Native planner training

This experiment replaces only tactical planning. Existing recruiter and
campaigner blocks remain selected by the match configuration. It implements
behavioral cloning; reinforcement updates and WASM integration are future work.

## Setup

From the game root, create a local virtual environment and install a CPU PyTorch
build using the official PyTorch package index:

```sh
python3 -m venv .venv
.venv/bin/pip install 'maturin>=1.9,<2'
.venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
.venv/bin/pip install -e training
node --import tsx .agents/skills/rust-toolchain/scripts/with-toolchain.ts \
  .venv/bin/maturin develop --manifest-path rust/training-python/Cargo.toml --profile simulation
.venv/bin/python -m unittest discover -s training -p 'test_*.py'
.venv/bin/python -m hex_command_rl.train --seconds 180
```

Training refuses budgets over 600 seconds. The budget covers data collection and
optimization; final inference checks and evaluation run afterward. Checkpoints,
exported weights, configurations, metrics and source fingerprints stay under
ignored `tmp/rl/imitation/`. The best validation cross-entropy checkpoint is
selected. A seed split separates complete training, validation and evaluation
matches; positions from a match never span splits. Default workload: four
training seeds, one validation seed, one evaluation seed, all five arena maps,
four directed faction pairings and both first seats. This is not a full balance
panel. The teacher defaults to Balanced against Balanced.

## Components

- `rust/src/training.rs`: versioned 32-feature candidate schema, legal candidate
  enumeration, teacher recording and small MLP inference. Weights require matching
  schema/names, valid dimensions and finite values.
- `rust/src/ai/planner/learned/`: native-only learned planner, selected through
  existing block composition when the `training` feature is enabled.
- `rust/training-python/`: PyO3/NumPy bridge. Python owns returned array buffers;
  subsequent calls never overwrite observations. Numeric observations and actions
  do not pass through JSON. Configuration and compact evaluation summaries do.
- `training/hex_command_rl/`: 64/64 ReLU scorer, masked candidate cross-entropy,
  held-out evaluation and policy export. No value network or PPO yet.

Teacher commands are recorded immediately before tactical forecast execution,
including the final stop decision. Recruitment and bookkeeping intents are
excluded; promotion timing is learned from the recruiter's chosen branch.
Missing teacher candidates are counted, and the training command rejects a
nonzero count. Candidates include end-turn, all reachable legal moves, immediate
attacks with each weapon, and recruiter-approved promotions. No heuristic top-K
shortlist restricts the learner. Moves and attacks are separate decisions.

Feature names are exported as `hex_command_training.FEATURE_NAMES`. Features use
relative quantities and fixed scales, without unit IDs or authored script
predicates. Objective features summarize the nearest matching target and actor
protection. They are deliberately an initial representation, not a complete
encoding of escort duties, terrain, support abilities or strategic state.

## Step interface

`BatchEnv.reset(config_json_strings)` and `step(candidate_indices)` return:

```text
features: float32 [total_candidates, 32]
offsets: int64 [environment_count + 1]
rewards: one float per environment
terminated: one bool per environment
truncated: one bool per environment
```

Indices are local to each environment's candidate slice. Completed slots have
empty slices and remain completed until reset. Ignore their action indices and
mask them in future rollout collection. Terminal rewards are +1 for success, -1
for defeat, 0 for truncation; these are scaffolding for RL, not imitation targets.
Reset currently replaces the entire batch. Use `close()` to cancel workers.
Each environment owns a native worker thread and its borrowed catalog/game
lifetimes. PyO3 releases the GIL while workers run. There are no leaked catalogs
or Python callbacks inside the game. Keep batches small on this four-core box;
threads beyond the CPU quota do not add compute.

The same learned planner drives interactive stepping and full native simulation.
The step interface automatically runs fixed opponents and existing orchestration
until the next player decision. Combat returns to the normal harness boundary
before replanning. Collection currently records teacher decisions for both sides.

## Validation

The integration checks cover owned arrays, teacher candidate coverage, invalid
actions, cancellation/reset, unchanged teacher outcomes, malformed models and
PyTorch/Rust inference parity. Training reports uniform and always-end baselines,
overall and nontrivial accuracy, per-action counts, and paired closed-loop arena
results. Imitation accuracy does not establish playing strength. Campaign
configurations can be supplied directly to the native APIs; campaign competence
requires separate evaluation and better objective features.

Custom workloads, including campaigns, can be supplied with `--train-configs`,
`--validation-configs` and `--evaluation-configs`, each a JSON array in the native
harness configuration format. Seed sets must be disjoint. For example, a campaign
configuration is:

```json
{"seed": 1, "campaign": "lantern-road", "player": "balanced", "enemy": "classic",
 "diplomacy": "diplomatic", "objective": "authored", "maxRounds": 100,
 "recruitLimit": null}
```

Evaluate existing weights without retraining:

```sh
.venv/bin/python -m hex_command_rl.evaluate tmp/rl/imitation/planner.json
.venv/bin/python -m hex_command_rl.evaluate tmp/rl/imitation/planner.json \
  --configs tmp/rl/campaign-configs.json --out tmp/rl/campaign-evaluation.json
```
