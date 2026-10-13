# AI policy implementation

The classic, rush and mixed policies use the same native battle commands and
resolved movement, damage and recruitment rules as the browser. Recruitment is
part of each policy. The live enemy uses classic; the simulation harness can
select each policy for either side.

Current unit and race values are authored in [game data](../../data/README.md).
CLI usage is in the
[simulation guide](../../../.agents/skills/combat-simulation/usage.md).

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
