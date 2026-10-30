---
name: design-campaign
description: Use to design or revise campaigns and scenarios, including their writing, objectives, maps, encounters, progression, and supporting mechanics.
---

# Design campaign

Design the situation, player decisions and supporting systems together. Writing
and objectives matter as much as the map. Make one scenario work well before
expanding the campaign or increasing its scale.

## Reuse mechanics

Don't create one-off shit if there is a better way. Inspect existing scenarios,
triggers, objectives and effects before proposing new mechanics. Express content
through reusable rules and data; extend the shared framework when needed. Avoid
scenario-specific engine branches and redundant events or flags.

Start with the actual player action and its condition. Distinguish the trigger,
its effects and the objective it satisfies. Specify eligible actors, repeat
behavior and consequences. Several systems responding to the same fact should
share that fact. Do not invent additional conditions beyond the intended design.
A named content instance does not require custom engine behavior.

## Premise, discovery and writing

Establish what happened, what the player knows, why it matters, and why the next
objective is a sensible response. Connect the premise to the player's actions.
Plan how discoveries change their understanding; revealing an answer too early
can undermine the reason to continue toward an objective.

Write dialogue from the characters' knowledge, roles and circumstances. Let them
react to what they encounter. Avoid stating the obvious or inventing story facts
to explain a placement that has not been thought through.

## Choices and consequences

Choices must have consequences. Define what changes when the player takes a
route, spends time, accepts a risk, helps someone or leaves something unresolved.
Consequences can affect information, relationships, resources, encounters,
objectives or later scenarios. Make the tradeoffs legible enough for an informed
decision, without requiring every outcome to be revealed in advance.

Give optional content a purpose and a cost. Rewards must be useful to the party
and fit its progression. Establish what carries forward and ensure later content
respects earlier outcomes rather than silently resetting them.

Urgency needs a reason and consequences for delay. Evaluate deadlines against
travel and combat costs; treat initial numbers as provisional. Define success,
failure and partial outcomes. Failure can advance the story when the player
still has a coherent reason and enough information to continue.

## Geography and encounters

Make geography explain movement, settlement placement and available routes.
Structures must connect physically to their surroundings. Use terrain to create
approaches, alternatives, concealment and encounter spaces. Decoration should
support those purposes rather than fill every empty area. Preserve useful paths
and intentional clearings; use symmetry deliberately.

Plan encounter placement, activation, enemy behavior and aftermath together.
Specify enemy counts, spawn locations and repeat behavior. Mechanics must enact
what the writing claims. Recruitment and reinforcement rules should support the
scenario's intended decisions rather than appear by default.

Define completion precisely. Reaching, controlling and securing a location are
different facts; decide which matter. Work out what opens progression and how
remaining threats interact with completed objectives.

## Preview and implementation

Use the bundled [map draft renderer](map-drafting.md) for all draft map previews.
Supply map data, unit placements and annotations through its input files. Extend
this shared renderer when a reusable capability is missing; do not create a
separate renderer per scenario or add scenario-specific branches.

Change only the intended area during map revisions and compare with the previous
layout. Use [design-terrain](../design-terrain/SKILL.md) and
[design-unit](../design-unit/SKILL.md) for new artwork. Review actual terrain and
connections through the game renderer; do not imply geometry or mechanics that
the map does not contain.

Separate existing capabilities, missing shared mechanics and undecided design
values. Inspect Rust schemas and scenario logic before claiming support. Runtime
scenario settings and game facts must come through Rust; rendering details can
remain in the frontend.

Keep drafts outside the playable catalog. Distinguish encounter illustrations
from implemented gameplay. Promote or publish within the user's authorized scope.
Evaluate routes, detour costs, encounter pressure and different outcomes before
expanding the campaign.
