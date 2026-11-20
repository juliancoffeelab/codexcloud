# Native learned planners

The learned planner rates legal action candidates. Rust retains recruitment,
campaign objectives, diplomacy, combat RNG and campaign progression. The Python
package supports imitation learning, CPU PPO, evaluation and checkpoint recovery.
Inference is native-only; browser integration is future work.

## Install and verify

From the game root:

```sh
python3 -m venv .venv
.venv/bin/pip install 'maturin>=1.9,<2'
.venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
.venv/bin/pip install -e 'training[dev]'
RUSTFLAGS="-C target-cpu=native" node scripts/toolchain/with-toolchain.mjs \
  .venv/bin/maturin develop --manifest-path rust/training-python/Cargo.toml --profile simulation
.venv/bin/python -m unittest discover -s training -p 'test_*.py'
.venv/bin/ruff check training --select F
.venv/bin/ruff format --check training
```

`training[trees]` additionally installs LightGBM. PyTorch's CPU distribution is
selected explicitly above; Matplotlib is required by the evaluation tools.
The native extension command targets the build machine, matching `ai:rust:build`.
Omit `RUSTFLAGS` when producing a portable extension: it retains runtime SIMD
dispatch. Host builds select a concrete SIMD kernel at compile time when the
required CPU features are enabled.

## Observations and actions

`BatchEnv.reset(config_json_strings)` and `step(candidate_indices)` return owned
NumPy candidate features and offsets, rewards, terminated flags and truncated
flags. Each index selects one candidate within that environment's slice. Completed
slots have empty slices until reset; reset replaces the batch. Use independent
one-slot environments to reset games individually. `close()` disconnects the
controller and cancels its native rollout at the next decision boundary.

Dense/tree schema 10 has 243 features: mechanical unit and weapon profiles,
terrain modifiers and movement/path costs, Campaigner duties, protected actors,
escort/hold progress, diplomacy and known unconditional defeat deadlines. Spatial
schema 11 supplies 79 board channels. The hex CNN shares two seven-neighbor
convolutions across candidates, gathers actor/destination/target embeddings and a
global pool, and scores candidates through a 64/64 head. The critic reuses the
same pooled board encoding with its gradient detached from the policy.

Native inference prepares flat dense weights and reordered convolution weights
once per model. Candidate heads use single-threaded `gemm`; seven-neighbor
convolutions use `pulp` to select scalar, AVX2 or AVX-512 kernels at runtime.
Each evaluator owns reusable buffers. Geometry is cached by board dimensions and
the complete valid-tile mask; activations are recomputed on every decision.
Exported weights and schemas retain their portable layout. Optimized reductions
can differ slightly in floating-point rounding from the scalar reference scorer.

Candidates contain EndTurn, all reachable legal moves, allowed attacks with each
weapon, and recruiter-approved promotions. Recruitment is performed by the
selected existing recruiter. Moves and attacks are separate decisions. Scoped
attack permissions are enforced before scoring. Survive and objective duties are
observations; the model can still make unsafe or unproductive legal choices.

The catalog crate owns JSON-derived definitions and their pure composition API.
Its source records and numeric tables are private. Consumers obtain definition
objects, opaque attacks, effect descriptors and resolved calculation outputs.
A unit composes explicit current equipment, gained abilities, battle-selected
effect occurrences, map modifiers and perception grants. Scoped effects are not
automatically applied to their emitter and incoming effects are not re-emitted.
Battle owns positions, effect recipients, diplomacy, legality, pathfinding,
visibility, random rolls and execution. It supplies the current terrain and
illumination for each hypothetical query. Recruitment, planners and CNN
observations must consume that composed API rather than reconstructing stacking
from authored contributions. Display metadata and readable own contributions do
not expose complete source definitions.

The 79 spatial channels retain their existing order. Movement cost, passability,
terrain healing and six terrain defense planes are evaluated for the actor and
its destination context. These nine planes join the five perception planes in
the sparse actor-context overlay; they are not shared type-only terrain tables.
Existing checkpoint shapes remain loadable, but these corrected observations
can change their behavior without changing their weights.

Unit/faction/terrain names are not learned identity embeddings. Live mechanical
stats expose balance changes, but observation compatibility does not guarantee
playing strength on new content. Travel-turn features are optimistic route-cost
estimates, omitting future occupancy and combat. Unconditional scripted defeat
clocks have a known flag and countdown; conditional deadlines remain unknown.

The dense, tree and spatial native loaders validate schema names, shapes and
finite parameters. Explicit legacy upgrades append zero weights to preserve old
scores. Preserve old model files and write new versions separately.

### Visibility observations

The following inputs extend the existing mechanical and campaign inputs.
Concealment inputs describe evaluated mechanics rather than forest-specific
abilities: changing eligible terrain from forest to water or sand should produce
equivalent inputs when the resulting concealment mechanics are equivalent.
Use shared Rust perception queries, including terrain eligibility, equipment,
effects, illumination, penetration and reveal distance. Separate availability
from distance so a reveal distance of zero does not mean no concealment.

Terrain channels:

| Name                                | Meaning                                                                                                                |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `tile_remembered`                   | The observing faction retains persistent terrain knowledge of this tile.                                               |
| `tile_in_sight`                     | At least one friendly sight source currently sees its terrain.                                                         |
| `possible_enemy_hide_place`         | An enemy could remain concealed here, based on public catalog capabilities; does not indicate actual hidden occupancy. |
| `actor_concealment_available`       | Concealment applies to the acting unit on this tile.                                                                   |
| `actor_concealment_reveal_distance` | Reveal distance for the actor's applicable concealment, before observer-specific penetration.                          |
| `actor_can_observe_terrain`         | The actor itself can see the tile's terrain, independently of friendly spotters.                                       |
| `tile_insight_of_enemy`             | Estimated current terrain visibility from known hostile sight sources.                                                 |

Remembered terrain and current sight are distinct. Persistent darkness with
`remember: false` does not automatically record exploration history. Visible
terrain can still contain concealed units. Do not add a separate
`tile_illuminated` input initially: lighting already contributes to evaluated
concealment and detection. Actor-relative channels use fourteen planes per candidate actor, selected by the
actor tile index. Python shares the common first-convolution contribution, adds
the actor contribution before ReLU, and batches the second convolution across
contexts. Rust caches each actor context within the decision. EndTurn uses the
actor-free encoding; the critic pools that shared encoding.

Unit channels, populated for friendly units and currently visible enemies or
neutral units:

| Name                                | Meaning                                                                                       |
| ----------------------------------- | --------------------------------------------------------------------------------------------- |
| `unit_sight_range`                  | Base finite range after group, race, unit, map/weather, equipment and scoped-effect stacking; target lighting may modify it. |
| `unit_sight_unlimited`              | Sight range is unlimited.                                                                     |
| `unit_concealment_available`        | Concealment applies to this unit on its current tile, even if detection currently succeeds.   |
| `unit_concealment_reveal_distance`  | Reveal distance for its applicable concealment, before observer-specific penetration.         |
| `unit_detectable_by_actor`          | The actor can directly detect this unit without another friendly spotter.                     |
| `unit_detectable_by_friendly_count` | Number of friendly sight sources that independently detect this unit.                         |
| `unit_can_detect_actor`             | This known unit can directly detect the actor.                                                |

Count independent detecting sources, not every unit receiving a shared faction
sighting. A unit's camouflage can be active while proximity, penetration or a
combat reveal makes it visible. Hidden enemies contribute no unit-channel
occupancy, stats or capabilities.

Dense candidate features:

| Before                                         | After                                         |
| ---------------------------------------------- | --------------------------------------------- |
| `actor_concealment_available_before`           | `actor_concealment_available_after`           |
| `actor_concealment_reveal_distance_before`     | `actor_concealment_reveal_distance_after`     |
| `actor_detectable_by_known_enemy_count_before` | `actor_detectable_by_known_enemy_count_after` |
| `target_detectable_by_actor_before`            | `target_detectable_by_actor_after`            |
| `target_detectable_by_friendly_count_before`   | `target_detectable_by_friendly_count_after`   |
| `target_can_detect_actor_before`               | `target_can_detect_actor_after`               |
| `friendly_visible_enemy_count_before`          | `friendly_visible_enemy_count_after`          |

Here `target` is the enemy attacked by the candidate; movement-only candidates
have no attack target. Target-specific after-values assume that target survives;
kill probability remains a separate existing input. Do not add detector-share
features: detector counts and the existing enemy count already supply that
information. Retain the target's before/after friendly detector counts initially,
including changes to lighting and other friendly sight sources. These distinguish
personal dark vision from establishing a persistent light that lets other allies
maintain contact. The total visible-enemy counts also describe sightings gained or
lost by movement candidates without an attack target.

Enemy sight and detection estimates must use known sources and observable
capabilities, not the enemy's actual exploration memory or unseen units. Apply
the same information contract to rule-based threat forecasts, which assume that other groups already remember every rememberable tile. This
conservative estimate is independent of their actual exploration history; hidden
sight sources are excluded. Different penetration capabilities can defeat otherwise applicable
concealment; a reveal-distance scalar alone is not a universal detection guarantee.

Before/after values must reflect supported sight, concealment, combat-reveal and
lighting effects. Existing mechanical forecasts do not execute campaign scripts;
scripted effects such as lighting a village need an explicit forecast contract
before being represented as known after-state facts. New input weights are zero-initialized when expanding older checkpoints,
preserving the starting policy. Distances and sight-source counts are divided
by 20; booleans are 0/1. Movement projections account for carried lights; attack
projections account for weapon and retaliation reveals. Promotions retain the
current perception profile in these after-values.

### Native spatial array contract

Spatial observations contain six owned NumPy arrays: board values (CHW), board
value offsets, board shapes (N×2), candidate tile indices (M×3), actor context
values (14 CHW planes), and context metadata (K×3: decision index, actor site,
value offset). Shared actor-relative planes are zero. Contexts contain only actors
present in the candidate list; neither path contains unseen enemy occupancy.
Disk datasets retain the context arrays alongside the common board buffer.

## Entry points

Each trainer's `--help` is the authority for its options. Run these from the game
root and place mutable runs under ignored `tmp/rl/`.

| Module under `hex_command_rl`     | Purpose                                                          |
| --------------------------------- | ---------------------------------------------------------------- |
| `train`                           | Dense linear/MLP imitation over disk-backed teacher decisions    |
| `trees`                           | Time-bounded LightGBM candidate ranker; install the trees extra  |
| `spatial_train`                   | CNN imitation and compatible policy fine-tuning                  |
| `evaluate`, `compare`             | Native evaluation of saved policies                              |
| `benchmark`                       | Frozen-position planner timing, including existing orchestration |
| `mixed_workload`                  | Isolated authored catalog and disjoint hierarchical seed panels  |
| `environments`                    | Offline authoring of fixed training-only tactical fixtures       |
| `diverse_workload`, `diverse_run` | Held-out validation and supervised PPO-only continuation         |
| `mixed_imitation`                 | Weighted imitation over losslessly compressed shards             |
| `mixed_ppo`                       | Independent native slots and decision-budgeted multi-task PPO    |
| `mixed_evaluate`, `handoff`       | Greedy validation and robust checkpoint selection                |
| `overnight`                       | Restartable imitation → PPO → held-out evaluation supervisor     |
| `remote_backup`                   | Local recovery export with explicit optional Git publication     |

For example, prepare a mixed workload and launch its supervisor:

```sh
.venv/bin/python -m hex_command_rl.mixed_workload --root tmp/rl/new-run
.venv/bin/python -m hex_command_rl.overnight --root tmp/rl/new-run \
  --posttune-seconds 5400 --rl-seconds 32400
```

The budget covers collection/optimization and periodic evaluation pauses; final
selection and testing follow it. Run a long supervisor in an independently
managed process. Restarting its command recovers existing atomic phase state.
A finished run is retained and reported as complete; use a new root for another
experiment.

The engine-owned `rust/data/simulation/rl.json` selects workload maps, roster
pairs, objective routes, opponents and recruiters. Python reads this validated
manifest through the native binding. Isolated catalogs copy the current engine
catalog; they do not create a second set of scenario definitions. See
[the catalog contract](../rust/data/CATALOG.md) for visibility and simulation setup.

Mixed sampling assigns equal decision budgets to arenas and campaigns, then
balances maps/campaigns and divides their objective/route variants. Recruiters and
fixed opponents are explicit workload axes. Seed partitions separate training,
validation and final testing. Fixed-opponent comparisons measure that opponent
set; authored rosters, diplomacy and objective routes remain part of the case.

Imitation learns a teacher's candidate selection. PPO uses fresh stochastic
rollouts, a critic, GAE, clipping, entropy regularization and a frozen-reference
KL anchor. The native step API returns 0 for truncation. The mixed PPO trainer
applies the family-specific terminal penalties documented below; historical
experiments retain their original reward parameters in their manifests.

The mixed handoff and final RL selection evaluate larger validation panels.
Candidates within 2.5 percentage points of the best minimum arena/campaign ×
opponent score compete on overall score and caps. Positive minimum scores take
precedence over a zero stratum. The starting policy remains eligible. Final test
seeds never select checkpoints. Other trainer entry points document their own
selection metric; teacher agreement alone does not establish game strength.

## Diverse tactical continuation

The active RL maps and campaigns are fixed, engine-owned JSON registered in
`rust/data/simulation/rl.json`. They include 14 arenas and the Underground Escape,
Woodland Defense, and Splitwake Crossing campaigns. The V3 training catalog and
workload remain available in the [archived experiment](experiments/2026-11-15-visibility/experiment.json).

Prepare a new workload with:

```sh
.venv/bin/python -m hex_command_rl.diverse_workload --root tmp/rl/new-run
.venv/bin/python -m hex_command_rl.mixed_ppo --root tmp/rl/new-run \
  --warm-start training/checkpoints/campaign-cnn-ppo-v1/resume.pt \
  --seconds 600 --campaign-cap -2
```

Run `mixed_evaluate --root tmp/rl/new-run --phase rl` in a second process during
training. Training uses the fixtures in `simulation/rl.json` with equal
arena and campaign decision budgets. Each task splits opponent coverage
between Classic, Predictor, and Full with the Predictor recruiter.
Checkpoint selection uses separate synthetic arenas and the Coastal Assault
and Whitebank Relief campaigns. The authored showcase evaluates Lantern Road
and the River Valley, Toll Road, and A Tide for Both battlefields after
selection. The three panels use separate layouts and seeds. The starting
policy stays eligible.
PPO uses a 100-round harness cap per arena or campaign stage; authored defeat
deadlines can end a stage earlier.
The full panel uses twelve campaign seeds per route; the small periodic panel
is only a progress indicator. A PPO-only supervisor can manage restarts and
evaluation with `diverse_run --root tmp/rl/new-run --warm-start <resume.pt>
--seconds 32400`. For a new warm-start phase, `--seed-start 9000001` supplies a
fresh rollout range; exact resumes restore their own seed cursor. Worker errors
include their configuration and action indices for replay through `BatchEnv`.

Wins/completions receive +1. Missing required escortees receive −2; losses of
other required companions receive −1.5; other defeats, including leader-only
losses, receive −1.
Campaign caps receive −2 by default; arena defeats and caps receive −1. These are
terminal outcomes, not per-casualty penalties, and do not accumulate. Terminal
summaries report required-character losses and reached stages without adding
privileged observations to live decisions. Optional losses do not automatically
become campaign-failure penalties. A configurable −1.5 campaign-cap experiment
requires a fresh warm start; exact recovery rejects changed reward settings.
Scripted defeats outside `requiredCompanions` retain their raw outcome text and
use the ordinary defeat penalty. This includes Lantern Road's scripted captive
failures; Lantern Road is held out in this continuation, so those rewards are
not used for optimization.

Visibility uses the current engine's permitted knowledge. This continuation adds
neither neural fog-memory features nor exceptions granting hidden information;
the limitations tracked in issue #141 remain separate.

## Recovery and publication

`recovery.save_bundle` writes policy/optimizer/RNG state, a native export and a
size/SHA-256 manifest into a temporary directory. It fsyncs and renames the
completed bundle before replacing a latest/best pointer. Readers validate one
captured pointer, so a concurrent save cannot change which model is attributed
to an evaluation. Completed atomic evaluation files recover interrupted JSONL
appends; an incomplete trailing record is repaired before writing resumes.

Supervised recovery also retains the shuffled shard/minibatch cursor. Compressed
shards are checksum-verified and reused. PPO restores policy, critic, both
optimizers, reference weights, RNGs and sampler budgets; in-progress games restart
on fresh seeds. Resume requires matching workload/content/engine provenance and
sampling temperature. For new units, maps or engine revisions, use a policy
warm start with a fresh run and fresh demonstrations instead of claiming an exact
resume.

`remote_backup` defaults to `<run>/recovery` and makes no Git changes. For an
explicitly authorized remote recovery stream, choose a distinct destination and
name the exact feature branch:

```sh
.venv/bin/python -m hex_command_rl.remote_backup --root tmp/rl/new-run \
  --destination training/recovery/new-run --branch feat/my-training-run
```

Publication commits only that destination and leaves unrelated staged work in
the caller's index. Completed experiment archives are immutable destinations.
A workspace replacement still requires restoring the catalog/workload and
relaunching processes; persisted optimizer/RNG state does not keep a process alive.

## Committed models and experiment evidence

- `checkpoints/campaign-cnn-imitation-v1/` is the original frozen CNN baseline.
- `checkpoints/campaign-cnn-ppo-v1/` contains the selected native policy,
  full recovery state and a manifest pointing to its experiment.
- `checkpoints/campaign-cnn-ppo-v2/` contains the diverse-environment continuation,
  selected on validation rather than the final update. It retains the same input
  schemas and network architecture. V1 remains available for comparisons.
- `checkpoints/campaign-cnn-ppo-v3/` retains the experimental six-hour policy from
  the nine-hour visibility continuation: 243 dense features, 79 spatial channels
  and 34,609 actor parameters. New input weights were initially zero, preserving
  the V2 policy before PPO. Validation still preferred frozen V2 overall; V3 is
  retained for comparison rather than promoted as a stronger default.
- `experiments/2026-11-15-visibility/experiment.json` records that continuation,
  including hourly validation, unused final-test seeds, actor-conditioned input
  diagrams, training curves, native rulebot comparisons and exact recovery data.
  Its `selected_model` is the safe validation choice; `trained_model` identifies
  the experimental continuation independently. Underground Lair diagnostics never
  participate in selection. The archive preserves the three-hour and four-hour
  policies that each completed one Lair case, alongside matched traces.
- `experiments/2026-11-14-diverse/experiment.json` inventories nine accounted PPO
  hours across three phases, including the changed curriculum and recovery
  boundaries. The broader phase warm-started from V1. Its archive contains frozen
  catalogs/workloads, raw metrics, selected/last recovery states, validation
  selection and matched final tests. Exact recovery uses the recorded training
  source; compatible engine changes use a new warm start.
- `experiments/2026-11-09-mixed/experiment.json` inventories the completed run.
  Its recovery directory losslessly preserves every formerly committed recovery
  file, including supervised/final policies, logs, figures and original manifests.
  Large JSON, JSONL and Torch state files use an additional gzip layer; their
  original hashes and restore paths are recorded in `recovery_files`.
- That experiment also retains the frozen catalog, written-bot comparisons,
  failure census and selected exact replay examples. The full corpus, all
  intermediate checkpoints and all 500 detailed traces remain in the original
  ignored run directory. They are preserved research evidence, not required
  checkout dependencies.

Inspect `evaluation.json` for balanced results and the archived CSVs for exact
campaign/route/opponent counts. Trace outcomes distinguish protected-character
deaths, leader deaths, scripted deadlines and harness caps. Forecasts and one-turn
counterfactuals establish only their recorded scope.

The diverse continuation's final held-out results were:

| Test stratum                 | V1 wins/completions | Selected V2 wins/completions |
| ---------------------------- | ------------------: | ---------------------------: |
| Existing arenas vs Classic   |             620/768 |                      642/768 |
| Existing arenas vs Predictor |             444/768 |                      439/768 |
| Lantern Road vs Classic      |              64/150 |                       72/150 |
| Lantern Road vs Predictor    |              40/150 |                       64/150 |

Both policies use Situational/Predictor recruiters. Campaign tests use 25 seeds
across authored/flight/fight routes. V2 capped six campaign runs against Classic;
V1 capped none. The final update fell to 24/150 against Predictor, so it is
archived rather than selected. These results describe the frozen catalog, not a
guarantee on future campaign or unit revisions.

The archive's `training-curves.png` shows stochastic training outcomes in
15-minute bins (the final bin is partial). `validation-curves.png` uses full
selection panels; `test-comparison.png` compares V1, selected/last V2 and written
bots. `evaluation.json` retains task/route/recruiter counts, terminal messages and
stage funnels; the compressed reports retain each exact case and outcome.

Verify the committed archive, or restore its original recovery layout into a new
ignored directory:

```sh
.venv/bin/python -m hex_command_rl.artifacts \
  training/experiments/2026-11-09-mixed/experiment.json
.venv/bin/python -m hex_command_rl.artifacts \
  training/experiments/2026-11-09-mixed/experiment.json \
  --extract-recovery tmp/rl/restored-recovery
```

The selected model remains experimental: protected-unit survival and deadline
execution are known weaknesses, especially against Predictor. Adding content
requires fresh matched evaluations. Neither these results nor native/PyTorch
parity establish browser deployment readiness.
