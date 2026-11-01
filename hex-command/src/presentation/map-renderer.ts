import { lightingKey, daylight } from "./battlefield-lighting";
import type { TerrainMapPlan, TerrainMapSprite } from "./terrain-map";

export interface MapFrameRequest {
  terrainKey: string;
  plan: TerrainMapPlan;
  prebuilt?: TerrainMapSprite;
  fog: readonly string[];
}
export interface MapRenderRequest {
  revision: number;
  fog: readonly string[];
  lighting: TerrainMapPlan["lighting"];
  scene?: Omit<MapFrameRequest, "fog">;
}
export type MapRenderResponse =
  { revision: number; bitmap: ImageBitmap } | { revision: number; error: string };

/** Main-thread presentation only. Map pixels are exclusively produced by the worker. */
export class MapRenderer {
  private worker: Worker;
  private revision = 0;
  private key?: string;
  private terrainKey?: string;
  private timeout?: ReturnType<typeof setTimeout>;
  private context: ImageBitmapRenderingContext;

  constructor(
    private canvas: HTMLCanvasElement,
    private onError: (message: string) => void,
  ) {
    const context = canvas.getContext("bitmaprenderer");
    if (!context) throw new Error("Map rendering requires ImageBitmapRenderingContext");
    this.context = context;
    this.worker = new Worker(new URL("./map-renderer.worker.ts", import.meta.url), {
      type: "module",
    });
    this.worker.onerror = (event) => {
      this.fail(`Map worker failed: ${event.message || "script could not load"}`);
      this.worker.terminate();
    };
    this.worker.onmessageerror = () => this.fail("Map worker returned an unreadable frame");
    this.worker.onmessage = (event: MessageEvent<MapRenderResponse>) => {
      const frame = event.data;
      if (frame.revision !== this.revision) {
        if ("bitmap" in frame) frame.bitmap.close();
        return;
      }
      clearTimeout(this.timeout);
      if ("error" in frame) {
        this.fail(frame.error);
        return;
      }
      try {
        // Resize only when the map dimensions change; ordinary updates are one swap.
        if (canvas.width !== frame.bitmap.width) canvas.width = frame.bitmap.width;
        if (canvas.height !== frame.bitmap.height) canvas.height = frame.bitmap.height;
        this.context.transferFromImageBitmap(frame.bitmap);
      } catch (error) {
        frame.bitmap.close();
        this.fail(`Map frame could not be displayed: ${String(error)}`);
      }
    };
  }

  private fail(message: string) {
    clearTimeout(this.timeout);
    this.worker.terminate();
    this.onError(message);
  }

  update(frame: MapFrameRequest) {
    const key = JSON.stringify([
      frame.terrainKey,
      lightingKey(frame.plan.lighting ?? daylight),
      frame.fog,
    ]);
    if (key === this.key) return;
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
    this.worker.postMessage({
      revision: ++this.revision,
      fog: frame.fog,
      lighting: frame.plan.lighting,
      scene,
    } satisfies MapRenderRequest);
    this.terrainKey = frame.terrainKey;
    clearTimeout(this.timeout);
    this.timeout = setTimeout(() => {
      this.worker.terminate();
      this.fail("Map worker did not finish within 30 seconds. Retry map rendering.");
    }, 30000);
  }

  destroy() {
    clearTimeout(this.timeout);
    ++this.revision;
    this.worker.terminate();
  }
}
