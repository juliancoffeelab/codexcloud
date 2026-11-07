import { MAPS } from "../game/maps";
import { terrainTexture } from "./terrain-texture";
import type { GameView, TileView } from "../game/view";

/** Warm artwork bytes only. Phaser owns texture loading and every rendering resource. */
function preloadTerrainView(
  game: Pick<GameView, "mapId" | "mapRevision" | "cols" | "rows" | "tiles">,
) {
  for (const page of terrainTexture(game.mapId, game.mapRevision).pages) {
    const image = new Image();
    image.src = page.url;
  }
}

export function preloadTerrainMap(mapId: string): void {
  const map = MAPS[mapId as keyof typeof MAPS];
  if (!map) return;
  preloadTerrainView({
    mapId: map.id,
    mapRevision: map.revision ?? 1,
    cols: map.cols,
    rows: map.rows,
    tiles: map.terrain as readonly TileView[],
  });
}
