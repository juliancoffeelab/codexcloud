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
- Artwork design: [unit lines](../.agents/skills/design-unit/SKILL.md),
  [terrain](../.agents/skills/design-terrain/SKILL.md).
- Shared artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)

## Battlefield presentation

An optional `lighting` object in a scenario's `map.json` selects `time` (`day`
or `night`) and `lights`. Rust loads and validates this object with the map,
includes it in the content fingerprint, and exports it through the WASM catalog.
Runtime obtains these definitions from `MAPS`; generated terrain modules contain
only opaque image keys, URLs and image dimensions. Devtools may read source JSON
to bake images, but must not export scenario definitions into runtime modules.

Each light has integer `col`/`row`, optional `dx`/`dy`, positive `radius` and
nonnegative `power`. Its optional `when` rule contains a `condition` and a
`remember` boolean. Conditions include `owner`, `proximity`, `event`, `round`,
`noUnits`, `all` and `any`. Owner/proximity conditions can omit `tile` to use the
light or fog tile. Proximity uses hex distance (`radius: 0` means occupying the
tile), side, and optional `unitType` and `name` filters. Conditions are evaluated
by Rust. `remember: false` follows current state; `true` retains satisfaction as a
durable fact. Event completion is already remembered by campaign state.

A tile's optional `fog` uses the same rule format and specifies what clears it.
Rust exports the resolved lights and fogged tile IDs in each engine view and the
initial presentation in the catalog. The frontend paints soft fog and hides
covered enemies; Rust rejects attacks on covered enemies. Remembered condition
facts persist in `conditionMemory`; resolved masks and rendered images do not.
Missing lighting defaults to daylight. Color transforms, palette colors and glow
rendering remain frontend presentation details.

Scenario events can use `trigger: {kind: "condition", condition: ...}`. An
optional `spawn` contains an ordered tile-ID `area` and units with stable `id`,
`type`, `side` and optional `name`/`role`. Activation fires once. Members appear
on free tiles they can traverse; unplaced members remain pending and retry on
subsequent gameplay updates even after the original condition becomes false.
Stable member markers prevent duplicate spawns after deaths or restoration.
`noUnits` includes activated pending members when determining whether a threat
has been cleared. Legacy visit/round/control triggers remain supported.

Production emits finished terrain PNGs with the authored lighting and fixed
lantern fixtures. Cache keys include terrain geometry, artwork inputs and every
lighting setting. Up to four condition-controlled lights have every combination
prebaked (at most 16 variants); larger combinations use runtime processing.
Ownership and event completion select a different image when a controlled light changes.
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
