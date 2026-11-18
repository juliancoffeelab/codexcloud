# Rust engine principles

## Authored rules and validation

Data owns game content and rules. Implement their execution generically from
authored JSON. Adding a hardcoded gameplay rule requires explicit user
authorization.

Defaults are forbidden. Required mechanical values must come from authored
map, terrain, unit or race JSON, including explicitly declared inheritance.
Missing or invalid data must fail catalog validation before gameplay starts.
Focused data-validation tests must load and validate the complete authored
catalog. Loading, restoration and runtime code must not invent missing values.

The Rust engine accepts only the latest data and save formats. The web layer
supplies up-to-date saves through its versioned migrations. Keep historical
compatibility at that boundary; follow the
[save-migrations skill](../.agents/skills/save-migrations/SKILL.md) when changing
the contract.

## Responsibility boundaries

Blocks communicate through narrow interfaces that expose their capabilities and
the information their consumers need.

| Block                  | Responsibility                                                                                                                          |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Map / battle mechanics | Own physical state and execute authored movement, combat and perception rules. Expose mechanical queries and legal commands.            |
| Event Engine           | Evaluate authored conditions and execute authored actions. Own campaign progression and authoritative objective completion.             |
| Campaigner             | Own strategic intent. Translate objectives known to the faction into priorities, assignments and constraints for Planner and Recruiter. |
| Planner                | Own tactical action selection and search. Carry out Campaigner guidance through legal action proposals.                                 |
| Recruiter              | Own force composition, recruitment selection and placement, and promotion options within available resources and Campaigner guidance.   |

Campaigner is the strategic top block. Planner and Recruiter consume its guidance
rather than independently interpreting campaign scripts. Event execution uses
authoritative facts; AI reasoning receives faction-appropriate observations.

## AI entrypoints

[Tactic::run_turn](src/ai/runtime/tactic.rs) owns turn orchestration: obtain
Campaigner orders, call Recruiter, then call Planner. Strategic ownership does
not make Campaigner the execution driver.

[play_blocks](src/ai/runtime/mod.rs) drives native play and simulation;
[choose_game_command](src/ai/runtime/mod.rs) supplies command-by-command execution.
The runtime applies proposals through battle mechanics, which also drives event
execution. Keep orchestration separate from the blocks' policy decisions.
