# Simulation performance

## Current-main validation

The consolidated branch includes main commit `1ce0c8c` (per-attack counts,
corrected arenas and woodland balance). The benchmark studies in the sections
below describe their original, pre-rebase builds and frozen `b1f553d` baseline.

After rebasing, all 12,240 complete Classic/Rush/Mixed gameplay records, endpoint
hashes, ordered outcomes and report aggregates match updated main. Single clean
whole-command observations were 31.36 seconds on main and 9.10 seconds on the
consolidated branch, with four workers and complete recording. These are single
samples, not repeated medians; builds and comparisons are excluded.

All eight project verification groups passed. Combined tests took 18.20 seconds
within the unchanged 20-second target. The restoration regression now reloads
edited catalog documents, preserving movement-class and cache identities.
Frozen combat imports preserve missing strike metadata while current records
retain explicit zero damage and retaliation values.

Frozen source and catalog checksums remain unchanged. A fresh seed-one comparison
checks 1,152 complete arenas and all 72 old-policy campaign pairings against the
standalone frozen engine, including full gameplay and report aggregates after
normalizing only policy labels and explicit run metadata. All eight policies
also complete 1,024 self-play games with timing rows. Priority/Battler complete
512 arenas and eight Priority campaign attempts on updated live data. These
smoke checks do not reestablish the historical paired quality claims.

Fingerprints, binary hashes, scope, equality results and check timings are in
[benchmarks/rebased-stack.json](benchmarks/rebased-stack.json). Complete journals
and logs remain in the validation checkout's ignored `tmp/rebase-validation/`.

The [planner cost investigation](planner-cost-investigation.md) explains the
remaining gap to Classic using matched positions and diagnostic work counters.

## Original exact-policy workload

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

## Opt-in policies and separate decision measurements

Priority supports battles and complete campaigns. Battler exposes the same battle
planner with campaign use explicitly rejected. The authored defaults remain
Classic/Rush/Mixed. Select the new policies with `--policies`; full schedules omit
Battler from both campaign sides.

The paired runner executes production games against each existing enemy policy,
using every authored faction/map pair and both arena initiatives, or every
campaign and diplomacy setting. It saves complete gzip JSONL traces, per-game
observations, all strata, caps, defeats, chapter reach and named losses. Campaigns
start at chapter one with earned carryover and no retries.

```sh
python3 scripts/compare-cheap-policies.py --mode arena --policies battler \
  --seed 1001 --seeds 50 --out artifacts/battler-comparison
python3 scripts/compare-cheap-policies.py --mode campaign --policies priority \
  --seed 8001 --seeds 200 --out artifacts/priority-comparison
```

Set `HEX_COMMAND_AI_PROFILE=1` for native decision measurements. Linux uses one
thread CPU interval around each chooser call, including queued-command handling;
production command execution and journal serialization run outside that interval.
Other platforms explicitly report elapsed-clock measurements instead. Profiling is
opt-in and disabled during full command acceptance timing.

Whole-game decision totals can change when strategies finish sooner or encounter
different states. The saved-position profiler also measures one own phase on the
same tactical states for each policy. It includes initial turn-start, reconstructed
mid/late arena boundaries with recorded next-phase objectives, and complete saved
campaign chapter starts. Reconstructed arena log/combat display metadata is omitted
from the tactical fixture; positions, HP, actions, ownership, resources, round and
unit identities are restored. This is a compute comparison, not a quality study.

```sh
uv run --with pyarrow python scripts/benchmark-policy-phases.py \
  --report artifacts/overnight-unit-zero-clean-3/report.json \
  --campaigns artifacts/priority-essential-crisis/mixed/details.jsonl.gz \
  --repeat 1000 --out artifacts/policy-phases
```

The native `--phase-stdio` protocol accepts `state`, `policy`, `seed`,
`recruitmentLimit`, optional `goals` and `diplomacy`, and either `beginTurn` or
`advanceTurn`. Generated arena states also supply their original `arena`
configuration so geometry and recruitment pools are restored correctly. The
response contains the complete resulting state and production action records.

## Final policy validation

The policy implementation is committed as `97076f1`. Its optimized native build
keeps the existing 12,240-game workload exact: all three repeats have identical
detail bytes, ordered outcomes and report aggregates against the accepted PR21
journal. Whole-command times were **10.21, 9.66 and 9.52 seconds**, with a
**9.66-second median**. The first repeat exceeded ten seconds; the earlier
performance-only build's three measurements remain **9.49, 9.74 and 9.59 seconds**.

Fresh paired quality studies used the final binary, production commands, a
30-round cap and one recruitment per phase. Campaign attempts began in chapter
one, earned their carryover and received no retries. Each configuration ran once
with Mixed and once with the new player policy against the same existing enemy.

| Study | Configurations per policy | Mixed completed | New policy completed | Mixed caps | New caps |
| --- | ---: | ---: | ---: | ---: | ---: |
| Battle, seeds 9001–9050 | 19,200 | 14,087 | 14,689 | 291 | 190 |
| Campaign, seeds 8001–8200 | 4,800 | 3,231 | 3,701 | 787 | 319 |

Battler gained **3.14 percentage points** in battle wins. A paired interval
clustered by seed is **2.41–3.87 points**. Every landscape and existing-opponent
aggregate improved. Coastal player armies declined from 3,528 to 3,404 wins out
of 4,800, and 53 of 192 smaller faction/map/opponent strata had fewer wins. The
aggregate improvement does not imply a win-rate improvement for every faction.

Priority gained **9.79 percentage points** in campaign completion; the paired
seed-clustered interval is **7.68–11.90 points**. Completion held or improved for
each whole campaign: Ashen Oath 1,087/1,087, Borrowed Tide 1,180/1,188, Green
Accord 442/796 and Lantern Road 522/630 (Mixed/Priority, each out of 1,200).
Four of the 24 campaign/diplomacy/opponent strata declined. Borrowed Tide against
Classic fell from 199 to 195 completions out of 200 under each diplomacy setting;
Green Accord against Classic fell from 133 to 128. All strata, named losses,
defeats and chapter counts are retained in the evidence.

Priority is a hybrid campaign controller. It uses full Mixed planning for essential
actors in crisis and throughout campaigns containing a defeat plus unrestricted
seal objective; Ashen Oath currently meets that condition. Its Ashen results are
therefore Mixed results. Battler rejects campaigns, and both new policies share
the same battle planner. Neither changes the authored default policies.

Across the fresh complete-game studies, player decision CPU fell from **16.23 to
13.92 seconds** in battles and **26.43 to 18.88 seconds** in campaigns. Those
totals include changes in game duration and encountered states. The separate
same-position study repeated 57 arena positions and 12 complete campaign chapter
starts 1,000 times per policy. Campaign decision CPU fell from **2.75 to 1.94
seconds**, about **29%**. Battle decision CPU was close: **3.15 seconds** for
Mixed, **3.05** for Priority and **3.11** for Battler. A substantial additional
battle-compute reduction, including the proposed fourfold target, has not been
achieved.

Replacing Mixed with Priority in both seats of the full tournament still schedules
**12,240 games** (11,520 arenas and 720 campaigns), with the same limits, seeds,
four workers and complete Arrow recording. Whole-command times were **8.99, 9.28
and 8.82 seconds**, giving an **8.99-second median**, about **7%** below the
existing-policy median in this build. This changes gameplay deliberately; it is
not part of the existing-policy equality claim. Repeats had identical ordered
outcomes, report aggregates and reported detail hashes.

The final full verification passed all eight groups in 71.12 seconds, with combined
tests taking 7.44 seconds against the unchanged 20-second limit. Complete raw
traces and timing logs are saved in the local `artifacts/` directories; archived
runs are lossless ZIP files whose CRCs were checked before removing their expanded
copies. Reviewed machine-readable measurements, every quality stratum, binary and
source fingerprints, and comparison exclusions are in
[benchmarks/priority-policies.json](benchmarks/priority-policies.json).

## Direct self-play and mean turn timing

Use `--self-play` to schedule arena games with the same strategy on both sides,
while retaining every faction pair and both initiatives. Use `--profile-turns`
to include `turnTiming` in the JSON report and console summary, plus a policy
comparison table in the Markdown report. These options also pass through the npm
simulation entry point. They are opt-in; default schedules and gameplay records
keep their existing format. `--self-play` requires arena mode.

```sh
rust/target/release/hex-command-sim --mode arena --self-play --profile-turns \
  --policies classic,rush,mixed,battler,priority --seeds 10 --seed 1 \
  --rounds 30 --recruits 1 --workers 1 --out artifacts/self-play/report.json
```

One turn is one side's completed own phase, rather than a complete two-sided
round. `turnTiming.policies` combines both seats by their actual turn counts;
`turnTiming.rows` also exposes each seat separately. `meanDecisionMicrosPerTurn`
measures chooser CPU, and `meanTurnMicros` includes command execution and phase
snapshots. Turn-transition events, journal encoding, startup and report aggregation
are outside the phase interval. Linux reports thread CPU time; other platforms
identify their elapsed-clock fallback in `turnTiming.clock`. Timing is run metadata,
explicitly excluded by the gameplay comparer.

Five sequential, instrumented native runs used 1,280 self-play games per strategy:
all faction/map pairs, both initiatives, seeds 1–10, a 30-round cap, one recruit and
one worker. Every game retained its complete Arrow history.

| Strategy | Whole command seconds | Own turns | Mean decision µs/turn | Mean complete phase µs/turn |
| --- | ---: | ---: | ---: | ---: |
| Classic | 1.73 | 30,154 | 35.24 | 41.69 |
| Rush | 3.55 | 43,614 | 59.33 | 67.36 |
| Mixed | 3.13 | 35,947 | 62.59 | 71.25 |
| Battler | 2.77 | 34,548 | 54.58 | 63.26 |
| Priority | 2.72 | 34,548 | 53.73 | 62.38 |

Battler and Priority produced identical outcomes and endpoints in all 1,280 games
after normalizing the policy labels; their timing difference is small. Other
strategies generate different game states and turn counts, so these are self-play
averages, separate from the earlier fixed-position comparison. These are single
whole-command samples, not repeated medians. The binary reused the accepted
compiler profile for this measurement-only change.

Profiling was also switched on and off for ten production games across all five
policies with two workers: per-game detail bytes and gameplay/report aggregates
matched, and independently decoded phase records verified the turn counts and
weighted means. Full verification passed all eight groups in 68.13 seconds;
combined tests took 7.39 seconds. Exact measurements, clock, limits, fingerprints
and provenance are in
[benchmarks/policy-selfplay.json](benchmarks/policy-selfplay.json).

## Main-branch self-play reference

The default branch was still `b1f553de69c0de8a340088c0a9ed42b2149fb27e` when
checked against GitHub. It contains Classic, Rush and Mixed. An isolated checkout
retained its AI, engine, allocator and ordinary release settings, adding only the
same chooser/phase CPU timers and a final timing summary. Each policy ran the same
1,280 self-play games, limits, seeds, faction/map pairs, initiatives and one worker.

| Strategy | Main AI µs/turn | Optimized AI µs/turn | Decision CPU speedup | Main command seconds | Optimized command seconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| Classic | 109.49 | 35.24 | 3.11× | 4.34 | 1.73 |
| Rush | 255.33 | 59.33 | 4.30× | 12.63 | 3.55 |
| Mixed | 262.07 | 62.59 | 4.19× | 10.79 | 3.13 |

All 3,840 ordered game outcomes, endpoint hashes and report aggregates matched
the optimized reference after excluding explicit run/journal metadata and its
self-play schedule marker. Actual phase counts, chooser-call counts and caps also
matched. Full histories remain saved locally; this comparison did not decode them
again. Battler and Priority have no main-branch versions.

These are production-build comparisons: main uses thin LTO, eight codegen units
and no PGO/native-target additions, while the optimized reference uses its native
fat-LTO/profile-guided build. The decision CPU improvement includes AI, engine,
allocator and compiler changes; journal encoding is outside the decision timer.
Whole-command numbers are single instrumented samples, not repeated medians.
The exact measurements, compiled fingerprints and instrumentation provenance are
in [benchmarks/main-policy-selfplay.json](benchmarks/main-policy-selfplay.json).
