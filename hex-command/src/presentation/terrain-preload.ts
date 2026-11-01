import terrainSprites from "virtual:terrain-prebuilt";
import { MAPS } from "../game/maps";

import { terrainAssetKey, terrainMapKey } from "./terrain-map";
import type { TileView } from "../game/view";

// Only the latest choice is retained; the browser owns its normal HTTP/image cache.
let pending: { url: string; image: HTMLImageElement } | undefined;

/** Start the authored PNG request before a new battle is created. */
export function preloadTerrainMap(mapId: string): void {
  const map = MAPS[mapId as keyof typeof MAPS];
  if (!map) return;
  const tiles = map.terrain as readonly TileView[];
  const sprite =
    terrainSprites[
      terrainAssetKey(terrainMapKey({ mapId: map.id, cols: map.cols, rows: map.rows, tiles }))
    ];
  if (!sprite || pending?.url === sprite.url) return;
  const image = new Image();
  pending = { url: sprite.url, image };
  image.src = sprite.url;
  void image.decode().catch(() => {
    if (pending?.image === image) pending = undefined;
  });
}
