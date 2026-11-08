---
name: performance
description: Optimize performance against a measured target. Use whenever reducing latency, CPU, allocation, memory, startup cost or code size is a task goal.
---

# Performance work

The order is **measure → diagnose the slowdown → optimize the identified cost →
measure again → clear the target → verify correctness**. Diagnosis must precede
optimization. Full verification is a later phase, not a way to finish an
optimization that is still too slow.

## Benchmark workflow

- Use the project's standard benchmark library directly. Do not build custom
  timing, calibration, warmup, sampling, replay or statistical controllers around it.
- Default Benchmark.js sampling budgets to **1 second per case/implementation**.
  Run a focused selection of workloads relevant to the current question; aim for
  **about two minutes or less** for an ordinary prepared-artifact run. Use the
  runner's case/variant selectors and `--time` override for deeper investigations.
- Keep complete matrices available through explicit `--full` runs; do not rerun
  every historical case/representation after each edit. A focused result only
  establishes evidence for its selected workloads, not unmeasured cases.
- Preserve raw library samples, error margins and exact source/artifact identities.
  Do not silently drop noisy rows, add automatic rerun gates, force GC, or add
  fixture validation/outcome-hashing passes to benchmark execution. Initialize
  modules and prepare actual arguments outside timing. Measure ownership cleanup
  inside operations where that is part of using the API.

## Establish the gate

- Recover the user's target, relevant workloads, comparison reference and required
  representations from the conversation and project instructions. State them
  briefly before changing code. Do not invent a weaker numerical threshold.
- Preserve a reproducible baseline and exact source/build/artifact identities.
  Commit the candidate before measuring; retain raw library samples.
- Measure equivalent work with matching library settings, ordinary GC and serial runs.
  Keep builds, profiling and tests out of timed benchmark runs. Separate component
  experiments from application measurements.

## Until measurements clear the gate

- Run benchmarks first, then diagnose the slowdown. Read emitted code and glue,
  profile, trace, instrument and isolate costs. Identify the responsible path,
  the extra work it performs and evidence connecting that work to the measured
  gap. A plausible explanation or a list of suspicious functions is not enough.
- Only then optimize that identified cost, including changing the architecture
  when warranted. Instrumentation and controlled experiments may establish the
  diagnosis; do not adopt speculative production optimizations before it exists.
  Remeasure, then diagnose any remaining slowdown before the next optimization.
- Use only build checks and focused probes necessary to obtain trustworthy
  measurements: matching outcomes, ownership/lifetime checks and failure cleanup
  where the measured path requires them. Do not launch full verification, broad
  correctness suites, demo/compiler matrices or final reviews in this phase.
- Inspect relevant rows individually. Repeat outliers under controlled conditions.
  An aggregate win, a faster different representation, or an isolated component
  result does not erase a required workload's regression. Explain representation
  differences; do not quietly remove them from the target.
- Distinguish measured attribution from hypotheses. If sampling cannot explain
  the remaining gap, obtain better evidence instead of asserting that the gap is
  unavoidable or that there is nothing left to optimize.
- When asked for a table, show the latest available table immediately, identifying
  its candidate and limitations. A status question steers ongoing work; it does
  not end the optimization unless the user asks to stop or changes the objective.

## Clear the gate explicitly

Before running full verification, present the measured candidate/reference table
and assess it against the original target. Account for every relevant unresolved
regression. Do not substitute “most cases”, “close enough”, passing probes or
planned fixes for the required result. If the target is qualitative, explain how
the evidence meets it; do not silently reinterpret it to admit a known gap.

Starting verification is permitted only after that assessment establishes the
target is met, or the user explicitly authorizes an exception. Never infer target
clearance from the fact that verification has already started.

## After the gate

Run the required correctness and integration checks, including stale builds.
If a fix changes a measured path, measure it again before resuming full
verification. Obtain a requested final review only when ready to return the work.
Finish with a commit/amend and report the performance table, check results and
material limitations. If blocked, report the specific blocker and current
measurements; do not present unfinished optimization as completed work.
