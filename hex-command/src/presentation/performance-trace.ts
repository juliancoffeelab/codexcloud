/** Opt-in Chrome/Perfetto trace. No observers or frame loop run before recording. */
export interface TraceEvent {
  name: string;
  cat: string;
  ph: "X" | "i" | "M";
  ts: number;
  dur?: number;
  pid: number;
  tid: number;
  s?: "t";
  args?: Record<string, unknown>;
}
export interface TraceConfig {
  id: number;
  origin: number;
  workerId?: number;
}
export interface TraceState {
  status: "empty" | "recording" | "stopping" | "ready";
  events: number;
  dropped: number;
  reason?: string;
}
interface Source {
  start(config: TraceConfig): void;
  stop(id: number): Promise<void>;
}
const sources = new Set<Source>(),
  listeners = new Set<(state: TraceState) => void>();
const maxEvents = 250_000,
  maxDuration = 600_000;
let state: TraceState = { status: "empty", events: 0, dropped: 0 };
let config: TraceConfig | undefined,
  serial = 0;
let events: TraceEvent[] = [],
  metadata: Record<string, unknown> = {};
let context: (() => unknown) | undefined;
let frame = 0,
  timer: ReturnType<typeof setTimeout> | undefined,
  notification: ReturnType<typeof setTimeout> | undefined;
let observer: PerformanceObserver | undefined, stopping: Promise<void> | undefined;
const clock = () => performance.timeOrigin + performance.now();
function notify() {
  for (const listener of listeners) listener({ ...state, events: events.length });
}
export function subscribeTrace(listener: (value: TraceState) => void) {
  listeners.add(listener);
  listener({ ...state, events: events.length });
  return () => {
    listeners.delete(listener);
  };
}
export function traceConfiguration() {
  return state.status === "recording" ? config : undefined;
}
export function registerTraceSource(source: Source) {
  sources.add(source);
  const current = traceConfiguration();
  if (current) source.start(current);
  return () => {
    sources.delete(source);
  };
}
export function receiveTrace(id: number, batch: TraceEvent[]) {
  if (id !== config?.id || !["recording", "stopping"].includes(state.status)) return;
  const room = maxEvents - events.length;
  events.push(...batch.slice(0, room));
  state.dropped += Math.max(0, batch.length - room);
  if (!notification)
    notification = setTimeout(() => {
      notification = undefined;
      notify();
    }, 500);
  if (events.length >= maxEvents && state.status === "recording")
    void stopPerformanceTrace("Event limit reached");
}
export function traceInstant(name: string, args?: Record<string, unknown>) {
  const current = config;
  if (!current || !["recording", "stopping"].includes(state.status)) return;
  receiveTrace(current.id, [
    {
      name,
      cat: "browser",
      ph: "i",
      s: "t",
      pid: 1,
      tid: 1,
      ts: (clock() - current.origin) * 1000,
      args,
    },
  ]);
}
export function traceCommand<A extends unknown[], R>(
  name: string,
  operation: (...args: A) => R,
): (...args: A) => R {
  return (...args) => {
    const current = traceConfiguration();
    if (!current) return operation(...args);
    const start = clock();
    receiveTrace(current.id, [
      {
        name: `${name} begin`,
        cat: "action",
        ph: "i",
        s: "t",
        pid: 1,
        tid: 1,
        ts: (start - current.origin) * 1000,
        args: args.length ? { input: args } : undefined,
      },
    ]);
    const finish = (error?: unknown) =>
      receiveTrace(current.id, [
        {
          name,
          cat: "action",
          ph: "X",
          pid: 1,
          tid: 1,
          ts: (start - current.origin) * 1000,
          dur: (clock() - start) * 1000,
          args:
            args.length || error !== undefined
              ? {
                  ...(args.length ? { input: args } : {}),
                  ...(error !== undefined ? { error: String(error) } : {}),
                }
              : undefined,
        },
      ]);
    try {
      const result = operation(...args);
      if (result instanceof Promise)
        return result.then(
          (value) => {
            finish();
            return value;
          },
          (error) => {
            finish(error);
            throw error;
          },
        ) as R;
      finish();
      return result;
    } catch (error) {
      finish(error);
      throw error;
    }
  };
}
export function measureTrace<T>(name: string, operation: () => T): T {
  return traceCommand(name, operation)();
}
export function startPerformanceTrace(getContext: () => unknown) {
  if (state.status === "recording" || state.status === "stopping") return;
  events = [];
  context = getContext;
  stopping = undefined;
  config = { id: ++serial, origin: clock() };
  state = { status: "recording", events: 0, dropped: 0 };
  metadata = {
    format: "hex-command-performance",
    version: 1,
    startedAt: new Date().toISOString(),
    browser: navigator.userAgent,
    viewport: { width: innerWidth, height: innerHeight, pixelRatio: devicePixelRatio },
    start: getContext(),
  };
  for (const [tid, name] of [[1, "Browser / actions"]] as const)
    receiveTrace(config.id, [
      { name: "thread_name", cat: "metadata", ph: "M", pid: 1, tid, ts: 0, args: { name } },
    ]);
  traceInstant("Recording started");
  let previous = performance.now();
  const tick = () => {
    if (state.status !== "recording") return;
    const now = performance.now();
    receiveTrace(config!.id, [
      {
        name: "Browser frame interval",
        cat: "frame",
        ph: "X",
        pid: 1,
        tid: 1,
        ts: (performance.timeOrigin + previous - config!.origin) * 1000,
        dur: (now - previous) * 1000,
        args: { visible: document.visibilityState },
      },
    ]);
    previous = now;
    frame = requestAnimationFrame(tick);
  };
  frame = requestAnimationFrame(tick);
  if (globalThis.PerformanceObserver?.supportedEntryTypes?.includes("longtask")) {
    observer = new PerformanceObserver((list) => {
      const active = config;
      if (!active || state.status !== "recording") return;
      receiveTrace(
        active.id,
        list.getEntries().flatMap((entry) => {
          const start = performance.timeOrigin + entry.startTime;
          const end = start + entry.duration;
          const clipped = Math.max(start, active.origin);
          return end < clipped
            ? []
            : [
                {
                  name: "Browser long task",
                  cat: "longtask",
                  ph: "X" as const,
                  pid: 1,
                  tid: 1,
                  ts: (clipped - active.origin) * 1000,
                  dur: (end - clipped) * 1000,
                },
              ];
        }),
      );
    });
    observer.observe({ type: "longtask" });
  }
  document.addEventListener("visibilitychange", visibility);
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);
  for (const source of sources) source.start(config);
  timer = setTimeout(() => {
    void stopPerformanceTrace("Ten-minute limit reached");
  }, maxDuration);
  notify();
}
function onError(event: ErrorEvent) {
  traceInstant("Browser error", { message: event.message });
}
function onRejection(event: PromiseRejectionEvent) {
  traceInstant("Unhandled rejection", { message: String(event.reason) });
}
function visibility() {
  traceInstant("Visibility changed", { visibility: document.visibilityState });
}
export function stopPerformanceTrace(reason = "Stopped by user"): Promise<void> {
  if (stopping) return stopping;
  if (state.status !== "recording" || !config) return Promise.resolve();
  state = { ...state, status: "stopping", reason };
  traceInstant("Recording stopped", { reason });
  clearTimeout(timer);
  cancelAnimationFrame(frame);
  observer?.disconnect();
  observer = undefined;
  document.removeEventListener("visibilitychange", visibility);
  window.removeEventListener("error", onError);
  window.removeEventListener("unhandledrejection", onRejection);
  notify();
  const id = config.id;
  stopping = Promise.allSettled([...sources].map((source) => source.stop(id))).then((results) => {
    metadata = {
      ...metadata,
      stoppedAt: new Date().toISOString(),
      end: context?.(),
      workerFlushFailures: results.filter((result) => result.status === "rejected").length,
    };
    state = { ...state, status: "ready" };
    clearTimeout(notification);
    notification = undefined;
    notify();
  });
  return stopping;
}
export async function downloadPerformanceTrace() {
  await stopPerformanceTrace();
  if (state.status !== "ready") return;
  const file = {
    traceEvents: events,
    displayTimeUnit: "ms",
    metadata: {
      ...metadata,
      events: events.length,
      dropped: state.dropped,
      stopReason: state.reason,
      gpuTiming:
        "Optional asynchronous GPU timer queries; no GPU fences. Browser frame intervals are scheduling intervals, not physical display FPS.",
    },
  };
  const url = URL.createObjectURL(new Blob([JSON.stringify(file)], { type: "application/json" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = `hex-performance-${new Date().toISOString().replaceAll(":", "-")}.json`;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
