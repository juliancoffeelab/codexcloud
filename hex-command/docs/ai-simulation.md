# Seeded AI comparisons

The simulation CLI defaults to the native Rust engine. Testing policies execute
the same Rust battle commands and rules as the browser: movement, terrain and race effects,
combat, healing, recruitment, XP, promotion, scripted arrivals and victory rules.
The ordinary enemy still uses classic AI. There is no new enemy selector.

Run campaign or tournament simulations only when the user explicitly requests it.

## Running a comparison

```sh
# Complete ten-seed workload: 11,520 arena matches and 720 campaign attempts.
npm run ai:full -- --out artifacts/full.json

# Race × complete policy × map, with both seats and initiatives.
npm run ai:simulate -- --seeds 10 --workers 4 --out artifacts/race-matchups.json

# Authored campaigns, each player/enemy policy pair, both diplomacy settings.
npm run ai:simulate -- --mode campaign --seeds 5 --workers 4 --out artifacts/campaigns.json

# Restrict the panel; each selected race can face every other selected race.
npm run ai:simulate -- --armies human,coastal --policies classic,mixed --landscapes crossings --seeds 2 --out artifacts/coast-panel.json

# Replay requires the report's exact source fingerprint.
npm run ai:simulate -- --replay artifacts/coast-panel.json

# Compare saved gameplay dumps without running matches. Convert each dump once.
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

`--seed` sets the first seed and `--seeds` the count. `--rounds` defaults to 30;
`--recruits` defaults to one recruitment per phase, or zero for starting-army-only
comparisons. `--workers` runs matches through up to eight scoped Rust threads.
Its default respects the available CPU quota; workers borrow one immutable catalog
and take the next match as they finish. Workers do not change outcomes.
`--diplomacy pragmatic,restrained` selects campaign settings; both are the
default. `--trace` additionally retains raw runner results; detailed phases and
initial/final states are recorded regardless of that flag.

Schema 3 reports keep compact outcomes, composition statistics and timings in
the main JSON. Each outcome's `detail` gives a file, byte offset, length and SHA-256
for its complete record in an adjacent `.details.jsonl` journal. This includes
all commands, combats, troop states, ownership, goals and initial/final states.
Records are written once as each match finishes. Progress files contain the
compact indexes, so the runner neither rewrites accumulated traces nor keeps
the entire tournament in memory. Keep a report and its journals together.
`--trace` adds `.trace.jsonl` files containing the raw runner return values.
Exact replay reads and validates one journal record at a time.

Dump conversion uses an explicit Arrow schema; pass every detail journal for a
sharded report. Comparison validates unique `(kind, config)` identities with a
Polars join, gathers corresponding rows and compares complete gameplay values in
Arrow without sorting or rerunning simulations. It excludes `stateHash`,
`elapsedMs`, `elapsedSeconds` and `detail` at match and mission level, preserving
array order within each match. Stage traces include wall time, CPU time and peak
memory; `--out PATH` saves the comparison result and wall-time spans.

`npm run ai:benchmark -- --out artifacts/baseline.json` measures 60 representative
runs spanning every race, terrain and policy plus all four campaigns. After an
optimization, use `--compare artifacts/baseline.json --out artifacts/optimized.json`
to require identical hashes for every complete result, including phase commands.
This benchmark is separate from the lightweight test suite.

The engine shares immutable indexed hex geometry between forecast copies, uses
integer movement queues, and precomputes fixed movement and defense rules. Routing
fields ignore occupancy and can be reused across commands; terrain revisions
invalidate them after edits, and planners share immutable distance arrays. Threat searches
reuse movement and firing positions within an unchanged board. Objective endpoints
and incoming waves retain exact forecasts. Rush and mixed retain the first best
plan directly, preserving the stable tie order without sorting every candidate.
These changes preserve the policies' scores and rules rather than reducing the
simulation schedule or turn limits.

## Native Rust engine

`npm run ai:simulate` and `npm run ai:full` build and invoke the native harness.
Rust reads the authored `rust/data` JSON directly and owns scheduling, battle/AI
execution, campaign transitions, recording and reports. Content fingerprints are
separate from the compiled engine fingerprint; JSON edits need no export or rebuild.

`npm run ai:schedule -- --mode full --seeds 10` enumerates configurations without
running matches. The browser uses the WASM engine and Rust-generated transfer
types through its JavaScript controller. See [`rust/README.md`](../rust/README.md)
for the public API and build details.

## Complete policies

**Classic** uses the shipped recruitment and combat/movement heuristics and stable
unit ordering. Recruit placement points toward the opposing keep. Equal scores
use home-relative ordering, with equal village choices favoring the home keep.
The campaign/test adapter adds objective reservations, required-actor safety and
selectable mediation behavior. Live enemy actions retain authored posts and patrols.

**Rush** recruits ordinary melee screens and favors efficient damage and durability.
**Mixed** aims for roughly 35% tanks, 15% fast melee flankers, 35% ranged and 15%
healers. Shares are normalized over the roles its recruitment pool offers. Fast
melee specialists (including scouts with a ranged sidearm) do not fulfill the tank
quota. A missing tank gets replacement priority. With village income, mixed saves
for that replacement instead of continually buying cheaper flankers; other preferred
roles save for at most two income ticks. Without income it buys an affordable option.
Recruitment effectiveness uses actual target cover and the attack category the
role delivers, rather than crediting a ranged unit for a risky melee sidearm.
Mixed places screens ahead of supports, heals adjacent
wounded allies, and recomputes plans after every command so followers can use newly
opened routes. Both policies use actual weapon/terrain effectiveness and objectives.
Their screens recruit toward contact and their supports behind it.

Beast-race units have 1 additional melee and ranged protection in forests and
groves, on top of terrain cover. Thorncaller and Briarweaver have no extra individual
woodland cover: their total protection is 1 melee and 2 ranged. Thorns deal 4 Pierce
damage (Briarweaver: 5). Rookie boars have 3-damage Impact Trample and 2-damage Pierce
tusks; Great boar attacks remain at 5. Boar HP and protection are unchanged.
Undead have 2 racial Pierce protection and 1 Impact vulnerability; skeletons and
revenants do not stack another individual Pierce modifier. Wardens retain their
spear and use a 3-damage Impact Shield bash. Wavemender costs 10 gold.
Historical reports describe their recorded rules snapshot, not these updated values.

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

The woodland regression reuses recorded starting states from the completed full
run and checks one policy turn, not a campaign simulation. Tournament and campaign
win rates must be measured again in a separately authorized run after this change.

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

Maps are rotationally paired radius-four hexagons: open, wooded, river crossings
and islands. Islands has two home islands and three separate objective islands:
the central fort and the northern/southern villages. Bridges link the home islands
to the fort; shallow crossings reach the village islands and give alternative
approaches to the fort, including the hex east of the northern village and its
southern counterpart. Open channels give merfolk routes around those crossings.
Seeds vary paired terrain, equal gold and small policy tie breakers. Water arenas
retain the river identity for classic's routing. Each side gets its selected race's leader,
two melee starters and its own recruitment pool. Human, woodland, undead and coastal
rosters can face each other; same-race baselines remain included. Undead arena
recruitment includes skeletons and bone archers; campaign pools remain authored
independently, including their existing adepts. Each side receives the same total
starting budget: the authored `startingBudget` of 36 gold plus a paired seed-dependent
0, 3 or 6 gold. Its leader and two starters are charged at their unit prices, and
the remainder is spendable cash. Positions and per-phase recruit limits are paired,
while race-specific stats differ.

An entrant is `(race, policy)`. Every unordered entrant pair is scheduled in both
seats and both initiatives; identical entrants run both initiatives once. Four races
and three policies give 12 entrants, 288 games per terrain, **1,152 games per seed**
across the four terrains. Each round contains both phases regardless of initiative.

Victory means defeating the opposing leader, or owning the central fort and both
central villages through four opposing turns, or six on islands to allow ground
troops time to reach the objectives. Only completed opposing turns count; losing
any of the three objectives resets the hold. Optional ruins record visits without
resources. Territory victory belongs to this testing scenario, not ordinary battles.

Campaigns use the actual authored player and enemy rosters, maps, objectives and
waves. They begin in chapter one, earn carryover and recall up to four eligible
survivors using the same ranking. No retries. Four campaigns, three player policies,
three enemy policies and two diplomacy settings give **72 attempts per seed**.
Campaign seeds vary policy tie breaks, not authored maps; deterministic classic
choices can repeat across seeds.

## Recorded data and limits

Reports retain every configuration, outcome, reason, unresolved cap,
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
