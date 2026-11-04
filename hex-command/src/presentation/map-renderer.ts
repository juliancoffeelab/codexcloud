import {
  registerTraceSource,
  receiveTrace,
  traceInstant,
  traceConfiguration,
  type TraceConfig,
} from "./performance-trace";
import type { TraceBatch } from "./worker-trace";
import type { BattlefieldScene } from "./battlefield-scene";
import type { MotionKind } from "./motion";
import { lightingKey, daylight } from "./battlefield-lighting";
import type { TerrainMapPlan, TerrainMapSprite } from "./terrain-map";

export interface MapFrameRequest {
  terrainKey: string;
  plan: TerrainMapPlan;
  prebuilt?: TerrainMapSprite;
  fog: readonly string[];
  battlefield?: BattlefieldScene;
}
export interface MapRenderRequest {
  resize?: { width: number; height: number };
  traceControl?: { config?: TraceConfig; stop?: number };
  preparePrograms?: { lighting: TerrainMapPlan["lighting"]; fog: boolean; prebuilt: boolean };
  revision: number;
  fog: readonly string[];
  lighting: TerrainMapPlan["lighting"];
  scene?: Omit<MapFrameRequest, "fog" | "battlefield">;
  battlefield?: BattlefieldScene;
  motion?: { tile: string; kind: MotionKind };
  interaction?: { hovered: string | null; focused: string | null };
  /** Transferred exactly once, with this renderer's first scene. */
  canvas?: OffscreenCanvas;
}
export type MapRenderResponse =
  | { revision: number; ready: true; width: number; height: number }
  | { revision: number; error: string };

const renderers = new Set<MapRenderer>();
let rendererSerial = 0;

/** The worker owns the visible canvas. Messages contain scene changes and acknowledgements only. */
export class MapRenderer {
  private worker: Worker;
  private workerId = ++rendererSerial;
  private canvas = document.createElement("canvas");
  private offscreen?: OffscreenCanvas;
  private revision = 0;
  private key?: string;
  private terrainKey?: string;
  private timeout?: ReturnType<typeof setTimeout>;
  private failed = false;
  private displayed = false;
  private battlefield?: BattlefieldScene;
  private unregisterTrace?: () => void;
  private traceStop?: { id: number; resolve: () => void; timer: ReturnType<typeof setTimeout> };
  private resizeObserver?: ResizeObserver;
  private host?: HTMLElement;
  private displaySize?: { width: number; height: number };
  private refreshSize = () => {
    if (this.failed || !this.host) return;
    const bounds = this.host.getBoundingClientRect();
    if (!bounds.width || !bounds.height) return;
    const size = {
      width: Math.ceil(bounds.width * devicePixelRatio),
      height: Math.ceil(bounds.height * devicePixelRatio),
    };
    if (this.displaySize?.width === size.width && this.displaySize?.height === size.height) return;
    this.displaySize = size;
    this.worker.postMessage({ resize: size });
  };
  private observeSize(host: HTMLElement) {
    this.resizeObserver?.disconnect();
    this.host = host;
    this.refreshSize();
    this.resizeObserver = new ResizeObserver(this.refreshSize);
    this.resizeObserver.observe(host);
  }
  private frameStarted = new Map<number, { time: number; trace?: TraceConfig }>();

  get hasFrame() {
    return this.displayed;
  }

  constructor(
    host: HTMLElement,
    private onError: (message: string) => void,
    private onFrame?: (revision: number) => void,
  ) {
    if (!this.canvas.transferControlToOffscreen)
      throw new Error("Map rendering requires transferControlToOffscreen");
    this.canvas.style.cssText =
      "position:absolute;inset:0;width:100%;height:100%;image-rendering:pixelated;visibility:hidden";
    this.canvas.setAttribute("aria-hidden", "true");
    this.worker = new Worker(new URL("./map-renderer.worker.ts", import.meta.url), {
      type: "module",
    });
    try {
      this.offscreen = this.canvas.transferControlToOffscreen();
      host.append(this.canvas);
    } catch (error) {
      this.worker.terminate();
      throw error;
    }
    this.observeSize(host);
    window.addEventListener("resize", this.refreshSize);
    renderers.add(this);
    this.worker.onerror = (event) =>
      this.fail(`Map worker failed: ${event.message || "script could not load"}`);
    this.worker.onmessageerror = () =>
      this.fail("Map worker returned an unreadable acknowledgement");
    this.worker.onmessage = (event: MessageEvent<MapRenderResponse | TraceBatch>) => {
      if ("trace" in event.data) {
        const { id, events, stopped } = event.data.trace;
        receiveTrace(id, events);
        if (stopped && this.traceStop?.id === id) {
          clearTimeout(this.traceStop.timer);
          this.traceStop.resolve();
          this.traceStop = undefined;
        }
        return;
      }
      const frame = event.data;
      const submission = this.frameStarted.get(frame.revision);
      if (submission?.trace)
        receiveTrace(submission.trace.id, [
          {
            name: "Map request to acknowledgement",
            cat: "latency",
            ph: "X",
            pid: 1,
            tid: 1,
            ts: (submission.time - submission.trace.origin) * 1000,
            dur: (performance.timeOrigin + performance.now() - submission.time) * 1000,
            args: { revision: frame.revision, error: "error" in frame ? frame.error : undefined },
          },
        ]);
      for (const revision of this.frameStarted.keys())
        if (revision <= frame.revision) this.frameStarted.delete(revision);
      if (this.failed || frame.revision !== this.revision) return;
      clearTimeout(this.timeout);
      if ("error" in frame) {
        this.fail(frame.error);
        return;
      }
      this.canvas.style.visibility = "visible";
      this.displayed = true;
      this.onFrame?.(frame.revision);
    };
    this.unregisterTrace = registerTraceSource({
      start: (config) =>
        this.worker.postMessage({
          traceControl: { config: { ...config, workerId: this.workerId } },
        }),
      stop: (id) =>
        new Promise<void>((resolve, reject) => {
          const timer = setTimeout(() => {
            this.traceStop = undefined;
            reject(new Error("Map worker trace flush timed out"));
          }, 1500);
          this.traceStop = { id, resolve, timer };
          this.worker.postMessage({ traceControl: { stop: id } });
        }),
    });
  }

  /** Adopt a pre-rendered canvas without recreating its worker or GPU context. */
  attach(
    host: HTMLElement,
    onError: (message: string) => void,
    onFrame?: (revision: number) => void,
  ) {
    if (this.failed) throw new Error("Cannot attach a failed map renderer");
    this.onError = onError;
    this.onFrame = onFrame;
    this.key = undefined;
    this.displayed = false;
    this.canvas.style.visibility = "hidden";
    host.append(this.canvas);
    this.observeSize(host);
  }

  preparePrograms(frame: MapFrameRequest) {
    if (this.failed) return;
    this.worker.postMessage({
      revision: 0,
      fog: [],
      lighting: frame.plan.lighting,
      preparePrograms: {
        lighting: frame.plan.lighting,
        fog: frame.fog.length > 0,
        prebuilt: !!frame.prebuilt,
      },
    } satisfies MapRenderRequest);
  }

  private fail(message: string) {
    if (this.failed) return;
    traceInstant("Map renderer error", { message });
    this.failed = true;
    clearTimeout(this.timeout);
    // Keep the context alive until replacement/disposal so recoverable image or
    // script errors do not intentionally clear the previous completed display.
    this.onError(message);
  }

  update(frame: MapFrameRequest) {
    if (this.failed) return;
    const key = JSON.stringify([lightingKey(frame.plan.lighting ?? daylight), frame.fog]);
    if (
      frame.terrainKey === this.terrainKey &&
      key === this.key &&
      frame.battlefield === this.battlefield
    )
      return;
    const battlefield = frame.battlefield === this.battlefield ? undefined : frame.battlefield;
    this.battlefield = frame.battlefield;
    this.key = key;
    const absolute = (url: string) => new URL(url, document.baseURI).href;
    const scene =
      frame.terrainKey === this.terrainKey
        ? undefined
        : {
            terrainKey: frame.terrainKey,
            prebuilt: frame.prebuilt && { ...frame.prebuilt, url: absolute(frame.prebuilt.url) },
            plan: {
              ...frame.plan,
              tiles: frame.plan.tiles.map((tile) => ({
                ...tile,
                terrainFrame: tile.terrainFrame && {
                  ...tile.terrainFrame,
                  url: absolute(tile.terrainFrame.url),
                },
              })),
            },
          };
    const canvas = this.offscreen;
    const revision = ++this.revision;
    const trace = traceConfiguration();
    if (trace) {
      this.frameStarted.set(revision, { time: performance.timeOrigin + performance.now(), trace });
      if (this.frameStarted.size > 128)
        this.frameStarted.delete(this.frameStarted.keys().next().value!);
    }
    this.worker.postMessage(
      {
        revision,
        fog: frame.fog,
        lighting: frame.plan.lighting,
        scene,
        battlefield: battlefield && {
          ...battlefield,
          prefetch: battlefield.prefetch?.map(absolute),
          units: battlefield.units.map((unit) => ({
            ...unit,
            url: unit.url && absolute(unit.url),
          })),
          items: battlefield.items.map((item) => ({
            ...item,
            url: item.url && absolute(item.url),
          })),
        },
        canvas,
      } satisfies MapRenderRequest,
      canvas ? [canvas] : [],
    );
    this.offscreen = undefined;
    this.terrainKey = frame.terrainKey;
    clearTimeout(this.timeout);
    this.timeout = setTimeout(
      () => this.fail("Map worker did not finish within 30 seconds. Retry map rendering."),
      30000,
    );
  }

  pulse(tile: string, kind: MotionKind) {
    if (!this.failed) this.worker.postMessage({ motion: { tile, kind } });
  }

  inspect(hovered: string | null, focused: string | null) {
    if (!this.failed) this.worker.postMessage({ interaction: { hovered, focused } });
  }

  destroy() {
    this.resizeObserver?.disconnect();
    window.removeEventListener("resize", this.refreshSize);
    this.unregisterTrace?.();
    this.frameStarted.clear();
    if (this.traceStop) {
      clearTimeout(this.traceStop.timer);
      this.traceStop.resolve();
      this.traceStop = undefined;
    }
    clearTimeout(this.timeout);
    ++this.revision;
    this.worker.terminate();
    renderers.delete(this);
    this.canvas.remove();
  }
}

type PreparedRenderer = {
  renderer: MapRenderer;
  key: string;
  promise: Promise<boolean>;
  resolve: (ready: boolean) => void;
  reject: (error: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};
let prepared: PreparedRenderer | undefined;

/** At most one hidden renderer, bounded to two minutes and claimed on first entry. */
export function prewarmMapRenderer(frame: MapFrameRequest): Promise<boolean> {
  if (!prepared && renderers.size) {
    for (const renderer of renderers) renderer.preparePrograms(frame);
    return Promise.resolve(false);
  }
  const key = JSON.stringify([
    frame.terrainKey,
    lightingKey(frame.plan.lighting ?? daylight),
    frame.fog,
  ]);
  if (prepared?.key === key) return prepared.promise;
  if (prepared) prepared.resolve(false);
  let resolve!: (ready: boolean) => void, reject!: (error: Error) => void;
  const promise = new Promise<boolean>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  if (prepared) {
    clearTimeout(prepared.timer);
    Object.assign(prepared, { key, promise, resolve, reject });
  } else {
    let renderer: MapRenderer;
    try {
      renderer = new MapRenderer(
        document.createElement("div"),
        (message) => {
          if (prepared?.renderer !== renderer) return;
          const entry = prepared;
          prepared = undefined;
          clearTimeout(entry.timer);
          entry.reject(new Error(message));
          renderer.destroy();
        },
        () => {
          if (prepared?.renderer === renderer) prepared.resolve(true);
        },
      );
    } catch (error) {
      reject(error instanceof Error ? error : new Error(String(error)));
      return promise;
    }
    prepared = { renderer, key, promise, resolve, reject, timer: undefined! };
  }
  prepared.timer = setTimeout(discardPrewarmedMapRenderer, 120_000);
  try {
    prepared.renderer.update(frame);
  } catch (error) {
    const entry = prepared;
    prepared = undefined;
    clearTimeout(entry.timer);
    entry.renderer.destroy();
    entry.reject(error instanceof Error ? error : new Error(String(error)));
  }
  return promise;
}

export function acquireMapRenderer(
  host: HTMLElement,
  onError: (message: string) => void,
  onFrame?: (revision: number) => void,
) {
  const entry = prepared;
  if (!entry) return new MapRenderer(host, onError, onFrame);
  prepared = undefined;
  clearTimeout(entry.timer);
  entry.resolve(false);
  entry.renderer.attach(host, onError, onFrame);
  return entry.renderer;
}

export function discardPrewarmedMapRenderer() {
  const entry = prepared;
  prepared = undefined;
  if (!entry) return;
  clearTimeout(entry.timer);
  entry.resolve(false);
  entry.renderer.destroy();
}
if (typeof window !== "undefined") window.addEventListener("pagehide", discardPrewarmedMapRenderer);
