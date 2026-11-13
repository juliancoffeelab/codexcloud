# Campaign authoring references

Read when implementing story events, characters, factions or objectives. Work from the project root and use current schemas and authored examples rather than copying a cached field list.

- **Loading and document boundaries:** `rust/catalog/src/data/catalog_documents.rs`; campaign `campaign.json` and stage `stage.json` files under `rust/data/game/campaigns/`.
- **Events, conditions, dialogue and objectives:** `rust/catalog/src/data/events.rs`, `rust/battle/src/logic/events.rs` and `event_queries.rs`; current stage `scripts.json` and `story.json` files.
- **Factions and rosters:** `rust/catalog/src/data/factions.rs` and the roster structures in `catalog_documents.rs`; current stage `roster.json` files.
- **Characters, chapter transitions and equipment:** `rust/catalog/src/data/scenario.rs`, `rust/battle/src/logic/campaign.rs` and current campaign character declarations and item definitions.

Before implementing, establish whether the shared engine expresses the intended encounter. Separate missing capability from an authoring choice. Keep campaign-specific decisions in authored content; tactical planners handle movement, combat and safety.

Reconcile the implementation with the design skill's rules: starting units and carryover, encounter activation, information reveals, optional versus required tasks, success and loss. Inspect the conditions and effects together, including pending spawns and repeat behavior.

Use the project's [testing skill](../testing/SKILL.md) for validation and [combat simulation](../combat-simulation/SKILL.md) for balance claims. A static preview establishes geometry and appearance, not playability. Keep diagnostics and disposable probes in ignored `tmp/`.
