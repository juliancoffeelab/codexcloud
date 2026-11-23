# Proposed replacement targeted environments

Three independent Sol6.2 agents inspected current maps and experiment evidence.
These are design proposals, not implemented or qualified environments. No
training workload, reward, playable catalog or model was changed. The first
three candidates below are the recommended prototype order; the other two are
alternatives, avoiding addition of every proposal at once.

| Priority | Proposed environment | Size | Decision and ordinary win condition |
|---|---|---|---|
|1|Commander Left Behind|17×12|Split force at ford; keep commander alive, secure a forward post and reach exit. Healthy support vs wounded retreat/heal pair.|
|2|Kiln Crossing|11×8|Locate two ranged battery units through fog, break a credible melee screen and defeat both. Full-vision/player-fog/both-fog comparisons.|
|3|Scree Relay|18×11|Change route when threat blocks a lane; ranger activates relay then captain reaches exit. Opposite-lane threat control makes waiting/continuation correct.|
|Reserve|Two Banks Under Pressure|15×11|Normal fixed-roster deathmatch, no commander waypoint. Weak front line needs commander support; strengthened front line permits safe rear coverage.|
|Reserve|Juniper Works|10×8|Hold two forts for ten consecutive rounds against ranged pressure and enemy replenishment; healthy/wounded commander pair.|

All force counts, starting HP, geometry and deadlines in the individual proposals
are provisional. In particular, a proposed route is not a demonstrated feasible
solution. Prototype in ignored catalogs and qualify before integrating.
Use fully active normal enemy play; no paralysis/order tricks, movement bonuses
or hardcoded commander behavior. Controls must demonstrate that waiting or
withdrawal can be appropriate. Commanders share fighting work when useful, not
whenever the designer demands a move.

## Evidence and scope

- Three escort maps are24/24 each, Bellmarket32/32 and Thorn Verge72/72 on fresh
  hour10 greedy cases. These are the proposed lower-budget rehearsal tasks.
- Echo Gate/North of Lantern Road remain0/8 vs matched Full/Queued8/8. They
  remain useful; new route geometry supplements their unsolved transfer gap.
- On Lair Outworks against Classic, hour10 vs Full reference: full vision5/8
  vs7/8; player fog0/8 vs6/8; both fog0/8 vs6/8. Fog failure causes were not
  action-traced; do not label them confirmed lost-contact hesitation. Raw
  configs/results are in lair-classic-reference.json.
- Commander losses were traced in four matched Lantern cases: detached troops
  die before Nera; prolonged stationary behavior or ineffective retreat follows.
  But hour8 seed480000013 also won while Nera waited safely through turn18.
  Support strength and threat state matter; never teach unconditional advance.

## Reward timing audit

Measured completed episodes in the final hour: targeted929 games, mean20.7
decisions; ordinary194, mean79.4; broad94, mean366.6. Thorn Verge161 games
averaged1.6 decisions. Targeted episodes yield about48.3 terminal outcomes per
1000 recorded decisions, ordinary12.6, broad2.73. These are outcomes, not
necessarily positive rewards. Broad campaigns also recorded94 chapter clears
and pay intermediate rewards; counting only final victories would misrepresent
their signal. See reward-timing.json for map-level counts.

Decision balance does not equalize episode endings or reward latency. The current
short scripted encounter finale pays+2, whereas a completed three-chapter campaign
pays+1,+1,+2. Thus simple one-turn encounters can yield immediate positive
feedback while longer tasks rely more heavily on critic bootstrapping. This is a
plausible training pressure, not a measured attribution of gradient dominance or
a proven explanation of Lantern regression. Do not automatically divide rewards
by duration; that changes the objective and can favor other unintended behavior.

For new prototypes, record decisions to first nonzero reward, decisions between
chapter rewards, terminal outcomes and wins per1000 decisions, and game length.
Prefer encounters requiring a few linked tactical decisions over more near-instant
kills. Keep full campaigns as well as short encounters. Reward coefficients and
new allocation rules remain separate decisions, not authorized by this proposal.

## Qualification

Start with a small matched tuning cohort under clear vision and one fixed
opponent. Compare hour8, hour10 and a successful observation-scoped reference.
Freeze promising layouts and verify on separate fresh seeds; then check explicit
player-only and both-sides fog, with one withheld layout for transfer. No score
claim for new maps exists yet. Accept only meaningful measured learner gaps on
feasible tasks, supported by action traces and the paired control. Rewards remain
normal win/chapter/finale rewards. Individual reports give provisional cohort
sizes and acceptance bars, which are design criteria rather than measured results.
