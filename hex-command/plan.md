# Learned policy training plan

Updated 2026-11-24.

## Goal

Improve the policy's ability to advance, fight, defend objectives, and complete
campaigns. Measure improvements through the model's own decisions on matched
evaluation cases.

## Current wiring and next decision

- **Active 2026-11-26:** another 12 hours, full-state resume from final hour12
  (not selected hour11). Start 43,200.194862209 → target 86,400.194862209 trainer
  seconds. Same frozen settings, curriculum and original hour10 KL reference.
  Hourly evaluations13–24; all earlier heldout candidates remain eligible.
  See `training/experiments/2026-11-24-ppo-hour10-12h/extension-twelve-hours-20261126`.

- Selected baseline: the unmodified final **hour-10 PPO checkpoint** at
  `training/experiments/2026-11-23-ppo-5m-9h/extension-one-hour/recovery/hour-10`.
  Do not initialize from the 1/3/5-minute imitation models. The 12-hour PPO warm-start
  run is complete: `tmp/rl/ppo-hour10-continuation-12h-20261124`.
  New-workload launch uses fresh critic/optimizers/sampler and hour-10 reference;
  recovery restores full state. See `training/experiments/2026-11-24-ppo-hour10-12h`.

- Imitation eligibility is now explicit via `pretrain_enabled`:15 conditions
  across eight targeted maps. Prepared imitation splits and collection enforce
  this allowlist; PPO remains complete. Mastered and unqualified cases stay
  disabled for imitation. PPO allocation remains unchanged.

- Implemented:25% ordinary arena /25% targeted /50% broad-campaign decisions,
  adaptive inside each group, authored mastery/base-weight controls, explicit
  single-stage+1 versus chapter+1/finale+2 rewards, eight new validated challenge
  conditions and a quarter-weight Marlow control sharing its map budget.
- The experimental relief/recovery/refuge planners have been removed. Crown
  Road, Briar Saddle and Lower Gallery use existing Full selectors and have
  pretraining disabled. Historical experiment artifacts remain archived.
- Teacher blocks and ordered recurrent imitation are wired. The 12-hour PPO
  continuation completed with no imitation; held-out-only selection chose its
  hour-11 candidate (28.41% versus 23.30% at start), not the final hour 12
  (24.62%). No candidate has been promoted or deployed. The
  eligible-only five-minute trial from hour 10 is complete: targeted wins improved
  24→78/128 while ordinary retention fell 14→5/28 and Lantern 4→0/24.
  Checkpoints 1/3/5 are saved; no policy was promoted or deployed. See
  `training/experiments/2026-11-24-targeted-imitation-5m`.
- Kill/objective-progress rewards and exploration experiments remain outstanding
  work; current vanilla workload keeps those bonuses off. Do not treat earlier
  proposals or historical experiment protocols below as current configuration.
- Current implementation/evidence is summarized in the final curriculum-wiring
  section and `training/experiments/2026-11-23-curriculum-wiring`.

## RL environment priorities

Better RL environments are needed regardless of whether we use targeted
imitation pretraining. Design situations around concrete decision failures and
the outcomes we want the agent to learn. The curriculum now contains28 targeted
logical maps (24 existing plus four new multi-objective challenges),14 ordinary
arenas and three broad training campaigns; two other broad campaigns remain
held out. Implementation and experiment records follow below.
Environment validation is not evidence that training has improved the policy.

Prioritize finishing objectives, defending and contesting points, advancing
without stalling, and engaging ranged threats appropriately. Include ordinary
combat alongside focused situations so specialization does not replace general
competence.

| Failure to address | Training situation and desired behavior | Counterpart situation | Measurements |
|---|---|---|---|
| Waiting indefinitely | Reach a feasible objective through manageable resistance before a meaningful deadline. | Holding position or waiting for support is the successful choice. | Completion, deadline failures, time to objective, actionable turns without progress. |
| Moving back and forth | Navigate a fork or detour that requires sustained progress along a viable route. | A changed threat makes reversing direction or retreating correct. | Repeated return moves, route progress, arrival time, losses and completion. |
| Failing to finish after an escort | In a mini-campaign, get captives safe, then take the commander to a separate exit. Learn the objective handoff. | The captives still require protection, so leaving them prematurely fails. | Escort completion, commander exit after escort completion, time between them, protected-unit deaths. |
| Abandoning captured ground | Hold a captured point against an active counterattack while other units operate elsewhere. | Giving up the point enables a more valuable objective or avoids an unwinnable defense. | Hold progress retained or reset, point losses, counterattack survival, overall outcome. |
| Ignoring an opponent's impending victory | Contest an enemy-controlled point before its public hold clock expires. | An unnecessary attack sacrifices a winning alternative or a secure defense. | Timely contesting, opponent clock resets, territory defeats, overall outcome. |
| Attacking piecemeal | Coordinate several units against a defended choke rather than feeding them in separately. | A brief opening makes immediate action better than waiting for everyone. | Arrival and attack timing, unsupported losses, breakthrough and completion. |
| Failing to advance after winning a fight | Clear an obstacle, then capture or exit before reinforcements arrive. | Regrouping or healing is necessary before the next engagement. | Delay after clearance, objective progress, losses to reinforcements, completion. |
| Refusing favorable melee against a stronger ranged attacker | Face an enemy with superior ranged damage but a favorable melee engagement. Cover both declining an already legal melee attack and failing to close distance when enduring repeated shots is worse. | A melee screen, strong retaliation, or exposure to supporting enemies makes charging suicidal; waiting, flanking or retreating can be correct. | Favorable legal melee opportunities declined, time to engagement, unanswered ranged attacks and damage, casualties and overall outcome. |

The ranged-enemy hesitation is a specific user-reported failure, potentially
inherited from rule-based teachers. Check the relevant rule-based and learned
policies on matching cases rather than assuming that every teacher or matchup
has the same defect. Targeted imitation must not simply reproduce that refusal.

Waiting, retreating, abandoning a point and declining an attack are not failures
by themselves. Every focused task needs cases where those actions are correct.
Do not use forced movement, unconditional charging, or movement frequency alone
as the learning goal. Success is choosing appropriately for the objective,
survival prospects and opposing force.

Environment design and validation:

- Use meaningful internal terrain, routes and obstacles, including caves,
  coastal terrain and chokepoints; jagged borders around an empty interior do
  not provide the intended diversity. Mix compact tactical arenas with short
  campaign chains and broader ordinary battles.
- Use supported arena objectives for standalone tasks. Escort and subsequent
  objective handoffs belong in mini-campaigns, with actual chapter transitions
  and carryover where relevant.
- Defenders start near what they defend. Holding tasks need active enemy
  pressure and enough enemy force, economy or reinforcement access to make
  losing possible. Avoid trivial unopposed holds and artificial paralysis
  through orders.
- Establish viable routes and meaningful deadlines with actual runs. Inspect
  whether failure comes from combat, routing, waiting or objective handoff;
  changing a deadline alone does not diagnose the failure.
- Vary layouts, faction matchups, force sizes, starting sides and initiative.
  Include clear, player-only fog and sight variants where appropriate, with
  returning-fog cave situations. Record which side receives each visibility
  restriction. Both-sided restrictions are a separate, explicitly selected case.
- Compare current learners and relevant rule-based bots on fixed matching
  cases. Validate that the intended opportunity and pressure actually occur.
  Retain separate held-out layouts and ordinary-play evaluations, including
  held-out Lantern Road; easier validation is not evidence of stronger play.

## Broad campaigns: authored version 2

Implemented all 15 chapter objective trees selected from the objective-generator
batch (seed 20261121). Existing terrain layouts remain; this is an objective,
encounter and carryover revision, not a new map generator. The special task is an
escort. Whitebank uses meeting a neutral courier, then escorting them, with
destroying the occupying army as an independent alternative.

| Campaign | Chapter 1 | Chapter 2 | Chapter 3 |
|---|---|---|---|
| Woodland Defense — training | Defeat a named guard → rescue captives → take a surrendered scout into custody | Rescue both prisoner groups | Destroy the occupying army |
| Underground Escape — training | Reach the far shore **or** destroy the garrison | Escort the charge → protect them at the refuge for six consecutive turns, under a counterattack | Destroy the pursuing army |
| Splitwake Crossing — training | Protect the tidekeeper at the keep for nineteen consecutive turns **or** capture the beachhead → destroy its defenders | Bring three named company members to the escape island **or** destroy the flotilla | Destroy the citadel army |
| Coastal Assault — held out | Destroy both actively recruiting hostile companies; two supported player landings | Escort the harbor pilot **or** destroy the harbor army | Hold the evacuation keep for twenty-one consecutive turns while keeping the tidekeeper alive |
| Whitebank Relief — held out | Meet → escort the courier **or** destroy the occupying army | Destroy the mountain army | Destroy the sanctuary army; an optional rescue supplies an ally |

Chapter loss occurs at the start of the listed deadline turn (completion must
precede it): Woodland 58/54/64; Underground 48/64/72; Splitwake 42/66/64;
Coastal 76/58/42; Whitebank 52/62/72. Carryover allows up to six veterans and
twenty gold, deployed within three tiles. Optional supplies award eight gold
(ten on Coastal's final map). Whitebank's courier supplies ten gold next chapter;
destroying Underground's opening garrison prevents a two-unit pursuit in chapter 2.

Enemy forces remain active, with recruiting keeps, income and selected timed or
objective-triggered reinforcements. Neutral prisoners/couriers are passive only
until contact. Whitebank's courier is an armed scout, rather than slow captives:
the first calibration lost the captives on the bridge before they could finish.
The initial Coastal opening had one landing against two recruiting economies;
a second three-unit landing and two contested supply villages now support the
southern front. The final defense starts six defenders near their keep and owns
the nearby supply village. Any player unit can garrison the keep while the
tidekeeper survives; moving the tidekeeper to heal does not itself reset the
hold. These are content changes, not measured learner improvements.

Escort deaths close that route without defeating a still-viable fight route.
Objective prerequisites, remembered arrivals and exact actor survival are authored
explicitly. Protection at a keep uses real consecutive hold conditions, so the
spatial countdown and victory rule describe the same task. All five fog routes
restrict **the player only**; enemy and neutral vision groups penetrate fog.
Coastal and Whitebank remain outside training, and Lantern Road is unchanged.

Keep the old run's archived catalog and results intact. Evaluate this as a new
benchmark version; its completion rates cannot establish improvements over the
old campaigns. Calibration records, baseline catalogs, full-campaign simulations
and chapter-entry probes are under `tmp/rl/broad-campaigns-v2-20261122/`.

Validation: 20 seeds (92001–92020) per campaign/visibility/tactic, Classic enemies,
normal 100-turn simulation cap and policy recruitment limits. Classic and Queued
with Predictor recruitment and Mission Campaigner were tested in clear and
player-only fog. Full-campaign completions on the final content:

| Campaign | Classic clear | Classic fog | Queued clear | Queued fog |
|---|---|---|---|---|
| Woodland | 11/20 | 1/20 | 3/20 | 0/20 |
| Underground | 0/20 | 0/20 | 4/20 | 2/20 |
| Splitwake | 4/20 | 1/20 | 9/20 | 12/20 |
| Coastal | 2/20 | 8/20 | 2/20 | 3/20 |
| Whitebank | 0/20 | 3/20 | 6/20 | 4/20 |

Every chapter also had a successful chapter-entry run across the tested tactics
and visibility settings. Those probes start with the authored roster, without
veterans or carried gold; they do not estimate the later chapter's difficulty
after a successful campaign opening. Coastal's opening remains difficult;
Underground and Whitebank still expose substantial Classic failures. The courier
meeting/escort opening succeeded in all 20 seeds for both tactics under each
visibility setting. Raw outcomes confirm completed 6-, 19- and 21-turn holds at
their full counters. No learned checkpoint strength claim is made.

The existing report analyzer encountered a `NoneType` counter error after the
matches completed. Results above were read from completed raw reports/outcomes;
HTML analytics are unavailable. `validation-summary.json` records the exact
source panels and limitations. The production campaign JSON matches the frozen
release catalog used for these runs; complete-catalog validation and JSON
formatting passed.

## Targeted environment 1: first draft findings

The 12×9 After the Breakthrough draft is a fixed four-survivor force against one
wounded spearman, with capture required before turn 9, clear visibility and no
recruitment. Tested 20 matched seeds per policy against Classic: Classic, Full,
Queued, V2, V3, the older six-hour policy, the 43-minute pretrain, and PPO hours
3/6/7/9. All 220 attempts captured the village. Classic took 3–5 turns, Full 3,
Queued 2, and every learned checkpoint 2. There were no player casualties or
player turns without movement/attacks. The defender moves off the village to
attack; all learned checkpoints capture while it remains alive.

This draft does not reproduce stalling. It is a basic success case, not evidence
that the known failure is fixed or a useful discriminating recovery task yet.
Its fresh-start state also lacks the preceding fight/history. A follow-up should
use actual failed post-combat positions or a played-through combat-to-advance
handoff and verify the failure before choosing further training variants. The
draft and executable experiment remain outside the playable/training catalogs in
`tmp/rl/targeted-map-1-draft/`; report, matched schedules, checkpoint hashes and
complete native phase traces are retained there.

## Solved targeted prototypes — withdrawn from training

Two one-chapter training campaigns were extracted from actual failed advances
on the original Underground Escape opening, then reduced to 16×20 boards:

- `rl-vault-aftermath`: nine surviving player units face the last necromancer.
  Source: older six-hour learner, seed 16010701, turn 23.
- `rl-vault-breakthrough`: a force stalled at the defended crossing.
  Source: PPO hour 9, seed 16010718, turn 17.

Both require any player unit to reach (10,2) before turn 25. Visibility is clear
for both sides. Starting health, promotions, allegiance, terrain and ownership
come from the source position; turn/action history resets for a new episode.
Enemies remain active. No movement overrides or frozen-defender orders are used.
They remain registered as simulation regression fixtures, but were removed from
`simulation/rl.json` because the hour-9 learner already solves both 20/20. No new
training run was launched. Existing archived run catalogs remain unchanged.

Validation used 20 matched seeds (94001–94020), fixed Predictor enemies, normal
100-round simulation caps and policy recruitment limits. Eleven player policies
were tested, 440 games total. Identity-only registration passed another 16
matched smoke runs, and complete-catalog validation passed.

| Player | Aftermath completions | Breakthrough completions |
|---|---|---|
| Classic | 20/20 | 3/20 |
| Full | 20/20 | 0/20 |
| Queued | 20/20 | 0/20 |
| V2 | 0/20 | 0/20 |
| V3 | 0/20 | 0/20 |
| Older six-hour | 0/20 | 0/20 |
| 43-minute pretrain | 20/20 | 18/20 |
| PPO hour 3 | 20/20 | 20/20 |
| PPO hour 6 | 20/20 | 19/20 |
| PPO hour 7 | 20/20 | 19/20 |
| PPO hour 9 | 20/20 | 20/20 |

Aftermath reproduces 18 consecutive idle turns in V2/the older six-hour model
and 20 in V3; successful policies take 8–9 turns. In Breakthrough, Full/Queued
can wait for 18 consecutive turns, while hour 9 has successful routes. These are
curriculum/retention cases, not an unresolved weakness demonstrated in the best
hour-9 checkpoint: cropping removed its original failure. The untouched source
runs and each reduction are retained in `tmp/rl/objective-stall-env/` along with
complete phase traces, schedules, summaries and previews. Do not claim that
historical success rates or frozen-state replays equal these fresh-start tasks.

Discovery also exposed a separate Woodland handoff problem: a completed rescue
still produced a critical movement duty. It was excluded from the clean movement
tasks. Review the authored live/permanent objective semantics before treating
that particular failure as something to solve through training.

## Acceptance rule for new targeted training environments

The target learner is the exact final checkpoint of the nine-hour PPO run, not
V2, V3 or the older six-hour learner. Before adding a targeted environment to the
training mix, demonstrate a substantial paired performance gap between that
checkpoint and an existing rule-based or explicitly described targeted bot.
Both must play the same final map, objective, forces, enemy policy, visibility,
recruitment budget and deadline. Validate a candidate on at least 20 matched
seeds, then confirm it on fresh seeds when a teaching policy was selected using
the first set. Retain the successful traces proving feasibility.

Any simplification, crop, force change or objective change invalidates the prior
gap: rerun the exact target learner and the oracle on the resulting environment.
A source failure, a weaker checkpoint's failure, or a strong oracle alone does
not qualify. Environments already solved reliably by hour 9 belong in retention
or regression checks, not in the targeted cure set. Keep investigations in
ignored experiments until this condition is met.

## Qualified targeted training tasks — final hour9 benchmark

The solved Vault Aftermath and Vault Breakthrough prototypes are excluded from
training. Three replacements now satisfy the strict hour9-versus-oracle rule.
Every comparison uses unchanged conditions within each pair, explicit disclosed
objectives, and a second independent20-seed cohort. No new training run started.

| Task | Exact hour9 | Reference bot | Reference completions |
|---|---|---|---|
| `rl-vault-rally` | 7/40 | Classic planner + Situational recruiter | 30/40 |
| `rl-urgent-crossing` | 9/40 | Offline targeted legal-action bot | 40/40 |
| `rl-refuge-counterattack` | 2/40 | Full planner + Predictor recruiter | 20/40 |

- Rally preserves the complete24×29 source encounter and economy, with a generous
 60-turn limit. Fixed Predictor enemy; identical Situational recruiters on both
 player planners. Source9 surviving troops must advance to (14,10); commander
 survival matters. Hour9 has23 commander losses and10 deadline failures.
 Discovery95001–95020: Classic14/20 versus4/20; confirmation97001–97020:
 16/20 versus3/20. This qualifies through real rule-based performance.
- Urgent crossing is10×10 with7 swordsmen plus commander versus3 active casters,
 clear visibility, no recruitment and a turn3 completion requirement. Fixed
 Classic enemy. Oracle20/20 versus hour9 6/20 on97301–97320; oracle20/20
 versus3/20 on97401–97420. The oracle prioritizes positive exchanges and
 engine-computed goal-path progress with nonleaders. This is tempo/objective
 priority, not prolonged stalling. The hand-designed oracle is a benchmark only;
 it does not alter the learner or authorize imitation pretraining.
- Refuge starts after arrival in the failed hour9 seed96022 chapter2 encounter.
 Hold the charge at (15,25) for6 consecutive turns under active pressure, before
 turn50. Source actors, health, terrain, ownership and gold are preserved; history
 and the already-zero hold clock reset. The arrival flag is restored and already
 spawned waves are not spawned again. Fixed Classic enemy and matching Predictor
 recruitment. Full7/20 versus2/20 on97001–97020, then13/20 versus0/20 on
 97101–97120. All20 Full witnesses end with the charge alive and6/6 hold progress;
 hour9 loses the charge38/40 times. Full50% means this remains a hard encounter.

All three are clear-only single-chapter campaigns registered in the training
catalog. They use existing campaign finale scoring; mix weights/reward per unit
of training time must be reviewed before the next long run. Held-out Lantern
Road supplied none of these tasks. Single extracted layouts plus RNG variation
are not a claim of generality; further layout diversity should retain the gap.
Full provenance is in each campaign's design.md and the ignored experiment
folders `objective-stall-env`, `hour9-hold-explore`, `hour9-combat-scout`.

## Refuge opponent follow-up

Matched seeds97101–97120 on registered `rl-refuge-counterattack`, clear for both
sides, unchanged forces/objectives/budgets: against Predictor (Predictor recruiter,
Mission Campaigner, Queued planner), Full10/20 and exact hour9 19/20. Against
Classic on those same seeds, Full13/20 and hour9 0/20. All successful Predictor
runs were verified charge-alive with6/6 hold progress. In seed97101, Classic
gets a promoted melee attacker onto the charge for19 damage on turn3 and kills
it on turn4. Predictor applies ranged pressure; healing permits the charge to
finish the hold on turn7. This example is not a claim of general opponent
strength. The targeted weakness is demonstrated against Classic; the Predictor
variant is mostly solved and should not be counted as a new hour9 failure.
Raw paired evidence is retained under `tmp/rl/refuge-predictor-check/`.

## All three tasks against Predictor

Clear visibility,20 matched seeds per map, unchanged registered content. Enemy
Predictor uses Predictor recruitment, Mission Campaigner and Queued planning.
Rally cohort97001–97020: Full11/20, hour9 3/20, Classic planner16/20.
Urgent cohort97401–97420: Full17/20, hour9 19/20, targeted oracle16/20.
Refuge cohort97101–97120: Full10/20, hour9 19/20.
Player recruitment remains matched within each map (Situational on Rally,
Predictor on Urgent/Refuge; recruitment disabled for both sides on Urgent).

Rally still exposes an hour9 weakness against Predictor. Urgent and Refuge are
mostly solved against that enemy; their demonstrated training deficits are
against Classic. Do not pool opponent variants into a single hardness claim.
Before the next training launch, scope targeted sampling to qualifying opponents:
the current workload builder expands every campaign across global opponents,
so merely registering a map does not enforce this scope. No such sampling
change was made during this comparison-only follow-up.
Evidence: `tmp/rl/all-targeted-predictor/` plus the earlier matching panels.

## All three tasks against Full

20 matched seeds per map, clear for both sides; enemy Full uses Predictor
recruitment, Mission Campaigner and Full planning. Forces, objectives, budgets
and seed cohorts match the Predictor follow-up. Results:

| Task | Full player | Exact hour9 player | Additional reference |
|---|---|---|---|
| Vault Rally | 12/20 | 7/20 | Classic planner12/20 |
| Urgent Crossing | 17/20 | 5/20 | Targeted bot11/20 |
| Refuge Counterattack | 10/20 | 19/20 | — |

Rally cohort97001–97020; Urgent97401–97420; Refuge97101–97120.
Urgent remains a demonstrated learner weakness against Full, with Full itself
providing17 successful witnesses. Refuge is mostly solved against Full as well
as Predictor; its clear failure is against Classic. Rally's strongest measured
gap remains against Predictor. Targeted training should select the qualifying
map/opponent combinations rather than assigning difficulty to the map alone.
All160 requested/supplementary comparisons completed; victories passed explicit
objective/required-actor checks. Raw evidence: `tmp/rl/all-targeted-full/`.

## Equal-weight opponent/map aggregate

Using20 matched confirmation-cohort seeds per map/opponent/player cell, give
Classic, Predictor and Full enemies one-third weight each. Full-player versus
exact hour9 completion rates are:

| Task | Full | Hour9 |
|---|---|---|
| Vault Rally | 23/60 =38.3% | 12/60 =20.0% |
| Urgent Crossing | 43/60 =71.7% | 27/60 =45.0% |
| Refuge Counterattack | 33/60 =55.0% | 38/60 =63.3% |
| Equal map/opponent weights | 99/180 =55.0% | 77/180 =42.8% |

Previously missing Classic-enemy cells were freshly measured on the same
cohorts: Rally Full0/20 and hour9 2/20; Urgent Full9/20 and hour9 3/20.
Refuge uses matched Classic13/20 versus0/20, rather than mixing the earlier
40-seed Classic panel with20-seed panels for other opponents. This is a scoped
three-task estimate, not general gameplay strength. Equal-opponent weighting
makes Refuge favor the learner overall, while its Classic-specific protection
defect remains. No training-sampling weights were changed by this calculation.
Evidence and exact cell cohorts: `tmp/rl/targeted-balanced-score/summary.json`.

## Classic-only environment investigation

The native match config already supports `enemy:classic` per episode. The native
bridge preserves the requested opponent while replacing only the player's planner
with Learned. PPO's sampler retains each template's opponent unchanged; it does
not switch opponents during the match or re-expand the global opponent list.

The authored metadata currently lacks per-environment opponent selection:
`TrainingCampaignVariant` accepts only objective/diplomacy, and configs() forms
the Cartesian product with the global opponent list. A proposed per-variant
`opponents:[classic]` field therefore requires schema/validation and builder
changes before it can be placed in simulation/rl.json.

A disposable probe filtered Refuge to Classic-only without changing production:
144 generated train configs /6 opponent-recruiter groups became48 configs /2
recruiter groups. Actual PPO weights preserved the task's full1/12=8.33% total
training-decision budget, concentrating it on Classic. Other task configs were
unchanged. Constructing the actual Sampler confirmed exactly2 Classic-only
Refuge templates; no Predictor or Full could be selected for that task.
Proof: `tmp/rl/classic-only-investigation/scheduling-proof.json`.

For a permanent implementation, carry the per-task opponent set through training,
validation, periodic evaluation and saved workloads. PPO weights() already counts
actual groups correctly. The imitation-only mass() helper still assumes global
opponent counts and must be made consistent as well. Keep restrictions in the
workload definition; roster AI policies alone do not constrain simulation configs.
No production scheduling behavior was changed during this investigation.

Proposed targeted scopes supported by the measured gaps: Refuge Classic-only;
Urgent Classic+Full; Rally Predictor-only. Keep broader tasks mixed, and retain
other opponent variants as separate generalization checks rather than averaging
away the targeted behavior.

## Training decisions after environment improvements

- Leave exploration settings unchanged for now. PPO already samples actions;
  temperature and entropy-bonus experiments are deferred. If revisited, vary
  them separately under matched models, rewards and budgets, and judge wins,
  objectives and stalls rather than merely reducing End Turn frequency.
- Targeted pretraining is not a prerequisite. First run a short measured PPO
  continuation with the implemented architecture and inspect the failures above.
  We do not yet know whether targeted demonstrations are needed.
- If the agent rarely discovers successful behavior, consider demonstrations
  of those situations from a teacher or validated rollout that actually succeeds.
  Mix them with ordinary combat. If successful behavior appears but is not
  retained, investigate reward credit and policy updates before prescribing
  more imitation.
- Teacher selection remains unresolved. Rule-based hesitation must not become
  the target behavior. Earlier successful intervention add-ons demonstrate
  recoverable errors, not proof that the current model needs the same remedies.
- Recurrent mixed imitation now preserves ordered teacher streams and explicit
  match/faction/chapter boundaries. It carries detached memory across windows
  and trains recurrence within them, like the implemented sequence PPO path.
  Legacy standalone imitation tools still use independent examples.

## Agreed direction

- Idle history is implemented directly in the existing candidate-scoring head. Its
  inputs are consecutive completed turns with no tactical action and whether the
  planner has acted during the current turn. The head changes from
  307 → 64 → 64 → 1 to 309 → 64 → 64 → 1. Per-unit history adds nine
  more inputs, bringing the single-branch head to 318 → 64 → 64 → 1.
  The coarse branch adds 64 encoded inputs (382); GRU memory adds another 64,
  making the current head 446 → 64 → 64 → 1.
- New input weights initialize to zero to preserve the starting checkpoint's
  scores. Idle turns are divided by ten. Per-faction history updates at turn
  boundaries and resets at match/chapter boundaries in training and native inference.
- Per-unit history is implemented in the same head: previous action type, known
  movement/attack history and turns since each event (divided by ten), plus
  whether a candidate move returns to its last move origin. Each actor and
  observer faction has independent history. End Turn gets zero actor history.
  The CNN is unchanged; new head columns initialize to zero.
- Let training learn how history affects waiting, advancing, attacking, and
  retreating. The preferred design feeds history into the head without a
  hard-coded two-turn intervention threshold or a separate recovery scorer.
- Keep campaign rewards at +1 per intermediate chapter and +2 for the finale.
  A three-chapter victory therefore earns +4 before discounting. Arena victory
  earns +1.

- Arena territory objectives now pass the acting faction's authoritative hold
  progress through Campaigner into the shared duty features, in training and
  inference. Countdown probes passed for both initiative orders and multi-point
  holds; deathmatch retains zero hold features. Opponent arena counters now
  travel separately through Campaigner into their own spatial marker/countdown
  planes. Both clocks are public objective scores under clear, fog and sight
  rules. Shared tiles carry both markers with independent countdowns, relative
  to the acting faction. Opponent progress does not overwrite our head features.
- Hold locations now also carry spatial marker/countdown planes, so separate
  holds retain separate timers. Multi-tile holds share their objective's counter;
  overlapping holds expose the smallest remaining count. Old CNN checkpoints
  expand with zero weights for the appended channels.

- The chosen raw-input coarse branch is now integrated into Torch training and
  native inference, alongside the existing detailed path. Both receive all 83
  channels; the coarse path starts with stride two. The model gathers 64 encoded
  features per branch and has 50,977 policy parameters. Old checkpoint scores are
  preserved with zero weights on new head inputs. Padding-invariant 2×2 masks and
  aligned hex stencils support odd shapes and jagged edges. The subsequent GRU
  integration adds 64 memory inputs and brings the policy to 70,817 parameters.
  A short PPO smoke verified trainable coarse weights and post-update native/Torch
  score parity; this does not establish improved strength.

## Proposed experiments

- Test a small intermediate reward for enemy kills, potentially weighted by unit
  value. Choose its magnitude explicitly and compare with a zero-kill-reward
  baseline. Measure campaign objectives as well as combat outcomes, because
  chasing kills can conflict with evacuation, escorts, and territory.
- Compare the current model with idle-history inputs under matched PPO budgets.
  Exploration remains an open issue when the starting policy overwhelmingly
  prefers End Turn. Exploration changes are deferred; if revisited, test them
  separately from observation and reward changes.
- A new mixed imitation pretrain remains a decision under discussion. Teachers
  are explicitly configured per validated condition; keep their full block
  compositions and the same25/25/50 base decision mixture. Do not start it merely
  because collection/training wiring exists.
- If imitation is evaluated, update the CNN's own weights, mix ordinary play
  with the targeted examples, evaluate the resulting policy, and test whether a
  subsequent PPO continuation retains the gains.
- GRU memory is now implemented in Torch and native inference. It reads the
  detailed global pool (16 values), keeps 64 learned values per faction, and
  resets at match/chapter boundaries. Both PPO trainers replay ordered windows
  of up to eight decisions with connected hidden states, and cache reference
  logits. Mixed imitation now also trains ordered teacher streams; old corpora
  without explicit sequence contexts must be recollected.
  A short mixed arena/campaign PPO smoke completed 34 updates and 544 decisions;
  input and recurrent weights both changed. Native hidden-state/logit parity and
  reset/isolation/repeated-scoring checks passed. Strength gains and complete
  recurrent PPO runtime remain unmeasured; earlier coarse-cost projections
  predate recurrence and sequence replay.

- The original raw-input coarse branch cost prototype is retained in
  `tmp/rl/coarse-branch-cost/`. It kept the two full-resolution hex convolutions,
  added stride-2 83→16 and coarse 16→16 convolutions, then concatenated 64 coarse
  features into the candidate head. On captured arena/campaign positions with
  actor contexts, one-thread PyTorch timings increased mixed forward/backward
  by 33.0% and mixed inference by 35.6%. Single-position inference rose 48.3%.
  Parameters increased from 35,761 to 50,977. These are prototype measurements;
  the raw-input design is now integrated, while strength remains untested. A historical timing-share
  projection suggests about 30% longer PPO cycles; end-to-end cost is unmeasured.

- A shared-feature coarse variant was also benchmarked, branching from the
  existing final 16-channel map instead of raw input. On the same observations,
  mixed forward/backward rose 25.7% (raw-input control 31.5%), mixed inference
  18.1%, and single-position inference 28.5%. Parameters total 43,473.
  The historical PPO-cycle projection is about +20%, still unmeasured end-to-end.
  This remains a prototype; the learned 16-channel compression is optimized for
  current tasks and is not guaranteed to retain all useful coarse-scale detail.
  Results: `tmp/rl/coarse-branch-cost/shared-16/results.json`.

## Training efficiency

- Frozen reference logits are now cached during rollouts and reused across PPO
  optimization epochs. Measure full training speed and memory use with the
  integrated architecture and sequence replay.
- A short mixed-workload profile attributed about 26% of training time to update
  forward passes, 24% to reference forward passes, 25% to backpropagation, 9% to
  rollout inference, 8% to environment stepping/results, and 8% to other work.
- Convolutions consumed about 75% of sampled network forward time and 74% of
  sampled backward CPU time. A 64-unit GRU on the pooled summary was cheap in a
  forward-only timing probe. Recurrent training cost remains unmeasured.

## Evaluation

- Use fixed matched maps, seeds, opponents, initiative, recruiters, and round
  limits for each baseline and candidate comparison.
- Cover several layout seeds and both initiative orders. Previous expanded
  arena validation used only one RNG seed per map despite its many configurations.
- Keep Lantern Road held out from training. Report clear and player-only fog
  separately wherever those variants exist and state enemy visibility explicitly.
- Track wins, defeats, caps, chapter entrants and clears, deadline failures,
  required-actor deaths, and previously won cases lost by a candidate.
- Chapter clear rates use runs that entered that chapter as the denominator.
  Whole-campaign results retain all campaign starts. Carryover changes which
  armies reach later chapters, so use identical entry states when isolating
  chapter behavior.
- Map edits create a separate benchmark version with matching reruns of every
  baseline. Improvements from easier map setup cannot establish policy gains.

## Implemented in the workspace

- Campaign reward plumbing now credits intermediate clears immediately and pays
  +2 on a successful finale. A live replay verified +1, +1, +2. Earlier chapter
  rewards survive later failure, and episode logs include cumulative returns.
- Arena IDs now match their names. `villages` became `toll-road` for The Toll Road
  arena. `sluice` became `a-tide-for-both` for A Tide for Both. Their catalog and
  workload references were updated.
- These changes have passed verification. They are workspace edits, with commit
  status to be checked before starting the next run.

## What the experiments established

- The experimental recovery add-on froze the CNN and fitted a small linear score
  adjustment using hand-designed recovery demonstrations. Its results belong to
  that combined system. Training the existing CNN with idle-history inputs has
  yet to be tested.
- On 192 fresh Islands deathmatches, the idle-conditioned add-on changed wins
  from 94 to 141 and caps from 82 to 19. Across 176 additional deathmatches on 11
  maps, wins changed from 95 to 111 and caps from 20 to 2. Those tests identify
  recoverable failures and a useful history signal.
- On The Toll Road campaign chapter, the hour-6 deadline failures usually
  occurred after the captives were safe. Nera then missed the exit. A scorer
  trained on synthetic handoffs improved the hour-9 policy from 7/23 to 22/23 on
  identical post-escort positions. It improved fresh full Lantern Road clears
  from 17/60 to 21/60, with mixed results across opponents.
- A blanket move-toward-objective override harmed escort campaigns. A model must
  learn when advancing is safe and how to coordinate protection.
- In The Toll Road arena, an experimental learned garrison scorer improved
  36/144 to 144/144 using ownership and departure features. Adding a countdown
  produced the same result there. Broader hold-clock benefits remain unresolved.
- A Tide for Both starts a spearman on a neutral scoring village that captures
  on entry. Disposable map edits removed caps and also changed difficulty.
  Production terrain and starting positions retain their existing setup.
- Chapter rewards were tested with two short matched PPO pairs. At 160 updates,
  Lantern Road clears were 14/60 with terminal-only rewards and 19/60 with chapter
  rewards. At 80 updates on a second training seed, they were 14/60 and 11/60.
  Other campaign results were mixed or lower. Long-run effects remain unresolved.

## Evidence locations

- Original run and archived catalog are in
  `tmp/rl/ppo-from-pretrain-43m-9h-20261121/`.
- Independent arena replays are in that directory's `astra20/results.json`.
- Paired policy and reward experiments are summarized in
  `one-hour-experiments/measured-summary.json`, with scripts and raw results nearby.
- Compute measurements are in `compute-profile/timing.json`,
  `compute-profile/inference-timing.json`, and
  `compute-profile/backward-timing.json`.

## Before the next long run

- Resolve which experiments to implement and which demonstration sources to use.
- Verify actual observed reward values at chapter transitions before launch.
- Verify history inputs and native/Python inference agree on identical states.
- Evaluate candidates on fresh cases and check retention during a short PPO
  continuation before committing to another long run.

## Eight-category expansion requested 2026-11-22

Target three distinct, qualified layouts/force compositions for each behavior:
1. Advance before deadline, including situations needing support first.
2. Commit to routes, including correct retreat after a newly revealed threat.
3. Complete escort then move commander to a separate exit, with a real task handoff.
4. Capture then defend against an active counterattack; retain useful secondary roles.
5. Interrupt an authoritative enemy hold, including alternative victory priorities.
6. Coordinate attacks through a choke, including short immediate-attack openings.
7. Exploit a breakthrough before reinforcements, including necessary healing first.
8. Close against ranged enemies with equal/better melee sidearms; include adjacent
   attacks, approaches, group attacks before enemy promotion, and suicidal-charge exceptions.

Nine Sol agents requested: eight searches and independent coverage/evidence audit.
Six scouts run initially because only six child slots are available concurrently;
remaining three are queued. All work is isolated under
`tmp/rl/targeted-eight-20261122/`; no new training launched. Evidence must use exact
nine-hour final model, matched conditions, qualified rule/targeted reference bots,
and independent confirmation after selection. Evaluate Classic/Predictor/Full
enemies in clear, player-only returning fog, and both-sides returning fog; verify
actual tile concealment and vision-group penetration in recorded states. Qualified
cases, counterexamples, retention cases, and rejected prototypes must remain
clearly distinguished. Final deliverable: readable tables, native map renders,
source provenance, failed decisions, feasibility evidence, and opponent/visibility
recommendations for each final environment. Do not quietly count easy solved
maps toward the targeted failure set.

### Expansion results and review bundle

All eight searches produced three primary-qualified scenarios each: 24 fixtures
on 23 terrain layouts (Lantern Maze and Lower Gallery reuse source geography
for different forces/objectives). Nine Sol agents handled eight searches and an
independent audit. Every scenario has an independent 20-seed primary comparison
against the exact nine-hour final policy. Full Classic/Predictor/Full enemy ×
clear/player-only fog/both-sides fog coverage is recorded; nonprimary cells in
route, interrupt and breakthrough groups use five seeds and are exploratory.
The remaining groups use twenty seeds per cell. Reports pin a declared reference
per scenario rather than choosing the strongest policy independently per cell.

Review `tmp/rl/targeted-eight-20261122/REPORT.md` for all 24 completion tables,
216 visibility/opponent cells, native map renders, decision evidence, source
provenance, controls and recommended scopes. `normalized-results.json` and
`audit/` retain machine-readable evidence. `bundle/` contains three runnable
catalogs and a per-case runner; the production training pool is unchanged.

The necessary-healing successor is Thorn Ford: Full 20/20 versus Hour9 0/20,
with a healer-only intervention dropping Full to 0/20. The same lance exchange
causes four thorn damage: healed courier enters at seven HP, unhealed at one.
The former survives and delivers; the latter dies. Lair Outworks separately
confirmed on fresh seeds at Full 17/20 versus Hour9 2/20. Earlier cave/quarry
controls did not prove necessary healing and are not presented as doing so.

Strict terrain/frame/floor validation covers every final authored fixture;
presentation metadata repairs are verified against complete native replays.
Native snapshots verify which sides actually receive fog. No new policy was
trained, no reward changed, and no live content was deployed by this expansion.

### RL and imitation wiring completed

`rust/data/simulation/rl.json` now registers all 24 targeted encounters and the
selected 66 opponent/visibility conditions, alongside every existing ordinary
training arena and broad campaign. Each training arena and campaign variant
explicitly declares `pretrain_imitator`: Classic, Queued, Full, the legal-action
`objective_progress` reference, or an authored `path_to` reference. The latter
two are pretraining-only controllers. PPO selects the learned planner and does
not consume imitation selectors. Unknown/missing teachers fail validation.

Authored decision/loss budgets are 25% ordinary arenas, 25% broad campaigns and
50% targeted encounters. The targeted half gives each of the eight behavior
families 6.25%, then divides equally among its three maps and selected
conditions/recruiters. Broad enemy/visibility pools are preserved. Target fog
routes use ordinary SetVisionGroup events and existing fog rules; no separate
copy of a battlefield or special gameplay rules is needed.

Both mixed and diverse workload preparers retain configured imitation teachers.
`imitation_validation` uses disjoint seeds on the training maps. Held-out gameplay
selection and showcase sets remain separate from the imitation corpus. The
native Python bridge honors each prepared catalog path. Teacher identities and
sampling weights participate in workload/corpus provenance checks.

Wiring evidence: `tmp/rl/wire-targeted-20261122/verification.json` records 172
native integration games, all 66 scoped conditions, both initial/final visibility
checks, learner control during RL, and 20/20 confirmations for each custom
imitator. Preparation plus real compressed data collection passed without
starting an optimizer. No pretraining or RL run was launched by this wiring task.

### Adaptive PPO sampling wired

`simulation/rl.json` explicitly enables `adaptiveSampling`: full base weight up
to 80% smoothed completion, linear decline afterward, and a 10% floor from 98%
upward. There is no separate fixed/dynamic mixture. The EMA rate is 0.1, prior
0.5, warmup ten outcomes per match template, and refresh every twenty completed
episodes. Templates distinguish map/objective/visibility, opponent, recruiter,
army pair and initiative; the seed does not distinguish a template.

`training/hex_command_rl/sampling.py` owns PPO allocation. Existing pool budgets
are preserved, and effective weights are normalized within pools. Changed pools
rebase their allocation debt, while unchanged pools retain their history. Live
episode reservations and variable episode lengths remain accounted for. Caps
are failures; errors and interrupted episodes do not supply outcomes.

Sampler checkpoint schema 2 retains estimates, observation counts, frozen
multipliers, refresh position, lifetime and allocation counters, and the next
seed. The containing PPO checkpoint retains RNG state. Metrics contain sampling
updates, and `sampler-status.json` exposes per-case mastery and decision shares.
Imitation, evaluation, checkpoint selection and rewards remain static.

Evidence: seven focused sampler tests, existing training/native contracts,
strict catalog setting validation, and a disposable PPO start/resume smoke under
`tmp/rl/adaptive-sampling-20261122/`. The smoke finished 31 episodes and 792
decisions over a 24-second cumulative budget. Its saved EMA exactly matches the
recorded actual outcomes; episode-20 refresh, recovery and latest diagnostics
were verified. It is an integration artifact, not a candidate training run.

### Recurrent imitation sequence wiring

The mixed pretrainer now collects explicit game/faction/map/reset-generation/
round/decision metadata alongside teacher labels. It partitions complete streams,
shuffles streams instead of decisions, advances up to eight streams together,
and backpropagates through windows of eight decisions. Memory carries detached
between windows and resets at true match/chapter boundaries. Validation follows
the same ordered path. Teacher selection and authored hierarchical sampling
weights are unchanged; no reward is added to imitation.

Resume checkpoints retain stream order, active batch, window cursor and detached
memory along with optimizer/RNG state. Old corpora without sequence boundaries
are rejected, and old independent-example checkpoints require a warm start rather
than exact resume. The existing standalone stateless collection API is preserved.

Verification and integration evidence is retained under
`tmp/rl/recurrent-imitation-20261122/`. This completes sequence wiring, not the
planned candidate pretraining/strength evaluation. Kill-reward experiments remain
outstanding PPO work.

Verification passed: 41 training tests (one optional dependency skip), full project
verification, and 10 arena/20 campaign collection probes. Legacy dense/spatial
buffers and teacher outcomes were unchanged on the paired arena panel; 7,762
campaign decisions formed 50 isolated faction/chapter streams. A 36-second
start/resume integration smoke made 215 updates on 1,680 decisions, changed both
GRU matrices plus both CNN branches/head, and resumed active memory at decision
64 of a stream. A serialized mid-stream checkpoint reproduced the exact next
optimizer update, including dropout, memory and momentum. No gameplay-strength
improvement is inferred from this smoke.

### One-hour recurrent imitation run authorized

Start from V4's policy expanded into the current architecture. Optimize for
3,600 training seconds after corpus preparation; validation and checkpoint writes
are excluded. Retain five-minute checkpoints and gameplay measurements at
0/15/30/45/60 minutes. Keep Lantern Road separate from selection. Use all 336
configured teacher groups, with four teaching games and two disjoint imitation
validation games per group. Corpus shards retain one whole game each to limit
memory. The exact frozen workloads, seed bands, evaluation panels and settings
are in `tmp/rl/pretrain-v4-recurrent-60m-20261122/protocol.json`.

Kill rewards remain an outstanding PPO experiment and are not part of this
imitation run. No PPO continuation is authorized by this launch.

### One-hour imitation completed: measured outcome

The run completed 3,600.005 training seconds, 40,283 updates and 311,262
decisions. Independent confirmation compared V4 and the 30-minute checkpoint.
Balanced targeted completion rose from 8.53% to 55.70%, while broad training
campaign completion fell from 24.72% to 20.56%, held-out gameplay from 32.49%
to 29.84%, and Lantern Road from 5/60 to 2/60. This recipe did not deliver the
intended broader improvement. The recorded selection rule weighted targeted,
training-campaign and held-out scores equally: two thirds therefore came from
training maps on separate seeds. Its winner is not an overall-strength claim,
and no default model was promoted.

A focused follow-up evaluated the saved five-minute checkpoint on the exact
monitor cases: targeted balanced score 45.22% (243/528 raw successes), Lantern
3/24. V4 scored 8.38% (52/528) and 5/24 on those same cases; the 30-minute
checkpoint scored 56.11% (351/528) and 2/24. Matched V2 Lantern results are
6/24. Each opponent has eight Lantern cases, so small differences remain noisy.

The completed archive, all five-minute recovery states, frozen workloads and
curves are under `training/experiments/2026-11-22-recurrent-imitation/`. The
large teacher corpus remains in the shared run backup. The completion monitor
is disabled. Native performance profiling remains outstanding: historical V2
and current imitation timings differed roughly sixfold on the sampled arenas,
but used different engine/catalog versions and played different decisions.


### Nine-hour PPO continuation — 2026-11-23

Authorized initialization: five-minute recurrent imitation checkpoint
`minute-0005-1795384800227451471`, not the final imitation model.
Run: `tmp/rl/ppo-from-pretrain-5m-9h-20261123`; shared backup has the same basename.
32400 seconds of trainer runtime, recovery from validated checkpoints.
38 equal-base-weight arena maps (14 ordinary + 24 targeted), 50% decision budget;
three broad campaigns share the other 50%. No reserved targeted-family allocation.
Existing adaptive mastery sampling remains enabled within pools. Existing chapter
+1/finale +2 rewards remain; no objective-progress or kill rewards, fixed temperature.
Hourly fixed panels: held-out 312, training diagnostics 168, Lantern showcase 24.
Only held-out results select the checkpoint; initialization remains eligible.
No large second confirmation sweep. Preserve every hourly checkpoint and raw results,
training and per-chapter gameplay curves. Hourly automation checks recovery.


### Arena allocation correction — 2026-11-23

User authorized pausing, fixing and continuing the existing nine-hour run.
Paused with a fresh checkpoint at 26041.891 trainer seconds (7h14m), leaving
6358.109 seconds. Ordinary arenas had only 860 decisions in six games; nine
received none. Pool-level accounting hid per-map starvation.
Fixed sampler: hierarchical map/condition allocation and persistent integrated
credits; no allocation-history resets when mastery changes. Schema 2 migration
preserves policy, optimizers, RNG, mastery and lifetime exposure, starting new
credits prospectively. The workload, reward rules, evaluation cases and original
32400-second total budget remain unchanged. All hourly results before correction
belong to the faulty-allocation phase, not the intended training distribution.
Validation: heterogeneous full-workload scheduling, changing mastery with four
concurrent slots, migration and deterministic resume; all ordinary maps receive
work. Full training suite: 41 tests pass, one optional LightGBM skip.


### Nine-hour run completed — 2026-11-23

Training and all hour0–9 evaluations finished; evidence archived under
`training/experiments/2026-11-23-ppo-5m-9h`. Held-out selector chose hour5; hour8
retained with 6/24 Lantern clears, final hour9 had 1/24. Corrected allocation
was active only for the final 1h46m; all 14 ordinary maps then received work.
Monitor disabled. No checkpoint promoted and no additional run started.
A possible later continuation without targeted encounters was discussed, not
authorized as a new run. Rewards/exploration proposals remain separate work.


### One-hour extension authorized — 2026-11-23

Continue from final hour9, not hour8, with the same corrected sampler, environment
mix, rewards and fixed evaluation cases. Preserve optimizer/critic/reference/RNG
and adaptive sampling state. Additional3600 seconds; saved cumulative target
36000.54316014299. Original nine-hour archive remains intact. Hour10 comparisons
will inform the next decision; no environment changes or next run authorized.


### One-hour extension completed — 2026-11-23

Hour10 fixed-panel results: held-out23.30%, ordinary39/84, targeted equal-map67.87%,
broad campaigns4/18, Lantern4/24 (7 caps). Lantern chapter clears/entrants
10/24→9/10→4/9. Same workload/settings as final hour9; all14 ordinary maps sampled.
Extra3600.20 saved trainer seconds,273 updates,1217 episodes,69888 decisions.
Evidence/checkpoint/curves preserved under the parent experiment's
`extension-one-hour/` folder, original nine-hour archive unchanged. Monitor
disabled; no checkpoint promoted and no new run started.

### Targeted curriculum audit and proposed revision — 2026-11-23

Measured hour10 on528 fresh-seed games (eight per configured condition), plus24
matched teacher games. Evidence: `training/experiments/2026-11-23-targeted-curriculum-audit`.
All three escort maps24/24 each, Bellmarket32/32, Thorn Verge72/72. Proposed,
not implemented: reduce each escort base budget to one-third and Bellmarket/
Thorn Verge to one-quarter, retaining rehearsal; account for existing adaptive
multipliers when choosing final allocations. Echo Gate and North of Lantern Road
remain0/8 while their configured teachers each win8/8; keep these gaps. Lower
Gallery learner0/8 vsFull3/8. Lair Outworks18/72 remains relevant. Fixed1/1 does
not establish mastery: Rally5/8 fresh, Flint7/8, training action sampling differs
from greedy evaluation.

Proposed new targeted candidate: Commander Left Behind, derived from observed
Lantern split-army failures. Commander reaches exit alive while supporting the
advance; rear pursuer punishes abandonment. Include a wounded-withdrawal variant
so following the army is not always correct. Validate a current learned-policy
gap against a successful rule-based/targeted reference before integration.
No map, allocation config, reward or new training run changed by this audit.


### New environment proposals from three Sol agents — 2026-11-23

Proposals archived in `training/experiments/2026-11-23-new-env-proposals`.
Prototype priorities: Commander Left Behind (healthy/wounded split-force pair),
Kiln Crossing (fog contact and coordinated battery assault), Scree Relay
(threat-dependent route switch plus named-actor handoff). Reserves: Two Banks
Under Pressure (deathmatch coordination without commander waypoint), Juniper
Works (ten-turn fort defense under ranged/recruitment pressure). Designs unrun;
qualify learner/reference gap and counterexamples before wiring any map.

Additional finding requested by user: short envs provide much faster terminal
feedback. Final hour completed-episode mean decisions: targeted20.7, ordinary79.4,
broad366.6; Thorn Verge1.6. Broad campaigns also pay intermediate chapter rewards.
Record reward latency/event density in subsequent prototypes; no automatic reward
normalization or new sampler/reward changes authorized. New Lair matched probe
againstClassic found hour10/reference5/8vs7/8 fullvision,0/8vs6/8 playerfog,
0/8vs6/8 bothfog. Full configs/results preserved.

### New environment prototypes simulated and confirmed — 2026-11-23

User authorized implementing/running proposals to completion. Three Sol agents
completed tuning and600 fresh native confirmation games, plus matched behavior
replays. Archive: `training/experiments/2026-11-23-new-env-validation`.
Strong candidates: Commander Left Behind healthy (Hour8=0/20,Hour10=2/20,Full20/20),
wounded+healer (0/20,6/20,Classic18/20), blocked-road Scree route (0/20,0/20,
Full20/20). Accepted qualification scope clear vision both sides, Classic enemy,
fixed authored forces, no recruits. Direct-route counterexample Hour10=20/20;
retain as control rather than advertise it as an unsolved training task. HP-only
wounded diagnostic referenceClassic13/20,Full1/20 vsbothlearners0/20.
Kiln rejected because discovery gap disappeared on fresh20seeds; Juniper untested.
No environment integration, sampler changes, rewards or new training started.
Native renders, raw counts, exact catalogs and trace evidence preserved.

### Explicit fog scopes and multi-objective validation — 2026-11-23

Completed user-authorized validation with three Sol workers. Archive:
`training/experiments/2026-11-23-fog-and-multi-objective-validation`.
Fog confirmation:900 commander+480 route games; blocked road stays0/20 for both
learners versusFull/Classic20/20 under player-only and both-sides returning fog.
Healthy commander player-only Hour10=4/20 versusQueued18/20; healer player-only
0/20 versusQueued15/20. Healthy both-fog lacks a competent reference and is
rejected; healer both-fog remains a harder diagnostic, Full10/20 only.

Multi-objective fresh20seed gaps against active Classic opponents, clear both:
Hollowbank capture/rescue/separate live exits Hour8/Hour10=8/20,8/20 versus
Classic/Queued18/20; post-rescue reserve12/20,6/20 versusClassic16/20.
Ashbank simultaneous ten-round staffed home hold plus forward fort1/20,3/20
versusClassic18/20; Hour10 takes fort20/20 but loses home17/20.
Marlow alternative-win priority pair: exposed commander13/20,12/20 versus
allthree ordinary references20/20; hold-interruption control bothlearners20/20.
Native replays verify actual objective handoffs, live exit positions, contested
home defense and enemy countdown observations. Failed prototypes preserved.
Multi-objective fog qualification is incomplete; Hollowbank player-fog rejected
in screening (reference2/8), not advertised as a validated fog encounter.
No integration, reward/sampling edits, promotion or training authorized here.
Next decision remains which validated scopes to wire, alongside the still-open
single-stage targeted +2 versus ordinary arena +1 reward proposal; not implemented.

### Expanded multi-objective opponents and fog — 2026-11-23

User requested establishing fog competence and challenged Classic-only coverage.
Existing authors ran2400 eight-seed discovery games across five starts, three
explicit sight scopes and Classic/Queued/Predictor/Full enemies;1020 separate
fresh-seed confirmation games followed, references selected from discovery.
Exact matrices/evidence: same experiment archive, `expanded-opponents/`.

Ashbank confirms non-Classic cases: clear Queued/Predictor enemies Hour10=4/20
versusClassic16/20; clearFull10/20versusQueued19/20; player-onlyfogFull9/20
versusQueued15/20; bothfogQueued/Predictor0/20versusQueued20/20; bothfogFull1/20
versusFull18/20. Fresh clearClassic14/20reference versusHour10 8/20 is weaker
than original18/20versus3/20; preserve both, not a blanket infeasibility claim.
Hollowbank fresh clearClassic eastguard11/20versus20/20, reserve5/20versus17/20.
Its selected fog/non-Classic cells failed reference reliability on confirmation.
Marlow clearClassic Hour10=14/20versusQueued20/20; fog/non-Classic screening
found no comparable deficit. Fog hides the exposed commander and defeat marker
initially, so it does not preserve the same visible priority choice.

Native two-side observations and living-faction re-hiding audited. Queued and
Predictor have equal win counts here, but Ashbank terminal rounds/reasons differ
on four matched pairs; do not describe the presets as identical. No integration,
workload/reward edits, checkpoint promotion or training launched.

### Multi-objective curriculum wired — 2026-11-23

User approved implementation after explicitly choosing **25% ordinary arena /
25% targeted / 50% broad-campaign decisions**, adaptive separately within all
three groups. This supersedes the earlier proposed equal-base shared arena pool.
No global adaptive transfer between groups. Existing EMA/warmup/80%-threshold/
10%-floor controls and persistent allocation credits remain in place.

Five base reductions now authored: each of the three escort maps1/3; Bellmarket
Courts and Thorn Verge1/4. Others1. Ashbank's physical fog variants share one
logical samplingMap. Four new challenges join the24 existing targeted maps:
Hollowbank east-road guard and west-bank reserve clear/Classic; Ashbank clear
Predictor/Full, player-onlyfogFull, both-fogPredictor/Full; Marlow exposed-commander
clear/Classic. Those are eight challenge combinations. Marlow's validated distant-
commander control shares its map budget at conditionWeight1/4, versus1 for Finish.

New imitators record full validated blocks per condition (Classic/Mission/Classic,
Balanced/Mission/Queued, Predictor/Mission/Full). Native collection applies all
three blocks; teacher-only collection parity was checked against explicit-config
reference play. New cases preserve fixed rosters, zero recruitment, initiative,
objectives and deadlines; no unqualified fog scope is registered for training.

Reward profiles are authored separately from scenario storage: ordinary and
single-stage targeted victories+1, no intermediate bonus; broad intermediate
chapters+1 and finale+2. Existing loss/cap penalties retained. Objective-progress
and kill bonuses remain OFF for the next vanilla run. They remain outstanding
experiments, not removed from the broader plan. Workload provenance includes
map weights, profiles, pool budgets, teachers and adaptive controls. Evaluation
keeps static family/map weights independent of training allocation changes.

No pretrain/PPO launched. Optional short imitation pretrain decision is still
open. Frozen earlier experiment workloads/checkpoints were not rewritten.

### Matched teacher audit before deciding on short pretraining — 2026-11-23

Completed all528 teacher matches for the exact66-condition fresh hour10 panel,
including the configured objective_progress/path_to controllers. No caps;24 prior
teacher results reproduced through terminal round. Teacher447/528 versus learner
383/528; equal-map85.99% versus70.55%. Teachers improve eleven maps, tie six and
are worse on seven. Splitpine0/24learner versus16/24teacher; Echo0/8versus8/8;
North of Lantern0/8versus8/8; Lower Gallery0/8versus3/8. Tidegate's teacher21/32
is materially worse than learner31/32; several solved maps also have weaker
teachers. Do not assume mixed imitation is universally beneficial. Short3–5min
pretrain remains a user decision, not launched. Evidence:
`training/experiments/2026-11-23-targeted-teacher-scores`.

### Better teachers: reuse and extend targeted bots — 2026-11-23

Existing objective_progress/path_to teachers were already included in the prior
scores. Tested them on weaker movement tasks, then screened other full block
configurations plus visible-target and defense specialists.1472 screening games,
600 matched fresh confirmations; failed defense heuristics rejected.

Five condition-specific teacher replacements now wired without changing any PPO
case, reward, sampling weight, map or objective: Force/clear/Full now uses
Situational+Mission+Full (fresh19/20 versus old12/20, learner18/20); Bellmarket/
both-fog/Full uses Situational+Mission+Queued (20/20 versus17/20, learner20/20);
Thorn clearPredictor/Full and both-fogFull use visible_target_path (60/60 versus
51/60, learner60/60). The specialist reuses the native path controller and only
observed hostile positions/defeat markers. Production parity100/100, additional
Predictor-recruiter specialist60/60. Pretraining remains undecided/not launched.

Rally Full did not confirm (12/20 versus Classic13/20 and learner12/20); keep
existing teacher. Tidegate still lacks a teacher matching the learner in several
conditions. Lower Gallery references remain5/20. Do not claim all teacher deficits
are resolved. Evidence: `training/experiments/2026-11-23-better-targeted-teachers`.


## Task-specific teachers — 2026-11-23 (historical; removed 2026-11-24)

Implemented explicit `pretrain_imitator.tactical` roles for relief timing,
recovery before assault, and retreat/screen/return-to-hold. Authored coordinates
and friendly actor bindings live in `simulation/rl.json`; controllers consume
legal candidates and faction-observed spatial features. They do not freeze
opponents, inspect hidden units or RNG, or change playable rules. The controller
recomputes from each supplied observation, including tentative planning states.

Fresh confirmation used twenty disjoint seeds per map on unchanged configured
opponents/visibility, with matched hour-10 and previous-teacher games:

| Map | Previous teacher | Hour 10 | Task planner | Decision |
|---|---:|---:|---:|---|
| Crown Road Relief, clear vs Classic | 14/20 | 18/20 | 20/20 | Wire relief teacher |
| Briar Saddle, clear vs Classic | 9/20 | 20/20 | 20/20 | Wire recovery teacher |
| Lower Gallery Refuge, clear vs Classic | 9/20 | 0/20 | 16/20 | Wire refuge teacher |
| Rally at the Vault, clear vs Predictor | 14/20 | 7/20 | 6/20 | Keep Classic |

Tidegate task-planner screens did not beat the existing learner/reference;
rejected prototypes are preserved rather than wired. Its four configured
clear/both-fog enemy conditions were screened, but the three promoted teachers
are qualified only on their current clear conditions. These are teacher-quality
results on fixed layouts, not evidence of post-imitation improvement or unseen
layout generalization. No pretraining or PPO was started. Pool budgets,
conditions, sampling controls, reward profiles and model weights are unchanged.
Evidence: `training/experiments/2026-11-23-task-planners`.
