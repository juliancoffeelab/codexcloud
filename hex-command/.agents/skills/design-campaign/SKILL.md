---
name: design-campaign
description: Use to design or revise campaigns and scenarios, including their writing, objectives, maps, encounters, progression, and supporting mechanics.
---

# Design campaign

Design the situation, player decisions and supporting systems together. Make one
scenario work well before expanding a campaign. For runtime mechanics or rendering
changes, apply [battlefield-contract](../battlefield-contract/SKILL.md).

## Shared mechanics and authoring

Start with the actual player action and its condition. Specify eligible actors,
repeat behavior and consequences; distinguish the trigger, effects and objective.
Inspect existing content and extend reusable rules instead of adding named
scenario branches. Several systems responding to one fact should share that fact.

For event fields and execution, consult `rust/catalog/src/data/events.rs`,
`rust/battle/src/logic/events.rs` and current campaign `scripts.json` files.
Event IDs are not flags; use explicit flags for remembered story facts. Deleting
a rescued or sacrificed unit does not emit a combat death. Account for pending
spawns when clearing encounters. AI intent describes planning; events execute
gameplay. Distinguish permanent achievement from current objective satisfaction.

For rosters, diplomacy, recruitment or named participants, read
[faction and identity authoring](references/factions.md).
For revisions to Green Accord or Lantern Road, reconcile the relevant story contract:
[Green Accord](../../../rust/data/game/campaigns/green-accord/design.md) or
[Lantern Road](../../../rust/data/game/campaigns/lantern-road/design.md).

## Premise and discovery

Establish what happened, what the party knows, why it matters and why its next
objective follows. Write dialogue from speaker knowledge, role and presence;
react to discoveries without inventing story facts to excuse an unexplained
placement. Briefings and objectives give actionable orders, not trigger mechanics
or the author's complete plan.

Preserve surprises. Foreshadow only what the party can know; meaningful choices
do not require disclosure of future rewards, losses or solutions. Keep unknown
people, destinations and rewards out of objectives, hints, markers and counters.
Reveal later objectives at the encounter that motivates them, without bypassing
their victory requirements.

Use one encounter activation for dialogue, spawns, lights and objective reveals.
Any player-controlled unit can discover a site unless the action requires a
particular person. Surviving companions may react with the party without being
marched to another invisible trigger. Author optional goals separately from
required victory conditions; a script effect is not automatically an objective.

## Decisions, geography and consequences

Define consequences for routes, delay, risks, help and unresolved encounters.
They may change information, relationships, resources, objectives or later stages.
Optional content needs a useful, progression-appropriate reward and a cost.
Respect carried outcomes instead of silently resetting them.

Urgency needs a reason and consequences. Evaluate deadlines against travel and
combat costs; initial numbers are provisional. Define success, failure and partial
outcomes. Failure can advance a story when the party still has a coherent reason
and enough information to continue.

Make geography explain settlements, movement and alternative approaches.
Structures connect physically to their surroundings; decoration supports encounter
spaces and intentional clearings. Use symmetry deliberately. Plan placement,
activation, enemy behavior, recruitment, reinforcements and aftermath together;
state enemy counts, spawn sites and repetition. Mechanics must enact the writing.
Distinguish reaching, controlling and securing a site, including how remaining
threats affect progression.

## Draft or implementation

For draft previews, use the [shared map renderer](map-drafting.md), including its
scope and completion rules. Extend it for reusable missing capabilities instead
of adding a scenario-specific renderer. Drafts stay outside the playable catalog.

For playable scenario or mechanics changes, read
[implementation and completion](implementation.md). Completion requires reconciling
starting units, dialogue, encounters and victory rules with the delivered state;
balance claims require simulation evidence.
