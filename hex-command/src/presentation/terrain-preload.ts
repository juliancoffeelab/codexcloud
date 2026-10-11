import terrainSprites from "virtual:terrain-prebuilt";

// Only the latest choice is retained; the browser owns its normal HTTP/image cache.
let pending: { url: string; image: HTMLImageElement } | undefined;

/** Start the authored PNG request before a new battle is created. */
export function preloadTerrainMap(mapId: string): void {
  const prefix = `[${JSON.stringify(mapId)},`;
  const sprite = Object.entries(terrainSprites).find(([key]) => key.startsWith(prefix))?.[1];
  if (!sprite || pending?.url === sprite.url) return;
  const image = new Image();
  pending = { url: sprite.url, image };
  image.src = sprite.url;
  void image.decode().catch(() => {
    if (pending?.image === image) pending = undefined;
  });
}
