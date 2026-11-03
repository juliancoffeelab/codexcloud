# Event system design

These examples preserve the agreed direction for Lantern Road's event redesign.
They are proposed authoring data, not inputs supported by the current engine.
The JSON spelling, selector shapes and variable-reference syntax are provisional;
the agreed behaviors below take precedence. Dialogue wording, descriptive location
IDs, faction IDs and spawn compositions in examples are illustrative, not changes
to the playable campaign. Do not treat this design as authorization to implement
or deploy it.

## Examples

- [Bear encounter and hammer](event-examples/bear.json): proximity spawns a
  particular identified bear; that bear being killed gives Nera the hammer.
  This is a death event, not an inference that no bears remain anywhere.
- [Ferryman](event-examples/ferryman.json): reaching the eastern ford in time
  saves him; the house has separate saved/dead conversation events. His letter
  is dialogue, not an inventory item.
- [Turn warning](event-examples/turn-warning.json): Oren delivers an informed
  warning if alive and present; otherwise Nera delivers the fallback.
- [Captive rescue and safety](event-examples/captives.json): proximity changes
  allegiance. Standing at shelter completes the explicitly signalled escort,
  deletes the captives and records their safety.
- [Monastery](event-examples/monastery.json): an eligible mage stands there for
  three consecutive turns. Completion sets the fog-cleared flag. The terrain's
  fog condition references that flag; no separate clear-fog action is needed.
- [Bell](event-examples/bell.json): three individually authored tolls and a
  captive-death event. This combines the agreed ritual behavior and schedule in
  the current draft notation; it is not a separately approved final JSON schema.

## Agreed semantics

### Identity, predicates and actions

Character and tile references use stable authored IDs, independent of display
names and coordinates. Unit types, character IDs and runtime unit IDs are distinct.
A selector can match one character, a role/type, or a collection of mark instances.

Conditions compose through `all`, `any` and `not`. `stand` means currently
occupying a tile, regardless of how the unit arrived. It replaces the proposed
`visit` predicate. `left` detects leaving. `killed` is an actual death occurrence.
`proximity.target` in the captive example identifies whom the player approaches,
rather than restricting the approaching unit's character identity.

Actions include spawning, giving equipment, changing allegiance, setting flags,
speaking, changing terrain, creating/removing marks and deleting units. Rescue
and ritual sacrifice use the same deletion action. Deletion does not emit a
killed event. The surrounding event records whether deletion meant rescue or
sacrifice. No distinct evacuation action is required by these examples.

Most examples are once-only chapter events. The captive-death event must handle
each matching death, including several on one turn. The precise repeat/once
schema remains to be specified rather than silently making this event once-only.
Existing blocked-spawn placement/retry behavior should be reused. Consuming a
mark after issuing a spawn must not discard an already pending spawn request.

### Dialogue

A `say` action contains ordered lines and optional `with` passages. A passage
can contain multiple speakers, including replies that must disappear together
with the initiating character's line. `with` selects the passage when its
character is alive and present. An optional `alternative` supplies a fallback;
without it the whole passage is skipped. Names are resolved from the campaign
registry. Narrators can use a speaker label.

Branching saved/dead Ferryman outcomes remains in separate event conditions.
Speaker-dependent exchanges belong inside dialogue. A general-purpose `If`
action language has not been approved or shown necessary.

### Hold and objectives

The monastery accepts any eligible mage, not specifically Oren. `held` maintains
consecutive-turn progress and resets on interruption. The current engine counts
at player-turn start; the exact new timing rules and actor substitution semantics
are still questions, not newly agreed rules.

An event may carry optional objective metadata in the same definition/file.
The objective explicitly describes its intent, including selector, destination
and duration: `hold` includes `turns: 3`, rather than requiring Campaigner to
infer instructions from arbitrary event logic. The event condition owns actual
completion/progress; objectives expose intent and information to the player and
AI without independently implementing the same counter. Existing campaigns other
than Lantern Road are not the design focus.

### Bell ritual

There are three original ritual marks and three captives. Each toll processes
its designated original mark, deleting an occupying captive and spawning the
associated group, then deactivates that original mark. A moved captive's death
creates a mark where it actually died, not at its original ritual location.
Each toll also processes all pending captive-death mark instances.

A death mark is a persistent instance attached to a tile, with its own unique ID
and shared type/tag `captive-death`. Two deaths on one tile create two instances
and therefore two spawn groups. Tile identity remains unique; duplicate IDs are
not needed for selection. A glow can be derived from mark presence. The precise
runtime representation of marks and their appearance is not yet settled.

Ritual deletion does not create a death mark. A dead captive cannot also be
standing on its original mark. Multiple eligible captives/death marks processed
by the same toll intentionally produce their respective groups. Do not add a
special automatic next-mark or recurring ritual scheduler.

The approved schedule is deliberately **not** a uniformly shifted schedule:

| Arrival | First toll | Second toll | Third toll |
| --- | --- | --- | --- |
| Normal | Turn 6 | Turn 12 | Turn 18 |
| Late | Chapter start | Turn 6 | Never |

- First: turn 6 OR (chapter started AND late).
- Second: turn 12 OR (late AND turn 6).
- Third: turn 18 AND NOT late.

Each toll fires only once even if its condition matches again later. The late
branch must not receive a third toll on turn 12. Applying the opening toll during
setup versus presenting it at chapter start, including sound/dialogue handling,
is still an execution-timing question.

## Remaining design questions

1. Evaluation timing: how do occurrence triggers such as killed/left/chapter start
   combine with state predicates, and when are they evaluated relative to actions?
2. Once/repeat behavior: what is the explicit authoring rule for per-death events
   versus once-per-chapter milestones, and how is completion persisted?
3. Action ordering and cascading: can one event's effects immediately enable
   another event in the same evaluation, and how are accidental cycles bounded?
4. Hold tracking: which turn boundary counts, what resets progress, and can a
   replacement eligible mage continue the same hold?
5. Dialogue selection: when are `with` branches resolved, and what exactly is
   saved if the player reloads midway through a conversation?
6. Mark representation and appearance: how are instances stored on tiles and
   connected to fog/light/terrain presentation without replacing base terrain?
7. Referenced content: where do item definitions such as quarry-hammer live?
   Item-catalog authoring has not been finalized.
8. Objective lifecycle: how do visibility, progress, failure, required completion
   and chapter victory/defeat connect to event state and explicit AI intent?

These are unanswered design questions, not implementation requirements to fill
in without discussion. Generalized conditional actions, special ritual types and
separate objective files are not prerequisites established by this design.
