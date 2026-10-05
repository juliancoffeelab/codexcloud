# Classic side bias and campaign failures

This investigation uses the retained tournament at source `578fae2`, including all
108 campaign attempts. Six selected campaign replays were made **before changing
the AI**, and each chapter's final-state SHA-256 matched its archived result.
There was no new large tournament. Campaign rules, stats and special objective
handling have not been tuned. Implementation changes address classic direction
bias, the arena round clock and the user's subsequent request for shared dynamic
leader restraint.

## Tournament maps and symmetry

The tournament layouts are CLI scenarios built by `createArena`, not maps in the
game picker. Their territory win condition also lives in the runner: own the
central fort and both central villages through two opposing phases, or defeat the
opposing leader. Adding playable tournament maps and configuration is deferred.

The archived tournament map itself is symmetric. Across all 108 combinations of
nine retained seeds, four rosters and three landscapes, every tile has its exact
180-degree partner with matching terrain, opposite ownership, matching starting
unit/HP, and matching adjacency. All 401,868 tile-pair distances matched under
rotation. These are geometry inspections, not simulated games.

The identity `mirror` used by the runner must not be confused with the ordinary
Classic mirror map: the runner replaces its board with the paired 61-hex arena.
The game's River crossings map is deliberately asymmetric; it is likewise
replaced by a symmetric board for crossing tournaments.

## Where classic's side bias came from

Archived classic self-play, wins by **board seat**, not by planner:

| Roster   | Left/player wins | Right/enemy wins |
| -------- | ---------------: | ---------------: |
| Human    |               17 |               37 |
| Woodland |                3 |               51 |
| Undead   |               15 |               39 |
| Coastal  |               21 |               33 |
| Total    |               56 |              160 |

With the left seat moving first: left 43, right 65. With the right seat moving
first: left 13, right 95. There were no unresolved classic self-games.

Three concrete defects were found and fixed:

1. **Directional recruitment.** `enemyRecruit` sorted spawn tiles by descending
   screen column for both sides. That is toward contact for a left-hand keep but
   behind a right-hand keep. This defect alone does not explain why the right
   side won more often: it coexisted with the other defects.
2. **Absolute coordinate ties.** Equal movement and retreat scores fell back to
   row-major tile/pathfinder order; equal target scores fell back to unit array
   order; equal objective distances fell back to the objective array. Those
   choices did not rotate with the board. A corpus of 324 isolated classic unit
   actions had 77 mismatched destinations after rotating the same position.
3. **Wrong round clock when the right seat started.** The arena called the normal
   game's `endTurn`, which increments the round after the enemy phase. An
   enemy-first opening therefore ran `enemy@1`, `player@2`, `enemy@2`,
   `player@3`. Woodland classic recruited a boar first on the right but a wolf
   first on the left; the pool rotation and the turn-5 leader release were no
   longer matched.

The new classic order uses a home-to-opposing-keep axis in axial coordinates.
Recruit deployment favors the opposing keep; equally useful village choices favor
the home keep; remaining movement, retreat and combat ties use that same relative
orientation. Pathfinder traversal and player movement rules remain unchanged.
The arena now completes both seats' phases before incrementing its round, whichever
seat starts. Normal-game and campaign round timing remains unchanged.

The same isolated-action corpus now has zero rotation mismatches. Eight quick
classic self-games (one open-map seed, all four rosters, both initiatives) have
exactly paired winners, reasons and durations under seat reversal: four left wins,
four right wins. Human and undead first movers won their pairs; woodland and
coastal second movers won theirs. This establishes the checked symmetry, **not** a
new statistical win-rate estimate or absence of initiative advantage.

Rush and mixed do not call classic's spawn-position rule when using their own
recruitment. They place screens nearer the closest current enemy, and ranged
troops/healers farther away. A recruitment override uses that placement even with
the classic planner. Their ID-seeded tie breakers and other policy weaknesses have
not been claimed to be free of all side effects.

### Subsequent leader-restraint request

All three planners now use the same live-army rule instead of classic's turn-5
release and rush/mixed's former keep penalty. Advancing toward contact needs two
nearby combatants with at least half their HP, with at least one no farther from
the enemy than the proposed leader position. Couriers and healers do not count
as bodyguards. An unthreatened leader retains its recruiting position while fewer
than four ordinary combatants survive and recruitment is possible. Retreats,
required leader escapes and immediately winning attacks remain possible.

The eight-game result above was obtained after the direction/clock fixes and
before this additional leader change. Focused checks cover unsupported leaders
on turns 1 and 99 in all three planners, loss of healthy escorts, recruitment
restraint and required leader escapes. The rule is not a guarantee against losing
a leader, and the historical campaign scores below have not been rerun. Repeating
the same eight small classic games after this addition still gives four wins per
seat, with matched winners/reasons/durations on reversal. Those paired results
changed: undead first movers win, while the other three rosters' second movers win.

## Campaign results by roster and planner

There is **one authored player roster per campaign**, not every race on every
campaign. Later chapters can add allied races. Consequently these figures cannot
separate race strength from map, story objectives, starting forces or enemy roster.

Each cell below is **completed / defeated / unresolved**, out of nine attempts
(three enemy planners times three seeds). A completion means the whole campaign.

| Authored player roster and campaign | Classic   | Rush      | Mixed     |
| ----------------------------------- | --------- | --------- | --------- |
| Human — Lantern Road                | 0 / 6 / 3 | 4 / 2 / 3 | 2 / 4 / 3 |
| Woodland — Green Accord             | 0 / 9 / 0 | 0 / 7 / 2 | 0 / 5 / 4 |
| Undead — Ashen Oath                 | 5 / 4 / 0 | 3 / 4 / 2 | 3 / 5 / 1 |
| Merfolk — Borrowed Tide             | 0 / 6 / 3 | 5 / 4 / 0 | 8 / 1 / 0 |

Full completions versus each enemy planner are shown below. **Every cell is out
of three attempts**, not a percentage. Three classic-only seeds can repeat the
same deterministic play; they are not three independently varied maps.

| Campaign      | Player planner | Enemy classic | Enemy rush | Enemy mixed |
| ------------- | -------------- | ------------: | ---------: | ----------: |
| Lantern Road  | Classic        |             0 |          0 |           0 |
| Lantern Road  | Rush           |             1 |          1 |           2 |
| Lantern Road  | Mixed          |             1 |          0 |           1 |
| Green Accord  | Classic        |             0 |          0 |           0 |
| Green Accord  | Rush           |             0 |          0 |           0 |
| Green Accord  | Mixed          |             0 |          0 |           0 |
| Ashen Oath    | Classic        |             0 |          2 |           3 |
| Ashen Oath    | Rush           |             2 |          1 |           0 |
| Ashen Oath    | Mixed          |             0 |          1 |           2 |
| Borrowed Tide | Classic        |             0 |          0 |           0 |
| Borrowed Tide | Rush           |             3 |          2 |           0 |
| Borrowed Tide | Mixed          |             3 |          3 |           2 |

## Why the campaigns fail

### Lantern Road: courier loss and blocked seal objectives

Nine attempts end at the ferry, all with Ivo dead. One ends at the villages, and
two end with Nera dead at the bell. Nine cap at the bell; six finish the campaign.
Eight bell caps still have unsealed graves; the ninth sealed the graves but did
not defeat the Bellwarden.

The exact classic/classic seed-1 replay kills the Bellwarden, seals only the
central grave, then stops taking actions. Nera stands on unsealed `9,1`; the mage
stands beside her at `10,2`. The nearest seal objective is blocked by an ally, but
classic has no command to clear an objective for its eligible actor. The same
stall repeats across the three classic-only seeds. Rush cannot recruit replacement
adepts because its recruitment excludes non-screen units; starting/carryover
adepts are therefore irreplaceable under its preferred recruitment.

### Green Accord: witness safety and an army that stops growing

Of 27 attempts, 18 lose at the cutting camps, six cap there, and just three reach
the vigil; all three then lose. Lark dies in 17 of the 21 defeats, Alder in four.
All six capped attempts recover the ledger but own neither logging camp at the
turn cap. None reaches the third chapter.

In the exact classic/classic replay, Lark advances away from her escort to make
ranged attacks. An archer and spearman kill her on enemy turn 2. Classic's ordinary
attack/retreat logic protects against immediate retaliation, not the complete
following enemy turn or loss of a required witness.

In the mixed/classic seed-1 replay, Alder leaves the keep on turn 2 after the army
reaches six units. There is no return-to-recruitment plan. After casualties, the
remaining army cannot breach the defended forts; Alder ends at `3,4`, Lark at
`3,5`, and the player has **116 unspent gold**. Late turns contain shuffling near
home while the enemy holds the camps. This is not evidence that the player simply
needs a lower recruit price or more starting gold.

### Ashen Oath: escape failures and ignored mediation rules

Nine attempts lose Orin at the quarry and one caps there. Four lose Orin at the
refuge. Eleven finish; two cap in the last chapter after breaking both stones but
not defeating the Master.

The mixed/classic seed-1 replay fails to establish the refuge hold while retreating
from pressure. On turn 8 a bone archer kills
Dain, ending mediation; Orin dies on enemy turn 10. The game correctly changes the
objective to defeating the remaining patrol, but the planner's combat score
rewards the leader kill without accounting for the peace objective it destroys.
It also permits essential actors to stand still when every evaluated action is
rejected as dangerous; doing nothing does not prevent the enemy from killing them.

Classic alone honors authored enemy posts and patrol limits. Rush/mixed read the
general objective list but do not enforce `enemyLeaderPost` or classic's patrol
radius. Thus switching the enemy planner changes mission-specific defensive
behavior as well as general tactical strength. The quarry is one reason a
nominally stronger enemy planner need not make every campaign harder.

### Borrowed Tide: the strongest mixed result, but still actor failures

Thirteen attempts finish; six lose at the outworks, four at the wreck and one at
the final sluices. Every one of those 11 defeats includes Edda's death. Three
classic-only attempts cap at the outworks despite owning both required forts.

In the exact classic/classic replay, an ordinary harpooner occupies Edda's goal
`9,3`; Edda is stranded next door at `9,4`. There are no enemies left. The blocker
has no remaining eligible objectives and never moves, while the actor cannot
enter its goal. This is an objective coordination failure, not coastal difficulty.

The rush/mixed seed-1 replay demonstrates a distinct safety failure at the wreck:
Edda moves to `6,0` and kills a spearman. The forecast sees four incoming damage
from heavy infantry and no reachable archer shot. On the actual enemy phase that
infantry moves to `6,1`, opening a route through its former hex for the archer to
reach `7,2`. The two attacks deal four damage each and kill 8-HP Edda. The top-two
threat estimate misses **routes opened by earlier enemy movements**, even with
only two attackers. This was confirmed by replaying the stored commands and
comparing the forecast and actual reachable sets.

## What to tune next — not implemented

1. Assign objectives to the actors who can complete them, reserve their endpoints,
   clear allied blockers, and choose reachable alternatives rather than waiting
   forever beside a blocked nearest objective.
2. Protect required actors against sequential enemy approaches; choose a survival
   fallback when no scored action is safe. A lethal-filter rejection is not a
   defense plan.
3. Include replenishment and return to a keep in planning; recruit units required
   for seals or other special objectives even when the general army style is rush.
4. Model hold/vigil/mediation as their actual rules, including forbidden kills and
   the value of maintaining the timer, rather than just destinations plus combat
   rewards. Preserve authored enemy duties across test planners where intended.
5. Only then use campaign completion data to decide whether unit stats or scenario
   difficulty need adjustment. Current failures conflate battle balance with
   demonstrable objective mistakes.

All archived results remain historical measurements of the recorded source. The
classic and arena fixes change its fingerprint; replay those archives on `578fae2`.
The new eight-game diagnostic is not substituted into the original tables.
