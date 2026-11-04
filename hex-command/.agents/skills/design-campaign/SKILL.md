---
name: design-campaign
description: Use to design or revise campaigns and scenarios, including their writing, objectives, maps, encounters, progression, and supporting mechanics.
---

# Design campaign

For runtime mechanics and rendering changes, apply the [battlefield contract](../battlefield-contract/SKILL.md).

Design the situation, player decisions and supporting systems together. Writing
and objectives matter as much as the map. Make one scenario work well before
expanding the campaign or increasing its scale.

## Reuse mechanics

Reuse shared rules. Inspect existing scenarios,
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

Surprises, good or bad, are important and necessary. Preserve discoveries,
unknown places and unforeseen consequences. Meaningful choices do not require
advance disclosure of their future effects. Foreshadow only what fits the
party's knowledge; do not invent advance knowledge to explain a later reward
or loss.

Write dialogue from the characters' knowledge, roles and circumstances. Let them
react to what they encounter. Avoid stating the obvious or inventing story facts
to explain a placement that has not been thought through. Briefings and objectives
use the party's current knowledge; let encounters reveal enemies and outcomes.
Give players actionable orders and necessary rules, without narrating trigger
conditions or alternate branches. Script events are implementation details,
not automatically optional objectives. Author optional goals explicitly, separately
from victory requirements; hints offer advice rather than hide side objectives.

## Knowledge and encounter activation

Objectives are orders based on what the party knows, not a checklist of the
scenario author's plan. Do not reveal undiscovered people, allies, destinations,
rewards or solutions through objectives, hints, map markers or counters. Keep
later objectives hidden until the discovery that gives the party a reason to
pursue them; revealing one must not bypass its actual victory requirement.

Use one encounter activation to drive its dialogue, spawns, lights and objective
reveals. Any player-controlled unit can discover a location unless the action
really requires a particular person. A surviving companion can react with the
party without being marched to a separate invisible trigger. Descriptions of
settlements, camps and fortifications must make sense in their geography.

## Choices and consequences

Choices must have consequences. Define what changes when the player takes a
route, spends time, accepts a risk, helps someone or leaves something unresolved.
Consequences can affect information, relationships, resources, encounters,
objectives or later scenarios. Give players enough grounding to act on the
situation they face; let later consequences reveal themselves through play.

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

Draft work may add artwork to the shared library and extend the draft renderer.
Adding artwork does not authorize gameplay or runtime code changes. Keep broader
tooling improvements separate; they must not delay the requested preview. Check
artwork with the artwork build, consistency checks and visual inspection; adding
art files alone does not justify a broader code audit. Missing gameplay support
must not block a visual draft.

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

## Completion

For every changed scenario, reconcile the authored content with the delivered
state before handing it over:

- Account for every starting player unit: carried leader, automatic surviving
  companion, selected recall, or explicitly authored new arrival. Check promotions,
  experience and equipment where progression is part of the change.
- Check every delivered dialogue line against speaker presence, prior outcomes
  and the party’s knowledge at that moment. Match every named route, settlement,
  light and destination to the actual map and encounter state.
- Account for every encounter activation and its dialogue, spawns, lights and
  objective reveals. Verify eligible actors, repeat behavior and consequences.
- Distinguish required victory conditions from optional goals. Check discovery,
  success and loss/absence cases relevant to the changed mechanics; inspect the
  opening before advancing the scenario.

A draft completes with the requested map-data preview and its supported/unsupported
mechanics stated. An implementation completes when the starting state and changed
encounter outcomes agree with the writing, the applicable content/mechanics checks
pass, and balance claims have simulation evidence. Keep review evidence in ignored
tmp/; use disposable probes where needed rather than adding permanent named-content
tests. Report unresolved mismatches explicitly.
