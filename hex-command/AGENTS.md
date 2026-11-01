# Hex Command development rules

- Use [docs/development.md](docs/development.md) for setup and technical references.
  The README is user-owned; modify it only when explicitly asked.
- Keep simulation experiments separate from the playable game. Requests to
  prototype, render, simulate or evaluate a map or balance change do not authorize
  adding it to the playable catalog. Use an ignored copy of the data and
  `--data-dir`; promote experimental content only when the user requests it.
- Commit and push completed requested repository changes by default, regardless
  of task duration, through the current feature branch and pull request. Honor
  explicit requests to keep changes local or hold a push.
- Deploy only when the user asks to deploy/update the live game, or the task
  explicitly intends a live build for a human to playtest. Editing playable game
  content, running simulations, or pushing changes alone does not establish that
  intent. If human playtesting is not established, finish with the pushed changes.
  For an intended human playtest, complete appropriate implementation validation
  and commit/push first, then use the [deploy skill](.agents/skills/deploy/SKILL.md)
  and return the live URL. Honor local-only, unpublished-release, and deployment
  holds. Reviews, documentation, tooling-only work, drafts, renders, and simulations
  do not trigger deployment on their own.
- An explicit deploy request is an immediate execution request, not a request
  for more development or validation. Invoke the deploy operation directly;
  run no separate tests, inspections, Git commands, builds or previews. The
  operation owns any necessary preparation and build. Use the
  [publish skill](.agents/skills/publish/SKILL.md) only for a requested unpublished
  release. Deployment sequencing belongs to the deploy skill.
  Deployment uses the current committed snapshot (`HEAD`); uncommitted changes
  are left alone, not checked as a prerequisite or automatically committed.
- A real deployment failure authorizes fixing its cause and retrying, without
  another permission request. Before any investigation or repair tool call,
  announce the failed stage/error, the next repair action and why deployment is
  delayed. Fix only the deployment blocker, commit/push required source changes,
  then rerun the deploy operation and deliver its successful URL. This is recovery
  after an observed failure, never permission to add preflight checks or a general
  test/review cycle. Follow the deploy skill for recovery and external blockers.

- **Permanent character canvas rule: every character/unit sprite and portrait is
  exactly 64×64 native pixels. This rule must never be changed; do not reintroduce
  alternate sizes, automatic cropping or resizing.** New generation, extraction,
  cleaning and generated frames must use 64×64. Most characters should be below
  40 painted pixels tall; cavalry and giants may go slightly above this. Compare
  mounted artwork with existing mounted units before setting scale. Distinguish
  body/mount height from raised weapons or staffs. Terrain uses a 64×64 logical hex at the same native pixel scale. Explicit
  terrain overflow may add transparent canvas padding with an authored anchor
  and drawing depth; it must never resize the hex or artwork.
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
- Do not add permanent tests unless you can guarantee that they will fail in
  the future. A reproduced bug alone does not justify a permanent regression test.
- For bug fixes, use a disposable test or probe: reproduce the failure, fix the
  code, verify that the probe passes, then delete it. Run the relevant existing
  suites and remove obsolete expectations rather than preserving incorrect behavior.
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
- Continue on the current feature branch and pull request unless the user asks
  to change branches. New branches must start from the freshly fetched upstream
  default branch (`main` here), never a stale local copy. Never push directly to
  `main` or bypass the repository's PR workflow.
- Before returning completed repository work, commit and push as described above;
  report check status and remaining work. For unfinished work, commit and push
  after more than 10 minutes unless the user requests otherwise. An explicit
  deploy request follows the operation-only rule above, including for source
  synchronization.

## Interruptions

When the user interrupts ongoing work, stop that work and keep it paused. Address
only the interruption and any explicitly requested follow-up. Do not resume the
earlier implementation, checks, commits or deployment until the user asks you to
continue.

## Permanent prohibition on map post-processing

Map post-processing is absolutely forbidden except for weather and atmospheric effects such as fog and lighting. Under no circumstances may this rule be changed, weakened, bypassed or given an exception. Runtime must select complete, prebuilt terrain frames from each tile’s explicit data. It must never assemble structures across the map, inspect neighboring geometry to inject artwork, distribute fragments, or paint terrain features onto other tiles. All terrain geometry and composition must be finished offline and baked into the selected tile frames.

## Connected terrain

Connected terrain must declare explicit geometry: castles use `walls`;
other connected terrain uses `connections`. Use numbered
`<mask>.txt` sources for every supported combination. Implicit orientation
defaults, special filename aliases such as `tile.txt` for one connected mask,
and unsuffixed connected frames are explicitly forbidden. Missing or invalid
geometry must fail; do not silently substitute, infer or backfill it in
runtime rendering or validation. This applies to all connected terrain,
including roads, crossings, settlements and castles. Unconnected terrain and
individual prop sources may use `tile.txt`.

## Runtime data ownership

Runtime code must obtain current authored game data, including scenarios and their
settings, through Rust. Do not bypass Rust by importing JSON or injecting game
data through generated frontend modules. Pure rendering details such as terrain
colors and shading may live in the frontend. Devtool scripts may read source data
directly.

## Failures

When a user asks to diagnose agent failure, you need to diagnose it. Failed
instruction is a failure of instruction, never the agent. Gaps or ambiguity in
instructions are instruction failures too. Prefer finding ways to fix code or
architecture to prevent recurrence, unless the cause is clearly a bad instruction.
For instruction failures, identify how the instruction should be edited or removed,
or, in rarer cases, extended.

## Skills

Read the linked `SKILL.md` before starting work that matches its trigger.

- [save-migrations](.agents/skills/save-migrations/SKILL.md): Use when changing or
  auditing save loading, save formats, migrations, or their data and dependencies.
- [publish](.agents/skills/publish/SKILL.md): Use when the user requests an
  unpublished release.
- [deploy](.agents/skills/deploy/SKILL.md): Use for an intended live human playtest, or when
  the user asks to deploy, publish or update the live game.
- [combat-simulation](.agents/skills/combat-simulation/SKILL.md): Use when asked
  to review, explain, compare or change AI policies; run or analyze combat
  simulations; or assess balance, campaign competence or simulation performance.
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
