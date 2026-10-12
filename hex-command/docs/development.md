# Development and publication

Hex Command uses OCaml/Melange and ReasonReact for the interface and Rust/WASM for battles
and enemy planning. It runs entirely in the browser. Development rules live in
[AGENTS.md](../AGENTS.md).

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
Python 3 and Dune 3.24.2 (pinned in `.dune-version`). React and ReactDOM install
through npm. Dune uses upstream package metadata and locks application OCaml
5.3.0, Melange and ReasonReact; no separate application Opam switch is needed.

With an existing Opam setup, run `opam install dune.3.24.2`. Alternatively, put a
bootstrap OCaml compiler 4.14+ on `PATH` and run:

```sh
bash scripts/install-dune.sh "$HOME/.local"
export PATH="$HOME/.local/bin:$PATH"
dune --version                 # Expected: 3.24.2
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

The source bootstrap needs `curl`, `tar`, Make and a C/C++ toolchain; use
`build-essential` on Debian/Ubuntu, Xcode Command Line Tools on macOS, or WSL on
Windows. Add the PATH line to your shell startup file and check `command -v dune`
for an older executable that might shadow it. The installer verifies the upstream
release checksum and TLS. Initial package builds need access to their upstream
archive hosts, including Erratique and Inria.

`npm run dev` prepares WASM, regenerates OCaml ABI bindings, builds artwork and
compiles the frontend, then supervises Dune's watcher and Vite together. The first
compiler/package build can take several minutes; preparation prints its stage
and elapsed time every 15 seconds. Later builds reuse the toolchain. OCaml and
Reason edits rebuild automatically and Vite reloads the emitted modules; CSS
uses normal hot updates. After startup, compilation errors leave the watcher
alive: fix the source to recover. The browser can retain previous compiled code
until that rebuild succeeds. Initial preparation failures require restarting.
Ctrl-C or an unexpected service exit releases both process trees.

Vite options pass through, for example `npm run dev -- --port 5174 --strictPort`.
Rust/catalog and artwork changes require restarting development preparation.
Stop development before formatting, other Dune builds or verification preparation;
the formatter uses Dune to resolve its pinned tools.

```sh
npm run format                # Apply project formatting; README is user-owned
npm run format:check          # Read-only formatting and whitespace checks
npm run verify:prepare        # Explicit cold toolchain/artifact preparation
npm test                      # All test suites; shared 20-second budget
npm run verify                # All checks/tests and prepared bundle freshness
npm run verify -- types tests # Selected checks, under the same deadline
npm run build                 # Actual release build
npm run preview
```

## Verification

Run `npm run verify:prepare` after changes to compiled sources, content, artwork,
compiler or bundle inputs. It builds release WASM, artwork, generated bindings,
OCaml and the Vite bundle, resolves formatters, and compiles native test executables
without running tests. Cold compilation is explicit preparation work.

The entire `npm test` and every verification invocation must finish within
20 seconds, including npm startup. One unchanged 18-second internal deadline
reserves startup time, bounds concurrency and kills process groups on timeout.
Tests include Vitest, native Rust and Rustdoc, plus artwork, generation, cleaning
and combat-analyzer Python suites. No simulation batch or browser automation is
part of verification. Analyzer tests use small synthetic saved reports.

`npm run verify` adds strict Rust declaration/TypeScript checks, generated binding
freshness, content, read-only formatting and prepared-artifact validation.
Formatting writes only through the explicit `npm run format` command (`style`
is a compatibility alias). Verification never changes sources while checking
compiled artifacts. Exact source and output digests reject stale, missing, added,
deleted or corrupt WASM, emitted OCaml, sprites, native tests and bundle files.
The build group validates the prepared production bundle without another Vite
compilation; `npm run build` remains the real publication build.
Groups are `style` (also `format`), `content`, `wasm`, `ui`, `types`, `rust`,
`python`, `art`, `tests` and `build`. Overlapping groups run each suite once.
Independent checks continue after failures while the shared deadline remains.

## Project layout

- `rust/data/` contains authored game content and simulation rosters/schedules.
  The [content reference](../rust/data/README.md) describes schemas and inheritance.
- `rust/src/data/`, `logic/` and `ai/` own the catalog, battles and planning.
  `simulation/` owns scheduling and recording; `protocol.rs` and `wasm.rs` expose
  the browser contract. See the [Rust guide](../rust/README.md).
- `frontend/Native.ml` owns opaque battle handles and detached publications;
  `Catalog.ml` owns indexed native metadata. `Domain.ml/.mli` are generated from
  Rust's declarations. Handwritten `.d.ts` files describe emitted APIs for tests.
- `frontend/Session.ml` owns commands, saves, undo and deferred work. Components
  receive its published model and stable actions, never a live battle handle.
  `.mli`/`.rei` files document module contracts and resource ownership.
- `Save.ml` and `Save_legacy.ml` validate browser envelopes and migrate historical
  saves before native restoration. See [save migrations](save-migrations.md).
- ReasonReact components render the shell, battlefield and panels. `Boot`, `Main`
  and `Startup` mount a catalog-free shell before loading the battle application.
  `Application` releases the session and factory, including on loading failure.
- `Art`, `Audio`, `Terrain` and `Viewport` own presentation resources;
  `Presentation` and `Story` project published data. TypeScript is limited to
  tooling, test embedding and generated data-only artwork metadata.

Native publications are acyclic foreign graphs, deep-frozen in place. Borrowed
arrays/dictionaries preserve references but retain mutable OCaml syntax; runtime
freezing enforces read-only access. Copy before editing. A failed accepted native
publication releases its handle, retains the last coherent view and saved bytes,
cancels deferred work and offers reload. Rejected-save backup failure suspends
recovery writes until an explicit new battle.

Generated bindings live in ignored `rust/pkg/`, sprites in ignored
`public/assets/sprites/`, and production bundles in ignored `dist/`. Edit source
files. Temporary experiments and screenshots belong in ignored `tmp/`. The
developer map displays every terrain and both sides of every playable unit rank.

## Artwork rendering implementation

Pixel sources are character matrices and explicit palettes under `pixel/art/`.
Character sources are always 64×64, stored in
`pixel/art/units/<race>/<unit-id>/`; race trees live in `pixel/art/units/lineups/`.
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

## References

- [Combat simulations and analysis](../.agents/skills/combat-simulation/usage.md)
- [AI policy implementation](../rust/src/ai/README.md)
- Artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)
