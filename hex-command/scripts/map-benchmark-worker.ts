/** Benchmark-only stage timings. Normal mode never waits for a GPU fence. */
import "../src/presentation/map-renderer.worker";
import { MapRendererGpu } from "../src/presentation/map-renderer-gpu";
import { PhaserBattlefield } from "../src/presentation/phaser-battlefield";
import type { MapRenderRequest, MapRenderResponse } from "../src/presentation/map-renderer";

type Timer = { TIME_ELAPSED_EXT: number; GPU_DISJOINT_EXT: number };
type Query = { stage: string; query: WebGLQuery };
type Measurement = {
  revision: number;
  receivedAt: number;
  cpu: Record<string, number>;
  queries: Query[];
  regions: { stage: string; x: number; y: number; width: number; height: number }[];
  renderStartedAt?: number;
  transferStartedAt?: number;
  fenceInsertedAt?: number;
  fence?: WebGLSync;
  gpuMs?: number | null;
  gpuStagesMs?: Record<string, number | null>;
  pollCount: number;
};
const mode =
  new URL(globalThis.location.href).searchParams.get("mode") === "normal" ? "normal" : "completed";
const scope = globalThis as unknown as {
  onmessage: (event: MessageEvent<MapRenderRequest>) => void;
  postMessage(message: unknown, options?: StructuredSerializeOptions): void;
};
let gl: WebGL2RenderingContext | undefined,
  timer: Timer | null = null;
let workerGpu: Record<string, unknown> | undefined;
let frame: Measurement | undefined,
  active: Query | undefined,
  phase = "setup";
const handle = scope.onmessage;
scope.onmessage = (event) => {
  if (event.data.preparePrograms) {
    handle.call(scope, event);
    return;
  }
  frame = {
    revision: event.data.revision,
    receivedAt: performance.now(),
    cpu: { compileShaderCount: 0, linkProgramCount: 0, createImageBitmapCount: 0 },
    queries: [],
    regions: [],
    pollCount: 0,
  };
  handle.call(scope, event);
};
const getContext = OffscreenCanvas.prototype.getContext;
OffscreenCanvas.prototype.getContext = function (this: OffscreenCanvas, ...args: unknown[]) {
  const started = performance.now();
  const context = Reflect.apply(getContext, this, args);
  if (args[0] === "webgl2" && context) {
    if (frame) frame.cpu.contextCreationMs = performance.now() - started;
    gl = context as WebGL2RenderingContext;
    timer = gl.getExtension("EXT_disjoint_timer_query_webgl2");
    const debug = gl.getExtension("WEBGL_debug_renderer_info");
    workerGpu = {
      renderer: debug
        ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL)
        : gl.getParameter(gl.RENDERER),
      vendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR),
      timerQueryExposed: !!timer,
      attributes: gl.getContextAttributes(),
    };
  }
  return context;
} as typeof getContext;
function endQuery() {
  if (!active) return;
  gl!.endQuery(timer!.TIME_ELAPSED_EXT);
  frame!.queries.push(active);
  active = undefined;
}
// TypeScript private methods remain ordinary prototype methods in the bundle.
// Instrument the real implementation without adding measurement work to production.
const prototype = MapRendererGpu.prototype as unknown as Record<
  string,
  (...args: unknown[]) => unknown
>;
const owners: [Record<string, (...args: unknown[]) => unknown>, string[]][] = [
  [prototype, ["setTerrain", "shade", "paintFog", "compose", "update"]],
  [PhaserBattlefield.prototype as unknown as typeof prototype, ["render"]],
];
for (const [owner, names] of owners)
  for (const name of names) {
    const original = owner[name]!;
    owner[name] = function (...args: unknown[]) {
      if (!frame) return Reflect.apply(original, this, args);
      const start = performance.now(),
        previous = phase;
      if (name === "render") frame.renderStartedAt = start;
      else phase = name;
      if (timer && gl && name !== "render" && name !== "update" && name !== "setTerrain") {
        active = { stage: name, query: gl.createQuery()! };
        gl.beginQuery(timer.TIME_ELAPSED_EXT, active.query);
      }
      try {
        return Reflect.apply(original, this, args);
      } finally {
        if (name !== "render" && name !== "update") endQuery();
        frame.cpu[name + "ApiMs"] = (frame.cpu[name + "ApiMs"] ?? 0) + performance.now() - start;
        phase = previous;
      }
    };
  }
const prepare = prototype.prepare!;
prototype.prepare = function (...args: unknown[]) {
  const measurement = frame,
    start = performance.now();
  const result = Reflect.apply(prepare, this, args) as Promise<void>;
  return result.finally(() => {
    if (measurement)
      measurement.cpu.prepareProgramsApiMs =
        (measurement.cpu.prepareProgramsApiMs ?? 0) + performance.now() - start;
  });
};
const originalFetch = globalThis.fetch;
globalThis.fetch = function (...args) {
  const measurement = frame,
    start = performance.now();
  return Reflect.apply(originalFetch, this, args).finally(() => {
    if (measurement)
      measurement.cpu.imageFetchMs =
        (measurement.cpu.imageFetchMs ?? 0) + performance.now() - start;
  });
};
const readBlob = Response.prototype.blob;
Response.prototype.blob = function () {
  const measurement = frame,
    start = performance.now();
  return readBlob.call(this).finally(() => {
    if (measurement)
      measurement.cpu.imageBodyMs = (measurement.cpu.imageBodyMs ?? 0) + performance.now() - start;
  });
};
const decode = globalThis.createImageBitmap;
globalThis.createImageBitmap = function (this: typeof globalThis, ...args: unknown[]) {
  const measurement = frame,
    start = performance.now();
  if (measurement)
    measurement.cpu.createImageBitmapCount = (measurement.cpu.createImageBitmapCount ?? 0) + 1;
  return (Reflect.apply(decode, this, args) as Promise<ImageBitmap>).finally(() => {
    if (measurement)
      measurement.cpu.imageDecodeMs =
        (measurement.cpu.imageDecodeMs ?? 0) + performance.now() - start;
  });
} as typeof createImageBitmap;
for (const method of ["compileShader", "linkProgram"] as const) {
  const original = WebGL2RenderingContext.prototype[method];
  Object.defineProperty(WebGL2RenderingContext.prototype, method, {
    configurable: true,
    writable: true,
    value: function (this: WebGL2RenderingContext, ...args: unknown[]) {
      if (frame) frame.cpu[method + "Count"] = (frame.cpu[method + "Count"] ?? 0) + 1;
      return Reflect.apply(original, this, args);
    },
  });
}
const scissor = WebGL2RenderingContext.prototype.scissor;
WebGL2RenderingContext.prototype.scissor = function (x, y, width, height) {
  frame?.regions.push({ stage: phase, x, y, width, height });
  scissor.call(this, x, y, width, height);
};
const post = scope.postMessage.bind(scope);
function readQueries(measurement: Measurement) {
  if (!gl || !timer || !measurement.queries.length) return true;
  if (
    !measurement.queries.every(({ query }) =>
      gl!.getQueryParameter(query, gl!.QUERY_RESULT_AVAILABLE),
    )
  )
    return false;
  const disjoint = gl.getParameter(timer.GPU_DISJOINT_EXT);
  const stages: Record<string, number | null> = {};
  for (const { stage, query } of measurement.queries) {
    stages[stage] = disjoint
      ? null
      : (stages[stage] ?? 0) + (gl.getQueryParameter(query, gl.QUERY_RESULT) as number) / 1e6;
    gl.deleteQuery(query);
  }
  measurement.queries = [];
  measurement.gpuStagesMs = stages;
  measurement.gpuMs = disjoint
    ? null
    : Object.values(stages).reduce<number>((sum, value) => sum + (value ?? 0), 0);
  return true;
}
function metadata(measurement: Measurement) {
  return {
    mode,
    presentation: "direct-worker-canvas",
    fullSizeBitmap: false,
    workerGpu,
    ...measurement.cpu,
    workerBeforePostMs: performance.now() - measurement.receivedAt,
    workerBeforeRenderMs:
      measurement.renderStartedAt === undefined
        ? null
        : measurement.renderStartedAt - measurement.receivedAt,
    gpuCompletionWaitMs:
      measurement.fenceInsertedAt === undefined
        ? null
        : performance.now() - measurement.fenceInsertedAt,
    gpuMs: measurement.gpuMs ?? null,
    gpuStagesMs: measurement.gpuStagesMs ?? null,
    fencePollCount: measurement.pollCount,
    regions: measurement.regions,
  };
}
scope.postMessage = (message, options) => {
  const response = message as MapRenderResponse;
  if (!("ready" in response) || !frame || !gl) {
    post(response, options);
    return;
  }
  const measurement = frame,
    context = gl;
  if (mode === "completed") {
    measurement.fence = context.fenceSync(context.SYNC_GPU_COMMANDS_COMPLETE, 0)!;
    measurement.fenceInsertedAt = performance.now();
    context.flush();
  }
  const deliver = () => {
    const ready = readQueries(measurement);
    post({ ...response, benchmark: metadata(measurement) }, options);
    // Timer queries are read asynchronously after normal-mode bitmap delivery.
    // These sideband messages are consumed by the benchmark, never MapRenderer.
    if (!ready) {
      const deadline = performance.now() + 5000;
      const pollQueries = () => {
        if (context.isContextLost() || performance.now() > deadline) {
          for (const { query } of measurement.queries) context.deleteQuery(query);
          return;
        }
        if (!readQueries(measurement)) {
          setTimeout(pollQueries, 8);
          return;
        }
        post({
          type: "benchmark-metrics",
          revision: measurement.revision,
          gpuMs: measurement.gpuMs,
          gpuStagesMs: measurement.gpuStagesMs,
        });
      };
      setTimeout(pollQueries, 8);
    }
  };
  if (!measurement.fence) {
    deliver();
    return;
  }
  const sync = measurement.fence;
  const poll = () => {
    const status = context.clientWaitSync(sync, 0, 0);
    measurement.pollCount++;
    if (status === context.TIMEOUT_EXPIRED) {
      setTimeout(poll, 1);
      return;
    }
    context.deleteSync(sync);
    if (status === context.WAIT_FAILED || context.isContextLost()) {
      post({ revision: response.revision, error: "Benchmark GPU fence failed" });
    } else deliver();
  };
  poll();
};
