# Benchmark.js runners

All active benchmark runners use Benchmark.js **2.1.4** directly. It owns
calibration, repetitions, sampling and statistics. `maxTime` defaults to
**1 second** per case/implementation; other options retain library defaults.
`--time SECONDS` overrides the sampling budget. This is not a hard wall-clock
timeout; calibration, sample minimums and cycle pauses take additional time.
There are no custom timing loops, workers, cohorts, A/A gates, forced GC or
correctness/hash passes. Async operations use `defer: true` and resolve only when
the operation completes. Initialization and constructing inputs happen before
registration; ownership cleanup stays inside lifecycle operations.

This follows [wasm-bindgen's driver](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/benchmarks/index.js)
and [benchmark declarations](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/benchmarks/js-benchmarks.js):
upstream initializes Wasm, constructs strings/arrays/objects, then registers
functions in `Benchmark.Suite`. It has no separate result-validation pass.
Some of its DOM functions have workload-specific branches that throw.
Our implementation does not reuse upstream binding code.

## SDK commands

Run `npm ci` in the SDK. Build artifacts separately, as documented in the
[glue guide](../benchmarks/glue/README.md) and
[async guide](../benchmarks/async-modes/README.md). Then:

```sh
npm run benchmark -- --artifacts /tmp/glue --out /tmp/focus.json
node benchmarks/glue/run.mjs --artifacts /tmp/glue --suite glue --full --out /tmp/glue.json
node benchmarks/glue/run.mjs --artifacts /tmp/glue --suite strings --full --out /tmp/strings.json
node benchmarks/async-modes/run.mjs /tmp/before /tmp/after /tmp/async.json
npm run benchmark:demo
npm run benchmark:async
```

The normal `npm run benchmark` profile selects 14 relevant cases across native,
selected-JS and wasm-bindgen paths: **42 measurements**, targeting about two
minutes or less with prepared artifacts. `benchmark:glue` and `benchmark:strings`
select their six/eight cases separately. Async modes default to eight cases
measured before/after. Explicit case filters bypass these presets. `--full`
selects complete matrices; combine `--full --time 5` for historical budgets.

There are 91 glue, 109 string and 64 async operation declarations. Available
exports filter cases when replaying older built artifacts. The demo commands
exercise existing compiled demo bindings with the same 1-second default. Every runner uses the actual library,
not a custom controller around its timer. Reports store library `hz`, `stats`,
`times`, iteration count and cycles. CLI case/variant filters select workloads; `--time` sets the library budget.
`stats.mean` is seconds per operation; `stats.rme` is the library's relative
margin of error percentage. Standard output prints its usual operations/sec.
No custom speedup verdict or significance gate is added.

## Reports

Reports retain Benchmark.js samples, error margins and artifact identities. Save
reports and generated artifacts under ignored `tmp/`, or an external directory.
They are run outputs, not source files. Wider margins mean less precise results;
point estimates alone do not establish parity or a performance improvement.
