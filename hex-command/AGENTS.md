# Hex Command development rules

- Use [docs/development.md](docs/development.md) for setup and technical references.
  The README is user-owned; modify it only when explicitly asked.
- Keep simulation experiments separate from the playable game. Requests to
  prototype, render, simulate or evaluate a map or balance change do not authorize
  adding it to the playable catalog. Use an ignored copy of the data and
  `--data-dir`; promote experimental content only when the user requests it.
- Publication requires a user instruction to publish, deploy or update the live
  game. Building, testing, showing an image, committing and pushing do not imply
  publication. Visual changes and time spent working are not publication triggers.
  Follow the [publish skill](.agents/skills/publish/SKILL.md) when publication is
  authorized; do not invoke the hosted-site workflow for simulation-only work.

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
- After changing the simulation harness, analyzer or AI behavior, run fresh
  simulations and include the report directly in the final reply without waiting
  for another request. A successful run or test status alone is not delivery.
  Report the affected workload; use the full panel when scope is unclear.
- Use feature branches and pull requests for publication as for other changes.
  Never push directly to `main` or bypass the repository's PR workflow.
- Commit and push before returning only if the current task took more than
  10 minutes, even if unfinished or checks fail. Otherwise, keep changes local.
  Report check status and remaining work.

## Skills

Read the linked `SKILL.md` before starting work that matches its trigger.

- [publish](.agents/skills/publish/SKILL.md): Use when the user authorizes
  publication, deployment or an update to the live game. Simulation, preview,
  commit and push requests alone do not trigger this skill.
- [combat-simulation](.agents/skills/combat-simulation/SKILL.md): Use when asked
  to review, explain, compare or change AI policies; run or analyze combat
  simulations; or assess balance, campaign competence or simulation performance.
- [pinkbox-enhancement](.agents/skills/pinkbox-enhancement/SKILL.md): Use when
  asked to recolor or redesign existing unit artwork, create upgrade artwork
  from an approved unit, or recolor terrain floors.
- [pinkbox-generation](.agents/skills/pinkbox-generation/SKILL.md): Use when
  asked to generate a new pixel unit or regenerate a unit's shape, pose or
  equipment through image generation.
- [pixel-cleaning](.agents/skills/pixel-cleaning/SKILL.md): Use when asked to
  inspect, clean or repair sprite or terrain pixels, outlines, shading, ground
  alignment or terrain joins.
- [pixel-lineup](.agents/skills/pixel-lineup/SKILL.md): Use when asked to show,
  preview or visually compare faction lineups, unit upgrade trees or progression
  grids.
- [rust-toolchain](.agents/skills/rust-toolchain/SKILL.md): Use when building,
  testing or formatting Rust code, compiling WASM or simulation binaries, or
  troubleshooting Cargo, Rustup or wasm-pack.
