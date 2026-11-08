# Glue and string benchmarks with Benchmark.js

These suites now use **Benchmark.js 2.1.4 directly**, matching the framework used
by [wasm-bindgen's browser benchmarks](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/benchmarks/index.js).
The library controls calibration, repeated invocation, sample collection and
statistics. The custom timing loops, workers, cohorts, warmup/calibration code,
correctness/hash passes, sampling summaries and one-off timing/profiling controllers
were removed. Historical evidence archives and reports remain preserved.

The suite declares **91 glue cases** and **109 string cases**. Available cases
are filtered by the selected build's exported capabilities. Variants cover
compiled native/JS-field/JS-record bindings, direct wasm-bindgen bindings and the
pinned game's wire transport. Native string output means OCaml byte strings;
explicit `host` cases include conversion to host text.

Module loading and preparing arrays, strings, records or the Rust store needed
by a call happen outside the measured function. This is the same role as
upstream's initialization and input construction. There are no additional
correctness checks, forced collections or benchmark-specific diagnostics.
Opaque allocation/disposal stays inside lifecycle operations; long-lived input
stores/owners are released after the library finishes the case.

After building the artifacts and installing dependencies, run from the SDK:

```sh
npm ci
node benchmarks/glue/run.mjs --artifacts /tmp/artifacts --out /tmp/focus.json
node benchmarks/glue/run.mjs --artifacts /tmp/artifacts --suite glue --full --out /tmp/glue.json
node benchmarks/glue/run.mjs --artifacts /tmp/artifacts --suite strings --full --out /tmp/strings.json
```

`--case` selects a name substring, `--case-regex` selects names by expression,
and `--variants native,js,wasm-bindgen` selects implementations. Those options
only select functions. The default sampling budget is `maxTime: 1`; other Benchmark.js options retain
their defaults. `--time SECONDS` overrides the budget.
Each function prints the library's standard ops/sec, RME and sample count.
JSON saves the unmodified library `hz`, `stats`, `times`, count and cycles plus
case/variant names and existing build metadata. No custom statistics are computed.

The existing Rust/bindings builder remains a separate build command:

```sh
node benchmarks/glue/build.mjs --upstream /tmp/upstream/hex-command --out /tmp/artifacts
```

It requires the pinned upstream commit `78289b2745c4dfbff5555c14f539ed6284a47802`,
Rust/Wasm toolchains, wasm-bindgen and the SDK's prepared Melange compiler.
This corresponds to wasm-bindgen building its benchmark crate before loading it.
The archiving utilities preserve generated artifacts and reports; they are not
part of benchmark execution.

The default combined profile has **14 cases × 3 variants = 42 measurements**:
opaque access/lifecycles, small/large borrowed floats, eight numeric arguments,
large native Unicode host conversion, ASCII/Latin input, ASCII host output, JSON
Result/throws output, ASCII/Unicode save throws and Unicode record round trip.
It targets about two minutes or less with prepared artifacts. `--suite glue`
selects its six glue cases and `--suite strings` its eight string cases.
Explicit case filters search the entire selected catalog instead of intersecting
with that preset. `--full` restores all cases and the full default variant list
(including game-wire for glue and js-record for strings). Add `--time 5` to
reproduce the historical sampling budget. Extra variants remain explicitly
selectable in focused runs.
