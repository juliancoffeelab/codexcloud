# AI policy implementation

The classic, rush and mixed policies use the same native battle commands and
resolved movement, damage and recruitment rules as the browser. Recruitment is
part of each policy. The live enemy uses classic; the simulation harness can
select each policy for either side.

Current unit and race values are authored in [game data](../../data/README.md).
CLI usage is in the
[simulation guide](../../../.agents/skills/combat-simulation/usage.md).

## Purpose and acceptance criteria

Classic is the simplest AI the player interacts with. It should provide a fair
challenge and avoid automatically losing fights, without requiring exceptional
tactics. It currently serves that purpose. This performance work does not require
making the live enemy a stronger opponent.

Simulation AIs model players to test roster-balance hypotheses and invariants.
The intended player strategies are:

1. **Infantry only:** recruit infantry melee units as the deliberately narrow
   baseline.
2. **Fair roster use:** use every unit fairly and maintain a stable recruitment
   pool, rather than repeatedly exploiting whichever unit looks strongest.
3. **Situational selection:** semi-intelligently choose the best unit for the
   current situation.

Compare each strategy against the same enemy AI, currently Classic, under matched
arena and campaign conditions. Fair-roster and situational players should have
better win/completion rates than the infantry-only player. The fair-roster player
should remain reasonably close to the situational player. This compares their
success against a fixed enemy; self-play timings or direct matches between the
simulation strategies do not establish those balance expectations.

A large advantage from selecting one roster unit indicates a balance problem.
An infantry melee unit dominating the roster is an especially obvious failure.
The acceptable fair-roster versus situational gap has no numeric threshold yet.
Report the actual differences and their strata rather than inventing one.

All three simulation-player roles must intelligently complete campaign objectives
as well as fight battles. Trials start in chapter one, earn their carryover and
receive no retries. Preserve objective completion, required actors and campaign
progression when making planning cheaper; weakening player competence would make
the balance comparison unreliable.

This PR's goal is faster simulation players, enabling more trials, larger maps,
longer campaigns and more units. Classic's speed is a useful reference, while
simulation correctness and the balance comparisons remain acceptance criteria.

### Current implementation gaps

The policy names below describe current code, not three completed implementations
of the intended simulation-player roles. Rush filters frontline units, including
both warrior and flank roles, so it does not guarantee strict infantry-only
recruitment. Mixed maintains role shares and scores types within each role; it
does not guarantee fair use of every individual roster option.

Priority and Battler are planning experiments that both reuse Mixed recruitment.
Neither supplies a separate unrestricted situational recruitment strategy.
Priority supports campaigns; Battler's arena-only restriction means it cannot
fulfill a complete simulation-player role by itself. Their shared arena planner
does not create an additional independent balance hypothesis.

ClassicOld, RushOld and MixedOld are frozen historical measurement references.
They preserve the old engine and catalog and remain separate from the intended
simulation-player roles; optimizing them would destroy that reference.

## Complete policies

**Classic** uses the shipped recruitment and combat/movement heuristics and stable
unit ordering. Recruit placement points toward the opposing keep. Equal scores
use home-relative ordering, with equal village choices favoring the home keep.
The campaign/test adapter adds objective reservations, required-actor safety and
selectable mediation behavior. Live enemy actions retain authored posts and patrols.

**Rush** recruits ordinary melee screens and favors efficient damage and durability.
**Mixed** uses relative weights of 0.3 warriors, 0.2 flanks, 0.2 rangers and 0.2
healers. Every unit JSON declares its role explicitly. Missing roster roles are
redistributed at runtime: all four roles normalize to 33.3%/22.2%/22.2%/22.2%; a
warrior/ranger-only roster normalizes to 60%/40%. It chooses the most underrepresented
role first, then an effective unit within that role. A missing warrior gets replacement priority. With village income, mixed saves
for that replacement instead of continually buying cheaper flankers; other preferred
roles save for at most two income ticks. Without income it buys an affordable option.
Recruitment effectiveness uses actual target cover and the attack category the
role delivers, rather than crediting a ranged unit for a risky melee sidearm.
Mixed places screens ahead of supports, heals adjacent
wounded allies, and recomputes plans after every command so followers can use newly
opened routes. Both policies use actual weapon/terrain effectiveness and objectives.
Their screens recruit toward contact and their supports behind it.

Required couriers and escaping leaders plan with current occupancy, allowing
detours that initially increase terrain-only distance. Allies clear their next
movement corridor instead of permanently occupying the destination's approaches.
Safety checks still apply to both the actor and units yielding the route.

All policies share leader reinforcement hysteresis. Leaders initially rebuild when
below five living ordinary combatants. They release at five, return to rebuilding
at two or fewer, and retain their previous mode at three or four. Rebuilding leaders
return to an owned keep and remain available there until replenished. Couriers,
healers and other leaders do not count toward the thresholds. Mode is saved with
the leader. Required leader escapes bypass rebuilding; no owned recruitment site
or no recruitment pool means no rebuild assignment. A side with neither an
affordable recruit nor village income seeks resources instead of waiting on a
keep indefinitely, while retaining its rebuilding mode. Retreat remains available.
Released leaders still need two nearby healthy escorts to advance toward contact.

Couriers, leaders and required companions use a conservative next-phase pressure
estimate. It includes scripted arrivals, sums potential attackers, and allows
opposing movers to vacate one another's routes while retaining friendly obstacles.
This catches attacks enabled by earlier enemy moves. It is a conservative heuristic,
not an exact enemy-turn search: attackers can have mutually incompatible positions
and it can overestimate danger. It does not model friendly screens dying and opening
further routes. Ordinary troops retain the cheaper top-two estimate.
Required actors strongly penalize exposure; couriers give combat much less weight
than safety. When every destination is lethal, an actor can still withdraw to a
less dangerous reachable position. A genuinely terminal escort/escape or arena
leader kill need not fear a nonexistent next phase.

## Objectives and diplomacy

Required goals have weight 24, ordinary uncaptured villages 8, optional visits 4.
Escort, escape and vigil endpoints belong to their specific actor; typed seals
belong to eligible unit families. Other troops support the surrounding approaches
and penalize occupying or remaining on the reserved endpoint. Classic clears
blockers before moving the actor; rush/mixed recompute after each command.
Rebuilding leaders also reserve their recruitment sites. An actor holds its endpoint
until the engine completes the battle. Authored enemy posts and objective patrol
limits apply to all policies; the scenario's duties do not disappear when changing AI.

Rush and mixed assign each unit a reachable objective before scoring destinations,
and retain that assignment throughout the turn until the objective is secured.
Assigned couriers and seal-capable units prioritize their personal duties. Equal
objective choices use home-relative ordering rather than coordinate labels.
Progress earns points and retreat loses them against that same objective; gaining
one movement-cost step is worth a quarter of its weight. Moves are compared with
waiting, so existing formation, healing and safety do not count as new gains.
Captured sites stop attracting troops. Hold duties retain a garrison, while other
troops pursue remaining goals. A rebuilding leader's duty does not recall the army.
Required-actor safety, retreat and legal movement restrictions still apply.

Campaign diplomacy is a matched comparison dimension:

- **Pragmatic:** ordinary combat scoring, including attacks that may end mediation.
- **Restrained:** during the authored refuge mediation, do not pursue retreating
  patrol units or kill the protected captain. Nonlethal defense against the captain
  remains possible. Penalize occupying their next steps toward friendly villages,
  keeps or the home edge, leaving a withdrawal route. Rush/mixed mark genuine
  wounded withdrawals just as classic does, clearing that status on renewed
  advance, attack or full recovery.

The setting changes AI choices, not legal attacks, enemy allegiance, battle stats
or the story's escalation triggers. Once the real campaign escalates to war,
mediation protection ends. The current engine has two sides; the refuge's captain
and withdrawing soldiers are protected by story conditions, not by an invented
third faction. Campaigns without those conditions retain ordinary combat behavior.

## Exact performance changes

Changes to existing policies preserve complete commands, tie breaks, movement
discovery order, safety forecasts and campaign progression. Threat forecasts may
use membership-only movement masks while decisions retain ordered destinations.
Static reverse routes and bounded blocker-keyed movement caches are shared across
forecasts; terrain changes detach them. Mission content is reused only while
its story-patch inputs match. Required-actor safety and conservative forecasts
remain part of the existing policies.

Static supplied objectives are expanded once per planning workspace; dynamic
campaign objectives and rebuilding duties refresh against the current battle.
Tactical units are evaluated by potential benefit, while original roster,
destination and weapon ordinals decide equal scores. Compact target slices avoid
one allocated list per tile. Arena safety forecasts reset action flags directly
and subtract eliminated enemies from independent forecasts; campaign forecasts
retain actual turn-start events, healing, waves and objective transitions.

Native recording uses direct Arrow snapshot dictionaries with complete ordered
phase references. Endpoint hashing streams the original sorted-key JSON bytes.
The native CLI uses a CPU-specific LLVM profile-guided build, trained on full
all-policy arena and supported campaign schedules at seeds 17–18. Compiler profiles
are cached by source, data, toolchain and CPU; simulation results are always
computed afresh. Wasm builds use separate compiler flags.

## Priority and Battler

These policies are opt-in; the authored simulation defaults and live enemy remain
unchanged. **Priority** plays battles and complete campaigns. **Battler** is an
arena-only entry point to the same battle planner. Campaign requests and saved
campaign planners reject Battler. Full schedules exclude it from both campaign
sides. Both recruit the balanced Mixed army through production recruitment rules.

The planner ranks actionable units, searches one unit at a time, and refreshes its
queue after actions. It retains objective commitments, movement corridors, reserved
endpoints, authored posts, promotion, healing, retreat and leader rebuilding. Each
unit has at most three searches per phase; new friendly units receive their own
budget. Stable ordinals and the seeded action jitter resolve ties.

Detailed evaluation normally considers the twelve most promising raw actions.
In battles, attack previews use at most four promising origins per target and
weapon. Dominated choices sharing an origin, eliminated enemy and terminal status
are removed before spending the shortlist on safety. An endangered essential actor
with no positive plan searches all legal moves using its existing pressure forecast;
a nearby leader with no bounded plan can fall back to a focused Mixed search.
These rules keep useful retreat positions from disappearing behind unsafe attacks.

Objective route fields are shared by unit kind and duty. Proximity, formation,
withdrawal lanes and rebuilding intents refresh against their actual dependencies.
Cheap ranking does not replace legal command validation or conservative pressure
forecasts. Campaigns retain scripted turn-start arrivals and mission transitions.
Derived fields stay private and rebuild after restoration; the unit queue, retries,
search attempts and cursor are part of the validated native/WASM continuation.

Priority's campaign controller uses full Mixed planning when an essential actor (a leader,
courier or required companion) is at or below half health and an opponent is within five hexes. It also
uses full Mixed planning from chapter one when any chapter combines defeat with
an unrestricted seal objective, preserving the army and companion carryover needed
for that assault. Ashen Oath currently matches this structural condition. This is a
hybrid controller; its Ashen results come from full Mixed planning. Other campaigns
use the bounded planner with focused Mixed fallback for required actors. Once only
an escort or escape remains, ordinary fighters give clearing the opposing leader
higher priority instead of camping indefinitely on the actor's approaches. This
priority change applies to generated duties and respects refuge mediation.

During an un-escalated refuge mediation, Priority prefers restrained behavior even
when pragmatic play permits escalation, except in those complex assault campaigns.
It changes its choices, not legal attacks, allegiance or story conditions. Real
campaigns start at chapter one, earn their carryover and receive no retries.

Measure quality and compute independently with the paired production-game runner
and the saved-position profiler described in
[the performance guide](../../../docs/simulation-performance.md). Report every
campaign/diplomacy/opponent stratum, caps, defeats, chapter reach and named losses;
aggregate wins alone do not establish campaign support. Compare battle factions,
landscapes, opponents and both initiatives. The proposed fourfold additional
reduction in decision CPU remains a performance target, not an achieved claim.
