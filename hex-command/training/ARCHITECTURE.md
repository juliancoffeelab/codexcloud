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

Dense/tree schema 16 has 254 features: mechanical unit and weapon profiles,
terrain modifiers and movement/path costs, Campaigner duties, protected actors,
escort/hold progress, diplomacy and known unconditional defeat deadlines. Spatial
schema 19 supplies 83 board channels to two parallel hex CNN branches. Each has
83→16 and 16→16 seven-neighbor convolutions with ReLU. The detailed branch keeps
full resolution; the coarse branch uses stride two in its first convolution.
Each branch supplies actor/destination/target embeddings (48 values) and a masked
average (16 values). A GRUCell takes the detailed branch's 16-value global pool and a 64-value
previous hidden state. Its updated state is shared by all candidates. The
446→64→64→1 head receives 254 candidate features, 128 spatial values, and 64
memory values. The policy has 70,817 parameters. The critic receives the detailed
pool, hidden state, and mean candidate features, all detached from the policy.

Coarse cells aggregate non-overlapping 2×2 axial blocks. Their first-convolution
centers are (2r+1, 2q), so the seven-hex stencil covers all four block positions.
Valid coarse cells have at least one valid source tile in their block. This mask
and asymmetric convolution padding make scores independent of batch padding,
including odd dimensions and jagged map edges. Candidate positions map by integer
division of row and column by two. Actor-relative contexts are retained in both
branches. Native inference caches separate geometry and SIMD buffers for each.
Serialized convolution order is detailed-first, detailed-second, coarse-first,
coarse-second. Legacy models expand with trainable coarse weights and zero head
weights for their 64 new encoded inputs, preserving their original scores.

The final four spatial channels mark our hold-objective tiles and remaining
turns, followed by the opponent's hold-objective tiles and remaining turns.
Countdowns are divided by 30. Separate locations carry independent countdowns;
all tiles of a multi-tile objective carry its shared countdown. At overlapping
holds, the smallest remaining count is shown. A marker distinguishes completed
zeroes from tiles without a known hold. These planes use Campaigner-visible
progress and targets, independent of actor assignments. Arena holds also supply
the acting faction's authoritative progress through Campaigner and the scalar
duty features. Opponent arena progress travels separately through Campaigner
into spatial planes, without changing our duty assignments or scalar progress.
Arena hold clocks are public objective scores in clear, fog and sight variants;
they expose no hidden unit positions. Shared objectives mark both sides' planes
on the same tiles, with independent countdowns relative to the acting faction.
Older CNN checkpoints append zero convolution weights for the new planes,
preserving their scores until further training.

The candidate head includes consecutive idle turns divided by ten and an
acted-this-turn flag. A tactical move, attack, or promotion marks the turn active.
History is maintained per observer faction, resets for each match and chapter,
and reaches both the live training bridge and native inference. New history
columns start with zero weights; older model exports and PyTorch policy weights
expand with zeros before the spatial embeddings. The spatial head input is 446
values (254 candidate features, 128 spatial values, and 64 memory values).

The candidate head also receives nine per-unit history features: a known-history
flag, previous move/attack/promotion one-hot flags, known-move and known-attack
flags, turns since each (divided by ten), and whether a candidate move returns to
the origin of the unit's most recent move. Unknown events have zero age and a
zero known flag. End Turn has no actor and receives zero unit-history features.
History records executed tactical commands per unit and observer faction; it
shares the match/chapter lifecycle of idle history. Considering candidates does
not update history. The history features do not add convolution channels.
Older schema-12/13/14/15/17/18 weights and datasets remain loadable; new head columns
start at zero, and old embedding weights retain their meaning.

Recurrent memory is explicit in Torch `forward_recurrent`: callers supply a
batch×64 tensor and receive updated state. Ordinary `forward` and independent
`score_spatial` calls use zero initial state. Native gameplay commits memory once
per executed-decision boundary, retaining the input state for repeated scoring.
State is independent per observer faction; explicit match/chapter generation,
map changes, counter rewinds and fresh episodes reset it. `BatchEnv` exposes these
decision contexts so Python uses the same lifecycle, including revisiting a map.

Both PPO trainers retain policy and reference memory during rollouts, cache
frozen-reference logits, and replay ordered windows of at most eight decisions.
Hidden states stay connected inside each window; window starts use detached
rollout states (truncated backpropagation through time). Windows are shuffled,
individual positions within a window are not. Chapter resets split windows
without ending campaign reward credit. Bootstrap evaluation does not commit
memory. Checkpoint recovery restarts environments and hidden state at fresh
seeds, as before. Legacy shuffled datasets used by the older standalone
imitation tools remain zero-initial-state examples. The mixed pretrainer uses
the ordered sequence path described below and rejects corpora without boundaries.

GRU exports use PyTorch reset/update/candidate ordering, including reset after
the hidden candidate bias. Native weights and explicit hidden-state parity are
validated separately from stateless scores. Older policy checkpoints initialize
the GRU and append zero memory-head columns, preserving their previous scores;
optimizer/critic states require a matching recurrent architecture to resume.

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
| `mixed_imitation`                 | Weighted recurrent imitation over ordered compressed teacher streams |
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

Mixed sampling uses authored `samplingWeights`: **25% ordinary arena decisions,
25% targeted-encounter decisions, 50% broad-campaign decisions**. Each pool keeps
its own decision budget while PPO adapts map/condition weights inside it. The 28
targeted maps include the four new multi-objective challenges. Their eight
validated opponent/visibility combinations are registered explicitly; fixed-force
encounters retain zero recruits and the validated initiative. Unqualified fog
variants do not enter training. The distant-commander control shares Marlow’s map
budget at conditionWeight0.25 versus1 for the exposed-commander challenge.

`samplingMap` identifies the logical map: Ashbank's physical visibility variants
share one budget. `mapWeights` gives each map its authored base mass. Each of the
three escort maps has weight1/3, Bellmarket Courts and Thorn Verge1/4; other maps
have weight1. Base weights multiply PPO mastery weights and normalize within the
same pool. Conditions share their map's base mass using explicit `conditionWeight`, independent of condition count.
Imitation uses the same base decision mixture; PPO additionally updates mastery
from learned-policy outcomes. Evaluation does not adapt and keeps equal mass per
represented family/map, independent of training curriculum weights.

Reward profiles are separate from storage format and sampling group. `arena`
pays +1 on victory and no intermediate chapter bonus, including single-stage
encounters authored as campaigns. `campaign` pays +1 per intermediate chapter
and +2 for the finale. Existing defeat/cap/loss penalties remain. Objective-progress
and kill bonuses remain disabled. Profiles and map weights participate in workload
provenance alongside cases, teachers, pool budgets, adaptive settings and native
content fingerprints.

`pretrain_imitator` may select an existing named controller or explicit
`{recruiter, campaign, planner}` blocks. New teachers record their validated full
composition: Classic/Mission/Classic, Balanced/Mission/Queued, or
Predictor/Mission/Full. Native collection applies all three teacher blocks; PPO
continues to use the learned planner. No pretraining is implied by configuration.
Seed partitions separate training, validation and final testing. Fixed-opponent
comparisons retain authored roster, diplomacy and objective scope.

PPO allocation is hierarchical: pool, then map, then condition. Map and condition
credits accumulate the decision share owed under the weights in effect at each
step. Mastery changes retain those credits; live episodes reserve estimated
remaining work. Schema 3 checkpoints persist both credit sets. Schema 2 resumes
retain lifetime exposure and mastery statistics but initialize prospective credits
at zero, avoiding catch-up against the historical starvation bug.

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
- `checkpoints/campaign-cnn-ppo-v4/` is the final nine-hour PPO checkpoint from
  `ppo-from-pretrain-43m-9h-20261121`, initialized from the 43-minute imitation
  checkpoint. It includes native weights and training resume state. V4 names
  the final checkpoint; it also won this run's recorded validation selection.
  See the detailed V4 training history below.
- `experiments/2026-11-22-recurrent-imitation/experiment.json` records the one-hour
  V4 imitation continuation with the coarse branch and GRU. Its targeted gains
  came with regressions on broader campaigns, held-out gameplay and Lantern.
  The archived mixed-score winner is experimental; no default model was promoted.
  Five-minute native policies and exact sequence-resume states are preserved.
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

### V4: nine-hour PPO after imitation

`checkpoints/campaign-cnn-ppo-v4/` freezes the **final** policy from
`tmp/rl/ppo-from-pretrain-43m-9h-20261121`. Its native `planner.json` and Torch
`resume.pt` are copied without changing their weights. The saved
[training record](checkpoints/campaign-cnn-ppo-v4/training-record/provenance.json)
contains the run identity, architecture dimensions, workload fingerprints,
training settings, panel inventories and final results. The adjacent files
preserve the launch scripts, parameters, launch checks and completion report;
they describe the historical run rather than today's trainer defaults.

**Architecture.** This is the older, stateless hex CNN: schema 11, 79 spatial
channels and 243 candidate features. Two seven-tap hex convolutions map
79 → 16 → 16 channels at the original map resolution. Candidate spatial
readouts add 64 values to the 243 dense features; the scoring head is
307 → 64 → 64 → 1, with ReLU hidden activations. The actor has 34,609
parameters. V4 predates the later idle/unit history inputs, per-objective hold
countdown channels, coarse spatial branch and GRU. The current architecture
sections above describe subsequent development, not the contents of V4.

**Training lineage.** Imitation started from the saved V2 policy, not V3.
The teacher used the Queued planner, Predictor recruiter and Mission campaign
controller. Cross-entropy trained the learner to select the teacher's action
among legal candidates. The one-hour imitation job used 408 training cases and
80 separate validation cases across the 14 arenas and three campaigns below.
The checkpoint chosen for PPO was epoch 6, at 2,605.216 seconds (43m 25s).
PPO then ran for 32,400.082 seconds: 5,143 updates, 10,566 completed episodes
and 1,316,608 learner decisions. Thus “nine-hour” names the PPO stage; imitation
and evaluation took additional time.

**Training environments.** The arena IDs were `rl-twin-fords`,
`rl-splitpine-clearing`, `rl-crown-gap`, `rl-jawbone-crossing`, `rl-sump-teeth`,
`rl-last-lantern`, `rl-switchback-saltworks`, `rl-cinderfault`, `rl-sable-sound`,
`rl-three-wells-trail`, `rl-frosthook-gate`, `rl-birch-pocket`, `rl-quarry-ring`
and `rl-fordstep`. They mixed deathmatch and territory objectives, the twelve
ordered cross-faction pairings among Human, Skeleton, Woodland and Coastal
armies, and both arena initiatives. The three training campaigns were
`rl-woodland-defense`, `rl-underground-escape` and `rl-splitwake-crossing`,
each with three chapters and authored/fog routes. The workload contained
33,120 enumerated cases; scheduling aimed for 50% arena and 50% campaign
**decisions**, then equal tasks and variants within each family. It did not
use the later targeted encounters or adaptive completion-based sampling.

Opponents were Classic, Predictor and Full, fixed for each match. Full is
encoded as `enemy: "predictor"` with an explicit `enemyBlocks.planner: "full"`
override and Predictor recruiter. The learner's recruiter alternated between
Situational and Predictor across cases; the campaign controller was Mission.
The stored `playerBlocks.planner: "queued"` describes the workload template;
the training harness replaced that planner with the learned policy.

**Visibility.** In the three training campaigns, fog restricted the player:
the enemy group explicitly penetrated `fog`, checked before launch. Arena
visibility was different: each arena had two authored map variants, including
fog or sight variants, and their roster settings generally applied to both
sides. For example, Twin Fords fog gave both sides range 5 without fog
penetration; Last Lantern used range 6 in its base roster and range 5 in its
sight roster. These are historical settings from the frozen catalog, not a
claim that every arena used player-only fog or that every base map was clear.

**PPO settings and rewards.** Four environments, horizon 64, minibatch 8,
three optimization epochs, actor learning rate 0.000005, critic learning rate
0.001, gamma 0.999, GAE lambda 0.95, clip 0.2 and maximum gradient norm 0.5.
Sampling temperature was 0.35; entropy coefficient 0.001, reference-policy KL
coefficient 0.02 and critic warmup 24 episodes. The historical scorer paid
**+1 only for completing a whole campaign, with no chapter-completion reward**.
Campaign defeat was −1 and a campaign cap −2; escort, protected-unit and
leader losses had recorded penalties of −2, −1.5 and −1 respectively. Arena
win/defeat/cap rewards were +1/−1/−1. The later +1-per-chapter/+2-finale scorer
was added after this run and did not train V4.

**Selection and results.** The fixed periodic panel had 156 cases. Full
selection used 1,344 held-out cases: eleven arenas (`open`, `wooded`,
`crossings`, `wood-edge`, `islands`, `alder-fen`, `hart-saddle`, `herring-road`,
`ochre-wells`, `siltbar-bend`, `drowned-kilns`) plus Coastal Assault and
Whitebank Relief. Six candidate checkpoints, including the starting policy,
were compared. Selection first retained candidates within 2.5 percentage
points of the best worst stratum, then compared overall score and caps.
The final checkpoint also won this recorded selection, with balanced score
40.97%. This establishes how it was selected, not consistent improvement
across all maps.

The separate 1,314-case showcase used River Valley, the historically named
`sluice` and `villages` arena IDs, and Lantern Road. Its seeds did not select
checkpoints. Balanced showcase score fell from 31.44% at the imitation start
to 29.39% at V4; total wins/completions fell from 477 to 449 and caps rose
from 179 to 240. Those totals combine arenas and campaigns; the balanced
score weights evaluation strata rather than dividing total wins by cases.
The saved completion report includes every opponent stratum. V4 is a frozen
experimental model with mixed results, not a blanket replacement for V2.

The recorded source revision is `6b99fae284cba5a55d21ceb797ec37d008a3971c`;
the revision alone does not reconstruct uncommitted run-time changes, so retain
the recorded content/engine fingerprints and frozen run catalog. Original hourly
checkpoints, traces and curves remain in the run directory. A second checkpoint
copy remains under `/workspace/shared/ppo-from-pretrain-43m-9h-20261121/rl`.

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

## Per-environment imitation teachers

`simulation/rl.json` explicitly declares `pretrain_imitator` for every training
arena and campaign variant. `classic`, `queued`, and `full` select the player
planner while preserving the authored opponent, recruiter and campaigner. The
named `objective_progress` and structured `{"path_to":{"col":13,"row":5}}`
references select pretraining-only legal-action controllers using the same
faction-observed candidate features and spatial inputs as the learned planner.
They are not PPO policies or reward shaping. Custom path targets are validated
against an authored single-chapter map. Unknown or missing teachers fail loading.

`mixed_workload.prepare` creates disjoint train/validation/test schedules for the
whole training pool. `mixed_imitation` collects its train and validation partitions
with those teachers. Imitation validation uses separate seeds on the training
map pool (`imitation_validation`), while gameplay selection/showcase panels stay
separate and never supply training gradients. Compressed shards retain imitator identities
and reject reuse after a teacher/provenance change. Held-out maps are not added to
the corpus. PPO receives ordinary match configs without imitation settings and
always selects the learned player planner. No training begins during preparation.

The 24 targeted maps expose 66 selected enemy/visibility conditions. Clear,
player-only fog and both-sides fog share the same maps/rosters and use ordinary
events to assign the appropriate authored vision groups before play. Later
recruits inherit the faction's selected vision group. Broad environments retain
their existing mixed opponent and visibility settings.

### Recurrent mixed imitation

`collect_spatial_sequences` returns the same dense and spatial buffers as
`collect_spatial`, plus one explicit context per label: source game index,
observing faction, map, reset generation, round and decision counter. The old
collection API remains available. Compressed corpus schema 1 preserves these
contexts, teacher identities and fingerprints; missing boundaries or changed
provenance require a new corpus rather than guessing sequence identity.

The mixed trainer partitions each shard into match/faction/chapter streams.
Map reuse does not merge matches or repeated chapters. It shuffles shards and
whole streams, never individual positions inside a stream. Up to eight streams
advance in parallel through windows of eight decisions (configurable through
`--sequence-batch-size` and `--sequence-length`). Memory starts at zero at each
stream boundary, remains connected inside a window, and carries detached values
into its next window. Shorter streams leave the batch without contaminating
other streams. Candidate padding contributes no loss. Cross-entropy updates the
CNN, coarse branch, GRU and action head jointly. The existing hierarchical
per-decision loss weights remain in use; ragged final windows use the same
fixed loss denominator as full windows rather than receiving extra weight.

Validation uses the same ordered forward path with dropout disabled and no
gradients. It carries memory across windows and reports loss/accuracy over all
teacher decisions with the existing group weights. Held-out gameplay panels
remain separate. Checkpoints retain the stream permutation, active stream-batch
position, window cursor, detached memory, optimizer and RNG states. Exact resume
requires matching sequence settings and sequence schema; older independent-example
checkpoints can initialize a new run through `--warm-start`, not exact resume.

## Adaptive PPO practice allocation

`adaptiveSampling` in `simulation/rl.json` controls PPO sampling only. For a
smoothed learner completion rate `p`, the multiplier is
`max(minimumMultiplier, min(1, (1-p)/(1-completionThreshold)))`. The authored
threshold is 0.8 and the floor is 0.1: full weight through 80%, half at 90%,
quarter at 95%, and one tenth from 98% upward. There is no separate fixed/dynamic
mixture. These are relative weights normalized within the existing sampling
pools; a pool whose cases are all mastered retains its authored total budget.

Mastery belongs to an entire match template without its seed. This separates
map/objective/visibility, enemy tactic, recruiter, army pair and initiative.
The EMA starts at 0.5, updates at rate 0.1 on actual terminal outcomes, retains
full weight for ten observations, and refreshes weights every twenty completed
episodes globally. Campaign success means completing the full campaign. Caps
are failures. Worker errors and unfinished episodes do not update completion.

The sampler still allocates decision budgets, including reservations for live
episodes and moving estimates of episode length. When effective weights change,
it resets within-pool allocation debt while preserving lifetime counts and live
reservations. This prevents old allocations delaying a newly needed case.
Checkpoint schema 2 preserves mastery, frozen multipliers, refresh position and
allocation counters. Live matches restart on recovery without a synthetic loss;
the surrounding PPO checkpoint preserves the RNG for future draws.

`sampling` metric events log recently observed cases, and `sampler-status.json`
contains every case's estimate, observation count, multiplier, normalized decision
weight and actual decision share. Imitation weights, gameplay evaluation panels,
checkpoint-selection metrics and all reward calculations stay fixed.

### Mixed-imitation run clock and collection bounds

The mixed pretrainer budgets active training pipeline time. Corpus collection,
initial/epoch validation, checkpoint writes and recovery downtime do not consume
`--seconds`. Checkpoints store `budget_kind: training-time` and separate training
`elapsed` and total `wall_seconds`; older wall-time checkpoints require a warm
start. `--checkpoint-seconds` sets training-time checkpoint spacing (300 seconds
by default). `--collection-workers` and `--games-per-shard` bound collection
concurrency and retained games without truncating their teacher trajectories.
The corpus manifest pins its shard layout for recovery.

The optional `visible_target_path` offline teacher reuses the actor-specific
path controller, selecting only observed hostile tiles and prioritizing observed
defeat markers. It is configured for three validated Thorn Verge conditions;
it is not a new PPO planner or a hidden-information oracle. Force the Crossing
and Bellmarket also select different teachers by opponent/visibility after fresh
matched qualification. Existing teacher selectors remain on other conditions.
See `experiments/2026-11-23-better-targeted-teachers` for positive and rejected
results; unresolved Tidegate/Lower Gallery deficits still matter when deciding
whether to run broad imitation.


### Task-specific offline teachers

`pretrain_imitator: {"tactical": {...}}` adds three explicit role policies:
`relief` stages troops until an authored reinforcement round, `recovery` heals a
protected attacker while stronger units engage the marked guard, and `refuge`
retreats the holder, fights its pursuit and returns to hold once screened.
Each declares its public goal/home coordinates and friendly roster actor IDs;
plan-specific staging coordinates, recovery fraction and relief timing are
explicit authored parameters. These selectors never enter PPO action selection.

Controllers use the learner's legal candidate set, public friendly state and
observed enemies. They keep no completion flags across calls because turn search
may present tentative states; current observations remain authoritative. Route
progress remains positively scored even far from the objective. Native collection
supports the selectors through the existing recurrent sequence path, without
changing observation channels, model architecture or reward scoring.

Fresh matched twenty-seed comparisons qualified Crown Road Relief, Briar Saddle
and Lower Gallery Refuge; Rally and Tidegate candidates were rejected. See
`experiments/2026-11-23-task-planners` for authored selectors, full outcomes,
failed prototypes, matched hour-10 results, parity checks and limits.
