// Offline asset build isolation: generated Wasm runtimes own process-global registration.
// A worker isolates asset preparation without changing runtime glue.
import { workerData, parentPort } from "node:worker_threads";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { createCanvas, loadImage } from "@napi-rs/canvas";
const { root, source, wasmPath } = workerData;
try {
  const { initialize, Terrain_plan, Terrain_assets } = await import(
    `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`
  );
  await initialize(await readFile(wasmPath));
  const plans = Terrain_assets.plans();
  const images = new Map(),
    watched = [],
    assets = [],
    keys = new Set(),
    assetKeys = new Map();
  for (const asset of plans) {
    const { plan, key, asset_key: assetKey } = asset;
    if (keys.has(key)) continue;
    keys.add(key);
    if (assetKeys.has(assetKey) && assetKeys.get(assetKey) !== key)
      throw new Error(`Terrain image key collision: ${assetKey}`);
    assetKeys.set(assetKey, key);
    for (const tile of plan.tiles) {
      const frame = tile.frame;
      if (!frame) throw new Error(`${asset.map_id}/${tile.id}: missing terrain artwork`);
      if (images.has(frame.url)) continue;
      const path = resolve(root, "public", frame.url.replace(/^(?:\.\/|\/)+/, ""));
      watched.push(path);
      const image = createCanvas(frame.width, frame.height);
      image.getContext("2d").drawImage(await loadImage(path), 0, 0);
      images.set(frame.url, image);
    }
    const frames = Object.fromEntries(images),
      dimensions = Terrain_plan.dimensions(plan),
      pages = [],
      interior = 1020,
      gutter = 2;
    for (let y = 0; y < dimensions.height; y += interior)
      for (let x = 0; x < dimensions.width; x += interior) {
        const interiorWidth = Math.min(interior, dimensions.width - x),
          interiorHeight = Math.min(interior, dimensions.height - y),
          width = interiorWidth + gutter * 2,
          height = interiorHeight + gutter * 2;
        const canvas = createCanvas(width, height),
          context = canvas.getContext("2d");
        context.imageSmoothingEnabled = false;
        context.translate(gutter - x, gutter - y);
        context.beginPath();
        context.rect(0, 0, dimensions.width, dimensions.height);
        context.clip();
        context.scale(plan.pixel_scale, plan.pixel_scale);
        Terrain_plan.paint(context, plan.tiles, frames);
        pages.push({
          metadata: { x, y, interiorWidth, interiorHeight, gutter, width, height },
          bytes: canvas.toBuffer("image/png"),
        });
      }
    assets.push({
      mapId: asset.map_id,
      key: assetKey,
      sprite: {
        width: dimensions.width,
        height: dimensions.height,
        displayWidth: dimensions.display_width,
        displayHeight: dimensions.display_height,
      },
      pages,
    });
  }
  parentPort.postMessage({ assets, watched });
} catch (error) {
  parentPort.postMessage({
    error: String(error.stack ?? error).replace(
      /data:text\/javascript;base64,[A-Za-z0-9+/=]+/g,
      "<compiled terrain module>",
    ),
  });
}
