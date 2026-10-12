# Development and publication

This is the setup and technical reference for agents. The project README is
user-owned; change it only when the user asks.

Hex Command uses Svelte and TypeScript for the interface and Rust/WASM for battles
and enemy planning. It runs entirely in the browser.

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

Development prepares WASM bindings and artwork. Common commands:

```sh
npm run format                # Apply project formatting
npm run wasm:build            # Prepare bindings before a fresh test run
npm test                      # All test suites; shared 20-second budget
npm run verify                # All checks and production build
npm run verify -- types tests # Selected checks
npm run build
npm run preview
```

## Verification

`npm run verify` runs automatic formatting, content validation, WASM bindings,
UI and TypeScript checks, artwork validation, all tests and the production build.
Independent checks continue after failures and report their statuses together.
Select groups with `npm run verify -- types tests`. Legacy `style` selects
formatting; focused `rust` selects native tests. The combined test group already
includes Rust, so verification does not run it twice.

Prepare ignored bindings with `npm run wasm:build` before a fresh test run.
`npm test` runs Rust, Python artwork/analyzer tests and Vitest under one shared
20-second budget. Browser automation is separate. `art:check` validates generated
artwork; its unit tests run in the combined test command.

## Project layout

- `rust/data/` contains authored game content and simulation rosters/schedules.
  The [content reference](../rust/data/README.md) describes schemas and inheritance.
- `rust/src/data/`, `logic/` and `ai/` own the catalog, battles and planning.
  `simulation/` owns scheduling and recording; `protocol.rs` and `wasm.rs` expose
  the browser contract. See the [Rust guide](../rust/README.md).
- `src/game/` adapts the WASM factory, catalog and battle handles. Browser code
  receives resolved metadata rather than importing live authored JSON.
- `src/save/` owns versioned envelopes, storage and sequential compatibility
  migrations. See [save formats](save-formats.md).
- `src/controller/` owns actions, autosaves, undo, turn sequencing and cancellation.
  `src/ui/` renders published views; `src/presentation/` owns artwork, motion,
  sound and icons. See [artwork rendering](#artwork-rendering-implementation).

Generated bindings live in ignored `rust/pkg/`, sprites in ignored
`public/assets/sprites/`, and production bundles in ignored `dist/`. Edit source
files. Temporary experiments and screenshots belong in ignored `tmp/`.

## Artwork rendering implementation

Pixel sources are character matrices and explicit palettes under `pixel/art/`.
Character sources are always 64×64, stored in
`pixel/art/units/<race>/<unit-id>/`; race trees live in `pixel/art/units/lineups/`.
Each unit folder contains only its matrix and material palette. Generation
history and previews stay in ignored `tmp/`, outside the committed sources.

```sh
npm run art:build
npm run art:check
```

The read-only build, `scripts/build-pixel-art.py`, emits content-hashed native
PNG sprites into ignored `public/assets/sprites/` and URL/layout metadata into
`src/presentation/pixel-art.ts`. It applies faction/floor palettes and flattens
prepared prop layouts at build time. Vite copies the sprites into `dist/assets/`.
The browser fetches individual images as they are rendered. Once an authored map
is displayed, idle work warms one canonical inspection sprite per terrain type,
two at a time with low network priority. Details use the default floor and
connections with the first prebuilt prop arrangement, regardless of the clicked
tile's map variation. It never downloads
an atlas, crops sprites, or encodes PNGs. Source matrices/palettes remain authoritative.
Production Vite builds also render every authored map into a native PNG using
the same terrain layouts and drawing code. Unchanged maps use these sprites
directly. Terrain edits overlay changed cells until a background canvas bake
finishes; missing map sprites also use that fallback. Generated PNGs are not tracked.

Terrain formats, floor palettes and prop layouts are described in
[terrain artwork implementation](terrain-artwork.md). Character
creation and editing workflows live in the [generation](../.agents/skills/pinkbox-generation/SKILL.md),
[enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
[cleaning](../.agents/skills/pixel-cleaning/SKILL.md) and
[lineup](../.agents/skills/pixel-lineup/SKILL.md) skills.

The selected unit panel displays attacks and permanent damage modifiers. Terrain
effects show movement and resolved incoming-damage modifiers. Numeric abilities
such as healing come from catalog metadata; descriptions explain roles without
duplicating the values.

## Sites publication

`scripts/publish-sites-call.js` coordinates the configured Sites deployment.
It invokes `npm run publish`, which commits project source, synchronizes a Sites
source checkout, builds that checkout and packages its generated output with
`scripts/package-site.py`. The adapter saves/deploys the archive, polls deployment
status and pushes the deployed source commit to GitHub after success.

The Sites source repository receives its commit before deployment; GitHub is a
separate push. Packages include generated files that ordinary `git archive`
would omit. Packaging verifies references, file coverage and archive bytes.
Credentials are passed to the publication command without being printed or saved.
On failure, retain version/deployment identifiers for recovery.

Publication and push policy is defined in [AGENTS.md](../AGENTS.md).

## Game overview

Capture villages for income and recruit while your leader occupies a keep.
Ordinary recruits act on the next turn. Use terrain and damage types to counter
the enemy; combat grants experience and eligible units can advance. Campaigns
provide their own objectives and let survivors be recalled between chapters.
Saves and sound preferences stay in the browser. The developer map displays
every terrain and both sides of every playable unit rank.

## References

- [Rust API and build guide](../rust/README.md)
- [Authored content](../rust/data/README.md)
- [Save formats and migrations](save-formats.md)
- [Combat simulations and analysis](../.agents/skills/combat-simulation/usage.md)
- [AI policy implementation](../rust/src/ai/README.md)
- [Artwork rendering](#artwork-rendering-implementation) and [terrain sources](terrain-artwork.md)
- Artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)
