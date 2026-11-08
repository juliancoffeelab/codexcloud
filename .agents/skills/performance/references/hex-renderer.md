# Hex renderer benchmark reference

Historical source reference using the production OCaml Phaser scene and Rust engine.
This is reference material, not a runnable benchmark or part of the game build.
The former developer catalog, synthetic JSON fixtures, asset plugin and launcher
were removed. Imports and module names below describe that former environment.

## Workloads and measurement semantics

The restored renderer workloads cover ordinary maps, map area, actor and light
counts, selection, legal movement, carried-item drops when available, session
replacement, thirty-enemy spawning, partial/global fog removal, one hundred
village lights and two hundred illuminated death marks. Synthetic fierce
navigation uses the actual input handlers.
The extended simultaneous peak retains 232 live actors, 200 death marks and
300 lights together, using native scripted spawning after the historical
removal transition. It measures both fierce navigation and fixed redraws.

This page measures continuous browser frame cadence and native scene submission
elapsed time from Phaser prerender to render, rather than repeated operation throughput.
That submission duration excludes engine commands, scene update and GPU/compositor
completion. The frame sampler establishes its baseline before invoking commands
and samples after they finish, so synchronous command stalls remain in cadence. It therefore records raw
scheduled animation-frame intervals for the cadence verdict, plus separate callback-arrival
intervals for responsiveness. Callback jitter is not counted as a missed frame;
Benchmark.js continues to own the operation benchmarks above. A quiet stationary
scene is reported separately from a forced production redraw. An unavailable
operation is reported explicitly rather than replaced with a fabricated one.

Fixture assertions run outside measured windows and read published state directly;
they do not serialize checkpoint JSON. This renderer suite does not run gameplay
persistence or establish its cost.

Downloads include workload assertions, raw frame and submission samples,
resource estimates, browser/device/GPU information and source identities.
A valid workload can still fail the 60 Hz target. Software rendering and
synthetic input do not establish physical-device performance; run the page on
the intended browser and hardware to assess that target.

The download includes per-export inclusive Wasm wall time, scene submission
elapsed time, and GPU elapsed queries when the browser exposes the disjoint
timer extension. Unsupported GPU timing is explicit. Browser long-frame/task
entries include script attribution where supported. These scopes overlap: script
and Wasm timings can include each other, and GPU work runs asynchronously.
They must not be added together or subtracted from frame intervals to invent an
exclusive JS or GPU wait estimate. Timer wrappers and GPU queries add profiling
overhead; this page reports an instrumented run. Pending GPU queries are polled
without blocking or forcing GPU completion; unresolved/disjoint samples are
reported, rather than treated as zero.

Whole-map navigation adds separate wide (0.5×), normal (1×), and close (2×)
spiral sweeps on Bell Night, the large-area fixture, and the simultaneous peak.
Preparation discovers actual pan limits through wheel input outside measurement.
The sweep uses the production wheel handler, with at most one movement per
animation frame and overlapping viewport steps. It visits outer edges and works
inward, recording actual positions, reached edges, event hit tests and a 16×16
viewport-coverage grid. This is sampled spatial coverage, not proof of displayed
pixels. A map that already fits the viewport is explicitly reported and redrawn.
The original fierce touch sequence now alternates four expanding/contracting
pinches rather than repeatedly magnifying the same small region. Synthetic input
still does not establish physical-touch performance. These additional cases
extend full-suite runtime according to map size and device speed.

Format 8 adds nested command-path spans: generated command encoding, bridge call,
response decoding and output cleanup; native terrain update and publication
reconciliation; battlefield-frame preparation; scene refresh and terrain-page
rebuild/lighting; and per-frame update/render scopes. Each span has a timestamp
relative to the measured window; decoder spans also include packet and string-pool
byte counts. Frame render setup includes the poststep-to-prerender interval. Summaries group spans by name. These are
inclusive, overlapping wall-time scopes—not additive CPU accounting. The decoder
scope starts when the generated bridge returns its Reader and ends before output
cleanup; bridge time still includes Wasm execution and its input preparation.
The instrumentation is enabled by the developer page; normal gameplay registers
no measurement listener or codec wrapper.

## benchmark.html

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <title>Hex Command · renderer benchmark</title>
    <link rel="icon" href="data:," />
    <link rel="stylesheet" href="/src/style.css" />
    <link rel="stylesheet" href="/src/benchmark/page.css" />
  </head>
  <body>
    <main class="benchmark-page">
      <header>
        <h1>Renderer benchmark</h1>
        <button id="run" disabled>Run</button>
        <button id="download" disabled>Download</button>
      </header>
      <p id="status" role="status">Initializing…</p>
      <div class="workspace map-only">
        <section class="map-section">
          <div
            id="map"
            data-phaser-battlefield
            class="map-scroll phaser-battlefield battlefield-window"
          >
            <div class="battlefield-clip" id="benchmark-host"></div>
          </div>
        </section>
      </div>
    </main>
    <script type="module" src="/src/benchmark/page.mjs"></script>
  </body>
</html>
```

## src/benchmark/page.mjs

```javascript
// Developer measurement UI; game behavior and rendering remain in production OCaml/Rust.
import { initialize, api as runtimeApi } from "@wasm/bindings.runtime.mjs";
import wasmUrl from "../../rust/pkg-benchmark/backend.wasm?url";
import documents from "virtual:benchmark-documents";
import terrain from "virtual:terrain-benchmark-prebuilt";
import candidate from "virtual:benchmark-identity";
import * as Game from "@frontend/Benchmark_game.js";
import { fierceGestures, syntheticTouchSupport } from "./gestures.mjs";
import { cadenceVerdict, aggregateCadence } from "./verdict.mjs";
import { prepareSweep, sweepMap } from "./sweep.mjs";
import { instrumentWasm, instrumentCodecs, telemetry } from "./telemetry.mjs";
const byId = (id) => document.getElementById(id);
const status = (value) => {
  byId("status").textContent = value;
};
const explain = (error) =>
  Game.error_message ? Game.error_message(error) : String(error?.stack ?? error);
let lastHardware;
const nextFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const errors = [];
addEventListener("error", (event) => errors.push(event.error?.stack ?? event.message));
addEventListener("unhandledrejection", (event) =>
  errors.push(String(event.reason?.stack ?? event.reason)),
);
let game,
  stopped = false,
  running = false,
  result;
let activeCapture, diagnostics, wasmSupport, codecSupport;
const source = {
  upstream: "28e4f05c40bc3718a755e8a900357a95546e9e08",
  workloads: "15d6c6fa75d41c17ba8b66adcc9ea6c725a4c6a1",
  implementation: "OCaml production scene; current checkout",
  candidate,
  fixtureCatalog: "separate developer-only runtime-catalog binary",
};
function check() {
  if (stopped) throw new DOMException("Benchmark stopped", "AbortError");
  Game.status(game);
  if (errors.length) throw new Error(errors.join("\n"));
}
function camera() {
  const c = Game.camera(game);
  return { x: c?.left ?? 0, y: c?.top ?? 0, zoomX: c?.zoom ?? 1, zoomY: c?.zoom ?? 1 };
}
function canvas() {
  const value = byId("benchmark-host").querySelector("canvas");
  if (!value) throw new Error("Production Phaser canvas missing");
  return value;
}
function wheel(x, y, zoom = false) {
  const node = canvas(),
    b = node.getBoundingClientRect();
  node.dispatchEvent(
    new WheelEvent("wheel", {
      bubbles: true,
      cancelable: true,
      clientX: b.left + b.width * 0.5,
      clientY: b.top + b.height * 0.5,
      deltaX: x,
      deltaY: y,
      ctrlKey: zoom,
      deltaMode: 0,
    }),
  );
}
async function ready() {
  const start = performance.now();
  while (Game.status(game)?.phase !== "ready") {
    check();
    if (performance.now() - start > 45000)
      throw new Error("Current resources/first frame did not become ready within 45s");
    await nextFrame();
  }
  return performance.now() - start;
}
async function replace(kind, id) {
  check();
  status(`Loading ${id}…`);
  const start = performance.now();
  Game.start(game, kind, id);
  await ready();
  Game.return_to_commander(game);
  const warmupStart = performance.now();
  // is_loading includes the planned decode/admission queues, not just network IO.
  while (Game.is_loading(game)) {
    check();
    if (performance.now() - warmupStart > 120000)
      throw new Error(`Resource prewarm for ${id} did not settle within 120s`);
    await nextFrame();
  }
  const backgroundWarmupMs = performance.now() - warmupStart;
  // An idle on-demand scene does not warm renderer code. Exercise its actual
  // production draw path after speculative loading settles, before sampling.
  let rendererWarmupFrames = 0;
  const rendererWarmupStart = performance.now();
  const off = Game.watch_render(game, () => rendererWarmupFrames++);
  try {
    while (performance.now() - rendererWarmupStart < 1000) {
      check();
      Game.redraw(game);
      await nextFrame();
    }
  } finally {
    off();
  }
  // Let the last warmup submission leave the compositor before recording an
  // otherwise stationary scene. This does not force GPU synchronization.
  await nextFrame();
  await nextFrame();
  if (Game.is_loading(game)) throw new Error(`Resource pipeline restarted during ${id} warmup`);
  if (!rendererWarmupFrames) throw new Error("Renderer warmup produced no complete frames");
  const rendererWarmupMs = performance.now() - rendererWarmupStart;
  const snapshot = Game.snapshot(game);
  const checkpoint = JSON.parse(Game.checkpoint(game));
  if (checkpoint.state?.mapId && checkpoint.state.mapId !== snapshot.map)
    throw new Error("Checkpoint and rendered map identity disagree");
  return {
    kind,
    id,
    ms: performance.now() - start,
    backgroundWarmupMs,
    rendererWarmupMs,
    rendererWarmupFrames,
    loaderStillActive: Game.is_loading(game),
    snapshot,
  };
}
function distribution(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const percentile = (p) =>
    sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))] ?? null;
  return {
    samples: values.length,
    p50Ms: percentile(0.5),
    p95Ms: percentile(0.95),
    p99Ms: percentile(0.99),
    maxMs: sorted.at(-1) ?? null,
    meanMs: values.length ? values.reduce((a, b) => a + b, 0) / values.length : null,
  };
}
function layout() {
  const node = canvas(),
    b = node.getBoundingClientRect(),
    v = visualViewport;
  const viewport = {
    left: v?.offsetLeft ?? 0,
    top: v?.offsetTop ?? 0,
    width: v?.width ?? innerWidth,
    height: v?.height ?? innerHeight,
    scale: v?.scale ?? 1,
  };
  const visibleArea =
    Math.max(
      0,
      Math.min(b.right, viewport.left + viewport.width) - Math.max(b.left, viewport.left),
    ) *
    Math.max(0, Math.min(b.bottom, viewport.top + viewport.height) - Math.max(b.top, viewport.top));
  const visibleFraction = b.width * b.height ? visibleArea / (b.width * b.height) : 0;
  return {
    x: b.x,
    y: b.y,
    width: b.width,
    height: b.height,
    backingWidth: node.width,
    backingHeight: node.height,
    scrollX,
    scrollY,
    viewport,
    visibleFraction,
    fullyVisible: visibleFraction >= 0.999,
  };
}
function hardware() {
  const node = byId("benchmark-host").querySelector("canvas");
  if (!node && lastHardware) return { ...lastHardware, canvasRemoved: true };
  const gl = node?.getContext("webgl") || node?.getContext("webgl2");
  const debug = gl?.getExtension("WEBGL_debug_renderer_info");
  return (lastHardware = {
    userAgent: navigator.userAgent,
    hardwareConcurrency: navigator.hardwareConcurrency,
    deviceMemoryGiB: navigator.deviceMemory,
    devicePixelRatio,
    viewport: { width: innerWidth, height: innerHeight },
    renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : "unavailable",
    visibility: document.visibilityState,
  });
}
function report(workloads, extra = {}) {
  return {
    format: "hex-command-browser-benchmark",
    version: 8,
    createdAt: new Date().toISOString(),
    source,
    device: hardware(),
    timingSupport: {
      wasm: wasmSupport,
      codecs: codecSupport,
      applicationSpans: true,
      ...diagnostics?.support,
    },
    browserLongFrames: diagnostics?.frames(),
    validity: {
      status:
        errors.length || extra.failure || workloads.some((row) => row.validity.status === "ERROR")
          ? "ERROR"
          : "VALID",
    },
    performance: aggregateCadence(workloads.filter((row) => row.cadence).map((row) => row.cadence)),
    workloads,
    errors: [...errors],
    ...extra,
  };
}
function publish(value) {
  result = value;
  byId("download").disabled = false;
}
async function measure(label, ms, action, verify) {
  check();
  status(label);
  const before = Game.snapshot(game);
  const intervals = [];
  const callbackIntervals = [];
  const frameTimestamps = [];
  let previous,
    frameId,
    recording = true;
  let waiters = [];
  const sampledFrame = () => new Promise((resolve) => waiters.push(resolve));
  const layoutBefore = layout();
  let viewportChanged = false;
  const onViewportChange = () => {
    viewportChanged = true;
  };
  const viewportTargets = [window, window.visualViewport].filter(Boolean);
  for (const target of viewportTargets) {
    target.addEventListener("resize", onViewportChange);
    target.addEventListener("scroll", onViewportChange);
  }
  const visibility = [document.visibilityState];
  const onVisibility = () => visibility.push(document.visibilityState);
  document.addEventListener("visibilitychange", onVisibility);
  function sample(at) {
    if (!recording) return;
    // Preserve both clocks: frame cadence and callback responsiveness differ.
    const arrived = performance.now();
    diagnostics.poll();
    frameTimestamps.push({ scheduledMs: at, arrivedMs: arrived });
    if (previous !== undefined) {
      intervals.push(at - previous.scheduled);
      callbackIntervals.push(arrived - previous.arrived);
    }
    previous = { scheduled: at, arrived };
    frameId = requestAnimationFrame(sample);
    const pending = waiters;
    waiters = [];
    for (const resolve of pending) resolve(at);
  }
  frameId = requestAnimationFrame(sample);
  await sampledFrame();
  const started = performance.now();
  const submissions = [];
  const timings = diagnostics.row();
  activeCapture = { started, submissions, measures: [], ...timings };
  const measures = activeCapture.measures;
  let details = {},
    failure;
  try {
    details = (await action?.()) ?? {};
    while (performance.now() - started < ms) {
      check();
      await sampledFrame();
    }
    // A queued RAF timestamp can precede a late timer callback. Keep the next
    // two boundaries so that callback’s stall cannot escape the recorded tail.
    await sampledFrame();
    await sampledFrame();
  } catch (error) {
    failure = explain(error);
  } finally {
    recording = false;
    cancelAnimationFrame(frameId);
    for (const target of viewportTargets) {
      target.removeEventListener("resize", onViewportChange);
      target.removeEventListener("scroll", onViewportChange);
    }
    activeCapture = undefined;
    document.removeEventListener("visibilitychange", onVisibility);
  }
  const ended = performance.now();
  const layoutAfter = layout();
  if (
    viewportChanged ||
    !layoutBefore.fullyVisible ||
    !layoutAfter.fullyVisible ||
    JSON.stringify(layoutBefore) !== JSON.stringify(layoutAfter)
  )
    failure ??= "Canvas clipped or viewport changed during measurement";
  let after;
  const verificationStarted = performance.now();
  try {
    after = Game.snapshot(game);
    await verify?.(details);
  } catch (error) {
    failure ??= explain(error);
  }
  return {
    label,
    startedAtMs: started,
    endedAtMs: ended,
    elapsedMs: ended - started,
    before,
    after,
    details,
    intervals,
    intervalClock:
      "requestAnimationFrame timestamp; browser frame cadence, not physical display FPS",
    callbackIntervals,
    callbackResponsiveness: distribution(callbackIntervals),
    callbackIntervalClock:
      "performance.now at animation-frame callback arrival; includes scheduling jitter",
    verificationElapsedMs: performance.now() - verificationStarted,
    frameTimestamps,
    layout: { before: layoutBefore, after: layoutAfter },
    sceneSubmissionScope:
      "Phaser prerender to render; excludes engine commands, scene update and GPU/compositor completion",
    timings,
    measures,
    measureSummary: Object.fromEntries(
      [...new Set(measures.map((s) => s.name))].map((name) => {
        const samples = measures.filter((s) => s.name === name).map((s) => s.durationMs);
        return [name, { ...distribution(samples), totalMs: samples.reduce((a, b) => a + b, 0) }];
      }),
    ),
    timingScopesOverlap: true,
    sceneSubmissions: submissions,
    renderSubmissionElapsed: distribution(submissions.map((sample) => sample.durationMs)),
    distribution: distribution(intervals),
    cadence: details.unavailable ? undefined : cadenceVerdict(intervals),
    visibility,
    validity: {
      status: details.unavailable
        ? "UNAVAILABLE"
        : failure || visibility.some((value) => value !== "visible")
          ? "ERROR"
          : "VALID",
      ...(failure && { error: failure }),
    },
  };
}
async function redrawFrames(ms = 1000) {
  const start = performance.now();
  let requests = 0;
  while (performance.now() - start < ms) {
    check();
    Game.redraw(game);
    requests++;
    await nextFrame();
  }
  return {
    redrawRequests: requests,
    semantics:
      "Invalidate the production game each browser frame; fixed viewport, no alternate renderer.",
  };
}
async function navigate(mode) {
  const before = camera();
  let events = 0;
  if (mode === "fierce") {
    const reason = syntheticTouchSupport(canvas());
    if (reason) return { unavailable: reason };
    const details = await fierceGestures(canvas(), camera, check);
    const after = camera();
    if (
      before.x === after.x &&
      before.y === after.y &&
      before.zoomX === after.zoomX &&
      !(details.immediateCameraChanges > 0)
    )
      throw new Error("Fierce input did not change camera");
    return details;
  }
  for (let i = 0; i < 50; i++) {
    check();
    if (mode === "zoom") wheel(0, i % 2 ? -18 : 18, true);
    else wheel(i % 4 < 2 ? 24 : -24, i % 6 < 3 ? 20 : -20);
    events++;
    await delay(16);
  }
  const after = camera();
  return { mode, events, before, after };
}
async function wholeMapSweeps(label, workloads) {
  const input = {
    canvas: canvas(),
    camera,
    wheel,
    check,
    nextFrame,
    redraw: () => Game.redraw(game),
  };
  for (const [name, zoom] of [
    ["wide", 0.5],
    ["normal", 1],
    ["close", 2],
  ]) {
    status(`Preparing ${label} ${name} sweep…`);
    const plan = await prepareSweep(input, zoom);
    workloads.push(
      await measure(
        `${label}: whole-map spiral (${name}, ${zoom}x)`,
        0,
        () => sweepMap(input, plan),
        (details) => {
          if (
            details.coverage.fraction < 0.99 ||
            !Object.values(details.edgesReached).every(Boolean)
          )
            throw new Error(
              `Whole-map sweep coverage incomplete: ${JSON.stringify(details.coverage)}`,
            );
          if (details.eventsOutsideVisibleCanvas)
            throw new Error("Whole-map input did not hit the visible canvas");
        },
      ),
    );
  }
}
function dynamicSnapshot() {
  return { ...Game.snapshot(game), ...Game.dynamic_snapshot(game) };
}
function simultaneousPeak() {
  const after = dynamicSnapshot();
  if (
    after.authoritativeActors !== 232 ||
    after.friendly !== 31 ||
    after.enemies !== 201 ||
    after.deathMarks !== 200 ||
    after.redDeathLights !== 200 ||
    after.villageFixtureLights !== 100 ||
    after.lights !== 300 ||
    after.fog !== 0
  )
    throw new Error(`Simultaneous peak workload invariant failed: ${JSON.stringify(after)}`);
  return after;
}
async function localChanges(label) {
  const move = Game.legal_move(game);
  const unit = move?.unit;
  const selection = await measure(`${label}: selection`, 1500, async () => {
    if (!unit) return { unavailable: "No controllable unit with a legal move" };
    for (let i = 0; i < 10; i++) {
      if (i % 2) Game.clear_selection(game);
      else if (!Game.select(game, move.from))
        throw new Error("Selection did not select the real actor");
      await delay(150);
    }
    Game.clear_selection(game);
    return { publications: 10 };
  });
  const movement = await measure(`${label}: legal movement`, 1000, async () => {
    if (!move) return { unavailable: "No legal player move in this fixture state" };
    const start = performance.now();
    Game.move(game, move.to);
    return { commandMs: performance.now() - start, move };
  });
  const item = Game.carried_item(game);
  const drop = await measure(`${label}: item drop`, 1000, async () => {
    if (!item) return { unavailable: "No carried item in this fixture state" };
    const start = performance.now();
    Game.drop_item(game, item.unit, item.item);
    return { commandMs: performance.now() - start, item };
  });
  return [selection, movement, drop];
}
const ordinary = [
  { kind: "scenario", id: "mirror" },
  { kind: "campaign", id: "mirror-night" },
  { kind: "campaign", id: "bell-night" },
];
async function run() {
  const suite = "full";
  if (running) throw new Error("Finish the active measurement first");
  running = true;
  stopped = false;
  byId("run").disabled = true;
  byId("download").disabled = true;
  result = undefined;
  errors.length = 0;
  const workloads = [],
    readiness = [];
  const start = performance.now();
  try {
    await reset();
    hardware();
    const axes = ["base", "area", "area-high", "actors", "actors-high", "lights", "lights-high"];
    for (const fixture of ordinary) {
      readiness.push(await replace(fixture.kind, fixture.id));
      workloads.push(await measure(`${fixture.id}: warm idle`, 1000));
      workloads.push(...(await localChanges(fixture.id)));
      workloads.push(await measure(`${fixture.id}: wheel pan`, 1100, () => navigate("pan")));
      workloads.push(await measure(`${fixture.id}: wheel zoom`, 1100, () => navigate("zoom")));
      workloads.push(
        await measure(`${fixture.id}: fierce navigation`, 3800, () => navigate("fierce")),
      );
      if (fixture.id === "bell-night") await wholeMapSweeps(fixture.id, workloads);
    }
    for (const axis of axes) {
      readiness.push(await replace("scenario", `benchmark-${axis}`));
      const before = Game.snapshot(game);
      const roster = JSON.parse(documents[`game/scenario/benchmark-${axis}/roster.json`]);
      const expectedActors = roster.factions.reduce(
        (total, faction) => total + faction.units.length,
        0,
      );
      if (before.actors !== expectedActors || before.authoritativeActors !== expectedActors)
        throw new Error(
          `Actor fixture expected ${expectedActors}, published ${before.actors}, authoritative ${before.authoritativeActors}`,
        );
      const map = JSON.parse(documents[`game/scenario/benchmark-${axis}/map.json`]);
      const expectedLights =
        map.tiles.filter((tile) => tile.light).length + (map.lighting?.lights?.length ?? 0);
      if (before.lights !== expectedLights)
        throw new Error(`Light fixture expected ${expectedLights}, published ${before.lights}`);
      workloads.push(await measure(`${axis}: warm idle`, 1000));
      if (axis === "base") workloads.push(...(await localChanges(axis)));
      workloads.push(await measure(`${axis}: fixed viewport redraw`, 1000, () => redrawFrames()));
      if (axis.endsWith("high"))
        workloads.push(await measure(`${axis}: fierce navigation`, 3800, () => navigate("fierce")));
      if (axis === "area-high") await wholeMapSweeps(axis, workloads);
    }
    for (const withNavigation of [false, true]) {
      readiness.push(await replace("campaign", "benchmark-burst"));
      wheel(0, Math.log(2) / 0.002, true);
      await delay(200);
      const before = Game.snapshot(game);
      workloads.push(
        await measure(
          `Thirty-enemy burst${withNavigation ? " + fierce navigation" : ""}`,
          withNavigation ? 4000 : 1800,
          async () => {
            const navigation = withNavigation ? navigate("fierce") : Promise.resolve();
            const at = performance.now();
            Game.move(game, "11,12");
            const commandMs = performance.now() - at;
            const navigationDetails = await navigation;
            await delay(500);
            return { commandMs, navigation: navigationDetails };
          },
          (details) => {
            const after = Game.snapshot(game);
            details.spawned = after.authoritativeActors - before.authoritativeActors;
            if (details.spawned !== 30)
              throw new Error("Expected exactly thirty newly spawned enemies");
          },
        ),
      );
    }
    for (const [name, tile, navigation] of [
      ["local fog", "18,10", false],
      ["whole-map fog", "17,9", false],
      ["100 village lights", "17,11", false],
      ["combined", "16,10", false],
      ["combined + fierce navigation", "16,10", true],
    ]) {
      readiness.push(await replace("campaign", "benchmark-dynamic"));
      Game.return_to_commander(game);
      await delay(200);
      const before = dynamicSnapshot();
      if (!before.fog || before.lights || before.actors < 200)
        throw new Error("Dynamic fixture initial workload assertion failed");
      workloads.push(
        await measure(
          `Dynamic ${name}`,
          navigation ? 4300 : 1300,
          async () => {
            const moving = navigation ? navigate("fierce") : Promise.resolve();
            const at = performance.now();
            Game.move(game, tile);
            const commandMs = performance.now() - at;
            const navigationDetails = await moving;
            await delay(600);
            return { commandMs, navigation: navigationDetails };
          },
          (details) => {
            const after = dynamicSnapshot();
            Object.assign(details, { before, after });
            const delta = (key) => after[key] - before[key];
            const unchanged =
              delta("friendly") === 0 && delta("enemies") === 0 && delta("deathMarks") === 0;
            const valid =
              name === "local fog"
                ? after.fog > 0 &&
                  after.fog < before.fog &&
                  !after.allVisibleFlag &&
                  delta("lights") === 0 &&
                  unchanged
                : name === "whole-map fog"
                  ? after.fog === 0 && after.allVisibleFlag && delta("lights") === 0 && unchanged
                  : name === "100 village lights"
                    ? delta("villageFixtureLights") === 100 &&
                      delta("lights") === 100 &&
                      delta("fog") === 0 &&
                      unchanged &&
                      after.villagesLitFlag
                    : delta("friendly") === 30 &&
                      delta("enemies") === -200 &&
                      delta("deathMarks") === 200 &&
                      delta("redDeathLights") === 200 &&
                      delta("villageFixtureLights") === 100 &&
                      delta("lights") === 300 &&
                      after.fog === 0 &&
                      after.allVisibleFlag &&
                      after.villagesLitFlag &&
                      after.skeletons === 0;
            if (!valid)
              throw new Error(
                `Dynamic workload invariant failed: ${JSON.stringify({ before, after })}`,
              );
          },
        ),
      );
    }
    Game.return_to_commander(game);
    await delay(250);
    workloads.push(
      await measure("Resolved combined: fixed viewport redraw", 1000, () => redrawFrames()),
    );
    // Extend the historical transition with a simultaneous peak: the original
    // workload removes its enemies when creating death marks. The engine's
    // separate saturated event then refills those cells with new enemies.
    readiness.push(await replace("campaign", "benchmark-dynamic"));
    workloads.push(
      await measure(
        "Simultaneous actors, lights and death marks + fierce navigation",
        4300,
        async () => {
          Game.move(game, "16,9");
          const navigation = navigate("fierce");
          const navigationDetails = await navigation;
          await delay(600);
          return { navigation: navigationDetails };
        },
        (details) => {
          details.after = simultaneousPeak();
        },
      ),
    );
    wheel(0, Math.log(camera().zoomX / 0.5) / 0.002, true);
    await nextFrame();
    Game.return_to_commander(game);
    await delay(250);
    simultaneousPeak();
    if (Math.abs(camera().zoomX - 0.5) > 0.000001)
      throw new Error("Simultaneous peak did not reach minimum native zoom");
    workloads.push(
      await measure("Simultaneous peak: wide-view fixed redraw", 1000, () => redrawFrames()),
    );
    await wholeMapSweeps("Simultaneous peak", workloads);
    for (let i = 0; i < 3; i++) {
      readiness.push(await replace("scenario", "benchmark-base"));
      workloads.push(await measure(`Replacement ${i + 1}: warm idle`, 1000));
    }
    await diagnostics.settle();
    publish(report(workloads, { suite, readiness, elapsedMs: performance.now() - start }));
    status(`Finished: ${result.validity.status}; cadence ${result.performance.status}.`);
    return result;
  } catch (error) {
    publish(
      report(workloads, {
        suite,
        readiness,
        elapsedMs: performance.now() - start,
        failure: explain(error),
      }),
    );
    status(`Stopped: ${error.message ?? error}`);
    return result;
  } finally {
    running = false;
    byId("run").disabled = false;
  }
}
byId("run").onclick = () => run();
byId("download").onclick = () => {
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(result, null, 2)], { type: "application/json" }),
  );
  const link = document.createElement("a");
  link.href = url;
  link.download = `hex-renderer-${Date.now()}.json`;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
window.hexBenchmark = {
  api: Game,
  run,
  stop: () => {
    stopped = true;
  },
  get result() {
    return result;
  },
  game: () => game,
};
let unwatch, unwatchStart, unwatchMeasures, unwatchSteps;
function recordMeasure(name, at, duration, source) {
  if (activeCapture)
    activeCapture.measures.push({
      name,
      atMs: at - activeCapture.started,
      durationMs: duration,
      source,
    });
}
async function reset() {
  unwatchMeasures?.();
  unwatchSteps?.();
  unwatchStart?.();
  unwatchStart = undefined;
  unwatch?.();
  diagnostics?.dispose();
  unwatch = undefined;
  if (game) Game.dispose(game);
  game = undefined;
  byId("benchmark-host").replaceChildren();
  game = await Game.create(
    byId("benchmark-host"),
    Object.entries(documents),
    Object.entries(terrain),
  );
  await ready();
  diagnostics = telemetry(canvas(), () => activeCapture);
  unwatchMeasures = Game.watch_measures(game, (name, at, duration) =>
    recordMeasure(name, at, duration, "OCaml application"),
  );
  let updateEnded, renderStarted;
  unwatchSteps = Game.watch_steps(game, (at, duration) => {
    updateEnded = at + duration;
    recordMeasure("frame.update", at, duration, "Phaser prestep to poststep");
  });
  unwatchStart = Game.watch_gpu(
    game,
    () => {
      const at = performance.now();
      if (updateEnded !== undefined)
        recordMeasure(
          "frame.render_setup",
          updateEnded,
          at - updateEnded,
          "Phaser poststep to prerender: renderer setup/clear and event overhead",
        );
      diagnostics.begin();
      renderStarted = performance.now();
    },
    () => {
      recordMeasure(
        "frame.render",
        renderStarted,
        performance.now() - renderStarted,
        "Phaser game prerender to postrender; includes final flush, excludes initial clear",
      );
      diagnostics.end();
    },
  );
  unwatch = Game.watch_render(game, (at, durationMs) => {
    if (activeCapture)
      activeCapture.submissions.push({ atMs: at - activeCapture.started, durationMs });
  });
}
try {
  const instance = await initialize(wasmUrl);
  wasmSupport = instrumentWasm(instance, () => activeCapture);
  codecSupport = instrumentCodecs(instance, runtimeApi, () => activeCapture);
  byId("run").disabled = false;
  status("Ready. Run the full suite.");
} catch (error) {
  errors.push(explain(error));
  status(`Initialization failed: ${explain(error)}`);
}
addEventListener(
  "pagehide",
  () => {
    stopped = true;
    diagnostics?.dispose();
    if (game) Game.dispose(game);
  },
  { once: true },
);
```

## src/benchmark/gestures.mjs

```javascript
// Workload contract preserved from upstream 15d6c6fa75d41c17ba8b66adcc9ea6c725a4c6a1.
function syntheticTouchSupport(canvas) {
  if (typeof Touch !== "function" || typeof TouchEvent !== "function")
    return "This browser does not expose Touch/TouchEvent constructors. Wheel-driven whole-map sweeps remain available.";
  try {
    new Touch({ identifier: 1, target: canvas, clientX: 1, clientY: 1 });
  } catch (error) {
    return `Synthetic Touch construction unavailable: ${String(error)}`;
  }
}
async function fierceGestures(canvas, camera, check) {
  const bounds = canvas.getBoundingClientRect();
  const active = /* @__PURE__ */ new Map();
  const timeline = [];
  const begin = performance.now();
  const plans = [];
  const distance = bounds.width * 0.72;
  const strokeMs = Math.max(50, Math.min(180, Math.hypot(distance, bounds.height * 0.4) / 4.5));
  const steps = Math.ceil(strokeMs / (1e3 / 120));
  for (let gesture = 0; gesture < 10; gesture++) {
    const at = gesture * 300,
      id = gesture + 1,
      reverse = gesture % 2 === 1;
    const x0 = bounds.width * (reverse ? 0.14 : 0.86),
      x1 = bounds.width - x0;
    const y0 = bounds.height * (reverse ? 0.3 : 0.7),
      y1 = bounds.height - y0;
    plans.push({ at, kind: "touchstart", id, x: x0, y: y0 });
    for (let step = 1; step <= steps; step++)
      plans.push({
        at: at + (step * strokeMs) / steps,
        kind: "touchmove",
        id,
        x: x0 + ((x1 - x0) * step) / steps,
        y: y0 + ((y1 - y0) * step) / steps,
      });
    if (gesture > 0 && gesture % 2 === 0) {
      const expanding = gesture % 4 === 2;
      const startDistance = expanding ? 24 : 72;
      const endDistance = expanding ? 72 : 24;
      const pinchId = id + 1e3;
      const pinchDirection = x1 < bounds.width / 2 ? 1 : -1;
      plans.push({
        at: at + strokeMs + 8,
        kind: "touchstart",
        id: pinchId,
        x: x1 + pinchDirection * startDistance,
        y: y1,
      });
      for (let step = 1; step <= 6; step++)
        plans.push({
          at: at + strokeMs + 8 + step * 8,
          kind: "touchmove",
          id: pinchId,
          x: x1 + pinchDirection * (startDistance + ((endDistance - startDistance) * step) / 6),
          y: y1,
        });
      plans.push({
        at: at + strokeMs + 64,
        kind: "touchend",
        id: pinchId,
        x: x1 + pinchDirection * endDistance,
        y: y1,
      });
      plans.push({ at: at + strokeMs + 72, kind: "touchmove", id, x: x1, y: y1 + 20 });
      plans.push({ at: at + strokeMs + 80, kind: "touchend", id, x: x1, y: y1 + 20 });
    } else plans.push({ at: at + strokeMs + 8, kind: "touchend", id, x: x1, y: y1 });
  }
  let changedResponses = 0;
  try {
    for (const plan of plans) {
      check();
      const delay = begin + plan.at - performance.now();
      if (delay > 0) await new Promise((resolve) => setTimeout(resolve, delay));
      check();
      const touch = new Touch({
        identifier: plan.id,
        target: canvas,
        clientX: bounds.left + plan.x,
        clientY: bounds.top + plan.y,
        screenX: bounds.left + plan.x,
        screenY: bounds.top + plan.y,
        pageX: bounds.left + plan.x + scrollX,
        pageY: bounds.top + plan.y + scrollY,
        force: 0.5,
      });
      if (plan.kind === "touchend") active.delete(plan.id);
      else active.set(plan.id, touch);
      const before = camera();
      const event = new TouchEvent(plan.kind, {
        touches: [...active.values()],
        targetTouches: [...active.values()],
        changedTouches: [touch],
        bubbles: true,
        cancelable: true,
      });
      const dispatchedAt = performance.now() - begin;
      const canvasHitAtDispatch =
        document.elementFromPoint(bounds.left + plan.x, bounds.top + plan.y) === canvas;
      canvas.dispatchEvent(event);
      const after = camera();
      if (
        before.x !== after.x ||
        before.y !== after.y ||
        before.zoomX !== after.zoomX ||
        before.zoomY !== after.zoomY
      )
        changedResponses++;
      timeline.push({
        ...plan,
        canvasHitAtDispatch,
        dispatchedAtMs: dispatchedAt,
        latenessMs: Math.max(0, dispatchedAt - plan.at),
        nativeEventTimestamp: event.timeStamp,
        before,
        after,
      });
    }
  } finally {
    if (active.size)
      canvas.dispatchEvent(
        new TouchEvent("touchcancel", {
          changedTouches: [...active.values()],
          touches: [],
          targetTouches: [],
          bubbles: true,
          cancelable: true,
        }),
      );
  }
  return {
    kind: "synthetic DOM TouchEvent lifecycle",
    limitation:
      "Runs on the main thread; cannot reproduce hardware input arriving during JavaScript stalls, OS event coalescing, or physical phone behavior.",
    requestedContacts: 10,
    requestedPinches: 4,
    contactSpacingMs: 300,
    targetMoveHz: 120,
    strokeMs,
    plannedEndpointSpeedCssPxPerSecond:
      (Math.hypot(distance, bounds.height * 0.4) / strokeMs) * 1e3,
    plannedEventCount: plans.length,
    actualEventCount: timeline.length,
    eventsOutsideVisibleCanvas: timeline.filter((event) => !event.canvasHitAtDispatch).length,
    immediateCameraChanges: changedResponses,
    timeline,
  };
}
export { fierceGestures, syntheticTouchSupport };
```

## src/benchmark/sweep.mjs

```javascript
// Developer navigation through real wheel handlers; never sets camera state.
export function spiralPoints({ min, max, viewport }) {
  let left = min.x,
    right = max.x,
    top = min.y,
    bottom = max.y;
  const points = [];
  const add = (x, y) => {
    const last = points.at(-1);
    if (!last || Math.hypot(last.x - x, last.y - y) > 0.01) points.push({ x, y });
  };
  while (true) {
    add(left, top);
    add(right, top);
    add(right, bottom);
    add(left, bottom);
    add(left, top);
    const dx = viewport.width * 0.8,
      dy = viewport.height * 0.8;
    if (right - left <= 2 * dx || bottom - top <= 2 * dy) {
      const x = (left + right) / 2,
        y = (top + bottom) / 2;
      add(left, y);
      add(right, y);
      add(x, top);
      add(x, bottom);
      add(x, y);
      break;
    }
    left += dx;
    right -= dx;
    top += dy;
    bottom -= dy;
  }
  return points;
}

export async function prepareSweep(input, zoom) {
  const { wheel, camera, nextFrame, check, canvas } = input;
  check();
  // Explicitly zoom out as well as in. Production controls clamp to 0.5–3×.
  wheel(0, Math.log(camera().zoomX / zoom) / 0.002, true);
  await nextFrame();
  await nextFrame();
  if (Math.abs(camera().zoomX - zoom) > 0.001) throw new Error("Sweep zoom did not settle");
  wheel(-1e7, -1e7);
  await nextFrame();
  await nextFrame();
  const min = camera();
  wheel(1e7, 1e7);
  await nextFrame();
  await nextFrame();
  const max = camera();
  wheel(-1e7, -1e7);
  await nextFrame();
  await nextFrame();
  check();
  const bounds = canvas.getBoundingClientRect();
  const plan = {
    zoom,
    min: { x: min.x, y: min.y },
    max: { x: max.x, y: max.y },
    viewport: { width: bounds.width, height: bounds.height },
  };
  return { ...plan, points: spiralPoints(plan) };
}

export async function sweepMap(input, plan) {
  const { camera, wheel, nextFrame, check, canvas, redraw } = input;
  const started = performance.now(),
    timeline = [],
    visited = new Set();
  const edgesReached = { left: false, right: false, top: false, bottom: false };
  const gridSize = 16,
    speed = 4500;
  let eventsOutsideVisibleCanvas = 0,
    distance = 0;
  function record() {
    const position = camera();
    timeline.push({ atMs: performance.now() - started, ...position });
    edgesReached.left ||= Math.abs(position.x - plan.min.x) < 1;
    edgesReached.right ||= Math.abs(position.x - plan.max.x) < 1;
    edgesReached.top ||= Math.abs(position.y - plan.min.y) < 1;
    edgesReached.bottom ||= Math.abs(position.y - plan.max.y) < 1;
    const width = plan.max.x - plan.min.x + plan.viewport.width;
    const height = plan.max.y - plan.min.y + plan.viewport.height;
    for (let row = 0; row < gridSize; row++) {
      const y = plan.min.y + ((row + 0.5) * height) / gridSize;
      if (y < position.y || y > position.y + plan.viewport.height) continue;
      for (let col = 0; col < gridSize; col++) {
        const x = plan.min.x + ((col + 0.5) * width) / gridSize;
        if (x >= position.x && x <= position.x + plan.viewport.width)
          visited.add(row * gridSize + col);
      }
    }
    return position;
  }
  let previousAt = performance.now();
  record();
  for (const target of plan.points) {
    while (true) {
      check();
      const before = camera(),
        dx = target.x - before.x,
        dy = target.y - before.y;
      const remaining = Math.hypot(dx, dy);
      if (remaining < 0.5) break;
      if (performance.now() - started > 120000) throw new Error("Whole-map sweep exceeded 120s");
      const now = performance.now();
      const step = Math.min(
        remaining,
        speed * Math.max(1 / 120, (now - previousAt) / 1000),
        plan.viewport.width * 0.4,
        plan.viewport.height * 0.4,
      );
      previousAt = now;
      const bounds = canvas.getBoundingClientRect();
      if (
        document.elementFromPoint(
          bounds.left + bounds.width / 2,
          bounds.top + bounds.height / 2,
        ) !== canvas
      )
        eventsOutsideVisibleCanvas++;
      wheel((dx / remaining) * step, (dy / remaining) * step);
      // Give the actual scene a render opportunity at each overlapping step;
      // do not catch up by dispatching an unseen batch after a stalled frame.
      await nextFrame();
      const after = record();
      const moved = Math.hypot(after.x - before.x, after.y - before.y);
      if (moved < 0.01) throw new Error("Camera stopped before reaching a sweep waypoint");
      if (Math.abs(after.zoomX - plan.zoom) > 0.001)
        throw new Error("Sweep zoom changed unexpectedly");
      distance += moved;
    }
  }
  const wholeMapAlreadyVisible = plan.max.x === plan.min.x && plan.max.y === plan.min.y;
  if (wholeMapAlreadyVisible) {
    while (performance.now() - started < 500) {
      check();
      redraw();
      await nextFrame();
      record();
    }
  }
  return {
    kind: "whole-map inward spiral via production wheel input",
    targetZoom: plan.zoom,
    targetSpeedCssPxPerSecond: speed,
    min: plan.min,
    max: plan.max,
    viewport: plan.viewport,
    waypoints: plan.points,
    wholeMapAlreadyVisible,
    distanceCssPx: distance,
    eventsOutsideVisibleCanvas,
    edgesReached,
    coverage: {
      columns: gridSize,
      rows: gridSize,
      visited: visited.size,
      fraction: visited.size / (gridSize * gridSize),
      scope:
        "Grid centers intersected by sampled camera viewports; not physical display verification",
    },
    timeline,
  };
}
```

## src/benchmark/telemetry.mjs

```javascript
// Developer instrumentation only. These scopes overlap: GPU work is asynchronous,
// and a Wasm export can call back into JS. Never subtract them from frame time.
export function instrumentWasm(instance, capture) {
  // The generated runtime returns its instance owner. Copy its export table;
  // WebAssembly.Instance.exports itself has non-writable function properties.
  if (!instance?.e) return { supported: false, reason: "Runtime export table unavailable" };
  function timed(name, fn) {
    return function (...args) {
      const row = capture();
      if (!row) return fn(...args);
      const started = performance.now();
      try {
        return fn(...args);
      } finally {
        const ms = performance.now() - started;
        const item = (row.wasm.byExport[name] ??= { calls: 0, inclusiveMs: 0 });
        item.calls++;
        item.inclusiveMs += ms;
        row.wasm.calls++;
        row.wasm.inclusiveMs += ms;
      }
    };
  }
  instance.e = Object.fromEntries(
    Object.entries(instance.e).map(([name, value]) => [
      name,
      typeof value === "function" ? timed(name, value) : value,
    ]),
  );
  for (const name of ["nativeCopy", "nativeRead"])
    if (typeof instance[name] === "function") instance[name] = timed(name, instance[name]);
  return {
    supported: true,
    scope:
      "Wall time inside generated-runtime Wasm exports and native string helpers, including JS imports; wrapper overhead is present",
  };
}

export function telemetry(node, capture) {
  const gl = node.getContext("webgl2") || node.getContext("webgl");
  const webgl2 = !!gl?.createQuery;
  const ext = gl?.getExtension(
    webgl2 ? "EXT_disjoint_timer_query_webgl2" : "EXT_disjoint_timer_query",
  );
  const gpuSupport = {
    supported: !!ext,
    scope:
      "GPU elapsed time from game prerender through postrender (includes final batch flush, excludes initial clear and compositor/display); may overlap CPU",
    ...(!ext && { reason: "Browser does not expose disjoint GPU timer queries" }),
  };
  const create = () => (webgl2 ? gl.createQuery() : ext.createQueryEXT());
  const begin = (q) =>
    webgl2 ? gl.beginQuery(ext.TIME_ELAPSED_EXT, q) : ext.beginQueryEXT(ext.TIME_ELAPSED_EXT, q);
  const end = () =>
    webgl2 ? gl.endQuery(ext.TIME_ELAPSED_EXT) : ext.endQueryEXT(ext.TIME_ELAPSED_EXT);
  const available = (q) =>
    webgl2
      ? gl.getQueryParameter(q, gl.QUERY_RESULT_AVAILABLE)
      : ext.getQueryObjectEXT(q, ext.QUERY_RESULT_AVAILABLE_EXT);
  const read = (q) =>
    webgl2
      ? gl.getQueryParameter(q, gl.QUERY_RESULT)
      : ext.getQueryObjectEXT(q, ext.QUERY_RESULT_EXT);
  const remove = (q) => (webgl2 ? gl.deleteQuery(q) : ext.deleteQueryEXT(q));
  let active;
  const pending = [];
  const longFrames = [];
  const supportedEntries = globalThis.PerformanceObserver?.supportedEntryTypes ?? [];
  const type = supportedEntries.includes("long-animation-frame")
    ? "long-animation-frame"
    : supportedEntries.includes("longtask")
      ? "longtask"
      : undefined;
  function ingest(entries) {
    for (const entry of entries)
      longFrames.push({
        type: entry.entryType,
        startMs: entry.startTime,
        durationMs: entry.duration,
        blockingMs: entry.blockingDuration,
        renderStartMs: entry.renderStart,
        styleAndLayoutStartMs: entry.styleAndLayoutStart,
        scripts: entry.scripts?.map((s) => ({
          sourceURL: s.sourceURL,
          functionName: s.sourceFunctionName,
          invoker: s.invoker,
          invokerType: s.invokerType,
          executionStartMs: s.executionStart,
          durationMs: s.duration,
          forcedStyleAndLayoutMs: s.forcedStyleAndLayoutDuration,
        })),
      });
  }
  const observer = type ? new PerformanceObserver((list) => ingest(list.getEntries())) : undefined;
  observer?.observe({ type });
  function poll() {
    if (!ext || !pending.length) return;
    const disjoint = gl.isContextLost() || gl.getParameter(ext.GPU_DISJOINT_EXT);
    for (let i = pending.length - 1; i >= 0; i--) {
      const item = pending[i];
      if (!disjoint && !available(item.query)) continue;
      item.owner.pending--;
      if (disjoint) item.owner.discarded++;
      else item.owner.samplesMs.push(read(item.query) / 1e6);
      remove(item.query);
      pending.splice(i, 1);
    }
  }
  return {
    support: {
      gpu: gpuSupport,
      browserFrames: {
        supported: !!type,
        type,
        scope:
          "Browser-reported long frames/tasks; script durations include Wasm and are not exclusive JS time",
      },
    },
    row: () => ({
      wasm: { calls: 0, inclusiveMs: 0, byExport: {} },
      gpu: { ...gpuSupport, samplesMs: [], pending: 0, discarded: 0 },
    }),
    begin() {
      poll();
      const row = capture();
      if (!ext || !row || active || gl.isContextLost()) return;
      if (pending.length >= 120) {
        row.gpu.discarded++;
        return;
      }
      const query = create();
      if (!query) {
        row.gpu.discarded++;
        return;
      }
      begin(query);
      active = { query, owner: row.gpu };
    },
    end() {
      if (!active) return;
      end();
      active.owner.pending++;
      pending.push(active);
      active = undefined;
    },
    poll,
    frames() {
      ingest(observer?.takeRecords() ?? []);
      return [...longFrames];
    },
    async settle() {
      const deadline = performance.now() + 1000;
      while (pending.length && performance.now() < deadline) {
        await new Promise((resolve) => requestAnimationFrame(resolve));
        poll();
      }
    },
    dispose() {
      observer?.disconnect();
      if (active) {
        end();
        remove(active.query);
        active = undefined;
      }
      for (const item of pending) {
        remove(item.query);
        item.owner.pending--;
        item.owner.discarded++;
      }
      pending.length = 0;
    },
  };
}

export function instrumentCodecs(instance, api, capture) {
  // This page deliberately instruments the generated runtime's global dispatch
  // table. The exported frozen API and ordinary game entry remain untouched.
  const readers = new WeakMap();
  const instrumented = { ...api };
  function span(row, name, started, wasmBefore, extra = {}) {
    row.measures.push({
      name,
      atMs: started - row.started,
      durationMs: performance.now() - started,
      wasmInclusiveMs: row.wasm.inclusiveMs - wasmBefore,
      source: "generated binding runtime",
      ...extra,
    });
  }
  for (const [name, call] of Object.entries(api)) {
    if (!name.startsWith("call_") || typeof call !== "function") continue;
    instrumented[name] = (...args) => {
      const row = capture();
      if (!row) return call(...args);
      // Only this argument is the generated writer closure, not a user callback.
      if (name === "call_dispatch_update") {
        const encode = args[3];
        args[3] = (writer) => {
          const started = performance.now(),
            wasmBefore = row.wasm.inclusiveMs;
          try {
            return encode(writer);
          } finally {
            span(row, "binding.encode_command", started, wasmBefore);
          }
        };
      }
      const started = performance.now(),
        wasmBefore = row.wasm.inclusiveMs;
      try {
        const result = call(...args);
        if (result?.owner === instance && typeof result.view === "function")
          readers.set(result, {
            row,
            name,
            started: performance.now(),
            wasmBefore: row.wasm.inclusiveMs,
            packetBytes: result.length,
            stringPoolBytes: result.poolLength,
          });
        return result;
      } finally {
        span(row, `binding.${name}`, started, wasmBefore);
      }
    };
  }
  instrumented.close = (reader) => {
    const pending = readers.get(reader);
    if (!pending) return api.close(reader);
    readers.delete(reader);
    span(pending.row, `binding.decode.${pending.name}`, pending.started, pending.wasmBefore, {
      packetBytes: pending.packetBytes,
      stringPoolBytes: pending.stringPoolBytes,
    });
    const started = performance.now(),
      wasmBefore = pending.row.wasm.inclusiveMs;
    try {
      return api.close(reader);
    } finally {
      span(pending.row, "binding.output_cleanup", started, wasmBefore);
    }
  };
  globalThis.Ocamljs_hex_command = Object.freeze(instrumented);
  return {
    supported: true,
    scope:
      "Writer callback, bridge call, Reader return to close (decoding), and output cleanup. Inclusive wall time; nested scopes overlap, and decoding can invoke Wasm string helpers.",
  };
}
```

## src/benchmark/verdict.mjs

```javascript
// Workload contract preserved from upstream 15d6c6fa75d41c17ba8b66adcc9ea6c725a4c6a1.
function cadenceVerdict(intervals) {
  const periodMs = 1e3 / 60,
    toleranceMs = 1.5,
    minimumCleanSamples = 2;
  const measured = intervals.filter((value) => Number.isFinite(value) && value > 0);
  const overBudgetIntervals = measured.filter((value) => value > periodMs + toleranceMs).length;
  const elapsedMs = measured.reduce((sum, value) => sum + value, 0);
  const cumulativeExcessMs = Math.max(0, elapsedMs - measured.length * periodMs);
  const status =
    overBudgetIntervals || cumulativeExcessMs > toleranceMs
      ? "FAIL"
      : measured.length < minimumCleanSamples || measured.length !== intervals.length
        ? "INCONCLUSIVE"
        : "PASS";
  return {
    status,
    targetHz: 60,
    periodMs,
    toleranceMs,
    budgetMs: periodMs + toleranceMs,
    samples: measured.length,
    discardedInvalidSamples: intervals.length - measured.length,
    minimumCleanSamples,
    overBudgetIntervals,
    observedHz: elapsedMs ? (measured.length * 1e3) / elapsedMs : null,
    cumulativeExcessMs,
    maxIntervalMs: measured.length ? Math.max(...measured) : null,
    criterion:
      "Any interval above period+tolerance, or cumulative elapsed time above samples*period+tolerance, fails. The cumulative tolerance is applied once per window, not once per frame. Fewer than2 clean intervals are inconclusive. PASS describes measured browser cadence only, not physical display60FPS.",
  };
}
function aggregateCadence(verdicts) {
  const failedPhases = verdicts.filter((verdict) => verdict.status === "FAIL").length;
  const inconclusivePhases = verdicts.filter((verdict) => verdict.status === "INCONCLUSIVE").length;
  const passedPhases = verdicts.filter((verdict) => verdict.status === "PASS").length;
  const status = failedPhases
    ? "FAIL"
    : inconclusivePhases || !verdicts.length
      ? "INCONCLUSIVE"
      : "PASS";
  return {
    status,
    testedPhases: verdicts.length,
    failedPhases,
    passedPhases,
    inconclusivePhases,
    scope:
      "Observable browser cadence at60Hz; independent of workload validity and physical display FPS.",
  };
}
export { aggregateCadence, cadenceVerdict };
```

## src/benchmark/page.css

```css
body:has(.benchmark-page) {
  margin: 0;
  padding: 0;
  overflow: hidden;
}
.benchmark-page {
  box-sizing: border-box;
  height: 100vh;
  height: 100dvh;
  padding: 6px;
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 4px;
  overflow: hidden;
}
.benchmark-page header {
  display: flex;
  align-items: center;
  gap: 6px;
}
.benchmark-page h1 {
  margin: 0;
  font:
    600 16px/1.2 system-ui,
    sans-serif;
  flex: 1;
  min-width: 0;
}
.benchmark-page button {
  margin: 0;
  padding: 8px 12px;
  font:
    14px/1.2 system-ui,
    sans-serif;
}
.benchmark-page #status {
  margin: 0;
  font:
    12px/1.3 system-ui,
    sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.benchmark-page .workspace,
.benchmark-page .map-section,
.benchmark-page .map-scroll {
  height: 100%;
  min-height: 0;
  max-height: none;
  min-width: 0;
}
```

## frontend/Benchmark_game.ml

```ocaml
(* Developer-only adapter. All rendering, resource ownership and mutations below
   use the same production Phaser scene and native engine as ordinary play. *)
type t = {
  factory : Native.factory;
  owner : Phaser_owner.t;
  runtime : Phaser_owner.battlefield;
  mutable battle : Native.battle;
  mutable game : Native.view;
  audio : Phaser_sound.t;
  previews : Terrain_preview.t;
  mutable selected : string option;
  mutable epoch : int;
  mutable progress : Battlefield_frame.progress option;
  mutable failure : string option;
}

let value = function
  | Ok value -> value
  | Error error -> Js.Exn.raiseError (Utf8.to_js (Native.message error))

let present t =
  Phaser_owner.present_battlefield t.runtime
    {
      Battlefield_frame.game = t.game;
      catalog = Native.catalog t.factory;
      artwork = Art.current ();
      selected = t.selected;
      target = None;
      inspected_tile = None;
      recruitment = false;
      session_version = t.epoch;
      sound = Phaser_sound.enabled t.audio;
      action = None;
      combat_playback = None;
    }
    false

let choose t tile =
  t.selected <- Option.map (fun (unit : Domain.public_unit) -> unit.id) (Native.at t.game tile);
  present t

let create parent entries terrain =
  Phaser_terrain_map.register terrain;
  let documents =
    Array.fold_left
      (fun documents (path, contents) ->
        Domain.String_map.add (Utf8.of_js path) (Utf8.of_js contents) documents)
      Domain.String_map.empty entries
  in
  Js.Promise.then_
    (fun outcome ->
      let factory = value outcome in
      let battle =
        value
          (Native.create factory
             (Domain.Start.Scenario { map = "mirror"; revision = None; armies = None; first = None })
             17.)
      in
      let audio = Phaser_sound.create_silent ()
      and previews = Terrain_preview.create ()
      and owner = Phaser_owner.prepare () in
      let state = ref None in
      let current () = Option.get !state in
      let runtime =
        Phaser_owner.mount_battlefield owner
          {
            Battlefield_frame.parent;
            audio;
            previews;
            actions =
              (fun () ->
                {
                  choose = (fun tile -> choose (current ()) tile);
                  clear_selection =
                    (fun _ ->
                      let t = current () in
                      t.selected <- None;
                      present t);
                });
            describe = (fun _ -> ());
            progress = (fun progress -> (current ()).progress <- Some progress);
            failed = (fun message -> (current ()).failure <- Some message);
          }
      in
      let t =
        {
          factory;
          owner;
          runtime;
          battle;
          game = Native.view battle;
          audio;
          previews;
          selected = None;
          epoch = 0;
          progress = None;
          failure = None;
        }
      in
      state := Some t;
      present t;
      Js.Promise.resolve t)
    (Native.create_factory ~documents ())

let start t kind id =
  let id = Utf8.of_js id in
  let start =
    if kind = "campaign" then Domain.Start.Campaign { id }
    else Domain.Start.Scenario { map = id; revision = None; armies = None; first = None }
  in
  let battle = value (Native.create t.factory start 17.) in
  let previous = t.battle in
  t.battle <- battle;
  t.game <- Native.view battle;
  t.selected <- None;
  t.epoch <- t.epoch + 1;
  t.progress <- None;
  Native.dispose previous;
  present t

let move t destination =
  let destination = Utf8.of_js destination in
  let unit =
    Array.find_opt
      (fun (unit : Domain.public_unit) ->
        Native.controllable t.game unit.id
        && Domain.String_map.mem destination (Native.reachable t.game unit.id))
      (Native.units t.game)
  in
  match unit with
  | None -> failwith ("No legal move to " ^ destination)
  | Some unit ->
      t.game <-
        value
          (Native.dispatch t.battle
             (Domain.Command.Move
                { unit = Native.unit_id unit.id; tile = Native.hex t.game destination }));
      present t

let select t id =
  choose t (Utf8.of_js id);
  Option.is_some t.selected

let end_turn t =
  t.game <- value (Native.dispatch t.battle Domain.Command.End_turn);
  present t

let enemy_step t =
  t.game <- fst (value (Native.enemy_step_recorded t.battle (Setup.ai_options Setup.default)));
  present t

(* save_json explicitly selects a host JS string; decoding it again corrupts Unicode. *)
let checkpoint t = value (Native.save_json t.battle)
let status t = match t.failure with Some message -> failwith message | None -> t.progress

let snapshot t =
  let state = Native.state t.game and publication = Native.presentation t.game in
  let resources = Phaser_owner.resources_battlefield t.runtime in
  let camera = Phaser_owner.camera_battlefield t.runtime in
  [%mel.obj
    {
      map = Utf8.to_js state.map_id;
      actors = Array.length (Native.units t.game);
      authoritativeActors = Array.length state.units;
      lights = Array.length publication.lighting.lights;
      villageFixtureLights =
        Array.fold_left
          (fun count (light : Domain.battlefield_light) ->
            count + if light.fixture = Some true then 1 else 0)
          0 publication.lighting.lights;
      redDeathLights =
        Array.fold_left
          (fun count (light : Domain.battlefield_light) ->
            count
            + if light.fixture = Some false && light.color = Some [| 255; 32; 32 |] then 1 else 0)
          0 publication.lighting.lights;
      fog = Array.length publication.fog;
      camera;
      resources = Option.map Phaser_resources.snapshot_json resources;
    }]

(* Benchmark assertions read the already-published typed state. They neither
   serialize a checkpoint nor invoke persistence. *)
let dynamic_snapshot t =
  let state = Native.state t.game in
  let campaign = state.campaign in
  let flag id =
    match Option.bind campaign (fun campaign -> Hashtbl.find_opt campaign.Domain.flags id) with
    | Some changes when Array.length changes > 0 -> changes.(Array.length changes - 1).state
    | _ -> false
  in
  let count predicate =
    Array.fold_left (fun n unit -> n + if predicate unit then 1 else 0) 0 state.units
  in
  let marks =
    Option.fold ~none:[||]
      ~some:(fun (scripts : Domain.event_state) -> scripts.marks)
      (Option.bind campaign (fun campaign -> campaign.Domain.scripts))
  in
  [%mel.obj
    {
      friendly = count (fun (unit : Domain.public_unit) -> unit.side = Domain.Side.Player);
      enemies = count (fun (unit : Domain.public_unit) -> unit.side = Domain.Side.Enemy);
      skeletons =
        count (fun (unit : Domain.public_unit) ->
            unit.type_ = "skeleton" && unit.ai_group = Some "benchmark-despawn");
      deathMarks =
        Array.fold_left
          (fun n (mark : Domain.map_mark) ->
            n
            +
            if
              mark.kind = "benchmark-death-light"
              && Option.fold ~none:false
                   ~some:(fun (light : Domain.tile_light) ->
                     light.fixture = Some false && light.color = Some [| 255; 32; 32 |])
                   mark.light
            then 1
            else 0)
          0 marks;
      allVisibleFlag = flag "all-visible";
      villagesLitFlag = flag "villages-lit";
    }]

let return_to_commander t = Phaser_owner.commander_battlefield t.runtime

let set_sound t enabled =
  Phaser_sound.set_enabled t.audio enabled;
  present t

let dispose t =
  Phaser_owner.dispose t.owner;
  Phaser_sound.close t.audio;
  Terrain_preview.close t.previews;
  Native.dispose t.battle;
  Native.dispose_factory t.factory

let is_loading t = Phaser_owner.loading_battlefield t.runtime
let camera t = Phaser_owner.camera_battlefield t.runtime

(* This adapter is consumed by JavaScript. Its callbacks are uncurried and may
   ignore arguments without depending on OCaml's function-arity convention. *)
let watch_render t (receive : (float -> float -> unit[@u])) =
  Phaser_owner.watch_render_battlefield t.runtime (fun at duration -> (receive at duration [@u]))

let redraw t = Phaser_owner.redraw_battlefield t.runtime
let error_message error = Utf8.to_js (Foreign_error.describe error)

type move_choice = {
  unit_id : string; [@mel.as "unit"]
  origin : string; [@mel.as "from"]
  destination : string; [@mel.as "to"]
}

let legal_move t =
  Array.find_map
    (fun (unit : Domain.public_unit) ->
      if not (Native.controllable t.game unit.id) then None
      else
        Domain.String_map.bindings (Native.reachable t.game unit.id)
        |> List.find_map (fun (destination, _) ->
            if Native.at t.game destination <> None then None
            else
              Some
                {
                  unit_id = Utf8.to_js unit.id;
                  origin = Utf8.to_js (Hex.key unit.tile);
                  destination = Utf8.to_js destination;
                }))
    (Native.units t.game)

let clear_selection t =
  t.selected <- None;
  present t

type item_choice = { unit_id : string; [@mel.as "unit"] item : string }

let carried_item t =
  Array.find_map
    (fun (unit : Domain.public_unit) ->
      if Native.controllable t.game unit.id && Array.length unit.items > 0 then
        Some { unit_id = Utf8.to_js unit.id; item = Utf8.to_js unit.items.(0).id }
      else None)
    (Native.units t.game)

let drop_item t unit_id item =
  t.game <-
    value
      (Native.dispatch t.battle
         (Domain.Command.Drop_item
            { unit = Native.unit_id (Utf8.of_js unit_id); item = Utf8.of_js item }));
  present t

let watch_gpu t (begin_frame : (unit -> unit[@u])) (end_frame : (unit -> unit[@u])) =
  Phaser_owner.watch_gpu_battlefield t.runtime
    (fun () -> (begin_frame () [@u]))
    (fun () -> (end_frame () [@u]))

let watch_measures (_t : t) (receive : (string -> float -> float -> unit[@u])) =
  Performance_trace.observe_measures (fun sample ->
      (receive sample.measure_name sample.measure_start sample.measure_duration [@u]))

let watch_steps t (receive : (float -> float -> unit[@u])) =
  Phaser_owner.watch_steps_battlefield t.runtime (fun started duration ->
      (receive started duration [@u]))
```
