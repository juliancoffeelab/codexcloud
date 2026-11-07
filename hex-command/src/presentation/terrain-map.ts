import { daylight, lightingKey, type BattlefieldLighting } from "./battlefield-lighting";
import { artFrame, artFrameKey, type Artwork } from "./art";
import { terrainLayout } from "./terrain-layout";
import { battlefieldSpriteSize, hexPoints, tileCenter } from "../ui/geometry";
import type { GameView, TileView } from "../game/view";
import type { FrameImage } from "./art";

export { terrainMapDimensions, terrainDrawingOrder, paintTerrainTiles } from "./terrain-painter";
export type { TerrainMapTile, TerrainMapPlan } from "./terrain-painter";

export const terrainPageInterior = 1020;
export const terrainPageGutter = 2;
export interface TerrainPage extends FrameImage {
  readonly x: number;
  readonly y: number;
  readonly interiorWidth: number;
  readonly interiorHeight: number;
  readonly gutter: number;
}
export interface TerrainMapSprite {
  readonly width: number;
  readonly height: number;
  readonly pages: readonly TerrainPage[];
  readonly displayWidth: number;
  readonly displayHeight: number;
}

/** This is also used by the build renderer, before any controller or browser exists. */
export function terrainTileView(tile: TileView, mapId: string, artwork: Artwork) {
  const { x, y } = tileCenter(tile);
  const frameName = terrainLayout(tile, mapId).name;
  const frame = artFrame(artwork, "terrain", frameName, "player", tile.floor);
  const scale = battlefieldSpriteSize / 64;
  return {
    id: tile.id,
    col: tile.col,
    row: tile.row,
    spriteX: x - (frame?.anchorX ?? 32) * scale,
    spriteY: y - (frame?.anchorY ?? 32) * scale,
    spriteWidth: (frame?.width ?? 64) * scale,
    spriteHeight: (frame?.height ?? 64) * scale,
    frameKey: artFrameKey("terrain", frameName, "player", tile.floor),
    x,
    y,
    points: hexPoints(x, y),
    clipId: `hex-clip-${tile.col}-${tile.row}`,
    spriteSize: battlefieldSpriteSize,
    terrainFrame: frame,
  };
}

/** Select each tile independently; no map-wide terrain assembly is permitted. */
export function terrainMapTiles(
  tiles: readonly TileView[],
  mapId: string,
  artwork: Artwork,
  roles: Readonly<Record<string, import("../../rust/pkg/hex_command").TerrainRole>>,
) {
  return tiles.map((tile) => ({
    ...tile,
    role: (() => {
      const role = roles[tile.terrain];
      if (!role) throw new Error(`Missing terrain role: ${tile.terrain}`);
      return role;
    })(),
    ...terrainTileView(tile, mapId, artwork),
  }));
}

/** Ownership, units and turn state do not affect the static terrain. */
export function terrainMapKey(
  game: Pick<GameView, "mapId" | "cols" | "rows" | "tiles">,
  lighting: BattlefieldLighting = daylight,
): string {
  return JSON.stringify([
    game.mapId,
    game.cols,
    game.rows,
    [...game.tiles]
      .sort((a, b) => a.row - b.row || a.col - b.col)
      .map((tile) => [
        tile.id,
        tile.col,
        tile.row,
        tile.terrain,
        tile.floor,
        tile.connections,
        tile.deadEnds,
        tile.walls,
      ]),
    lightingKey(lighting),
  ]);
}

export type TerrainTileView = ReturnType<typeof terrainTileView>;
type TerrainGame = Pick<GameView, "mapId" | "cols" | "rows" | "tiles">;

function sameTile(a: TileView, b: TileView) {
  if (a === b) return true;
  const edges = (x: readonly string[] | undefined, y: readonly string[] | undefined) =>
    x === y || (!!x && !!y && x.length === y.length && x.every((edge, index) => edge === y[index]));
  return (
    a.id === b.id &&
    a.col === b.col &&
    a.row === b.row &&
    a.terrain === b.terrain &&
    a.floor === b.floor &&
    edges(a.connections, b.connections) &&
    edges(a.walls, b.walls) &&
    edges(a.deadEnds, b.deadEnds)
  );
}

/** One static scene, reused through moves, ownership changes and selection. */
export class TerrainViewCache {
  private game: TerrainGame | undefined;
  private artwork: Artwork | undefined;
  private scene: { key: string; tiles: ReadonlyMap<string, TerrainTileView> } | undefined;

  prepare(game: TerrainGame, artwork: Artwork) {
    const previous = this.game;
    const unchanged =
      previous &&
      previous.mapId === game.mapId &&
      previous.cols === game.cols &&
      previous.rows === game.rows &&
      previous.tiles.length === game.tiles.length &&
      (previous.tiles === game.tiles ||
        previous.tiles.every((tile, index) => sameTile(tile, game.tiles[index]!)));
    if (!unchanged) {
      this.scene = {
        key: terrainMapKey(game),
        tiles: new Map(
          game.tiles.map((tile) => [tile.id, terrainTileView(tile, game.mapId, artwork)]),
        ),
      };
    } else if (
      this.artwork !== artwork &&
      [...this.scene!.tiles.values()].some(
        (tile) => tile.terrainFrame !== artwork.frames[tile.frameKey],
      )
    ) {
      this.scene = {
        key: this.scene!.key,
        tiles: new Map(
          game.tiles.map((tile) => [tile.id, terrainTileView(tile, game.mapId, artwork)]),
        ),
      };
    }
    this.game = game;
    this.artwork = artwork;
    return this.scene!;
  }
}
/** Opaque asset identity; generated image indexes must not export scenario definitions. */
export function terrainAssetKey(key: string): string {
  let first = 2166136261;
  let second = 2246822519;
  for (let index = 0; index < key.length; index++) {
    const code = key.charCodeAt(index);
    first = Math.imul(first ^ code, 16777619);
    second = Math.imul(second ^ code, 3266489917);
  }
  return [first, second].map((value) => (value >>> 0).toString(16).padStart(8, "0")).join("");
}
