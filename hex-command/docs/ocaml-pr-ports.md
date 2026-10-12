# Sequential OCaml ports

PR #14 replaces the frontend with OCaml/Melange and ReasonReact. The rejected hybrid was discarded; the new baseline starts at the main commit immediately before PR #13. Each incoming PR description and implementation was read, then rebased and ported in order: #13, #15, #16, #17, #18. The published branch is based on main `e59d224`.

The machine-readable [delta ledger](ocaml-pr-deltas.json) contains exact revisions and every counted file. [sync-deltas.ts](../scripts/sync-deltas.ts) reproduces the counts. Local chronological checkpoints preserve measurements made before each subsequent rebase; `publishedPort` names the corresponding reachable commit in PR #14. Rebased patches can differ from chronological snapshots because upstream changes move into the base.

## Scope and final result

Runtime means handwritten frontend implementation, including CSS and startup JavaScript. Authored frontend also includes `.mli`, `.rei` and test-embedding `.d.ts` declarations. Whole code additionally includes tests, build tools and shared Rust/Python changes. Generated code, lockfiles, JSON data, artwork, documentation and dependency/compiler output are excluded. Historical save JSON is retained as runtime data, not counted as code.

| Compared with current main                 |  Added | Removed |   Net |
| ------------------------------------------ | -----: | ------: | ----: |
| Frontend runtime                           |  +4865 |   −6680 | -1815 |
| Frontend including interfaces              |  +5720 |   −6684 |  -964 |
| Whole authored code, including tests/tools | +11692 |   −9142 | +2550 |

**Only the frontend has a negative delta.** The whole-code count increases because it includes the test migration, additional regressions and build tooling. There are no handwritten TypeScript/JavaScript application helpers. Runtime TypeScript is generated data-only sprite metadata; Rust WASM glue and React are foreign dependencies. The terrain prebuild adapter is Node-only I/O around the same compiled OCaml renderer.

## Each incoming PR and its port

Numbers are net lines; the per-file tables below show additions and removals.

| PR                                                           | Original runtime | OCaml runtime | Difference | Original with interfaces | OCaml with interfaces | Original whole code | Port whole code |
| ------------------------------------------------------------ | ---------------: | ------------: | ---------: | -----------------------: | --------------------: | ------------------: | --------------: |
| [#13](https://github.com/juliancoffeelab/codexcloud/pull/13) |             +488 |          +688 |       +200 |                     +492 |                  +833 |               +1454 |           +2329 |
| [#15](https://github.com/juliancoffeelab/codexcloud/pull/15) |              +87 |          +113 |        +26 |                      +87 |                  +140 |                +207 |            +536 |
| [#16](https://github.com/juliancoffeelab/codexcloud/pull/16) |               +0 |            +0 |         +0 |                       +0 |                    +0 |                  +0 |              +0 |
| [#17](https://github.com/juliancoffeelab/codexcloud/pull/17) |             +130 |          +223 |        +93 |                     +130 |                  +274 |                +317 |            +772 |
| [#18](https://github.com/juliancoffeelab/codexcloud/pull/18) |              +16 |           +10 |         -6 |                      +16 |                   +19 |                 +56 |            +225 |

Ports #13, #15 and #17 are larger than their TypeScript frontend changes; #18 uses fewer runtime lines. Interface declarations are included, not hidden. Whole-code port counts include unchanged upstream Rust, artwork-builder and publication-tool changes carried by the rebase; they do not mean those systems were rewritten in OCaml.

### #13 — native artwork, terrain, startup and saves

The original removes alternate art and atlases, publishes standalone 64×64 sprites, prebuilds all authored maps, expands the developer board, hides showcase maps and mounts the shell before WASM.

The port uses `Art` for immutable image metadata, `Terrain_plan` for shared geometry/frame selection/native painting and `Terrain` for scene ownership, prebuilt matching, changed-cell overlays and bounded fallback work. Browser and production prebuild use the same OCaml renderer. Unit sprites and health nodes retain identity through movement/undo. `Catalog` filters hidden maps; `Save`/`Save_legacy` perform version-four historical board expansion.

`Boot`, `Main`, `Startup` and `Application` separate early HTML failure handling, catalog-free shell, async acquisition and loaded application cleanup. Those ownership and rendering contracts explain much of this larger port. Obsolete atlas/SVG markup paths and artwork loading commands are removed. The handwritten startup script is counted: the original changes it by +3/−2; the chronological port removes its previous 40 lines. It was not a new 41-line addition.

### #15 — bounded inspection prefetch

`Terrain_prefetch` owns one shared two-request bound across map changes. It waits for the displayed map, then a frame and idle time, cancels queued/stale work, retains only successful URL identities and leaves failures retryable through ordinary foreground loading. Tests use controlled callbacks and promises. The extra runtime lines implement owner lifetime and scheduling; the public interfaces add 27 lines.

### #16 — PR workflow

The feature-branch/PR-only instructions are carried directly. There is no runtime or interface port. No direct main push, merge or Sites publication was performed.

### #17 — inspection and mobile layout

`Session.choose` preserves move/attack behavior while a second inspected-tile tap clears inspection. `Viewport` owns measured scrolling, inspected/combatant visibility, clamping and fenced cleanup. A single catalog-free `Header` is shared by startup and the live battle. Compact cards and overscroll CSS retain the upstream layout; attack controls scroll into view. The publication adapter and its success/failure tests are carried from upstream.

The larger runtime delta includes an explicit viewport owner and the shared header. CSS is counted on both sides. Native browser pull-to-refresh and physical-device layout were not tested in this rewrite.

### #18 — canonical terrain details

Details and prefetch use one complete canonical image per terrain type: default connections/floor and the first prepared prop arrangement. Only terrain types present on the displayed map are warmed. Battlefield and hover retain authored floors/directions/arrangements. Canonical selection stays inside `Terrain_plan`; the existing prefetch owner needs no new mechanism.

## Baseline and independent-review changes

| Stage                               | Runtime net | Frontend with interfaces net | Whole code net |
| ----------------------------------- | ----------: | ---------------------------: | -------------: |
| Fresh baseline                      |       -2287 |                        -1709 |           -772 |
| Review repairs after all five ports |        +159 |                         +204 |          +1129 |

Review repairs are separate from the five port measurements. Five fresh independent reviewers assessed code quality, deep interfaces, performance, reliability and public documentation. Their findings produced:

- DTO-only generation; removal of unused hybrid-helper generator branches and stale Svelte configuration.
- Original foreign-exception preservation, cleanup that releases both session and factory, late React failure handling, and terminal native-publication failure that retains the last coherent view without changing saved bytes.
- Terminal audio cleanup, correct historical browser developer saves, recursive freezing even beneath already shallow-frozen parents, changed-artwork terrain fallback, and release of completed preload image references.
- Lazy map previews: closed dialogs no longer construct/reconcile 1,236 hidden polygons per publication. Choices commit before native dialog focusing; delayed close events cannot remove choices from reopened map/campaign dialogs.
- Accurate ownership/migration documentation and fixed bounded verification.

Foreign arrays/dictionaries retain ordinary mutable OCaml types; runtime freezing enforces the documented borrowed read-only contract. This is not compile-time collection immutability. Browser paint latency remains unmeasured. The single-page shared map-preload owner is bounded; successful loads release image references, but it does not expose a separate embedding shutdown API.

## Validation and the 20-second requirement

These measurements precede the Dune 3.24.2 environment upgrade in `02f2cb6`.
They are retained as historical results; checks have not been rerun for that
upgrade in this session.

- Complete `npm test`: **10.738 seconds**; 241 frontend tests, 74 native Rust tests and 39 Python checks pass. Both native executables and Rustdoc discovery run, including targets with no current tests.
- Complete `npm run verify`: **14.693 seconds**; every test plus strict TypeScript/WASM ABI, exact generated bindings, OCaml/Reason/JSON/Rust formatting, whitespace, content and prepared artifact freshness checks pass.
- Explicit `npm run verify:prepare` compiles the pinned OCaml toolchain, release WASM, authoritative artwork, native test executables and Vite output. Production compilation emitted all 16 authored terrain PNGs. Timed checks fingerprint source inputs and entire compiled/artwork/bundle file sets; missing, stale, added, removed or corrupted outputs reject. Preparation never runs tests.
- Verification uses one unchanged 18-second internal deadline, reserving time for npm startup inside the user’s 20-second command limit. Subprocess groups are killed on timeout/interruption. Vitest keeps file isolation; transformed-module caching, three frontend workers, shared TypeScript compilation and parallel independent formatting remove repeated work.
- No browser automation or campaign/tournament batches were run.

There was no instruction permitting verification over 20 seconds. The old project rules already required the complete test command to stay under 20; the old verification implementation had no aggregate timeout and repeated lifecycle/build work. Failed full runs were terminated and rejected. The rules now explicitly cover every verification invocation; no budget was increased.

## Development workflow follow-up

Commit `12f1190` fixes the development workflow separately from incoming feature ports:

- `npm run dev` owns preparation, ABI regeneration, the Dune watcher and Vite. It prints preparation progress, preserves compiler diagnostics, forwards Vite arguments and releases both process trees on interruption or unexpected service exit.
- At this checkpoint, README bootstrap commands installed the official standalone Dune 3.21.0 binary and explained prerequisites, first-build work, supported architectures and recovery. That installer was exercised, followed by a successful frontend build. The setup change below supersedes those bootstrap instructions.
- Actual HTTP/WebSocket checks confirmed that a Reason edit rebuilt and reached Vite, an intentional compiler error left the watcher alive, and fixing the source recovered without restarting. These were server/compilation checks, not browser automation.
- Four small subprocess tests cover argument forwarding, interruption with descendants, watcher/server failure and initial preparation failure. Fixtures have independent lifetime and cleanup bounds. Independent code and documentation reviews completed with their findings repaired.

This follow-up adds **330 authored tooling/test lines**: `scripts/dev.ts` +137,
`scripts/test-budget.ts` +1 and `tests/dev.test.ts` +192. Frontend runtime and
interface deltas remain unchanged. The machine ledger preserves the original
migration comparison and records this follow-up separately from the five ports.
The Dune directory configuration, package scripts and Markdown instructions are
outside the source-extension counting scope described above.

## Environment setup follow-up

Setup commit `02f2cb6` pins Dune **3.24.2** in `.dune-version`, upgrades the Dune
language, uses upstream OCaml packages and removes the local package metadata
overrides. README now documents an existing Opam installation or the
checksummed `scripts/install-dune.sh` source bootstrap. The latter needs a
bootstrap OCaml compiler 4.14+; application OCaml remains pinned to 5.3.0 in the
package lock. Preparation fingerprints include the Dune version and installer.

The setup adds **35 authored code lines net**: the bootstrap shell script +34
and preparation fingerprint changes +2/−1. Authored `.sh` files are now included
in whole-code counting; frontend runtime/interface counts are unchanged. The
machine ledger records the setup separately, and the original PR port counts
remain unchanged. Lockfiles, package metadata and documentation stay outside
these code counts.

## Per-file frontend changes

Interface rows are declarations; all other rows contribute to runtime. Whole-code per-file lists, baseline, review and final comparisons are retained in the JSON ledger.

### PR #13

Published port: `fbec25074447f38641d4c9931cce37abd2c86263`.

**Original**

| File                                      | Added | Removed | Kind      |
| ----------------------------------------- | ----: | ------: | --------- |
| `public/startup.js`                       |    +3 |      −2 | Runtime   |
| `src/App.svelte`                          |   +23 |      −2 | Runtime   |
| `src/Startup.svelte`                      |  +118 |      −0 | Runtime   |
| `src/battle-app.ts`                       |   +16 |      −0 | Runtime   |
| `src/controller/battle.ts`                |    +2 |     −77 | Runtime   |
| `src/controller/view.ts`                  |    +2 |      −6 | Runtime   |
| `src/game/campaign-progress.ts`           |    +1 |      −1 | Runtime   |
| `src/game/maps.ts`                        |    +5 |      −0 | Runtime   |
| `src/game/types.ts`                       |   +19 |      −3 | Runtime   |
| `src/main.ts`                             |    +3 |     −15 | Runtime   |
| `src/presentation/art.ts`                 |   +53 |    −144 | Runtime   |
| `src/presentation/campaign-art.ts`        |    +0 |     −25 | Runtime   |
| `src/presentation/sprite-images.ts`       |    +0 |     −79 | Runtime   |
| `src/presentation/terrain-connections.ts` |   +58 |      −0 | Runtime   |
| `src/presentation/terrain-layout.ts`      |   +21 |      −0 | Runtime   |
| `src/presentation/terrain-map.ts`         |  +273 |      −0 | Runtime   |
| `src/presentation/terrain-preload.ts`     |   +17 |      −0 | Runtime   |
| `src/save/migration/index.ts`             |    +4 |      −0 | Runtime   |
| `src/save/migration/v2-to-v3.ts`          |    +4 |      −2 | Runtime   |
| `src/save/migration/v3-to-v4.ts`          |   +57 |      −0 | Runtime   |
| `src/save/schema.ts`                      |    +1 |      −1 | Runtime   |
| `src/style.css`                           |    +8 |     −12 | Runtime   |
| `src/terrain-prebuilt.d.ts`               |    +4 |      −0 | Interface |
| `src/ui/Battlefield.svelte`               |  +234 |     −43 | Runtime   |
| `src/ui/MapPicker.svelte`                 |   +20 |     −11 | Runtime   |
| `src/ui/SettingsDialog.svelte`            |    +0 |     −18 | Runtime   |
| `src/ui/TerrainPanel.svelte`              |    +5 |      −1 | Runtime   |
| `src/ui/battlefield-view.ts`              |   +23 |     −39 | Runtime   |
| `src/ui/geometry.ts`                      |   +15 |     −17 | Runtime   |
| `src/ui/terrain-view.ts`                  |    +1 |      −0 | Runtime   |

**OCaml port**

| File                         | Added | Removed | Kind      |
| ---------------------------- | ----: | ------: | --------- |
| `frontend/App.d.ts`          |    +4 |      −0 | Interface |
| `frontend/App.re`            |   +29 |      −8 | Runtime   |
| `frontend/Application.d.ts`  |    +3 |      −0 | Interface |
| `frontend/Application.ml`    |   +38 |      −0 | Runtime   |
| `frontend/Application.mli`   |    +5 |      −0 | Interface |
| `frontend/Art.d.ts`          |    +7 |     −20 | Interface |
| `frontend/Art.ml`            |   +23 |    −147 | Runtime   |
| `frontend/Art.mli`           |   +11 |     −18 | Interface |
| `frontend/Battlefield.re`    |  +460 |    −206 | Runtime   |
| `frontend/Boot.ml`           |   +40 |      −0 | Runtime   |
| `frontend/Catalog.d.ts`      |    +2 |      −0 | Interface |
| `frontend/Catalog.ml`        |    +2 |      −1 | Runtime   |
| `frontend/Catalog.mli`       |    +4 |      −2 | Interface |
| `frontend/Dialogs.re`        |   +16 |     −36 | Runtime   |
| `frontend/Geometry.d.ts`     |    +5 |      −1 | Interface |
| `frontend/Geometry.ml`       |   +14 |     −12 | Runtime   |
| `frontend/Geometry.mli`      |    +5 |      −1 | Interface |
| `frontend/Main.ml`           |   +27 |     −20 | Runtime   |
| `frontend/Panels.re`         |    +2 |      −1 | Runtime   |
| `frontend/Save.d.ts`         |    +2 |      −1 | Interface |
| `frontend/Save.ml`           |   +51 |      −7 | Runtime   |
| `frontend/Save.mli`          |    +3 |      −1 | Interface |
| `frontend/Session.d.ts`      |    +1 |      −3 | Interface |
| `frontend/Session.ml`        |   +17 |     −29 | Runtime   |
| `frontend/Session.mli`       |    +2 |      −2 | Interface |
| `frontend/Shell.re`          |   +56 |      −0 | Runtime   |
| `frontend/Shell.rei`         |    +3 |      −0 | Interface |
| `frontend/Startup.d.ts`      |   +13 |      −0 | Interface |
| `frontend/Startup.ml`        |   +47 |      −0 | Runtime   |
| `frontend/Startup.mli`       |   +10 |      −0 | Interface |
| `frontend/Terrain.d.ts`      |   +29 |      −0 | Interface |
| `frontend/Terrain.ml`        |  +209 |      −0 | Runtime   |
| `frontend/Terrain.mli`       |   +43 |      −0 | Interface |
| `frontend/Terrain_plan.d.ts` |   +15 |      −0 | Interface |
| `frontend/Terrain_plan.ml`   |  +112 |      −0 | Runtime   |
| `frontend/Terrain_plan.mli`  |   +27 |      −0 | Interface |
| `frontend/Ui.re`             |   +20 |      −5 | Runtime   |
| `public/startup.js`          |    +0 |     −40 | Runtime   |
| `src/style.css`              |   +49 |     −12 | Runtime   |

### PR #15

Published port: `bf66534e70c4871a24bd206dd178890bd4b9ebca`.

**Original**

| File                                   | Added | Removed | Kind    |
| -------------------------------------- | ----: | ------: | ------- |
| `src/presentation/terrain-prefetch.ts` |   +76 |      −0 | Runtime |
| `src/ui/Battlefield.svelte`            |   +11 |      −0 | Runtime |

**OCaml port**

| File                             | Added | Removed | Kind      |
| -------------------------------- | ----: | ------: | --------- |
| `frontend/Battlefield.re`        |   +33 |      −5 | Runtime   |
| `frontend/Terrain_prefetch.d.ts` |    +9 |      −0 | Interface |
| `frontend/Terrain_prefetch.ml`   |   +85 |      −0 | Runtime   |
| `frontend/Terrain_prefetch.mli`  |   +18 |      −0 | Interface |

### PR #16

Published port: `8f08a37eac1f4656ae3cb23389a0f59169af25a3`.

**Original**

No frontend code changes.

**OCaml port**

No frontend code changes.

### PR #17

Published port: `b88b96a78573e5f7543d35970bf9641c8d5b27d2`.

**Original**

| File                          | Added | Removed | Kind    |
| ----------------------------- | ----: | ------: | ------- |
| `src/App.svelte`              |   +29 |     −14 | Runtime |
| `src/Startup.svelte`          |   +22 |     −56 | Runtime |
| `src/controller/battle.ts`    |   +18 |     −14 | Runtime |
| `src/style.css`               |   +46 |     −29 | Runtime |
| `src/ui/AttackChoices.svelte` |    +0 |      −1 | Runtime |
| `src/ui/BattleBar.svelte`     |  +104 |      −0 | Runtime |
| `src/ui/Battlefield.svelte`   |   +49 |      −2 | Runtime |
| `src/ui/TurnBar.svelte`       |   +17 |     −64 | Runtime |
| `src/ui/UnitCard.svelte`      |   +17 |      −5 | Runtime |
| `src/ui/UnitDetails.svelte`   |    +1 |     −12 | Runtime |
| `src/ui/UnitPanel.svelte`     |   +25 |      −1 | Runtime |

**OCaml port**

| File                      | Added | Removed | Kind      |
| ------------------------- | ----: | ------: | --------- |
| `frontend/App.re`         |   +66 |     −83 | Runtime   |
| `frontend/Battlefield.re` |   +29 |      −1 | Runtime   |
| `frontend/Dialogs.re`     |    +1 |      −1 | Runtime   |
| `frontend/Header.re`      |  +116 |      −0 | Runtime   |
| `frontend/Header.rei`     |   +24 |      −0 | Interface |
| `frontend/Panels.re`      |   +48 |     −37 | Runtime   |
| `frontend/Session.ml`     |   +40 |     −28 | Runtime   |
| `frontend/Shell.d.ts`     |    +3 |      −0 | Interface |
| `frontend/Shell.re`       |   +24 |     −48 | Runtime   |
| `frontend/Ui.re`          |    +3 |      −5 | Runtime   |
| `frontend/Viewport.d.ts`  |    +7 |      −0 | Interface |
| `frontend/Viewport.ml`    |   +82 |      −0 | Runtime   |
| `frontend/Viewport.mli`   |   +17 |      −0 | Interface |
| `src/style.css`           |   +46 |     −29 | Runtime   |

### PR #18

Published port: `2c90674c818cd0cc236af682bd9768b25343e871`.

**Original**

| File                                   | Added | Removed | Kind    |
| -------------------------------------- | ----: | ------: | ------- |
| `src/presentation/terrain-layout.ts`   |    +7 |      −0 | Runtime |
| `src/presentation/terrain-prefetch.ts` |   +16 |      −1 | Runtime |
| `src/ui/Battlefield.svelte`            |    +7 |      −2 | Runtime |
| `src/ui/TerrainPanel.svelte`           |    +3 |     −14 | Runtime |

**OCaml port**

| File                             | Added | Removed | Kind      |
| -------------------------------- | ----: | ------: | --------- |
| `frontend/Battlefield.re`        |    +3 |      −3 | Runtime   |
| `frontend/Panels.re`             |    +1 |      −2 | Runtime   |
| `frontend/Terrain.d.ts`          |    +1 |      −0 | Interface |
| `frontend/Terrain.ml`            |    +8 |      −0 | Runtime   |
| `frontend/Terrain.mli`           |    +3 |      −0 | Interface |
| `frontend/Terrain_plan.d.ts`     |    +1 |      −0 | Interface |
| `frontend/Terrain_plan.ml`       |    +8 |      −3 | Runtime   |
| `frontend/Terrain_plan.mli`      |    +3 |      −0 | Interface |
| `frontend/Terrain_prefetch.d.ts` |    +2 |      −1 | Interface |
| `frontend/Terrain_prefetch.ml`   |    +5 |      −7 | Runtime   |
| `frontend/Terrain_prefetch.mli`  |    +3 |      −3 | Interface |
