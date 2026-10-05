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
- Before announcing final verification, freeze implementation. Run required checks
  once; fix concrete failures and rerun only affected checks. Put unrelated
  improvements in TODO.md. Do not restart broad review or add features after this
  cutoff. Report a failed check explicitly instead of continuing to say "done".
  Once required checks pass, commit/push the authorized work and respond promptly.
- Campaign and tournament simulations require the user's explicit instruction to
  run them. Verification must not start those batches as an extra finishing step.
- `dist/` contains generated output and is intentionally Git-ignored. Commit the
  source and public assets. For Sites publication, build the selected Site checkout
  with `npm run build`, commit and push its exact source, then package it with
  `python3 scripts/package-site.py /tmp/hex-command.tar.gz`. Publish the returned
  archive and source commit. Do not use `git archive` for generated output: it
  omits ignored bundles. The packager checks client references and archive bytes.
- Atlases are inputs to artwork loading only. Publish isolated frame images in
  `BattleView.artwork`; renderers must consume that snapshot. Do not put a full
  sheet in SVG, HTML or a CSS background and rely on overflow/viewports to crop it.
  Extract pixel frames at native resolution with smoothing disabled.
