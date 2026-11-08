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

Training accepts a finite positive time budget. The budget covers data collection and
optimization; final inference checks and evaluation run afterward. Checkpoints,
exported weights, configurations, metrics and source fingerprints stay under
ignored `tmp/rl/deathmatch-classic/`. The best validation cross-entropy checkpoint is
selected. A seed split separates complete training, validation and evaluation
matches; positions from a match never span splits. Default workload: deathmatch (opposing leader elimination), sixteen
training seeds, one validation seed, all five arena maps,
four directed faction pairings and both first seats for training/validation.
Final evaluation defaults to three held-out seeds, all twelve directed non-mirror
faction pairings, five maps and both first seats per recruiter (1,080 games total).

## Components

- `rust/src/training.rs`: teacher sessions and validated small MLP inference.
- `rust/src/training/features.rs`: versioned 172-feature schema, mechanical unit
  profiles, terrain/path encoding and legal candidates. Weights require matching
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

Feature names are exported as `hex_command_training.FEATURE_NAMES`. Schema 2
has 172 inputs and rejects older checkpoints instead of silently misinterpreting
weights. There are no unit, faction or terrain identity embeddings. Each candidate
includes mechanical profiles for the actor, attack target and nearest enemy:
max HP, cost, movement, level/experience, healing including equipment, thorns,
armor modifiers including equipment, current action flags and strongest raw-damage
weapon in each category. Selected attack weapon properties are encoded separately.
Alternative weapons within a category are summarized rather than fully enumerated
in the unit profile; attack candidates still enumerate every legal weapon.

Destination and target terrain encode all six signed category/damage modifiers,
entry cost, healing, keep status and ownership. Modifiers use the game's additive
mechanics, not assumed percentage resistances. Path features use cached reverse
route fields for matching active goals and enemy locations. Cost divided by base
movement is an optimistic travel-turn estimate, not a turn simulation: it ignores
occupancy, future blockers, per-turn wasted movement, and future combat. Separate
reachability flags distinguish unreachable locations from zero-cost routes.
Profiles and route references are prepared once per decision; the engine shares
route fields by movement profile, goal set and terrain revision.

Nearest-enemy observations provide raw properties, not a hand-written danger
score. They are not a full roster or spatial encoder and can miss other dangerous
units. Supplying live stats makes balance changes observable to the policy but
does not guarantee successful generalization; substantial rebalances may require
fresh imitation data or later RL training. Models remain compatible only with the
same explicit feature schema.

## Step interface

`BatchEnv.reset(config_json_strings)` and `step(candidate_indices)` return:

```text
features: float32 [total_candidates, 172]
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
before replanning. Collection defaults to player-side labels; `--collect-side both` restores both-side recording.

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
.venv/bin/python -m hex_command_rl.evaluate tmp/rl/deathmatch-classic/planner.json
.venv/bin/python -m hex_command_rl.evaluate tmp/rl/deathmatch-classic/planner.json \
  --configs tmp/rl/campaign-configs.json --out tmp/rl/campaign-evaluation.json
```

## Deathmatch rules

The default Python training and evaluation workloads explicitly select
`"objective": "deathmatch"`. Only leader elimination wins; surviving to the round
limit is truncation, never a territory or score victory. The campaigner receives
leader-elimination and economic village-capture targets, with territory-holding
and optional ruin-visit targets removed. Village income, recruitment and normal
combat mechanics remain active. Existing native arena configurations that omit
`objective` retain their original territory rules. The current experiment does
not train campaign or escort behavior.

## Logging and evaluation caps

Training/validation matches retain 30 rounds. Final evaluation defaults to 100
rounds (`--eval-rounds` in training, `--rounds` in standalone evaluation). Custom
arena evaluation configs are also overridden with this explicit limit. Final
seeds never overlap train, data-validation, or periodic-validation seeds.

`metrics.jsonl` flushes one compact record per epoch, plus periodic evaluations.
It records full validation metrics, a fixed training probe of at most 1,024
positions, online optimization loss, action-specific accuracy, predicted action
frequencies, optimization throughput, elapsed times, and peak process RSS. Dataset
diagnostics sample at most 1,024 decisions once to measure identical-candidate
features and teacher ambiguity. Ambiguity is not an accuracy ceiling: deterministic
tie order may correlate with the teacher's choices. Timing separates optimizer
work from diagnostics; train-probe and validation loss use the same checkpoint.

Every five epochs, thirty short validation matches (Humans across five maps, both
first seats and all three recruiters, separate seed) record wins/losses/caps and mean rounds. Their fixed
baseline is cached once. Use `--eval-every 0` to disable. They are diagnostics,
not final test results or the checkpoint selection metric. The best full-data
validation cross-entropy still selects the saved checkpoint.

Plot logs without retraining:

```sh
.venv/bin/pip install -e 'training[plots]'
.venv/bin/python -m hex_command_rl.plot tmp/rl/deathmatch-classic/report.json
```

## Recruiter diversity and disk-backed datasets

Recruitment is an independent workload axis. Defaults are Balanced, Situational
and Predictor (`--recruiters balanced situational predictor`). Within each stratum,
our side uses the selected recruiter. The enemy retains the full Classic preset,
including its Classic recruiter and default two-recruit phase limit. The teacher planner remains Queued and the
campaigner remains Mission. The learned planner replaces only the player planner
in evaluation. This diversifies army composition without treating different
recruiters as different tactical planning algorithms. Results include a separate
wins/losses/caps summary for each recruiter pair.

The default training panel is 16 seeds × 3 recruiters × 5 maps × 4 directed faction
pairings × 2 first seats = 1,920 matches. Validation uses a separate seed with all
three recruiters (120 matches). Final evaluation uses all twelve directed
non-mirror faction pairs, three other seeds and a 100-round cap: 360 matches per
recruiter, 1,080 total. These are fixed-opponent comparisons, not direct matches between recruiter
strategies.

`data.py` collects eight-game native shards into `data/train/` and
`data/validation/`. Candidate features are written as a float32 binary buffer and
mapped copy-on-write for training; offsets and labels use small NumPy arrays.
Manifests retain schemas, fingerprints, full configurations and shard boundaries.
All configurations are collected by default. Optional `--max-samples` is now a
hard limit that raises an error rather than silently discarding later examples.
The per-shard native guard also raises if reached. Tests compare sharded and
single-call observations, offsets and labels exactly.

For an approximately twenty-minute collection/training run, followed by final
evaluation:

```sh
.venv/bin/python -m hex_command_rl.train --seconds 1200 --epochs 1000000 \
  --out tmp/rl/deathmatch-classic-20m
```

Training is still imitation over a fixed dataset, not RL. More data increases
cost per epoch; the time budget limits repeated passes. Large mapped datasets
also consume disk space and file-backed resident memory as batches touch pages.

## Fixed Classic opponent and collection side

The default opponent is now the complete Classic preset. `--opponent` selects
another preset; its recruitment/planning remain independent of our recruiter
axis. `--mirror-recruiters` explicitly overrides the enemy recruiter when a
matched-recruiter experiment is wanted. To reproduce the earlier self-play
workload, use `--opponent balanced --mirror-recruiters --collect-side both`.

`--collect-side player` is the default: only our side's teacher decisions become
labels, even when Classic moves first. The native collector also accepts enemy
or both-side recording, with tests proving that player/enemy decision counts
partition both-side recording and that sharded player data matches native data.
Manifests record the collection side. Opponent actions still run normally and
influence positions, but do not supply supervision.

The current teacher remains Queued, with Balanced/Situational/Predictor recruiters.
This change does not introduce Full teachers, stat randomization, spatial encoders
or reinforcement updates. The time budget continues to include collection and
training; final evaluation follows it.

A reproducible bot comparison against the same full Classic enemy is available:

```sh
.venv/bin/python -m hex_command_rl.compare \
  --model tmp/rl/deathmatch-classic-20m/planner.json \
  --out tmp/rl/deathmatch-classic-20m/bot-comparison.json
```

It runs Classic, Rush, Rotated, Balanced, Situational and Predictor presets,
Full with each of the three recruiters, and the learned planner with those same
recruiters. Every panel uses the same 360 non-mirror faction/map/initiative/seed
configurations and a 100-round cap. Preset recruitment limits remain intact;
these are whole-bot comparisons, not equalized recruitment-budget experiments.
JSONL retains partial results by panel. `--baselines` reuses a completed panel
only after exact workload and engine/content fingerprint checks.
