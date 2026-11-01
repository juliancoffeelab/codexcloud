import { paintTerrainTiles, terrainMapDimensions } from "./terrain-painter";
import { paintLights, shadePixels } from "./battlefield-lighting";
import { paintFog } from "./battlefield-fog";
import type { MapFrameRequest, MapRenderRequest, MapRenderResponse } from "./map-renderer";

let base: OffscreenCanvas | undefined;
let baseKey: string | undefined;
type RenderJob = MapFrameRequest & { revision: number };
let scene: Omit<MapFrameRequest, "fog"> | undefined;
let pending: RenderJob | undefined;
let running = false;

async function loadImage(url: string) {
  const response = await fetch(url, { signal: AbortSignal.timeout(15000) });
  if (!response.ok) throw new Error(`Map image failed to load (${response.status}): ${url}`);
  return createImageBitmap(await response.blob());
}

async function render(request: RenderJob) {
  const { plan, terrainKey, prebuilt, fog } = request;
  const dimensions = terrainMapDimensions(plan);
  if (!base || baseKey !== terrainKey) {
    const next = new OffscreenCanvas(dimensions.width, dimensions.height);
    const context = next.getContext("2d");
    if (!context) throw new Error("Map worker requires an OffscreenCanvas 2D context");
    context.imageSmoothingEnabled = false;
    if (prebuilt) {
      const image = await loadImage(prebuilt.url);
      try {
        context.drawImage(image, 0, 0, next.width, next.height);
      } finally {
        image.close();
      }
    } else {
      const urls = [
        ...new Set(
          plan.tiles.map((tile) => {
            if (!tile.terrainFrame) throw new Error(`Missing terrain frame at ${tile.x},${tile.y}`);
            return tile.terrainFrame.url;
          }),
        ),
      ];
      const images = new Map<string, ImageBitmap>();
      try {
        // Wait for every decode before releasing images, including when a load fails.
        const results = await Promise.allSettled(
          urls.map(async (url) => images.set(url, await loadImage(url))),
        );
        const failed = results.find((result) => result.status === "rejected");
        if (failed?.status === "rejected") throw failed.reason;
        context.scale(plan.pixelScale, plan.pixelScale);
        paintTerrainTiles(context, plan.tiles, images);
        if (plan.lighting?.time === "night") {
          const pixels = context.getImageData(0, 0, next.width, next.height);
          shadePixels(pixels.data, next.width, 0, next.height, plan.pixelScale, plan.lighting);
          context.putImageData(pixels, 0, 0);
          paintLights(context, plan.lighting);
        }
      } finally {
        for (const image of images.values()) image.close();
      }
    }
    base = next;
    baseKey = terrainKey;
  }
  const output = new OffscreenCanvas(dimensions.width, dimensions.height);
  const context = output.getContext("2d");
  if (!context) throw new Error("Map worker requires an OffscreenCanvas 2D context");
  context.drawImage(base, 0, 0);
  context.scale(plan.pixelScale, plan.pixelScale);
  paintFog(
    context,
    fog.map((id) => {
      const [col, row] = id.split(",").map(Number);
      return { col: col!, row: row! };
    }),
  );
  return output.transferToImageBitmap();
}

self.onmessage = (event: MessageEvent<MapRenderRequest>) => {
  if (event.data.scene) scene = event.data.scene;
  if (!scene) {
    self.postMessage({
      revision: event.data.revision,
      error: "Map worker received no scene",
    } satisfies MapRenderResponse);
    return;
  }
  pending = { ...scene, revision: event.data.revision, fog: event.data.fog };
  if (!running) void drain();
};
async function drain() {
  running = true;
  while (pending) {
    const request = pending;
    pending = undefined;
    try {
      const bitmap = await render(request);
      if (pending) {
        bitmap.close();
        continue;
      }
      self.postMessage({ revision: request.revision, bitmap } satisfies MapRenderResponse, {
        transfer: [bitmap],
      });
    } catch (error) {
      self.postMessage({
        revision: request.revision,
        error: `Map rendering failed: ${String(error)}`,
      } satisfies MapRenderResponse);
    }
  }
  running = false;
}
