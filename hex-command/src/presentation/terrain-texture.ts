import textures from "virtual:terrain-prebuilt";
import { terrainAssetKey, terrainTileView, type TerrainMapSprite } from "./terrain-map";
import { AUTHORED_SCENARIOS } from "../game/authored-content";
import type { Artwork } from "./art";
import type { TileView } from "../game/view";

export function terrainTexture(mapId: string, revision = 1): TerrainMapSprite {
  const texture = textures[terrainAssetKey(`${mapId}:${revision}`)];
  if (!texture) throw new Error(`Missing prebaked terrain: ${mapId}. Rebuild map assets.`);
  return texture;
}

/** Compare published state with the authored starting map supplied by Rust. */
export function initialTerrain(mapId: string, revision: number, artwork: Artwork) {
  const map = AUTHORED_SCENARIOS.find((m) => m.id === mapId && (m.revision ?? 1) === revision);
  if (!map) throw new Error(`Missing authored map: ${mapId}`);
  return map.terrain.map((tile) => ({
    ...terrainTileView(tile as TileView, mapId, artwork),
    terrain: tile.terrain,
  }));
}
