import { MapRendererGpu } from "./map-renderer-gpu";
import type { MapFrameRequest, MapRenderRequest, MapRenderResponse } from "./map-renderer";

let gpu: MapRendererGpu | undefined;
let baseKey: string | undefined;
type RenderJob = MapFrameRequest & { revision: number };
let scene: Omit<MapFrameRequest, "fog" | "battlefield"> | undefined;
let battlefield: MapFrameRequest["battlefield"];
let displayedBattlefield: MapFrameRequest["battlefield"];
let displayedJob: RenderJob | undefined;
let animation: number | undefined;
let interaction = { hovered: null as string | null, focused: null as string | null };
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
  if (request.battlefield && request.battlefield !== displayedBattlefield) {
    const images = new Map<string, ImageBitmap>();
    const controller = new AbortController();
    const timer = setTimeout(
      () => controller.abort(new Error("Battlefield sprites did not load within 15 seconds")),
      15000,
    );
    try {
      const urls = [
        ...new Set(
          [...request.battlefield.units, ...request.battlefield.items].flatMap((sprite) =>
            sprite.url && !gpu!.hasSprite(sprite.url, request.battlefield!.key) ? [sprite.url] : [],
          ),
        ),
      ];
      const loaded = await Promise.allSettled(
        urls.map(async (url) => images.set(url, await loadImage(url, controller.signal))),
      );
      const failed = loaded.find((result) => result.status === "rejected");
      if (failed?.status === "rejected") throw failed.reason;
      if (pending) return false;
      gpu.setBattlefield(request.battlefield, images);
      gpu.inspect(interaction.hovered, interaction.focused);
      displayedBattlefield = request.battlefield;
    } finally {
      clearTimeout(timer);
      for (const image of images.values()) image.close();
    }
  }
  gpu.render(plan.lighting, fog);
  displayedJob = request;
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
  if (event.data.motion || event.data.interaction) {
    if (event.data.motion) gpu?.pulse(event.data.motion.tile, event.data.motion.kind);
    if (event.data.interaction) {
      interaction = event.data.interaction;
      gpu?.inspect(interaction.hovered, interaction.focused);
    }
    scheduleAnimation();
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
  if (event.data.battlefield) battlefield = event.data.battlefield;
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
    battlefield,
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
      displayedJob = undefined;
      self.postMessage({
        revision: request.revision,
        error: `Map rendering failed: ${String(error)}`,
      } satisfies MapRenderResponse);
    }
  }
  running = false;
  if (gpu?.animating) scheduleAnimation();
}

// Worker animation runs only while feedback is active. Terrain/light/fog caches
// survive every tween frame; idle maps have no continuous render loop.
function scheduleAnimation() {
  if (animation !== undefined || !displayedJob || running) return;
  animation = requestAnimationFrame(() => {
    animation = undefined;
    if (running || !gpu || !displayedJob) return;
    try {
      gpu.render(displayedJob.plan.lighting, displayedJob.fog);
      if (gpu.animating) scheduleAnimation();
    } catch (error) {
      self.postMessage({
        revision: latestRevision,
        error: `Battlefield animation failed: ${String(error)}`,
      } satisfies MapRenderResponse);
    }
  });
}
