# Hex Command instructions

The README is user-owned; modify it only when explicitly asked.

**Cornerstone rule: never resolve merge or rebase conflicts autonomously.**
Resolve conflicts only in response to an explicit user request for conflict
resolution. Do not initiate a merge, rebase, cherry-pick or branch synchronization
to address known conflicts without that explicit request. Requests to implement,
integrate, push, publish or prepare a PR do not imply permission to resolve
conflicts. If conflicts block requested work, report them and stop the affected
operation.

Commit and push completed requested changes by default through the current
feature branch and pull request. Honor explicit requests to keep changes local
or hold a push. For unfinished work, commit and push after more than 10 minutes
unless the user requests otherwise; report check status and remaining work.
Continue on the current branch and PR unless asked to change branches. New
branches start from the freshly fetched upstream default branch (`main`), never
a stale local copy.

Diagnose the underlying cause and fix the general rule or mechanism instead of
special-casing the current scenario. Keep changes quick to make and review.

Keep temporary data, experiments and diagnostics in ignored `tmp/`, outside
tracked content. Tracked Markdown must contain permanent information; keep
current-work tracking and diagnostics out of tracked Markdown files.

Startup should be fast. Lazy-load what is possible. Loading failures must
provide a way to diagnose the error and restart cleanly.

When the user interrupts ongoing work, stop that work and keep it paused. Address
only the interruption and explicitly requested follow-up. Resume the earlier
implementation, checks, commits or deployment only when the user asks to continue.

# Skills

- [testing](.agents/skills/testing/SKILL.md): Use for bug fixes, test changes and implementation validation.
- [battlefield-contract](.agents/skills/battlefield-contract/SKILL.md): Use when changing unit artwork, terrain authoring, battlefield rendering or runtime game-data ownership.
- [save-migrations](.agents/skills/save-migrations/SKILL.md): Use when changing or
  auditing save loading, save formats, migrations, or their data and dependencies.
- [publish](.agents/skills/publish/SKILL.md): Use when the user requests an
  unpublished release.
- [deploy](.agents/skills/deploy/SKILL.md): Use for an intended live human playtest, or when
  the user asks to deploy, publish or update the live game.
- [combat-simulation](.agents/skills/combat-simulation/SKILL.md): Use when asked
  to review, explain, compare or change AI policies; run or analyze combat
  simulations; change simulation harnesses or analyzers; or assess balance, campaign competence or simulation performance.
- [design-campaign](.agents/skills/design-campaign/SKILL.md): Use to design or
  revise campaigns and scenarios, including writing, objectives, maps, encounters,
  progression, and supporting mechanics.
- [design-unit](.agents/skills/design-unit/SKILL.md): Use when designing or
  integrating unit artwork, promotion branches or shared sprite scale.
- [design-terrain](.agents/skills/design-terrain/SKILL.md): Use when authoring,
  revising or integrating terrain artwork, connected variants or shared tile geometry.
- [pinkbox-generation](.agents/skills/pinkbox-generation/SKILL.md): Use when
  generating or regenerating native pixel artwork from reference matrices,
  including units, terrain and foreground props.
- [pixel-cleaning](.agents/skills/pixel-cleaning/SKILL.md): Use when inspecting,
  cleaning or repairing native pixel artwork, including units, terrain and props.
- [pinkbox-enhancement](.agents/skills/pinkbox-enhancement/SKILL.md): Use when
  revising approved pixel artwork manually or through another pinkbox generation,
  including unit upgrades and terrain changes.
- [pixel-lineup](.agents/skills/pixel-lineup/SKILL.md): Use when asked to show,
  preview or visually compare faction lineups, unit upgrade trees or progression
  grids.
- [rust-toolchain](.agents/skills/rust-toolchain/SKILL.md): Use when building,
  testing or formatting Rust code, compiling WASM or simulation binaries, or
  troubleshooting Cargo, Rustup or wasm-pack.
