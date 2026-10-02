# Hex Command development rules

- Keep changes quick to make and review. Tests must be small, fast, and lightweight.
- The entire `npm test` command must finish within 20 seconds. Use controlled
  promises or an injected clock instead of real delays in tests. The runner
  enforces a shared time budget; do not raise it to accommodate slow tests.
- Do not add or retain Playwright, browser automation, screenshot comparisons,
  large migration/parity suites, or test infrastructure requiring browser installs,
  extra services, or Git-history fixtures. Use focused TypeScript unit checks for
  meaningful rules, state transitions, publication, and cancellation behavior.
- Components derive display state exclusively from the immutable published
  `BattleView`. Pass `BattleCommands` for event handlers. Never query mutable
  controller state during rendering or pass a mutable `Game` into a component.
- All game changes go through controller commands and publish a coherent view.
  Deferred work must check that its request/session/render revision is current
  before publishing results or applying feedback.
- Preserve the game rules, artwork, layout, and existing visible behavior. Keep
  strict TypeScript and Prettier checks passing. Keep this project self-contained.
