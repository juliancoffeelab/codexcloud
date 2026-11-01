import terrainSprites from "virtual:terrain-prebuilt";
import { MAPS } from "../game/maps";
import { TERRAIN } from "../game/catalog";
import { currentArtwork, type Artwork } from "./art";
import { TerrainViewCache, terrainAssetKey } from "./terrain-map";
import { prewarmMapRenderer, type MapFrameRequest } from "./map-renderer";
import { battlefieldSpriteSize, mapDimensions } from "../ui/geometry";
import type { GameView, TileView } from "../game/view";

let pending: { url: string; image: HTMLImageElement } | undefined;

/** Identical scene identity and artwork metadata to the live battlefield. */
export function terrainPreloadFrame(
  game: Pick<GameView, "mapId" | "cols" | "rows" | "tiles" | "presentation">,
  artwork: Artwork = currentArtwork(),
): MapFrameRequest {
  const scene = new TerrainViewCache().prepare(game, artwork);
  const tileById = new Map(game.tiles.map((tile) => [tile.id, tile]));
  return {
    terrainKey: JSON.stringify([
      scene.key,
      [...scene.tiles.values()].map((tile) => tile.terrainFrame),
    ]),
    plan: {
      ...mapDimensions(game),
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: [...scene.tiles].map(([id, tile]) => ({
        ...tile,
        role: TERRAIN[tileById.get(id)!.terrain].role,
      })),
      lighting: game.presentation.lighting,
    },
    prebuilt: terrainSprites[terrainAssetKey(scene.key)],
    fog: game.presentation.fog,
  };
}

function preload(frame: MapFrameRequest) {
  // A hidden, worker-owned canvas can be adopted at entry. Existing live
  // renderers only prepare future shaders; they never display speculative maps.
  void prewarmMapRenderer(frame).catch(() => {
    // Actual entry creates a fresh renderer and reports failures with retry.
  });
  const url = frame.prebuilt?.url;
  if (!url || pending?.url === url) return;
  const image = new Image();
  pending = { url, image };
  image.src = url;
  void image.decode().catch(() => {
    if (pending?.image === image) pending = undefined;
  });
}

export function preloadTerrainView(
  game: Parameters<typeof terrainPreloadFrame>[0],
  artwork?: Artwork,
) {
  preload(terrainPreloadFrame(game, artwork));
}

/** Speculative map-picker preparation; all definitions still come from Rust. */
export function preloadTerrainMap(mapId: string): void {
  const map = MAPS[mapId as keyof typeof MAPS];
  if (!map) return;
  preloadTerrainView({
    mapId: map.id,
    cols: map.cols,
    rows: map.rows,
    tiles: map.terrain as readonly TileView[],
    presentation: map.presentation,
  });
}
