# Seeded AI comparisons

These testing policies execute the real `Game` commands. Movement, terrain/race
modifiers, retaliation, healing, gold, recruitment, XP, promotion, scripted arrivals
and campaign victory conditions use the same rules as gameplay. They need no
language model, browser or service. The ordinary enemy still uses classic AI;
this change does not add an enemy selector to the game.

```sh
# Preferred recruitment: classic, melee rush and layered mixed armies.
npm run ai:simulate -- --seeds 10 --workers 4 --out artifacts/preferred.json

# All six combinations of three planners and two recruitment styles.
npm run ai:simulate -- --mode factorial --seeds 10 --workers 4 --out artifacts/crossed.json

# Hold recruitment style constant while comparing the planners.
npm run ai:simulate -- --recruitment mixed --seeds 10 --out artifacts/fixed-recruitment.json

# Authored campaigns, all player/enemy policy combinations, no retries.
npm run ai:simulate -- --mode campaign --seeds 5 --workers 4 --out artifacts/campaigns.json

# Smaller panel or detailed command trace; these flags work in every arena mode.
npm run ai:simulate -- --mode factorial --policies mixed --armies coastal --landscapes wooded --seeds 2 --trace --out artifacts/example.json
npm run ai:simulate -- --replay artifacts/example.json
```

`--seed` sets the first seed; `--seeds` sets the number of consecutive seeds.
`--rounds` defaults to 30 and `--recruits` to one successful recruitment attempt
per phase. `--workers` partitions whole seeds between up to eight Node processes,
then checks coverage and combines their results in seed order. Workers change
execution speed, not outcomes. Reports can also be replayed from `.json.gz` archives.

## What each policy does

**Classic** reuses the existing enemy recruitment and action methods, including
their stable ordering and early leader restraint. Those methods now accept either
side, retaining the original enemy defaults. When given testing objectives, classic
uses them as movement destinations; it still chooses its ordinary combat actions.
The player-side campaign adapter permits an escape leader to leave early. The
live enemy retains its authored posts and campaign behavior.

**Rush** recruits melee units without healing and favors efficient damage and
durability. It pursues objectives, captures income and protects indispensable
actors, while accepting more risk for expendable troops. Existing leaders and
starting troops remain present; melee recruitment does not delete a ranged leader.

**Mixed** targets approximately 50% screens, 35% ranged and 15% healers among
ordinary troops, subject to the available roster, prices and casualties. It scores
weapon effectiveness, finishing attacks, terrain healing and promotion. Screens
support endangered actors and ranged troops; ranged units favor positions behind
the front; healers favor adjacent wounded allies with nonstacking healing accounted
for. Healthy healers defer formation moves while screens can still move. The planner
recomputes the whole army after each chosen action, so followers can use newly
vacated routes instead of being trapped by a fixed unit order.

Both new planners estimate next-phase pressure through a restored game and the
live pathfinder, including authored arrivals. They preserve couriers, leaders and
required companions. The top two potential attackers form a local heuristic;
this is not a prediction of a complete enemy turn or an optimal search. More than
two attackers, jointly opened routes and policy mistakes can still cause losses.

## Objectives and fair starts

Required campaign goals have movement weight 24, ordinary uncaptured villages 8,
and optional visits 4. Escorts and vigils select their actual actors, seals select
the required family, and other troops can clear blockers and support the approach.
An actor remains assigned to its endpoint until the game actually wins. Campaign
success is decided by `Game`, never by the planner's estimate of success.

Arena maps are rotationally paired radius-four hexagons, with open, wooded and
river-crossing layouts. The seed controls paired terrain, equal starting gold and
small planning tie breakers. Crossing arenas retain the river map identity so
classic uses its existing river route planning; open and wooded arenas use its
ordinary mirror-map movement heuristic.
Both sides receive the same leader, two identical
melee troops, recruitment pool and per-phase recruitment limit. Pools cover human,
woodland, undead and coastal units. Both seats and both initiatives are run for
each nonidentical bot pairing; self-play runs both initiatives. One seed contains
216 preferred-recruitment games, or 864 games with all six factorial bots.

Arena victory is either the real engine's leader defeat or control of the central
fort and two central villages through two opposing phases. Optional ruins record
visits without awarding resources. This territory victory condition belongs to
the testing scenario; it does not change normal battles.

Campaign comparisons retain authored asymmetry and real chapter requirements.
They start at chapter one, earn carryover, recall up to four eligible survivors
using the same value ranking for every policy, and receive no retries. Their seeds
vary the new planners' tie breaks, not the authored map or classic AI. A deterministic
classic-only campaign can therefore repeat identically across seeds.

## Reading results

Preferred-recruitment wins combine planner and army effects. Factorial labels use
`planner/recruitment`: compare `mixed/mixed` with `mixed/rush` to change recruitment
while holding the planner fixed, and compare `mixed/mixed` with `rush/mixed` to
change the planner while holding recruitment fixed. Recruitment styles are policies,
not guaranteed fixed unit counts: affordability and losses can change the army.
The report records actual purchases and losses by unit, by roster and by bot, as
well as every individual result. Loss totals include starting troops and promoted
types, so they are not purchase survival percentages.

Results include overall matchup tables, per-roster/per-landscape tables, self-play
seat and initiative diagnostics, casualties, purchases, promotions and endpoint
state hashes. Turn caps remain **unresolved**, separate from wins and defeats.
The net completed-win margin is `(wins - losses) / all games`; JSON includes a 95%
bootstrap interval obtained by resampling whole seed blocks, preserving dependence
between seats, initiatives and maps. More seeds reduce sampling uncertainty;
they do not remove planner weakness, board-direction bias or scenario bias.

Reports record the source commit and a SHA-256 fingerprint of all game TypeScript
and the runner. `--replay` requires the same source fingerprint and exactly reproduces
each recorded outcome, commands summarized into purchases/losses/promotions, and
endpoint state hash. `--trace` also retains all commands and states for diagnosis.
Partial checkpoints preserve completed batches if a run fails; failed workers are
errors, never manufactured defeats. Local reports live in ignored `artifacts/`;
reviewed results can be archived under `docs/simulations/`.

These comparisons measure these policies on these scenarios, not optimal play or
human campaign difficulty. Crossed comparisons help distinguish army composition
from planning strength, and the roster/terrain breakdown helps identify where a
unit or strategy succeeds. Stat changes should follow those breakdowns rather
than treating every mixed-policy victory as evidence that support units are strong.
A unit that the recruiter never buys is not meaningfully assessed by those wins;
check the purchase breakdown before inferring that an individual unit is balanced.
To compare movement/combat planners with an identical fixed starting army and no
subsequent purchases, use `--recruits 0`.
