# Hex Command

A client-only hex strategy game with recruitment, terrain combat, unit progression,
and four story campaigns. Svelte and TypeScript provide the interface; Rust,
compiled to WASM, runs battles and enemy planning. No account or server is needed.

## Development

Prerequisites: Node.js 22.12+, Rust stable with `rustfmt`, the
`wasm32-unknown-unknown` target, and Python 3 for the pixel artwork build.
`wasm-pack` is pinned in the npm development dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

Development builds prepare WASM bindings and artwork.
Generated bindings live in ignored `rust/pkg/`; production files live in ignored
`dist/`. Edit source files and artwork, rather than those directories.

```sh
npm run format                 # Apply TypeScript/Svelte/JSON/Rust formatting
npm run style                  # Formatting and staged/unstaged whitespace
npm run verify                 # All checks, then a production release build
npm run verify -- types tests  # Selected verification groups
npm run build                  # Release WASM, artwork and Vite bundle
npm run preview                # Serve the production bundle locally
```

`npm run verify` continues independent checks after a failure and reports the
failed groups together. Its groups are `style`, `content`, `wasm`, `ui`, `types`,
`rust`, `art`, `tests` and `build`. It never starts simulation batches.

For a focused test run after a fresh checkout, prepare the ignored bindings once:

```sh
npm run wasm:build
npm test
```

Vitest exercises browser adapters, controller transitions and compatibility rules
without a browser or server. The entire command has a 20-second budget.
`npm run check:rust` covers native rules, commands, catalog validation, saves and
recording. Playwright is available for opt-in browser spot checks; scratch probes
and screenshots belong in ignored `tmp/`, outside the default test suite.

## Code and content

- `rust/data/game/` is the authored source for units, races, terrain, scenarios and
  campaign stages. Each stage separates its map, roster, objectives, story and
  scripts. `rust/data/simulation/` contains independent arena rosters and schedules.
- `rust/src/data/`, `logic/` and `ai/` implement the catalog, battle rules and
  complete AI policies. `simulation/` runs native workers and records outcomes;
  `protocol.rs` and `wasm.rs` expose typed browser operations.
- `src/game/authored-content.ts` requests typed catalog metadata from WASM for
  display. Rust resolves all race/unit inheritance; browser code imports no live
  authored JSON. The save importer owns its historical metadata separately. `npm run content:check` checks document
  references. Native runs read JSON from disk at startup; WASM embeds it.
- `src/game/native-engine.ts` and `live-game.ts` adapt generated WASM types to
  the controller. Every live battle runs Rust; there is no TypeScript fallback.
- `src/save/` owns the versioned save envelope, storage, loading and writes.
  `migration/` contains sequential, data-only compatibility steps.
- `src/controller/` owns actions, autosaves, undo, turn sequencing and cancellation.
  `src/ui/` renders immutable published views. `src/presentation/` handles artwork,
  motion, sound and icons.

See the [Rust API and build guide](rust/README.md),
[content layout](rust/data/README.md), and [AI comparison guide](docs/ai-simulation.md).
Native checkpoints preserve enemy-planner continuation. Supported older browser
saves migrate to version 3 only after a successful native restore; rejected checkpoints are backed
up locally and leave a playable new battle.

## Playing

Capture villages for income, recruit while your leader occupies a keep, and use
terrain and damage types to counter the enemy. Ordinary recruits act on the next
turn. Combat grants experience; eligible units can advance into a second rank.
The selected campaign supplies its own required objectives, hints, dialogue and
optional rewards. Surviving troops can be recalled between chapters.

The game uses pixel artwork throughout. Sound preferences and saves stay in the
current browser. The developer map shows every terrain and both factions of
every playable unit rank.

The toolbar and battlefield frame mount while the engine
loads. Real map data and statistics appear when WASM is ready; loading failures
offer reload without replacing the saved battle.

## Artwork

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
The browser fetches individual images as they are rendered; it never downloads
an atlas, crops sprites, or encodes PNGs. Source matrices/palettes remain authoritative.
Production Vite builds also render every authored map into a native PNG using
the same terrain layouts and drawing code. Unchanged maps use these sprites
directly. Terrain edits overlay changed cells until a background canvas bake
finishes; missing map sprites also use that fallback. Generated PNGs are not tracked.
Art workflows live in short skills: [generation](.agents/skills/pinkbox-generation/SKILL.md),
[enhancement](.agents/skills/pinkbox-enhancement/SKILL.md),
[cleaning](.agents/skills/pixel-cleaning/SKILL.md), and
[lineup previews](.agents/skills/pixel-lineup/SKILL.md).

## Simulations and publication

Simulations are opt-in and separate from game verification:

```sh
npm run ai:schedule -- --mode full --seeds 10 # Enumerate only
npm run ai:full -- --out artifacts/full.json # Run arenas and campaigns
```

Rust owns scheduling, workers and reporting. Complete gameplay records are saved
in indexed JSONL journals alongside compact summaries. The comparison tools
convert saved journals to Arrow with an explicit schema and compare unordered
match records without rerunning matches. See the [AI guide](docs/ai-simulation.md)
for filters, replay and comparison commands.

For the configured Sites deployment, `npm run publish` builds the release,
commits and pushes its source, then prepares the deployment archive and manifest.
The tool adapter in `scripts/publish-sites-call.js` submits that prepared archive
and waits for deployment. Publication is separate from verification and requires
an explicit request.
