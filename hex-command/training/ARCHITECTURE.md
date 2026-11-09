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

Dense/tree schema 8 has 229 features: mechanical unit and weapon profiles,
terrain modifiers and movement/path costs, Campaigner duties, protected actors,
escort/hold progress, diplomacy and known unconditional defeat deadlines. Spatial
schema 9 adds 65 board channels. The hex CNN shares two seven-neighbor
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

Unit/faction/terrain names are not learned identity embeddings. Live mechanical
stats expose balance changes, but observation compatibility does not guarantee
playing strength on new content. Travel-turn features are optimistic route-cost
estimates, omitting future occupancy and combat. Unconditional scripted defeat
clocks have a known flag and countdown; conditional deadlines remain unknown.

The dense, tree and spatial native loaders validate schema names, shapes and
finite parameters. Explicit legacy upgrades append zero weights to preserve old
scores. Preserve old model files and write new versions separately.

## Entry points

Each trainer's `--help` is the authority for its options. Run these from the game
root and place mutable runs under ignored `tmp/rl/`.

| Module under `hex_command_rl` | Purpose |
| --- | --- |
| `train` | Dense linear/MLP imitation over disk-backed teacher decisions |
| `trees` | Time-bounded LightGBM candidate ranker; install the trees extra |
| `spatial_train` | CNN imitation and compatible policy fine-tuning |
| `evaluate`, `compare` | Native evaluation of saved policies |
| `benchmark` | Frozen-position planner timing, including existing orchestration |
| `mixed_workload` | Isolated authored catalog and disjoint hierarchical seed panels |
| `mixed_imitation` | Weighted imitation over losslessly compressed shards |
| `mixed_ppo` | Independent native slots and decision-budgeted multi-task PPO |
| `mixed_evaluate`, `handoff` | Greedy validation and robust checkpoint selection |
| `overnight` | Restartable imitation → PPO → held-out evaluation supervisor |
| `remote_backup` | Local recovery export with explicit optional Git publication |

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

PPO optionally collects actions with `mixed_ppo --rollout-backend native`.
The default remains `torch`. Native rollout inference loads a persistent evaluator
once per rollout and returns candidate scores plus the pooled board embedding.
The existing Torch critic consumes that embedding; action sampling, log
probabilities, all gradient updates and frozen-reference scoring remain in Torch.
Weights refresh between rollouts, after optimization. This mode changes reduction
rounding slightly and does not promise bit-identical training trajectories.
Collection and optimization durations are recorded in update metrics; overall
throughput also includes weight export, checkpointing and other orchestration.

Mixed sampling assigns equal decision budgets to arenas and campaigns, then
balances maps/campaigns and divides their objective/route variants. Recruiters and
fixed opponents are explicit workload axes. Seed partitions separate training,
validation and final testing. Fixed-opponent comparisons measure that opponent
set; authored rosters, diplomacy and objective routes remain part of the case.

Imitation learns a teacher's candidate selection. PPO uses fresh stochastic
rollouts, a critic, GAE, clipping, entropy regularization and a frozen-reference
KL anchor. Campaign completion/wins give +1, defeat gives -1, campaign caps give
-1 and arena caps give 0. The native step API itself returns 0 for truncation;
the campaign PPO trainers apply the campaign-failure rule.

The mixed handoff and final RL selection evaluate larger validation panels.
Candidates within 2.5 percentage points of the best minimum arena/campaign ×
opponent score compete on overall score and caps. Positive minimum scores take
precedence over a zero stratum. The starting policy remains eligible. Final test
seeds never select checkpoints. Other trainer entry points document their own
selection metric; teacher agreement alone does not establish game strength.

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
