# Simulation performance

The exact-policy workload is 12,240 complete games: 11,520 arenas and 720 campaign
attempts, seeds 1–10, all authored races, landscapes and campaigns, Classic/Rush/
Mixed, both arena initiatives/seats and both campaign diplomacy settings. Every
match has a 30-round cap and one recruitment per phase. Campaigns start in chapter
one, earn carryover and receive no retries. Complete gameplay is recorded.

Build time and comparison time are separate from simulation command time. The
reference environment has a four-CPU cgroup quota; the harness uses four workers.
Time the entire native command, including startup, recording, aggregation and
report cleanup. The report's `elapsedSeconds` stops before final aggregation and
is therefore not the acceptance measurement.

```sh
npm run ai:rust:build
/usr/bin/time -p rust/target/release/hex-command-sim \
  --data-dir rust/data --mode full --seeds 10 --seed 1 \
  --rounds 30 --recruits 1 --policies classic,rush,mixed --workers 4 \
  --out artifacts/exact-current/report.json
python3 scripts/compare-simulation-reports.py \
  artifacts/exact-baseline/report.json artifacts/exact-current/report.json \
  --out artifacts/exact-current/equality.json
```

Install PyArrow when comparing different journal encodings. JSONL and direct Arrow
journals contain the same gameplay. Comparison checks all fields, hashes, ordered
arrays, numeric values and missing/null distinctions, ignoring only JSON key order
and the explicitly listed run metadata. Compiler profiles optimize code; they do
not contain or replay simulation results.

## Exact implementation

Occupancy is maintained incrementally as a dense mask. Forecasts reuse that mask
instead of rescanning every tile for each opponent. Small attack fields use direct
128-bit unions, with the general geometry path retained for larger maps.
Movement uses dense terrain masks for threat membership and ordered cost buckets
for decisions. Terrain layouts share reverse routes and geometry; bounded movement
caches include their complete blocker sets. Global tactics evaluates promising
units first and applies safe upper bounds, retaining the original roster/tile/
weapon ordinals for ties. Home-relative ordering is prepared once per decision. A conservative whole-unit
ceiling skips destination discovery when that unit cannot beat the current action.
The ceiling includes objective, formation, healing, combat and terminal bonuses;
exact zero operations avoid creating subnormal rounding operands.

Arena incoming-damage forecasts survive HP/action-flag changes. The turn workspace
invalidates them on identity, kind, side, position or terrain changes. Forecasted
coverage and damage do not depend on current HP. Campaign forecasts still execute
turn-start events, waves and objective transitions. Derived caches are rebuilt
when restoring a saved planner; they do not alter its continuation.

Workers construct complete gameplay, encode lossless Arrow snapshot dictionaries,
hash endpoints and release match data before sending compact outcomes and encoded
bytes to the writer. Recording retains every phase. Report aggregation indexes
scenario and policy groups once and preserves the original bootstrap seed stream,
quantiles and output ordering.

Native builds use mimalloc, CPU-specific code generation, fat LTO and a cached LLVM
profile trained on separate seeds 17–18. A stable active profile path avoids accumulating
a separate dependency cache for every source profile; a tracked profile key forces the engine to rebuild
when training inputs change. Browser/WASM builds keep their ordinary flags.

## Measurements

The frozen main baseline (`b1f553d`) took **39.13 seconds** end to end. The overnight
reference measurements and complete equality evidence are saved under
`artifacts/overnight-workspace-clean-*`. Rejected experiments are retained as
verified lossless ZIP archives. Do not substitute the fastest sample for repeated
clean measurements, or interpret concurrent build/benchmark runs as acceptance
measurements.

The final three clean command measurements are **9.49, 9.74 and 9.59 seconds**,
with a **9.59-second median** (4.08× the baseline speed). Machine-readable
measurements, training seeds, CPU quota and the independently checked source
fingerprint are in [benchmarks/exact-simulation.json](benchmarks/exact-simulation.json).

All **12,240** complete gameplay records and every report aggregate match the
frozen main baseline, including endpoint hashes, ordered arrays and missing/null
distinctions. All three repeats also have byte-identical Arrow detail records
against the previously verified journal. Full final verification passed all eight
groups; the combined tests remained within their unchanged 20-second budget.
