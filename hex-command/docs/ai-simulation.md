# Seeded AI comparisons

Testing policies execute real `Game` commands: movement, terrain and race effects,
combat, healing, recruitment, XP, promotion, scripted arrivals and victory rules.
The ordinary enemy still uses classic AI. There is no new enemy selector.

The five AI followups are implemented. **No new campaign or tournament simulations
have been run for them. Run simulations only when the user explicitly requests it.**

## Running a comparison

```sh
# Race × complete policy × map, with both seats and initiatives.
npm run ai:simulate -- --seeds 10 --workers 4 --out artifacts/race-matchups.json

# Authored campaigns, each player/enemy policy pair, both diplomacy settings.
npm run ai:simulate -- --mode campaign --seeds 5 --workers 4 --out artifacts/campaigns.json

# Restrict the panel; each selected race can face every other selected race.
npm run ai:simulate -- --armies human,coastal --policies classic,mixed --landscapes crossings --seeds 2 --out artifacts/coast-panel.json

# Replay requires the report's exact source fingerprint.
npm run ai:simulate -- --replay artifacts/coast-panel.json
```

These are instructions for a later authorized run, not records of completed runs.
`--seed` sets the first seed and `--seeds` the count. `--rounds` defaults to 30;
`--recruits` defaults to one recruitment per phase, or zero for starting-army-only
comparisons. `--workers` partitions complete seed blocks across up to eight Node
processes and validates their coverage before combining them. Workers do not change
outcomes. `--diplomacy pragmatic,restrained` selects campaign settings; both are the
default. `--trace` additionally retains raw runner results; detailed phases and
initial/final states are recorded regardless of that flag.

The planner/recruiter split and factorial mode have been removed. Every policy
chooses its own recruits. Historical split results remain in `docs/simulations/`;
replaying them requires their original source, rather than converting their labels
or pretending that they describe these changed policies.

## Complete policies

**Classic** uses the shipped recruitment and combat/movement heuristics and stable
unit ordering. Recruit placement points toward the opposing keep. Equal scores
use home-relative ordering, with equal village choices favoring the home keep.
The campaign/test adapter adds objective reservations, required-actor safety and
selectable mediation behavior. Live enemy actions retain authored posts and patrols.

**Rush** recruits ordinary melee screens and favors efficient damage and durability.
**Mixed** aims for roughly 50% screens, 35% ranged and 15% healers, subject to its
race's pool and resources. Mixed places screens ahead of supports, heals adjacent
wounded allies, and recomputes plans after every command so followers can use newly
opened routes. Both policies use actual weapon/terrain effectiveness and objectives.
Their screens recruit toward contact and their supports behind it.

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

## Arena coverage and victory

Maps are rotationally paired radius-four hexagons: open, wooded and river crossings.
Seeds vary paired terrain, equal gold and small policy tie breakers. Crossings retain
the river identity for classic's routing. Each side gets its selected race's leader,
two melee starters and its own recruitment pool. Human, woodland, undead and coastal
rosters can face each other; same-race baselines remain included. Initial resources,
positions and per-phase recruit limits are paired, while race-specific stats differ.

An entrant is `(race, policy)`. Every unordered entrant pair is scheduled in both
seats and both initiatives; identical entrants run both initiatives once. Four races
and three policies give 12 entrants, 288 games per terrain, **864 games per seed**
across the three terrains. Each round contains both phases regardless of initiative.

Victory means defeating the opposing leader, or owning the central fort and both
central villages through two opposing phases. Optional ruins record visits without
resources. Territory victory belongs to this testing scenario, not ordinary battles.

Campaigns use the actual authored player and enemy rosters, maps, objectives and
waves. They begin in chapter one, earn carryover and recall up to four eligible
survivors using the same ranking. No retries. Four campaigns, three player policies,
three enemy policies and two diplomacy settings give **72 attempts per seed**.
Campaign seeds vary policy tie breaks, not authored maps; deterministic classic
choices can repeat across seeds.

## Recorded data and limits

Schema 2 reports contain every configuration, outcome, reason, unresolved cap,
phase command, combat damage/retaliation, casualty, recruit, promotion and initial/
final state. Phase records include beginning/end rosters with identities, positions,
HP, XP, action flags and leader mode; gold, ownership, goal progress and full campaign
state retain mediation escalation and named deaths. Casualties include troops
recruited and killed during the same phase. Arena goal ownership/holding and optional
visits are recorded. Endpoint hashes and a SHA-256 source fingerprint permit exact replay.

Aggregates compare race/policy entrants, ordered race matchups and terrain; self-play
reports seat and initiative diagnostics. Campaign aggregates retain campaign,
player/enemy policies, diplomacy setting, completion, defeat, unresolved caps,
escalation and chapter reach. Recruitment, losses and promotions are totaled by
race/policy entrant. Loss totals include starting and promoted troops, so they are
not purchase survival percentages.

Bootstrap intervals resample complete seed blocks, preserving seat/initiative/map
correlations. They describe these scenarios and policies, not optimal play or human
difficulty. More seeds reduce sampling uncertainty, not policy weakness. Partial
checkpoints retain completed configurations; worker failures are errors, not defeats.
