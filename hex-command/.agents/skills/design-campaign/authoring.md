# Campaign authoring

For the campaign author. Use the selected design and [shared design rules](world-design.md) to write player-facing content and implement the campaign.

## Player-facing elements

**Progressive disclosure.** Give the player only information the characters already know. Revealing information they do not yet know is forbidden in every channel, including briefings, hints, objectives, markers, counters and logs.

**Dialogue** drives and motivates the other elements. Dialogue must express the characters' intentions rather than grant permission for the desired gameplay. Give each exchange a clear subject and purpose. Ask: what would we lose if this dialogue were removed? Characters say things that matter to them, from their knowledge, role and circumstances. They speak to each other; the player may not fully understand them yet. Prior experience and expertise are valid sources of knowledge; a character need not discover everything on screen or explain their credentials.

**Briefing** gives short, current information about what is happening and what to do, in prose. Update it when the characters gain context. Put mechanical objectives and numbers in objectives instead.

**Hints** help a stuck player find an approach that progresses the story without revealing what it will unlock. They can, and usually should, be empty when the briefing is sufficient. Use them sparingly to guide early campaigns or chapters: “You never know what lurks in hidden places.”

## In-game objectives

Translate a task established by dialogue into a precise target and completion condition, such as “Reach X by turn N.” Reveal the objective after that dialogue; precision must still respect progressive disclosure. The mechanics need not be things characters state or know: a turn limit can quantify urgency.

This part is complete when every disclosed story fact has a source in character knowledge, dialogue establishes each task before its objective appears, and each channel serves its own purpose.

## Implementation costs

Assess the implementation cost of the proposed gameplay. Distinguish existing content, new authored data using supported mechanics, and changes requiring engine or artwork work. Inspect the relevant definitions and code before classifying a requirement. Explain what needs to be built and how substantial that work is, separately from judging whether the design is worth making. Missing content is a cost to account for, not a reason to replace the design with existing units. Apply implementation limits when the user establishes them.

## Implementation references

Read when implementing story events, characters, factions or objectives. Work from the project root and use current schemas and authored examples rather than copying a cached field list.

- **Loading and document boundaries:** `rust/catalog/src/loader.rs` and `rust/catalog/src/world.rs`; campaign `campaign.json` and stage `stage.json` files under `rust/data/game/campaigns/`.
- **Events, conditions, dialogue and objectives:** `rust/catalog/src/schema/events.rs` and `rust/catalog/src/schema/conditions.rs`, `rust/battle/src/logic/events.rs` and `rust/battle/src/logic/event_queries.rs`; current stage `scripts.json` and `story.json` files.
- **Factions and rosters:** `rust/catalog/src/schema/factions.rs` and the roster structures in `rust/catalog/src/world.rs`; current stage `roster.json` files.
- **Characters, chapter transitions and equipment:** `rust/catalog/src/schema/characters.rs`, `rust/catalog/src/schema/scenario.rs` and `rust/catalog/src/schema/units.rs`, `rust/battle/src/logic/campaign.rs` and current campaign character declarations and item definitions.

Before implementing, establish whether the shared engine expresses the intended encounter. Separate missing capability from an authoring choice. Keep campaign-specific decisions in authored content; tactical planners handle movement, combat and safety.

Reconcile the implementation with the shared design rules: starting units and carryover, encounter activation, information reveals, optional versus required tasks, success and loss. Inspect the conditions and effects together, including pending spawns and repeat behavior.

Use the project's [verification skill](../verification/SKILL.md) for validation and [combat simulation](../combat-simulation/SKILL.md) for balance claims. A static preview establishes geometry and appearance, not playability. Keep diagnostics and disposable probes in ignored `tmp/`.
