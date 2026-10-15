# Development and publication

Hex Command uses Svelte and TypeScript for the interface and Rust/WASM for battles
and enemy planning. It runs entirely in the browser. Development rules live in
[AGENTS.md](../AGENTS.md).

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

`npm run dev` prepares WASM bindings and artwork before starting Vite. Common commands:

```sh
npm run format                # Apply project formatting
npm run wasm:build            # Prepare bindings before a fresh test run
npm test                      # All test suites; warns after 20 seconds
npm run verify                # All checks and production build
npm run verify -- types tests # Selected checks
npm run build
npm run preview
```

## Verification

`npm run verify` runs automatic formatting, content validation, WASM bindings,
UI and TypeScript checks, artwork validation, all tests and the production build.
Independent checks continue after failures and report their statuses together.
Select groups as shown above. Focused `rust` selects native tests. The combined
test group already includes Rust, so verification does not run it twice.
UI and TypeScript groups share one WASM binding build during verification.

`npm test` runs Rust, Python artwork/analyzer tests and Vitest under one shared
20-second duration target. Slow runs warn and continue; only failed tests fail
the command. Vitest tests and hooks have no hard timeout. Browser automation is separate. `art:check` validates generated
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
  migrations. See [save migrations](save-migrations.md).
- `src/controller/` owns actions, autosaves, undo, turn sequencing and cancellation.
  `src/ui/` renders published views; `src/presentation/` owns artwork, motion,
  sound and icons. See [artwork rendering](#artwork-rendering-implementation).

Generated bindings live in ignored `rust/pkg/`, sprites in ignored
`public/assets/sprites/`, and production bundles in ignored `dist/`. Edit source
files. Temporary experiments and screenshots belong in ignored `tmp/`. The
developer map displays every terrain and both sides of every playable unit rank.

## Artwork rendering implementation

Pixel sources are character matrices and explicit palettes under `pixel/art/`.
Character sources are always 64×64, stored in
`pixel/art/units/<race>/<unit-id>/`. A lineup is a faction; its unit progression
tree lives in `pixel/art/units/lineups/<race>.json`. A faction can have sprites
before its units are implemented in the gameplay catalog.
Each unit folder contains only its matrix and material palette.

```sh
npm run art:build
npm run art:check
```

`scripts/build-pixel-art.py` reads the sources and emits content-hashed native
PNG sprites into ignored `public/assets/sprites/` and URL/layout metadata into
`src/presentation/pixel-art.ts`. It applies faction/floor palettes and flattens
prepared prop layouts at build time. Vite copies the sprites into `dist/assets/`.

The browser fetches individual images as they are rendered. Once an authored map
is displayed, idle work warms one canonical inspection sprite per terrain type,
two at a time with low network priority. Details use the default floor and
connections with the first prebuilt prop arrangement, regardless of the clicked
tile's map variation. The browser uses the generated sprites directly, without
an atlas or sprite cropping. Source matrices/palettes remain authoritative.

Production Vite builds render every authored map into a native PNG using
the same terrain layouts and drawing code. Unchanged maps use these sprites
directly. Terrain edits overlay changed cells until a background canvas bake
finishes; missing map sprites also use that fallback.

Terrain formats, floor palettes and prop layouts are described in
[terrain artwork implementation](terrain-artwork.md). Character creation and
editing workflows live in the artwork skills linked below.

The selected unit panel displays attack counts, damage per hit and permanent modifiers.
Rust applies armor and terrain modifiers per hit, then multiplies by count.
Terrain effects show movement and resolved incoming-damage modifiers. Numeric abilities
such as healing come from catalog metadata; descriptions explain roles without
duplicating the values. See the [authored catalog](../rust/data/README.md)
for healing, thorns and life-drain rules.

Restoration caps saved HP at the current unit maximum while preserving resolved
game state. Saves never contain queued AI actions or planning workspaces.
All data migrations belong in `src/save/migrations/` and run during loading;
unmigratable or invalid saves are wiped. See [save migrations](save-migrations.md).

Simulation reports can audit actual army composition and support use with
`npm run ai:simulate -- --unit-usage`, or add `--unit-usage` to `ai:analyze`
for a saved run. Presence includes initial troops and promotions and weights
survival by living unit-turns, excluding commanders. Engine records attribute
actual adjacent healing and thorn reflection to their sources; equal strongest
sources share credit. See the [combat-simulation skill](../.agents/skills/combat-simulation/SKILL.md)
for export definitions.

Fresh runs also export armour/terrain damage prevention, objective capture and
occupation, casualty gold and leader-risk evidence. Use analyzer options
`--battlefield-value` and `--leader-audit` to request these independently.
Leader deaths link to the last own-phase decision and retain army strength and
forecast alternatives; these distinguish evidence of collapse from exposure
without assigning an automatic causal verdict. The forecasts hold other units
fixed and do not establish that an alternative fulfills the mission.

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

## AI contracts

Named presets construct fixed
Recruiter/Campaigner/Planner compositions. Native simulations retain those
components through the whole match or campaign. `Tactic::run_turn` derives
explicit orders, applies recruitment to one draft, then plans against that updated
draft and returns ordered engine commands with recruitment and planning reports.
Recruiter owns promotion branch selection and passes explicit unit/type choices
in tactical orders. Planner decides when taking the selected promotion is safe;
it does not choose a different branch. Simulation-player recruiters can retain
gold for an expensive choice when income is available; Classic preserves the
fixed live-enemy behavior.

Policy descriptions and evaluation methodology live in the
[combat-simulation purpose](../.agents/skills/combat-simulation/purpose.md).
[Recruiter evaluation](../rust/src/ai/recruiter/evaluation.rs) owns scoring
formulas and support estimates; estimates must remain distinguishable from
recorded effects. [Army design](../.agents/skills/combat-simulation/design.md)
defines the intended faction and unit roles.

Only Campaigner receives authored mission data and progression. Recruiter and
Planner use restricted mechanical battle views. Generic capture, defeat and visit
targets supplement typed duties; mission kinds are normalized before reaching a
planner. Full and Queued use shared search and safety helpers without replacing
the selected planner. Recruiter selection rules remain independent of tactical planning.

The command-at-a-time host entry point adapts a completed turn plan to the
existing browser/native command protocol. Its queue exists only during execution
and is never saved. Browser persistence brackets the entire enemy turn; loading
an interrupted phase builds a fresh plan from its pre-turn state. Save envelopes
and host adapters remain separate from the block interfaces.

## References

- [Combat simulations and analysis](../.agents/skills/combat-simulation/SKILL.md)
- [AI preset composition](../rust/src/ai/presets.rs) and [planner dispatch](../rust/src/ai/planner/mod.rs)
- Artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)
