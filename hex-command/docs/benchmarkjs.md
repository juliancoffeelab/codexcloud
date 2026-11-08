# Benchmark.js runners

All benchmark commands use Benchmark.js **2.1.4** directly with its default
measurement options except `maxTime`, which defaults to **1 second** per
case/implementation. `--time SECONDS` overrides that sampling budget; it is not
a hard timeout. Calibration, minimum samples and slow operations add wall time. The library owns calibration, repeated invocation, samples
and statistics. There are no custom timers, worker samplers, paired cohorts,
warmup loops, forced GC or correctness/hash passes. Inputs and modules must exist
before measurement, just as in wasm-bindgen's own benchmark suite.

Install this project's dependencies with `npm ci`. Build the desired game,
compiled processing adapter and native release executable before running the
relevant commands. These commands do not silently rebuild implementations.

## Game processing in Node or Chromium

```sh
node scripts/benchmark-processing.mjs --upstream /path/to/upstream --corpus /path/to/corpus.json --out tmp/processing.json
node scripts/benchmark-processing.mjs --upstream /path/to/upstream --corpus /path/to/corpus.json --browser --out tmp/processing-browser.json
```

`--port PATH` selects another built port. `--heavy FILE` uses a saved heavy enemy
checkpoint instead of a corpus. `--input N` selects one checkpoint. `--artifacts
PATH` saves the actual bundled adapters, Wasm and corpus outside measurement.

The default selects the first, middle and last corpus checkpoints (or all when
fewer than three exist). `--full` selects the entire corpus; `--input N` selects
one explicit checkpoint. Reports record the selected indices.

One operation restores, executes the enemy phase, serializes the final save and
disposes every selected checkpoint in the corpus. The corpus loop is the
workload; Benchmark.js controls repetitions. This complete lifecycle differs
from historical phase-only timings with separate restoration clocks. Node and
Chromium run the same operation; Playwright launches Chromium and loads inputs,
while Benchmark.js executes inside the page.

## Browser operations

```sh
node scripts/benchmark-browser.mjs --variant port=/path/to/port --variant upstream=/path/to/upstream --out tmp/browser.json
```

Each variant needs its production `dist/` and compiled bindings. `--case TEXT`
filters operations; `--startup-only` selects startup. `--map NAME` selects a map.
`--cache cold|warm`, `--network unthrottled|fast-4g|slow-4g|3g`, `--cpu-rate N`
and `--software` select browser conditions, not sampling parameters.
`CHROMIUM` can override `/usr/bin/chromium`.

The suite includes movement/neighbor/recruitment/placement queries, an unmounted
session command lifecycle, and two operations using actual production pages:

- **Startup-to-playable iframe lifecycle:** create/navigate an iframe, wait for
  the enabled end-turn button, visible terrain canvas, visible image decoding
  and two animation frames, then remove it.
- **Rendered commands iframe lifecycle:** perform that startup, select/move,
  undo and end-turn, waiting for saved state changes and two animation frames, then remove the iframe.
  Startup and disposal are included in every measured operation.

Playwright is the launcher. Benchmark.js and deferred completion run inside
Chromium. These are optional performance workloads; browser correctness tests
are unchanged. The separately bundled unmounted session adapter uses the normal
Vite development terrain-manifest default `{}`. Rendered/startup operations load
the actual production bundle and terrain assets. Asset inventory reports raw and
gzip sizes outside the timed operation. This is not a retained-memory, paint-only
or isolated input-latency measurement.

## Native simulation requests

```sh
node scripts/benchmark-rust.mjs --limit 1 --out tmp/native.json
python3 scripts/benchmark-policy-phases.py --report /path/to/report.json --out tmp/policy
node scripts/benchmark-policy-phases.mjs --positions /path/to/positions.json --case arena/full --out tmp/policy.json
```

The native match command selects representative configs from the simulator's
schedule. The default selects one representative match; `--full` restores the
60-match selection. `--alternate`, `--skip N`, `--limit N`, `--binary PATH` and
`--compare FILE` retain workload selection and optional Benchmark.js comparison. Its default
executable is `rust/target/release/hex-command-sim` (respecting `CARGO_TARGET_DIR`).
One deferred invocation sends one match request to persistent native stdio and
awaits the response. Process initialization is outside measurement; request/
response transport and native match work are included.

The Python policy command exports positions from an existing simulation report
(and optional `--campaigns FILE`), then invokes the JS runner. The JS runner
registers arena/campaign × full/queued operations; each invocation processes that
selected position corpus through persistent native stdio. The default selects
one position per arena/campaign scope; `--full` uses every position. The Python
launcher forwards `--full`, `--case` and `--time`. `--binary PATH`
selects an executable. Internal Rust profiling and custom repeat counts are not
enabled. These request lifecycle measurements differ from historical internal
phase profiles.

## Reports

Each report records the library's `hz`, `stats`, `times`, count and cycles;
`stats.mean` is seconds per operation and `stats.rme` is its relative margin of
error percentage. Standard output uses Benchmark.js's usual operations/sec.
Lifecycle readiness waits define the actual asynchronous workload; they are not
custom clocks or a second measurement harness. Two animation frames provide
rendering opportunities; they do not measure compositor/GPU completion. Compare equivalent workloads and
browser conditions, and retain wide error margins rather than declaring winners
from noisy point estimates. Historical reports retain their original definitions.

Save raw reports and generated benchmark artifacts under ignored `tmp/` or an
external directory. They are run outputs and are not committed.
