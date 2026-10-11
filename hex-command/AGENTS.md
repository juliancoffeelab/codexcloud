# Hex Command development rules

- Store each pixel unit in `pixel/art/units/<race>/<unit-id>/`, with its
  only `sprite.txt` and `palette.json`. L2/L3 variants have
  their own unit-ID folders. Store race trees separately in
  `pixel/art/units/lineups/<race>.json`; each entry's `source` equals its unit ID.
  Do not nest sprites under lineup folders or restore family/tier directories.
  Keep generated candidates, prompts, extraction reports and edit logs in ignored
  `tmp/`; do not commit art archives, duplicate renders or reference-only poses.

- **Permanent character canvas rule: every character/unit sprite and portrait is
  exactly 64×64 native pixels. This rule must never be changed; do not even try
  to reintroduce 48×48 outputs, alternate sizes, automatic cropping or resizing.**
  The one-time migration places each old 48×48 matrix at `(8,16)`: 8 transparent
  pixels left, 8 right, 16 above, none below. Preserve every painted pixel.
  New generation, extraction, cleaning and atlas frames must use 64×64.
  Most characters should be below 40 painted pixels tall; cavalry and giants may
  go slightly above this. Compare mounted artwork with the existing rider,
  horse-archer, elf deer rider and skeletal rider references before setting scale;
  distinguish body/mount height from a raised weapon or staff. Canvas size is not
  permission to enlarge ordinary infantry. Terrain hex tiles use their separate
  terrain grid. During generation, keep raw evidence unchanged in `tmp/`.

- Keep changes quick to make and review. Tests must be small, fast, and lightweight.
- Matrices and palettes in `pixel/art/` are authoritative. `npm run art:build`
  packs them with `scripts/build-pixel-atlas.py`; builds must never replay
  recipes, regenerate characters or write source artwork. Keep generation,
  cleaning and preview scratch output in `tmp/`.
- Run `npm run style` for Prettier and staged/unstaged whitespace checks together.
  Use `npm run format` to apply formatting; do not repeat individual style checks.
- The entire `npm test` command must finish within 20 seconds. Use controlled
  promises or an injected clock instead of real delays in tests. Vitest runs
  independent named scenarios; the batch launcher enforces a shared time budget; do not raise it to accommodate slow tests.
- Keep browser automation out of the default test suite. Playwright is available
  for explicitly requested browser spot checks; keep scratch probes in ignored
  `tmp/`. Do not add mandatory browser installs, extra services, large migration
  suites, or Git-history fixtures. Use focused unit checks for meaningful rules,
  state transitions, publication, and cancellation behavior.
- Components derive display state exclusively from the immutable published
  `BattleView`. Pass `BattleCommands` for event handlers. Never query mutable
  controller state during rendering or pass a mutable `Game` into a component.
- All game changes go through controller commands and publish a coherent view.
  Deferred work must check that its request/session/render revision is current
  before publishing results or applying feedback.
- Preserve the game rules, artwork, layout, and existing visible behavior. Keep
  strict TypeScript and Prettier checks passing. Keep this project self-contained.
- Campaign and tournament simulations require the user's explicit instruction to
  run them. Verification must not start those batches as an extra finishing step.
- `dist/` contains generated output and is intentionally Git-ignored. Commit the
  source and public assets. For Sites publication, build the selected Site checkout
  with `npm run build`, commit and push its exact source, then package it with
  `python3 scripts/package-site.py /tmp/hex-command.tar.gz`. Publish the returned
  archive and source commit. Do not use `git archive` for generated output: it
  omits ignored bundles. The packager checks client references and archive bytes.
- For an authorized publication, use `scripts/publish-sites-call.js` in one
  `functions.exec` call instead of manually repeating the publication steps.
  Read the adapter with `tools.exec_command`, evaluate its exported function,
  and invoke it with the native `tools`, project `cwd`, a commit message and
  `notify` for progress. The adapter runs `npm run publish`, requests credentials,
  saves/deploys and polls to completion. Example body (set `cwd` to this project):

  ```js
  const source = await tools.exec_command({
    cmd: "cat scripts/publish-sites-call.js",
    workdir: cwd,
    max_output_tokens: 8000,
  });
  if (source.exit_code !== 0) throw new Error(source.output);
  const publish = new Function("return (" + source.output.replace("export default ", "") + ")")();
  text(await publish(tools, { cwd, message: "Describe the changes" }, notify));
  ```

  Never print or persist the credential response. Report a native deployment
  error without blindly repeating save/deploy; retain any saved version or
  deployment IDs returned by the service for recovery.

- Atlases are inputs to artwork loading only. Publish isolated frame images in
  `BattleView.artwork`; renderers must consume that snapshot. Do not put a full
  sheet in SVG, HTML or a CSS background and rely on overflow/viewports to crop it.
  Extract pixel frames at native resolution with smoothing disabled.
