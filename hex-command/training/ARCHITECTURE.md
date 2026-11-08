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

- `rust/src/training.rs`: teacher sessions and validated dense/tree inference.
- `rust/src/training/features.rs`: versioned 227-feature schema (172 legacy + 47 campaign duty + 8 diplomacy inputs), mechanical unit
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

Feature names are exported as `hex_command_training.FEATURE_NAMES`. Dense/tree
schema 6 has 227 inputs. Legacy schema-2/4 models are explicitly migrated by name
and shape, appending zero weights; unsupported schemas are rejected. There are no unit, faction or terrain identity embeddings. Each candidate
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
features: float32 [total_candidates, 227]
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

## Native runtime timing

`benchmark.py` compares Queued, Full and Learned on identical frozen player-phase
entry positions captured from fresh native matches. It reuses the simulation
skill's capture protocol and never repeats begin/end-turn preparation. Different
recruiters are benchmarked separately while keeping each row's inputs identical.

```sh
.venv/bin/python -m hex_command_rl.benchmark \
  --model tmp/rl/deathmatch-classic-20m/planner.json \
  --source-report tmp/rl/deathmatch-classic-20m/report.json \
  --out tmp/rl/deathmatch-classic-20m/timing
```

Use `--fixtures <timing>/positions.json` to reuse the same captured positions.
The artifact preserves source fingerprints and full states; output includes the
fixture SHA256 and tested-engine fingerprints. Workers start in fresh processes
with `HEX_COMMAND_AI_PROFILE=1`, warm the selected planner, then measure repeated
panels using Linux thread CPU time. Timings include recruiter/campaigner/planner
work inside tactical `run_turn`, including learned feature extraction and forward
passes. They exclude fixture restore/validation, JSON/Python serialization, and
real command execution outside the planner's own forecasts. Returned action counts
identify differences in the work chosen from the same position.

The reported statistic is the median of repeated-panel mean CPU milliseconds per
own phase. It is not a worst-turn latency or a browser/WASM measurement. Both
native replay embeddings and the existing phase-stdio path use the same validated
phase executor. These JSON methods are offline fixture tools; numeric training
observations still use NumPy buffers.

## Conditional logistic-regression baseline

`--architecture linear` replaces the 172→64→64→1 ReLU scorer with one 172→1
linear score. Training still applies a softmax across legal candidates and the
same masked cross-entropy. This is conditional/multinomial logistic regression,
not independent binary classification of candidates. Its shared intercept would
cancel across candidate logits, so it uses 172 learned weights and exports a zero
intercept for the existing native model format. No runtime-engine change is needed.

```sh
.venv/bin/python -m hex_command_rl.train --architecture linear \
  --dataset-dir tmp/rl/deathmatch-classic-20m/data \
  --seconds 180 --epochs 1000000 --out tmp/rl/deathmatch-classic-linear
```

Dataset reuse checks requested configurations, collection side, feature schema
and current content fingerprint. Saved source-engine fingerprints are retained
alongside the current runtime fingerprint. Reusing frozen examples does not
assert that source and current engine revisions are identical. This lets model
families use exactly the same examples without recollection. The report records
architecture, parameter count, data provenance, learning rate and weight decay.
`--learning-rate` defaults to 0.001 and `--weight-decay` to zero.

A linear candidate scorer cannot make preferences depend on features that are
identical across every candidate in one position: their common contribution
cancels in softmax/argmax. Nonlinear interactions or explicit feature crosses are
needed to use that global context conditionally. Per-candidate mechanical
features, including combat forecasts, remain available to both model families.

## Small boosted-tree ranker

`trees.py` trains LightGBM LambdaMART on the same frozen candidate matrix. Each
decision is one ranking group, with relevance 1 for the teacher's candidate and
0 for its alternatives. The model sums tree leaf scores and chooses the highest
scoring legal candidate, retaining the first on ties. This remains imitation;
match rewards do not train the model.

```sh
.venv/bin/pip install -e 'training[trees]'
.venv/bin/python -m hex_command_rl.trees \
  --dataset-dir tmp/rl/deathmatch-classic-20m/data \
  --source-report tmp/rl/deathmatch-classic-20m/report.json \
  --seconds 1200 --trees 64 --depth 6 \
  --out tmp/rl/deathmatch-classic-trees
```

The source report supplies the held-out evaluation panel and teacher outcomes.
Dataset content fingerprints and validation configurations must match. LightGBM
uses indexed internal column names because human-readable feature names can
repeat; exported native names and feature order remain unchanged.

Training defaults to 64 trees, with a maximum of 1,024, depth 6, and at most 1,200
seconds, using three CPU threads. `--early-stopping-rounds` defaults to twelve
iterations without improved validation choice accuracy. For a larger ensemble,
use `--trees 512 --early-stopping-rounds 64` while retaining the same time budget.
A supervising process enforces the time budget, including data
loading and training. Evaluation follows the budget. Each improved iteration saves
a versioned model pair before atomically publishing its checkpoint pointer, so a
deadline can preserve the best completed checkpoint. Per-iteration JSONL records
elapsed time, validation choice accuracy and candidate-softmax cross-entropy.
Cross-entropy is a diagnostic; LambdaMART does not optimize calibrated logits.

Exported trees contain flat numeric split/leaf arrays. Native Rust validates
feature indices, finite values, unique forward children, reachability, size and
depth, and rejects models mixing dense layers with trees. Runtime inference has
no LightGBM or Python dependency. Float32 observations are compared to float64
thresholds and leaf values are summed in float64 to preserve LightGBM's ranking.
The diagnostic `native.score` API returns float64; dense models still compute
their original float32 forward pass. Export parity and grouped held-out choices
are checked before evaluation. Reuse `benchmark.py --fixtures ...` to compare
complete native planning phases on the same saved positions as other models.

## Shared hex CNN

`spatial.py` adds an optional board encoder to 227 candidate features. The first
53 of its 65 channels include a valid-tile mask, friendly/hostile/
neutral occupancy, the existing complete 34-value unit profile, base terrain
cost/passability/healing/keep/ownership and six damage modifiers, and weighted
capture/defeat/visit goal planes. Empty tiles retain terrain; absent units have
zero profiles. Terrain channels describe base mechanics; candidate features still
provide the acting unit's adjusted terrain costs and resistances. Goal planes
aggregate targets; candidate path features retain actor/family restrictions.
No unit, faction or terrain identity embeddings are added.

Coordinates use `q = col - floor(row/2), r = row`, preserving the engine's six
neighbors even for negative coordinates. Two seven-position convolutions encode
65→16→16 channels, each followed by ReLU and multiplication by the valid-tile mask.
Padding stays zero and is excluded from mean pooling. The mask describes map
membership, independently of terrain passability or candidate legality. Each
candidate gathers 16 values at its actor, destination and target, plus the shared
16-value masked board mean. Missing references use zeros. Concatenating those 64
values with existing features feeds a 291→64→64→1 head. A 10% dropout follows the
first hidden ReLU during training only. No normalization or convolution dropout is
used. The complete model has 32,017 parameters.

The board is encoded once per decision, shared across all candidates. Native Rust
executes exported seven-position convolution weights and dense layers directly;
no Python or PyTorch is needed in a game. Spatial model schema 7 includes explicit
channel names and validates both convolution shapes. Legacy schema-3/5 CNNs migrate
with zero input-channel filters and zero dense columns inserted before the 64
spatial gathers. Dense/tree schemas 2/4 also migrate to schema 6. Dropout is absent
from the inference export.

`native.collect_spatial` returns `(dense, spatial)`: the original four-array/data
collection tuple plus `(board_values, board_offsets, shapes, tile_indices)`.
Board values are float32 CHW buffers, offsets/shapes are int64, and candidate
indices are int64 `[candidate_count, 3]`, with -1 for missing references. There is
one board per teacher decision. `spatial_data.py` streams both feature/board
buffers to disk and pads only the current batch, rebasing tile indices to its
width. `BatchEnv(spatial=True).spatial_observation()` exposes the same owned arrays
for paused decision slots; completed slots have shape `[0, 0]` and no candidates.
`native.score_spatial` checks offsets, masks, shapes, padding and tile references.
Spatial arrays use numeric NumPy buffers, not JSON.

```sh
.venv/bin/python -m hex_command_rl.spatial_train --seconds 1200 \
  --source-report tmp/rl/deathmatch-classic-20m/report.json \
  --out tmp/rl/deathmatch-classic-cnn
```

This supervised experiment reuses the source report's complete train/validation/
evaluation configurations, collects fresh board snapshots and selects validation
cross-entropy. Available old dense arrays are compared exactly to the recollected
features, offsets and labels. Seed splits, configuration/side/content provenance
and channel schema are checked. `--dataset-dir` can reuse a completed spatial
corpus; `--dropout 0` supports an ablation on the same data. The supervising process
enforces the requested budget covering child startup, collection, comparison
and optimization (up to 30 minutes for CNN and 20 for MLP). Versioned weights/model pairs are published through an atomic
best-checkpoint pointer. Final native parity, paired matches and timing follow
training. Logs include full validation, a fixed training probe, action accuracy,
prediction frequencies, throughput and peak RSS. CNN observations require fresh
collection; the old 172-value rows alone cannot reconstruct full boards.

## Representative combat corpus and bounded comparisons

`representative.py` prepares an isolated catalog with Plains, current River
Valley, Lantern Road's Villages, and Borrowed Tide's Sluice map. Villages and
Sluice use `LandscapeConfig.authored`: starting units, roles, equipment, faction
gold, relations and recruitment pools are retained instead of replacing them with
paired arena rosters. They are standalone scenarios with no campaign state,
events, scripted orders or escort requirements. Conditional tile presentation is
removed. Villages drops the passive prisoner faction and enables normal human
recruits for its existing patrol. The Tollkeeper and Edrin remain separate hostile
leaders, both of whom must fall for player victory. These copies never alter the
playable catalog. Plains and River Valley retain the normal paired-roster setup.

```sh
.venv/bin/python -m hex_command_rl.representative --collect \
  --out tmp/rl/representative
.venv/bin/python -m hex_command_rl.representative_run --root tmp/rl/representative
```

Training keeps the Queued teacher, Classic opponent and our Situational/Predictor
recruiters. Each map has the same number of configured games. Authored rosters
have more seeds instead of artificial faction substitutions. Training, validation
and evaluation seed sets are disjoint. Collection shards remain homogeneous by
map and record decision boundaries. Every model reuses the same numeric feature
rows; the CNN additionally reads the shared board buffers. Raw datasets may be
losslessly archived with a verified per-file hash index to reclaim scratch space;
model exports, reports and curves remain independently usable.

`map_weights` gives each map equal aggregate decision-loss weight, preserving all
collected examples. Both micro and map-balanced validation metrics are logged.
CNN/MLP selection uses map-balanced cross-entropy; LightGBM selects map-balanced
choice accuracy and applies matching per-decision weights to candidate rows.
These are imitation targets, not battle rewards or campaign scripting.

The shared supervised runner accepts `--architecture cnn` or `mlp`, with default
CNN head dropout 0.1 and the existing plain MLP unchanged. Explicit maximum budgets
are now 1,800 seconds for CNN and 1,200 seconds for MLP; LightGBM retains its
1,200-second maximum. Reusing a completed shared corpus means these budgets cover
loading/checks/optimization/checkpointing rather than repeated collection.
`--defer-evaluation` preserves exports/parity/metrics while leaving game comparisons
until all fits finish. The orchestrator runs CNN and trees with two threads each,
then starts the two-thread MLP when trees finish, preventing training jobs from
oversubscribing a four-core box. Each supervisor independently enforces its
requested cutoff. Cancellation terminates the complete process groups.

The tree fit is a single boosted ensemble of at most 1,024 trees, depth six, with
64-round early stopping in this experiment. Prefixes can subsequently be exported
at different tree counts without retraining. Final evaluation uses 100-round
leader-elimination matches on the same four scenarios, including separate Classic,
Standard and Veteran opponent panels. Authored multi-faction encounters must be
reported as such, not presented as a full synthetic faction matrix.

## Campaign duties and warm starts

The original 172 feature positions and first 53 board channels retain their
meaning and normalization. Forty-seven appended candidate values describe actor-
bound GoTo/Occupy/ClearPassage travel, priorities and arrival flags; escort
subjects/eligible guards, requested guards, nearby eligible allies and distances;
shortest-terrain-route corridor occupancy; mission progress and hold duration; terminal arrivals/attack targets; and unfinished/actionable
duty counts. Global duty/progress values are present on EndTurn as well. Route
turn estimates are optimistic movement-cost/base-movement estimates. Nearby
guards means eligible non-leader allies within one hex, not planner assignment.
Corridor flags mean membership of a shortest terrain-cost route, not proof that
all alternative routes are blocked. These observations make no tactical choices.

Nine appended board channels mark GoTo/Occupy/ClearPassage actors and destinations
(six channels), escorted subjects and requested guard counts (two), and Survive
units (one). Priority is encoded as 1/3, 2/3 or 1; guard counts divide by ten.
Survive also reuses the existing protected candidate flag. Dense duty features
preserve actor-target binding when spatial markers overlap. ClearPassage remains
an explicit input; no automatic native rescue or escort policy is added to the
learned planner.

Progress is supplied by the visible Campaigner read model. Hold counts come from
the engine's live counters. A known flag distinguishes absent hold duration from zero remaining turns.
This metadata does not change scripted completion or failure behavior. Deadline
reasoning is outside this observation contract.

Runtime preserves a canonical snapshot of Campaigner orders, including recruiter
promotions. Teacher recording and learned candidate generation both use that
snapshot, before a heuristic planner's coordination rewrites. The planner still
owns its private coordination state; policy choices are not moved into the
observation builder. Snapshots reset when the controlled roster changes.

The appended values/channels are zero outside campaigns. Existing arena datasets
with schemas 2/3 or 4/5 can therefore be reused with batch-local zero padding, without
rewriting their large disk buffers. Legacy campaign datasets are rejected and
need recollection because their policy filtering or observation annotations are outdated.
`native.upgrade_model(json)` validates and exports a migrated native model; normal
model loading also performs this migration. Scoring APIs accept the known legacy
172-feature/53-channel and 219-feature/62-channel buffers with zero extension. Unknown layouts are rejected.

`spatial_train --warm-start <weights.pt>` loads either current or legacy CNN/MLP
weights, preserving old columns and spatial embeddings while initializing added
connections to zero. Adam starts with fresh state. The default learning rate is
1e-4 for warm starts and 1e-3 for new models, with an explicit override available.
Initialization provenance and the source checkpoint SHA256 are recorded. Keep
old checkpoints and write fine-tuning results to a separate output directory.
Adding these inputs alone does not teach campaign behavior; actual campaign
examples and held-out campaign/arena evaluation are still required.

## Diplomatic restraint

Attack candidates pass the same `TacticalOrders::attack_allowed` policy used by
Full and Queued before they reach the scorer. Under withdrawal protection this
excludes attacks on withdrawing enemies, and attacks on mediated enemies whose
preview damage reaches their remaining HP. This is a policy restriction in
addition to engine legality and scoped attack areas; the network cannot override
it. The damage preview is an estimate, not a guarantee of a nonlethal outcome.

Eight candidate inputs append diplomatic mode, withdrawal-protection mode,
protected-retreater and mediated-enemy counts (divided by ten), whether the nearest
enemy at the resulting location is withdrawing, whether an attack target is
mediated, and withdrawal-lane membership before/after the action. The first four
are available on EndTurn. Three board channels mark protected retreaters, mediated
enemies and withdrawal lanes. Mode flags are separate dense inputs, including
when no corresponding units are present. These annotations are campaign-only;
protection-specific observations are zero when withdrawal protection is inactive.
Old weights initially ignore these inputs through zero-weight expansion. Their
campaign examples must be recollected under the current policy before fine-tuning.

## Escort teacher scoring

Full and Queued share bounded campaign courier/captive travel scoring in
`ai/policies/escort/scoring.rs`: advancing earns up to five points, scaled by
fraction of the remaining route covered, and arrival adds ten. Standing at the
endpoint earns nothing again; leaving it reverses the arrival term. Endpoint
membership is checked explicitly because zero-cost terrain can also have zero
remaining route cost. Village
capture and healing retain their existing values.

For these actors, predicted exposure costs at most twenty points and leaving
exposure credits at most twenty. Predicted lethal exposure separately costs forty;
the departure credit never refunds that death cost. The generic extra reward for
reducing incoming damage is disabled here to preserve the twenty-point bound.
Queued also evaluates reachable escape moves for threatened `Survive` actors in
campaigns, even if its preliminary shortlist already contains a positive plan.
This prevents progress/attack variants from crowding safe moves out before threat
assessment. These are teacher policy changes; existing learned weights require
fresh demonstrations and fine-tuning to acquire the resulting preferences.
