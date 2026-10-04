# Tides, race defaults and veteran companies

Night task, 2026-10-05. Target: 09:00 Europe/Kyiv (06:00 UTC).

## Rules contract

Terrain defines a finite movement cost and explicit passability. Race and unit
movement modifiers each contain an optional signed cost and optional passability.
Passability resolves unit → race → terrain; costs add terrain + race + unit and
clamp to at least one. An impassable tile costs Infinity only in the resolved
movement query, never in the terrain definition. Recruitment, reinforcements,
recall, route finding and save loading must all use that same query.

Race defaults are ordinary data: movement and signed damage-received modifiers.
Unit specifications contain deviations from the race. Resolved unit definitions
expose the final resistances so previews, AI and the field manual stay consistent.
Human defaults are neutral. Undead resist piercing and are vulnerable to impact.
Woodland beasts move freely in forests; their bodies are vulnerable to piercing.
Merfolk move freely in water, shallows and marsh. Plains remain normal footing.
Forests, groves and hills cost three movement and expose merfolk to one additional
point of damage after base terrain protection. Marsh exposes ground races to
one additional point of damage but protects merfolk by one. Terrain, race and
unit modifiers sum by attack category and damage type in the same combat query.
Individual specializations can override access and alter these defaults.

Units earn experience for damaging enemies and for kills, including retaliation.
Zero damage earns no experience. Experience never resets movement or attacks.
Promotions are explicit player choices once experience reaches the threshold;
enemy troops use the same progression rules. Promotion preserves identity, name,
role and accumulated wounds, and finishes that unit's actions for the turn.
No full-heal promotion or mid-combat statistic changes.

Swordsmen and spearmen are inexpensive initial infantry. Their promotions become
the existing armored sword/mace soldier and spear/hammer warden. Other troop
families have explicit specialist upgrades. Leaders also have a finite veteran
path. Higher ranks have real strengths and costs, never an unlimited scaling loop.

Campaign veterans retain type and experience. Named leaders and companions return
once, with their progression retained. Recalled recruits retain their progression
and names. Retrying reconstructs the mission's incoming roster, not defeated
survivors or experience farmed during a failed attempt. Existing saves remain
readable; missing experience means zero and legacy type-only recall is migrated.

## Merfolk campaign: The Borrowed Tide

The estuary's low tide is lasting too long. The river towns blame merfolk for
stranding their ferries; the merfolk blame a human sluice whose keepers refuse
to open it. Tidekeeper Mira discovers that both communities are being deceived.
A salvage factor has redirected the estuary into an old quarry to uncover
a wreck carrying the original accounts he needs to destroy. Releasing the water too soon
would drown the workers he sent into the wreck.

1. **The Empty Nets.** Recover a stranded pilot and take the two sluice outworks.
   The pilot's sounding chart shows water being diverted, rather than a natural
   failure of the tide. Crossing the water gives merfolk mobility; holding the
   dry outworks still requires protection and ranged support.
2. **The Drowned Ledger.** Reach the wreck while defending a channel refuge long
   enough for its trapped workers to escape. The factor's hired guards try to
   burn the records, and scavengers threaten the narrow rescue route. The workers
   establish what happened; a ledger alone is not a reason for everyone to trust
   Mira. Attacking guards serves rescue and preservation of evidence.
3. **A Tide for Both.** Secure both sluice controls and escort the surviving pilot
   to the signal tower before reopening the tide. Human defectors help on land;
   merfolk attack through the water. The ending restores the estuary after the
   evacuation, creates shared maintenance of the sluice, and places the factor
   before the towns' council. It does not claim water or violence magically
   settles every disagreement.

Objectives are concrete actions with visible destinations and a consequence in
the story. Named casualties and veteran continuity survive chapter transitions.
No objective appears underneath a starting unit and no encounter ends merely
because a character announces a truce while enemies keep inexplicably attacking.

## Balance and verification

Mixed forces should offer cheaper scouting/capture, robust melee screening,
ranged pressure, and a damage type that answers enemy resistances. Terrain must
create different useful routes. Strengths come with vulnerabilities and action
costs; no recruit should provide every damage type, speed and durability at once.

When the implementation is ready, Luna agents play actual deterministic games
under matched scenarios, gold, starting troops and enemy behavior. Compare the
instruction to rely on one recruit type with an informed mixed-army instruction.
Record wins/objective progress, turns, losses and gold. Keep the same seed and
rules for both; do not weaken the opponent or tell the mixed agent the winning
move sequence. A dominating single-unit strategy triggers another balance pass.
Retain the transcripts and explain practical limits of a small agent sample.

After balancing, independently review architecture, persistence, combat/progression
timing, passability and the new campaign. Verify review findings before correcting
them. Run only lightweight checks within the repository's existing time budget.
