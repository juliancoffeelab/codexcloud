# Development and publication

Development rules live in [AGENTS.md](../AGENTS.md).

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

## Verification

```sh
npm run format                # Apply project formatting
npm run wasm:build            # Prepare bindings before a fresh test run
npm test                      # All test suites; warns after 20 seconds
npm run verify                # All checks and production build
npm run verify -- types tests # Selected checks
npm run build
npm run preview
```

Verification covers formatting, content, artwork, tests and the production build.
Independent checks continue after failures and report their statuses together.
The test duration target warns without terminating a successful run. Browser
automation is separate from the default checks.

## Content and artwork

A lineup is a faction. A faction can have sprites before its gameplay roster is
implemented. Character and terrain artwork use the permanent 64×64 canvas and
shared battlefield scale defined in the development rules.

```sh
npm run art:build
npm run art:check
```

Edit authored content and artwork rather than generated output. Keep temporary
experiments and screenshots in ignored `tmp/`.

## Simulations

Use the [combat-simulation skill](../.agents/skills/combat-simulation/SKILL.md)
for commands, reports and interpretation. [Army design](../.agents/skills/combat-simulation/design.md)
defines faction identities and unit-role tradeoffs. Evaluate actual battlefield
contributions, and distinguish observed outcomes from proposed causes.

The default arena roster is Plains, Forests, Crossing, Islands and Wood Edge.
Permanent arena-only layouts are registered in the manifest's `arenaScenarios`
and kept under `rust/data/simulation/maps/`; they do not enter the playable map
picker. A scoped `--landscapes` selection must not silently replace this roster.

Sprite palette entries use straight RGBA, including partial alpha for spectral
materials. Keep faction cloth readable and preserve material alpha in all faction
variants. Native layer composition uses source-over blending without resizing.

## Publication

Use the [publish skill](../.agents/skills/publish/SKILL.md) when the user requests
deployment or an update to the live game. Simulation experiments, image previews
and Git pushes do not imply publication.

```sh
npm run publish
```

Publish the feature branch through the repository's PR workflow. Validate the
production build and publish the exact reviewed source. Keep credentials out
of logs and source control. Retain deployment identifiers if publication fails.

## Further guidance

- [Authored content](../rust/data/README.md)
- Artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)

## Battlefield presentation

An optional `presentation.json` beside an authored scenario or campaign stage
selects browser-only lighting. It contains `time` (`day` or `night`) and optional
`lights`, each with integer `col`/`row`, optional `dx`/`dy`, positive `radius` and
nonnegative `power`. A light may specify `whenOwner: "player"` or `"enemy"`
to follow the owner of its tile. Capturing or losing that tile changes the light;
loading and undo use the same ownership state. Positions and radii use battlefield coordinates; terrain
baking converts them to native pixels at `64 / battlefieldSpriteSize`. Scenarios
without this file use daylight. These settings do not change simulation rules.

Production emits finished terrain PNGs with the authored lighting and fixed
lantern fixtures. Cache keys include terrain geometry, artwork inputs and every
lighting setting. Up to four ownership-controlled lights have every combination
prebaked (at most 16 variants); larger combinations use runtime processing.
Ownership only selects a different image when a controlled light changes.
Selection and ordinary unit moves reuse the same image.
Units receive the corresponding color transform at their position, while health,
selection and interaction overlays retain their normal colors. Static light
settings need no per-turn terrain processing.

Custom or changed terrain uses the existing batched canvas bake. Uncached night
processing transfers an RGBA pixel buffer to a disposable Worker, then draws
the returned pixels and swaps the completed result. Workers need no canvas or
image APIs. Worker failure, lack of support or
timeout falls back to yielding row batches. Replaced or disposed requests cancel
processing and stale results never replace the active map. Daylight needs no
pixel processing. Development uses this same uncached path.
