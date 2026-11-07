# Development

Standing instructions and skill invocation conditions are in
[AGENTS.md](../AGENTS.md). Current npm commands are defined in
[package.json](../package.json).

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

For Rust/WASM tool discovery and proxy setup, use
[rust-toolchain](../.agents/skills/rust-toolchain/SKILL.md).

## Verification

```sh
npm run format
npm run wasm:build            # Prepare bindings before a fresh test run
npm test
npm run verify               # All default checks and production build
npm run verify -- types tests
npm run build
npm run preview
```

Verification continues independent checks after failures and reports their
statuses together. Browser automation is separate from the default checks.
Use [testing](../.agents/skills/testing/SKILL.md) for validation scope and test policy.

## Task references

- [Rust core](../rust/README.md) for engine architecture and the browser bridge.
- [Authored content](../rust/data/README.md) for game data ownership.
- [Campaign design](../.agents/skills/design-campaign/SKILL.md) for scenarios,
  objectives, events and story.
- [Unit design](../.agents/skills/design-unit/SKILL.md) and
  [terrain design](../.agents/skills/design-terrain/SKILL.md) for artwork.
- [Combat simulation](../.agents/skills/combat-simulation/SKILL.md) for AI,
  balance experiments and reports.
- [Save migrations](../.agents/skills/save-migrations/SKILL.md) for persistence.
- [Deploy](../.agents/skills/deploy/SKILL.md) for live publication.
- [Live play](../.agents/skills/live-play/SKILL.md) for explicitly authorized
  browser gameplay.

## Performance diagnostics

Settings → Debug tools provides **Start recording trace**, **Stop recording**
and **Download trace**. Start closes Settings; reopen it to stop or download.
Downloading during capture stops and flushes the recording. Recordings stay in
memory until reload, and starting another replaces the previous recording.
Nothing is uploaded automatically.

The exported Chrome/Perfetto trace includes engine and browser timings,
render workload, optional GPU timings and start/end battle state. Inspect its
reported dropped events and flush failures before drawing conclusions.
Browser callback intervals are not physical display frame rates; API submission
time is not GPU execution time.

Software GPU results do not establish hardware frame rates. Draw readiness,
completed GPU work, an animation-frame opportunity and physical display scanout
are distinct. Completion measurements using a GPU fence include polling overhead;
optional asynchronous GPU queries measure execution separately. Trace recording
must not add completion fences.
