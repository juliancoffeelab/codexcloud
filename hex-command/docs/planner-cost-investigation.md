# Why the planners cost more than Classic

This investigation uses the consolidated PR25 code at `38be55d` and current
combat data. It changes no production tactics. The three Old implementations
remain frozen reference engines.

The [simulation purpose and policy roles](../.agents/skills/combat-simulation/SKILL.md#purpose-and-policy-roles)
define why speed matters: Classic remains the player-facing fair challenge,
while simulation players test infantry-only, fair-roster and situational
recruitment against a fixed Classic enemy in arenas and complete campaigns.
Classic's timing is a reference, not a reason to weaken simulation-player
competence. The existing policy names do not yet implement all three roles.

## Recruitment and tactical search are separate

Rush restricts recruitment to frontline types and scores their effectiveness,
durability and affordability. It can consider several eligible types; it does
not restrict the battle to one unit or one tactical search.

Classic chooses recruitment through a simple rotation or preference, then visits
units in a stable order. Rush and Mixed repeatedly call `best_plan` to select
the best next action across actionable units. They also forecast next-phase
pressure for ordinary troops. Classic uses cheaper current-position danger for
ordinary troops, retaining conservative forecasts for essential actors.

The earlier self-play observations measured 33.26µs per own phase for Classic
and 57.17µs for Rush. Rush had 6.06 chooser calls per phase versus 4.76 for
Classic, and 9.43µs per call versus 6.99µs. Recorded mean own armies at phase
start were 4.20 units for Rush versus 3.76 for Classic. These strategies create
different states and durations; the self-play ratio does not isolate search cost.

## Same-position production measurements

61 Mixed self-play positions cover all four factions and landscapes, including
initial, middle, late and capped-game boundaries. Each policy starts a phase
with the same state, seed, objectives and recruitment budget. The production
binary runs 200 repetitions per position, counterbalancing all 120 orders of the
five policies in one process. Each row measures 12,200 complete own phases.

| Policy | Mean chooser CPU per phase | Chooser calls per phase |
| --- | ---: | ---: |
| Classic | 41.08µs | 5.56 |
| Rush | 48.79µs | 5.72 |
| Mixed | 52.14µs | 5.79 |
| Battler | 50.05µs | 5.72 |
| Priority | 50.24µs | 5.72 |

Rush is about 19% slower on these matched positions, compared with 72% in its
own self-play states. Earlier separate-process measurements gave 38.80µs for
Classic and 50.05µs for Rush. Cache state, allocator reuse and run order affect
timings; a universal ratio cannot be inferred from either sample.

Battler and Priority produce identical complete phase states and action/combat
records on every fixture. In an arena, `priority::next_plan` selects the Battler
planner for either entry point. Priority's campaign controller does not run.
Battler currently provides a capability boundary, not a separately cheaper
arena algorithm.

## Diagnostic cost attribution

A temporary copy of the native crate adds CPU scopes and work counters. All
five policies reproduce every reference phase result. Instrumentation increases
cost and changes one LLVM profile entry, so the following percentages locate
work rather than establish precise production costs.

Rush spends about 32% of diagnostic chooser CPU in global search setup, 27% in
per-unit preparation, and 24% in destination/candidate evaluation. Recruitment
is about 3%. It visits 13.05 unit plans per phase versus Classic's 4.25 unit
visits, with 3.92 global searches, 103.69 destination visits and 50.59 detailed
attack previews per phase. Simplifying recruitment cannot close the main gap.

Battler/Priority split their diagnostic cost approximately as follows:

| Work | Share of chooser CPU |
| --- | ---: |
| Refresh goals, formation and spatial context | 16% |
| Prepare each unit's duties/routes/support | 17% |
| Generate and shortlist candidates | 14% |
| Check shortlisted candidate safety | 11% |
| Rank units and rebuild the queue | 10% |
| Focused full-planner fallback for essential actors | 13% |

Their queues refresh about 2.97 times per phase and rank 12.93 unit entries.
They call the cheap planner 4.92 times and focused fallback 1.26 times per
phase. The fallback remains relevant to arena leaders even without campaigns.
The twelve-candidate limit bounds later safety evaluation; it does not remove
the work needed to prepare units, rank opportunities or generate that shortlist.

## Optimization order

1. **Reuse global search context in Rush/Mixed.** Cache foe-distance grids,
   target slices, tile-order context and objective/route preparation across
   actions, refreshing each part against its actual dependencies. Friendly
   movement does not change enemy-position distance grids; nonlethal damage
   does not change attack geometry. Preserve HP-dependent kill/retaliation
   decisions, original tie ordering, campaign events and restoration behavior.
2. **Split Battler/Priority cache invalidation.** Avoid clearing goal fields,
   reservations and route assignments when only a different part of the state
   changed. Reuse buffers for destinations, support and candidate work. Update
   unit scores incrementally where dependencies allow it, rather than blindly
   freezing a queue that depends on HP and occupancy.
3. **Make focused fallbacks genuinely focused.** Reuse the existing context and
   pressure forecast instead of rebuilding full-search scaffolding for one
   leader. Reject provably hopeless searches with conservative bounds. Retain
   full retreat and essential-actor safety coverage.
4. **Then tune candidate generation.** Reuse legal origins and attack previews;
   maintain the small sorted shortlist without sorting it afresh after each
   insertion. Reducing twelve candidates to six or dropping safety is a behavior
   change and needs a separate quality study, not an exact-performance claim.

The matched-position reductions needed to reach this Classic baseline are
about 16% for Rush, 21% for Mixed and 18% for Battler/Priority. These are targets,
not achieved improvements. Beating Classic requires further savings and
validation on full self-play workloads, whose encountered states differ.

An alternative scheduler, including a fixed unit pass, is acceptable only if
the simulation players retain their intended recruitment distinctions and can
intelligently finish campaigns. Measure wins/completions against the same Classic
enemy: fair-roster and situational players should outperform infantry-only, and
fair-roster performance should remain reasonably close to situational selection.
Also measure caps, essential-actor losses and campaign/saved-planner behavior.
Removing campaign support from Battler does not fulfill those player roles or
offer a corresponding arena gain because its battle path is shared with Priority.

Start with exact context/cache work in the current five policies. Check complete
gameplay against retained journals, including the existing-policy workload and
new-policy self-play/campaign continuations. Changes to search order, safety or
shortlist width also need the fixed-Classic balance comparisons on current combat
data. Existing self-play timing measurements do not establish those invariants.

Machine-readable timings, counters, fixture scope and methodological limits are
in [planner-cost-investigation.json](benchmarks/planner-cost-investigation.json).
Raw fixtures, phase results, logs and temporary instrumentation remain in ignored
`tmp/planner-investigation/`. No campaigns or browser/WASM timings were measured.
