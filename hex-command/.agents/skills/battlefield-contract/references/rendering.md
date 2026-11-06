# Battlefield rendering

Read when changing runtime map rendering or measuring its performance.
The implementation lives under `src/presentation/`; use its worker, protocol and
terrain modules for current cache layouts, limits and message shapes.

## Ownership and presentation

One persistent module worker owns the transferred OffscreenCanvas and WebGL 2
context. The browser composites that canvas directly. Terrain, lighting, fog,
units, items and geometric overlays are drawn there from Rust-published facts
and presentation snapshots. Main-thread SVG retains accessible interaction
geometry. Preserve the native artwork scale and complete-frame terrain contract
in [SKILL.md](../SKILL.md).

Keep native terrain, lighting and fog caches independent of the displayed canvas
size and device pixel ratio. Texture sampling remains nearest for pixel artwork;
geometric outlines may use antialiasing. Presentation changes must preserve
terrain inspection portraits and keyboard/click targets.

Workers, transferable OffscreenCanvas, worker WebGL 2 and createImageBitmap are
required capabilities. Failures use diagnostics and retry. Retry transfers a
fresh canvas; retain the previous display until its replacement is ready when
possible. Dispose workers, canvases and GPU resources with their owning session.

Speculative preparation must share actual-use work where possible and yield to
the current scene. Its failures are retried and diagnosed on actual use. Idle
rendering stops; superseded requests cannot paint over a newer scene.

## Measurement

Use the commands and trace controls in the
[development guide](../../../../docs/development.md#performance-diagnostics).
Normal readiness, completed GPU work, a browser animation-frame opportunity and
physical display scanout are different observations. Completed benchmark mode
waits for a GPU fence and includes polling overhead. Optional asynchronous GPU
timer queries measure execution; API wall times can include driver synchronization
and nested calls. Trace recording must not add completion fences.

Software GPU results are diagnostic, not a hardware performance budget. Keep
browser benchmarks outside the default tests, and distinguish presentation-only
workloads from gameplay or AI cost.
