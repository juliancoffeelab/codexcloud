import type { TraceConfig, TraceEvent } from "./performance-trace";
export interface TraceBatch {
  trace: { id: number; events: TraceEvent[]; stopped?: boolean };
}
let config: TraceConfig | undefined,
  events: TraceEvent[] = [];
let accepting = false;
let pollTimer: ReturnType<typeof setTimeout> | undefined;
let timer: ReturnType<typeof setTimeout> | undefined;
let gl: WebGL2RenderingContext | undefined;
let extension: { TIME_ELAPSED_EXT: number; GPU_DISJOINT_EXT: number } | null = null;
let queries: { query: WebGLQuery; started: number; config: TraceConfig }[] = [];
let frameQuery: { query: WebGLQuery; started: number; config: TraceConfig } | undefined;
const clock = () => performance.timeOrigin + performance.now();
type Stamp = { time: number; id: number };
function stamp(): Stamp | undefined {
  return config ? { time: clock(), id: config.id } : undefined;
}
export function workerTraceStart() {
  return accepting ? stamp() : undefined;
}
export function workerTraceEnd(
  name: string,
  started: Stamp | undefined,
  args?: Record<string, unknown>,
) {
  if (!config || started === undefined || config.id !== started.id) return;
  events.push({
    name,
    cat: "render",
    ph: "X",
    pid: 1,
    tid: 2 + (config.workerId ?? 0) * 2,
    ts: (started.time - config.origin) * 1000,
    dur: (clock() - started.time) * 1000,
    args,
  });
  if (events.length >= 128) flush();
  else if (!timer) timer = setTimeout(flush, 100);
}
function flush(stopped = false) {
  clearTimeout(timer);
  timer = undefined;
  if (!config) return;
  if (events.length || stopped)
    self.postMessage({ trace: { id: config.id, events: events, stopped } } satisfies TraceBatch);
  events = [];
}
export function configureWorkerTrace(
  next: TraceConfig | undefined,
  context?: WebGL2RenderingContext,
) {
  for (const pending of queries) gl?.deleteQuery(pending.query);
  queries = [];
  flush();
  config = next;
  accepting = !!next;
  events = [];
  clearTimeout(pollTimer);
  pollTimer = undefined;
  if (!next) return;
  gl = context;
  for (const [offset, name] of [
    [2, "Map worker"],
    [3, "GPU duration"],
  ] as const)
    events.push({
      name: "thread_name",
      cat: "metadata",
      ph: "M",
      pid: 1,
      tid: offset + (next.workerId ?? 0) * 2,
      ts: 0,
      args: { name: `${name} #${next.workerId ?? 0}` },
    });
  if (gl) {
    extension = gl.getExtension("EXT_disjoint_timer_query_webgl2");
    const debug = gl.getExtension("WEBGL_debug_renderer_info");
    workerTraceEnd("GPU capabilities", stamp(), {
      renderer: debug
        ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL)
        : gl.getParameter(gl.RENDERER),
      version: gl.getParameter(gl.VERSION),
      maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
      gpuTimers: !!extension,
      antialias: gl.getContextAttributes()?.antialias,
    });
  }
}
export function beginGpuTrace(context: WebGL2RenderingContext) {
  if (!config || !accepting || !extension || queries.length >= 32 || frameQuery) return;
  gl = context;
  const query = gl.createQuery();
  if (!query) return;
  gl.beginQuery(extension.TIME_ELAPSED_EXT, query);
  frameQuery = { query, started: clock(), config };
}
export function endGpuTrace() {
  if (!frameQuery || !gl || !extension) return;
  gl.endQuery(extension.TIME_ELAPSED_EXT);
  queries.push(frameQuery);
  frameQuery = undefined;
  if (!pollTimer) pollTimer = setTimeout(poll, 16);
}
function poll() {
  clearTimeout(pollTimer);
  pollTimer = undefined;
  if (!gl || !extension) return;
  const disjoint = gl.getParameter(extension.GPU_DISJOINT_EXT);
  queries = queries.filter((pending) => {
    if (!disjoint && !gl!.getQueryParameter(pending.query, gl!.QUERY_RESULT_AVAILABLE)) return true;
    if (!disjoint && pending.config.id === config?.id)
      events.push({
        name: "GPU render",
        cat: "gpu",
        ph: "X",
        pid: 1,
        tid: 3 + (config.workerId ?? 0) * 2,
        ts: (pending.started - config.origin) * 1000,
        dur: (gl!.getQueryParameter(pending.query, gl!.QUERY_RESULT) as number) / 1000,
        args: { clock: "CPU submission start; duration measured asynchronously on GPU" },
      });
    gl!.deleteQuery(pending.query);
    return false;
  });
  if (disjoint) workerTraceEnd("GPU timer disjoint; samples discarded", stamp());
  flush();
  if (queries.length) pollTimer = setTimeout(poll, 16);
}
export async function stopWorkerTrace(id: number) {
  if (config?.id !== id) {
    self.postMessage({ trace: { id, events: [], stopped: true } } satisfies TraceBatch);
    return;
  }
  accepting = false;
  const deadline = performance.now() + 800;
  while (queries.length && performance.now() < deadline) {
    poll();
    if (queries.length) await new Promise((resolve) => setTimeout(resolve, 16));
  }
  if (queries.length)
    workerTraceEnd("GPU queries incomplete at stop", stamp(), { count: queries.length });
  flush(true);
  configureWorkerTrace(undefined);
}
