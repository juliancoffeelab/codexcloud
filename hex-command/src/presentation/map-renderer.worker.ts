import { MapRendererGpu } from "./map-renderer-gpu";
import type { MapFrameRequest, MapRenderRequest, MapRenderResponse } from "./map-renderer";

let gpu: MapRendererGpu | undefined;
let baseKey: string | undefined;
type RenderJob = MapFrameRequest & { revision: number };
let scene: Omit<MapFrameRequest, "fog"> | undefined;
let pending: RenderJob | undefined;
let running = false;
let latestRevision = 0;
let activeLoad: { terrainKey: string; controller: AbortController } | undefined;

async function loadImage(url: string, signal: AbortSignal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw new Error(`Map image failed to load (${response.status}): ${url}`);
  return createImageBitmap(await response.blob(), { premultiplyAlpha: "none" });
}

const decodedBases = new Map<string, ImageBitmap>();
const decodedPixelBudget = 4_194_304;
let preparedProfile: string | undefined;

async function loadBase(url: string, signal: AbortSignal) {
  const cached = decodedBases.get(url);
  if (cached) {
    decodedBases.delete(url);
    decodedBases.set(url, cached);
    return { image: cached, cached: true };
  }
  const image = await loadImage(url, signal);
  if (signal.aborted) {
    image.close();
    throw signal.reason;
  }
  if (image.width * image.height > decodedPixelBudget) return { image, cached: false };
  decodedBases.set(url, image);
  let pixels = [...decodedBases.values()].reduce(
    (sum, bitmap) => sum + bitmap.width * bitmap.height,
    0,
  );
  while (decodedBases.size > 2 || pixels > decodedPixelBudget) {
    const [key, bitmap] = decodedBases.entries().next().value!;
    decodedBases.delete(key);
    pixels -= bitmap.width * bitmap.height;
    bitmap.close();
  }
  return { image, cached: true };
}

async function preparePrograms(request: RenderJob) {
  const lighting = request.plan.lighting;
  const profile = JSON.stringify([
    lighting?.time ?? "day",
    request.fog.length > 0,
    !!request.prebuilt,
  ]);
  if (profile === preparedProfile) return;
  await gpu!.prepare(lighting, request.fog.length > 0, !!request.prebuilt);
  preparedProfile = profile;
}

async function render(request: RenderJob) {
  const { plan, terrainKey, prebuilt, fog } = request;
  if (!gpu) throw new Error("Map worker received no visible canvas");
  if (baseKey !== terrainKey) {
    const load = { terrainKey, controller: new AbortController() };
    activeLoad = load;
    const timeout = setTimeout(
      () => load.controller.abort(new Error("Map images did not load within 15 seconds")),
      15000,
    );
    try {
      if (prebuilt) {
        const [loading, preparing] = await Promise.allSettled([
          loadBase(prebuilt.url, load.controller.signal),
          preparePrograms(request),
        ]);
        if (loading.status === "rejected") throw loading.reason;
        const { image, cached } = loading.value;
        try {
          if (preparing.status === "rejected") throw preparing.reason;
          if (pending && pending.terrainKey !== terrainKey) return false;
          gpu.setTerrain(plan, new Map(), image);
        } finally {
          if (!cached) image.close();
        }
      } else {
        const urls = [
          ...new Set(
            plan.tiles.map((tile) => {
              if (!tile.terrainFrame)
                throw new Error(`Missing terrain frame at ${tile.x},${tile.y}`);
              return tile.terrainFrame.url;
            }),
          ),
        ];
        const images = new Map<string, ImageBitmap>();
        try {
          const results = await Promise.allSettled([
            ...urls.map(async (url) =>
              images.set(url, await loadImage(url, load.controller.signal)),
            ),
            preparePrograms(request),
          ]);
          const failed = results.find((result) => result.status === "rejected");
          if (failed?.status === "rejected") throw failed.reason;
          if (pending && pending.terrainKey !== terrainKey) return false;
          gpu.setTerrain(plan, images);
        } finally {
          for (const image of images.values()) image.close();
        }
      }
      baseKey = terrainKey;
    } finally {
      clearTimeout(timeout);
      if (activeLoad === load) activeLoad = undefined;
    }
  }
  await preparePrograms(request);
  if (pending) return false;
  gpu.render(plan.lighting, fog);
  return true;
}

self.onmessage = (event: MessageEvent<MapRenderRequest>) => {
  if (event.data.preparePrograms) {
    const input = event.data.preparePrograms;
    if (gpu)
      void gpu.prepare(input.lighting, input.fog, input.prebuilt).catch(() => {
        // Background preparation is speculative. Actual entry retries and reports
        // failures through the ordinary renderer diagnostic/retry path.
      });
    return;
  }
  latestRevision = event.data.revision;
  if (event.data.canvas) {
    if (gpu) {
      self.postMessage({
        revision: latestRevision,
        error: "Map worker canvas was already initialized",
      } satisfies MapRenderResponse);
      return;
    }
    const canvas = event.data.canvas;
    canvas.addEventListener("webglcontextlost", () =>
      self.postMessage({
        revision: latestRevision,
        error: "Map GPU context was lost. Retry map rendering.",
      } satisfies MapRenderResponse),
    );
    try {
      gpu = new MapRendererGpu(canvas);
    } catch (error) {
      self.postMessage({
        revision: latestRevision,
        error: `Map GPU initialization failed: ${String(error)}`,
      } satisfies MapRenderResponse);
      return;
    }
  }
  if (event.data.scene) scene = event.data.scene;
  if (activeLoad && scene && activeLoad.terrainKey !== scene.terrainKey)
    activeLoad.controller.abort(new Error("Map scene was replaced"));
  if (!scene) {
    self.postMessage({
      revision: event.data.revision,
      error: "Map worker received no scene",
    } satisfies MapRenderResponse);
    return;
  }
  pending = {
    ...scene,
    plan: { ...scene.plan, lighting: event.data.lighting },
    revision: event.data.revision,
    fog: event.data.fog,
  };
  if (!running) void drain();
};
async function drain() {
  running = true;
  while (pending) {
    const request = pending;
    pending = undefined;
    try {
      if (!(await render(request)) || pending) continue;
      self.postMessage({
        revision: request.revision,
        ready: true,
        width: gpu!.canvas.width,
        height: gpu!.canvas.height,
      } satisfies MapRenderResponse);
    } catch (error) {
      if (pending) continue;
      self.postMessage({
        revision: request.revision,
        error: `Map rendering failed: ${String(error)}`,
      } satisfies MapRenderResponse);
    }
  }
  running = false;
}
