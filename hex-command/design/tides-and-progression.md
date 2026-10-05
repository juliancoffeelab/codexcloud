# Tides, race defaults and veteran companies

Night task, 2026-10-05. Extended target: 13:00 Europe/Kyiv (10:00 UTC).

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

Thorncallers specialize in protected woodland support: forest and grove add one
extra point of ranged cover, retained by Briarweavers. They keep seven HP and no
melee armor. This answers the observed woodland attrition without making exposed
casters or their staves into substitute front-line boars. A sword plus an arrow
kills an exposed rookie; in woodland it leaves one HP. Boars still supply the
durable melee screen, while wolves supply speed and thorncallers supply range.

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

The sixth coastal comparison improved mixed-army speed but retained higher
attrition. Wavemenders gain one HP (8; veterans 11) to survive a rookie sword
and bow while a screen withdraws, without adding armor or removing woodland
exposure. A fresh matched comparison is required before claiming improvement.

The wreck evacuation uses three consecutive defended enemy turns after capture,
with the final boat departing no earlier than global turn six. Ownership may be
held without an occupant. Enemy recapture resets the ferry crossings. Late
arrival cannot claim elapsed global rounds as defended time; saves and undo
retain the duration, and old ongoing saves start this new counter at zero.

The Ashen refuge's recurring heavy infantry suppressed bone bows while melee
skeletons retained an Impact club counter. Its initial armored patrol stays;
regular recruits and the turn-three sword reinforcement are now rookie
swordsmen. Bone bows gain an unarmored target, while recruits must earn heavy
armor through XP. The briefing makes ownership defense and the actual lethal
mediation boundary explicit. Both players receive those same rules in fresh
trials; earlier runs retain their stricter interpretation and failed attempts.

The fresh Ashen comparison exposed an unforgiving quarry opening: the Ledgerkeeper
began pursuing on global turn five, reaching an exposed Orin together with regular
troops and leaving no surviving retreat on the next turn. His mission now assigns
the southern office as a post. He guards it and its immediate approaches, while
regular soldiers and the incoming adept/warden still pursue. This is encounter
pacing, not a global unit buff. The game manual and player interface clarify that
troops do not intercept attacks or exert a zone of control. A fresh full comparison
is required; the earlier defeats and interrupted control remain recorded.

The extended pass gives both Luna recruitment policies the same
[tactics guide](../docs/campaign-strategy-guide.md) before chapter one.
Guidance covers escorts, damage types, cover, retreat, healing and objectives,
without supplying a winning route. Earlier interrupted and unguided records
remain identified separately. Slightly better mixed outcomes are sufficient;
the aim is useful composition choices, not forcing every roster to use all types.

Adepts and bone archers gain one HP (veterans also gain one) without extra damage
or armor; they can survive one ordinary mixed hit pair while still dying to
their counters or concentrated heavy attacks. Living bows cost seven rather
than eight. One southern village bow arrives on turn seven instead of starting
deployed, and the Master's boar arrives on turn four. These reduce overlapping
opening pressure while preserving the later troops and disclosed wave schedule.

## Visual advancement, after balance

Every family must visibly change when it advances. Lower infantry wear leather
and cloth before earning metal armor. Veteran scouts can use dark clothing and
two knives; other families need similarly recognizable equipment or silhouette
changes that express their role. Keep faction recognition and the approved pixel
material palette. Compare all ranks on one full unit sheet: humanoids should have
roughly the same height and comparable detail, with deliberate exceptions for
beasts and tails. Edit the matrices, then render the normal game atlas. This work
follows the balance pass, as requested on 2026-10-05.
