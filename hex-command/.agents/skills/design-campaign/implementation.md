# Campaign implementation

Read when changing playable scenarios or supporting mechanics, after establishing
the intended situation and player actions in [SKILL.md](SKILL.md).

Review the changed area against the previous map. Use the unit/terrain design
skills for artwork and the normal game renderer for geometry and connections.
Inspect Rust schemas and existing scenario logic before claiming support;
separate existing capabilities, missing shared mechanics and undecided values.
Follow [project instructions](../../../AGENTS.md) for runtime data ownership.

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

An implementation completes when the starting state and changed
encounter outcomes agree with the writing, the applicable content/mechanics checks
pass, and balance claims have simulation evidence. Keep review evidence in ignored
tmp/; use disposable probes where needed rather than adding permanent named-content
tests. Report unresolved mismatches explicitly.
