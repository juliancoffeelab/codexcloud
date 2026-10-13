# Hex Command development rules

- Use [docs/development.md](docs/development.md) for setup and technical references.
  The README is user-owned; modify it only when explicitly asked.

- **Permanent character canvas rule: every character/unit sprite and portrait is
  exactly 64×64 native pixels. This rule must never be changed; do not reintroduce
  alternate sizes, automatic cropping or resizing.** New generation, extraction,
  cleaning and generated frames must use 64×64. Most characters should be below
  40 painted pixels tall; cavalry and giants may go slightly above this. Compare
  mounted artwork with existing mounted units before setting scale. Distinguish
  body/mount height from raised weapons or staffs. Terrain tiles also use exactly
  64×64 native pixels, with a pointy-top regular hex centered in the square frame.
  Terrain, troops and commanders share one battlefield pixel scale; do not add
  per-unit scaling to emphasize leaders.
- **Native row 63 is the ground line. Grounded units must stand on it with no
  transparent rows beneath their ground-contacting body.** Preserve every painted
  pixel when aligning existing units: translate within the canvas, never resize
  or crop. Intentionally suspended animal poses must declare `"flying": true` in
  `palette.json`; this artwork-only flag covers hovering and swimming poses.
  Missing or false means grounded.
- Keep changes quick to make and review. Tests must be small, fast, and lightweight.
- Data-only changes do not justify new behavioral tests or extra named-content
  cases in existing test matrices.
- Validate content through generic schema and consistency checks. Measure
  balance through simulations.
- Add tests for new mechanics, changed code paths, or reproduced bugs that
  existing coverage misses. Identify that missing coverage before writing tests.
- Use a small, fixed set of representative fixtures. Do not freeze current
  unit stats, promotion choices, recruitment lists or roster sizes.
- Do not duplicate existing coverage merely because new content uses it.
  Remove redundant cases when found.
- Keep temporary data, experiments and diagnostics in ignored `tmp/`, outside
  tracked content.
- No tracked markdown file is allowed to contain information that is only relevant
  for the current work. Markdown files whose sole purpose is to track current work
  are not allowed to be in tracked places. Docs must contain permanent information.
- Run `npm run format` to apply project formatting automatically.
- Target 20 seconds for all test suites together, including Rust, Python
  and Vitest. Use the shared `npm test` runner; exceeding the target warns
  without terminating the run or failing successful tests. Use
  controlled promises or an injected clock instead of real delays.
- Keep browser automation out of the default test suite. Playwright is available
  for explicitly requested browser spot checks; keep scratch probes in ignored
  `tmp/`. Do not add mandatory browser installs, extra services, large migration
  suites, or Git-history fixtures. Use focused checks for meaningful rules,
  state transitions, publication, and cancellation behavior.
- Startup should be fast. Lazy-load what is possible. Loading failures must
  provide a way to diagnose the error and restart cleanly.
- Run combat simulations when needed to explore or validate balance. Do not wait
  for separate permission to use them during balance work.
- `dist/` contains generated output and is Git-ignored. Commit source and public
  assets. For Sites publication, build the selected checkout with `npm run build`,
  commit and push its exact source to the Sites source repository, then package
  it with `python3 scripts/package-site.py /tmp/hex-command.tar.gz`. Publish the
  archive and source commit. Do not use `git archive` for generated output; the
  packager checks client references and archive bytes.
- Use feature branches and pull requests for publication as for other changes.
  Never push directly to `main` or bypass the repository's PR workflow.
- For work lasting more than 10 minutes, commit and push the feature branch to
  GitHub before returning to the user, even if checks fail or work is unfinished.
  Report the actual check status and remaining work.
- For visual changes, deploy before pushing to GitHub so the user can test them
  sooner. If deployment fails, report it and still make the required GitHub push.
