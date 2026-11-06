# Hex Command instructions

Startup should be fast and show as much of final UI to the user as possible as
early as possible, by lazy-loading unnecessary parts.
The data should be prefetched, user must never wait for the data loading.
Loading failures must provide a full backtrace and a way to restart cleanly.

Runtime code obtains current authored game data, including scenarios and their
settings, through Rust. Frontend JSON imports or generated game-data modules must
not bypass Rust. Pure presentation details may live in the frontend; devtools may
read source data directly.

For battlefield rendering changes, preserve the
[native artwork scale](.agents/skills/design-unit/SKILL.md#native-artwork) and
[offline terrain contract](.agents/skills/design-terrain/SKILL.md#terrain-contract).

# Skills

- [live-play](.agents/skills/live-play/SKILL.md): Use only when the user explicitly authorizes live browser gameplay.

- [testing](.agents/skills/testing/SKILL.md): Use for bug fixes, test changes and implementation validation.
- [save-migrations](.agents/skills/save-migrations/SKILL.md): Use when changing or
  auditing save loading, save formats, migrations, or their data and dependencies.
- [deploy](.agents/skills/deploy/SKILL.md): Use for an intended live human playtest, or when
  the user asks to deploy, publish or update the live game.
- [combat-simulation](.agents/skills/combat-simulation/SKILL.md): Use when asked
  to review, explain, compare or change AI policies; run or analyze combat
  simulations; change simulation harnesses or analyzers; or assess balance, campaign competence or simulation performance.
- [design-campaign](.agents/skills/design-campaign/SKILL.md): Use to design or
  revise campaigns and scenarios, including writing, objectives, maps, encounters,
  progression, and supporting mechanics.
- [design-unit](.agents/skills/design-unit/SKILL.md): Use when creating, revising
  or integrating unit artwork, promotion branches or shared sprite scale, or
  previewing and comparing faction lineups.
- [design-terrain](.agents/skills/design-terrain/SKILL.md): Use when authoring,
  revising or integrating terrain artwork, connected variants or shared tile geometry.
- [pinkbox-generation](.agents/skills/pinkbox-generation/SKILL.md): Use when
  generating or regenerating native pixel artwork from reference matrices,
  including units, terrain and foreground props.
- [pixel-cleaning](.agents/skills/pixel-cleaning/SKILL.md): Use when inspecting,
  cleaning or repairing native pixel artwork, including units, terrain and props.
- [artwork-enhancement](.agents/skills/artwork-enhancement/SKILL.md): Use when
  revising approved pixel artwork, including manual edits, recoloring,
  regeneration, unit upgrades and terrain changes.
- [rust-toolchain](.agents/skills/rust-toolchain/SKILL.md): Use when building,
  testing or formatting Rust code, compiling WASM or simulation binaries, or
  troubleshooting Cargo, Rustup or wasm-pack.
