# Async glue benchmarks with Benchmark.js

All timing, iteration calibration, sampling and statistics are handled directly
by **Benchmark.js 2.1.4**, the version used by wasm-bindgen's browser benchmarks.
There are no custom timers, warmup loops, worker controllers, batch counts,
cohorts, CPU pinning, A/A gates, replayed measurements or correctness passes.
Save generated reports and artifacts under ignored `tmp/` or an external
directory.

The complete suite retains 64 named operations covering Promise/Lwt delivery, scalars,
strings, records, borrowed/owning opaques and callbacks. Each callback is an
ordinary synchronous Benchmark.js function. Async cases use its documented
`defer: true` API and resolve the deferred operation after the generated Promise
completes. Owning-result disposal is part of the operation being measured.

Like upstream, module initialization and creating inputs occur before benchmark
registration. The only case preparation is loading Wasm, preparing the host
argument or borrowed owner needed by a call, and releasing those owners afterward.
Runtime result decoding is part of using the raw generated bindings. The Lwt
cases exercise that runtime's JS receiver, not a compiled Lwt application.

From the SDK directory after `npm ci`, using existing before/after artifact folders:

```sh
node benchmarks/async-modes/run.mjs /tmp/before /tmp/after /tmp/results.json
```

`ASYNC_MODE` filters a mode prefix; `ASYNC_CASE` selects one exact
`mode/operation`; `ASYNC_CASES` accepts a comma-separated list. These select
functions only. The sampling budget defaults to `maxTime: 1`; other library defaults remain.
Use `--time SECONDS` after the three paths to override the budget.
It prints Benchmark.js's standard operations/second, RME and sample count, and
saves its `hz`, `stats`, `times`, cycle count and iteration count directly.
There is no custom statistical summary or speedup verdict.

The optional artifact builder is unchanged:

```sh
cargo build --manifest-path Cargo.toml -p ocamljs_pack
node benchmarks/async-modes/build.mjs /tmp/after target/debug/ocamljs_pack
```

Its fourth argument selects comma-separated Rust artifact kinds;
`--generate-only` reuses saved input Wasm. Building Rust/bindings is separate from
running benchmarks, just as wasm-bindgen builds its benchmark crate first.

Reference inspected: [wasm-bindgen benchmark driver](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/benchmarks/index.js)
and [architecture](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/benchmarks/README.md).
Upstream initializes Wasm and constructs input strings, DOM arrays and objects,
then calls `Benchmark.Suite.add`/`run`. It has no separate result-validation pass.
Its DOM benchmark functions contain their own workload-specific branches that can
throw; that is separate from adding a correctness harness around every sample.

The default profile selects eight cases (16 before/after measurements): ready
and suspended Promise delivery, nested owning results, JS-string completion,
js_of_ocaml host completion, Lwt unit completion, ReScript native-string
completion and a callback. Explicit `ASYNC_*` filters select from all 64 cases,
without intersecting that preset. `--full` runs the whole matrix; `--full --time 5`
restores its historical sampling settings.
