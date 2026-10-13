# Frozen native reference AI

This is the Rust source and authored catalog from commit
`b1f553de69c0de8a340088c0a9ed42b2149fb27e`, the reference used for the recorded
Classic/Rush/Mixed CPU timings. Builds never read `main`, `origin/main` or Git.
`PROVENANCE.json` records every original file checksum and each integration adapter.

ClassicOld, RushOld and MixedOld preserve the original complete match runner,
tactics, forecasts, geometry, system allocator, thin LTO and eight codegen units.
The native build compiles `hex-command-legacy` alongside `hex-command-sim`.
Comparison batches run the original worker pool and original JSONL recording path
to completion before the host imports the records into its Arrow journal. This
preserves allocation lifetime as well as algorithms: serial per-game transport
and overlapping transport both changed measured chooser costs. Timing comes from
the actual original chooser and phase intervals in the frozen runtime. IPC and
host report encoding are outside those intervals. Old command wall time includes
both the frozen recording and the import; use the reported CPU intervals to
compare AI costs. This avoids
silently accelerating old routines with current helpers or measuring IPC as AI.

These are native reference entrants for complete arena/campaign runs. Old versus
old opponents work; use `--self-play` to compare old and current entrants together.
Cross-version opponents and live/native/WASM planner continuations are rejected.
The frozen catalog is required; select `--data-dir rust/legacy/data` when the live
catalog has changed. Frozen source/data are never refreshed after a main merge.

The added profiler and reference transport, module/timer declarations, extra binary
entry point, supplied batch schedule, suppressed duplicate console output and
self-contained road-data paths are the only adapters. AI and engine
algorithms remain original. The build verifies all frozen and adapted checksums.
