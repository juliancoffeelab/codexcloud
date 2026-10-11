import { artFrame, artFrameKey, type Artwork } from "./art";
import { pixelDefaultFloors, pixelFloorColors } from "./pixel-art";
import { terrainLayout } from "./terrain-layout";
import { battlefieldSpriteSize, hexPoints, tileCenter } from "../ui/geometry";
import type { GameView, TileView } from "../game/view";
import type { FrameImage } from "./art";

export interface TerrainMapTile {
  readonly points: string;
  readonly x: number;
  readonly y: number;
  readonly spriteSize: number;
  readonly fill: string;
  readonly terrain: string;
  readonly terrainFrame: Readonly<FrameImage> | undefined;
}

export interface TerrainMapPlan {
  readonly width: number;
  readonly height: number;
  readonly pixelScale: number;
  readonly tiles: readonly TerrainMapTile[];
}

export interface TerrainMapSprite extends FrameImage {
  readonly displayWidth: number;
  readonly displayHeight: number;
}

/** This is also used by the build renderer, before any controller or browser exists. */
export function terrainTileView(tile: TileView, mapId: string, artwork: Artwork) {
  const { x, y } = tileCenter(tile);
  const frameName = terrainLayout(tile, mapId).name;
  return {
    frameKey: artFrameKey("terrain", frameName, "player", tile.floor),
    x,
    y,
    points: hexPoints(x, y),
    clipId: `hex-clip-${tile.col}-${tile.row}`,
    spriteSize: battlefieldSpriteSize,
    fill:
      (tile.floor && pixelFloorColors[tile.floor]) ||
      pixelFloorColors[pixelDefaultFloors[tile.terrain]]!,
    terrainFrame: artFrame(artwork, "terrain", frameName, "player", tile.floor),
  };
}

export function terrainMapDimensions(plan: Omit<TerrainMapPlan, "tiles">) {
  const width = Math.ceil(plan.width * plan.pixelScale);
  const height = Math.ceil(plan.height * plan.pixelScale);
  return {
    width,
    height,
    displayWidth: width / plan.pixelScale,
    displayHeight: height / plan.pixelScale,
  };
}

export interface TerrainMapImage {
  readonly canvas: HTMLCanvasElement;
  readonly width: number;
  readonly height: number;
  readonly displayWidth: number;
  readonly displayHeight: number;
}

/** Ownership, units and turn state do not affect the static terrain. */
export function terrainMapKey(game: Pick<GameView, "mapId" | "cols" | "rows" | "tiles">): string {
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
      ]),
  ]);
}

/** For edits, keep the authored sprite underneath and repaint only changed cells until baking finishes. */
export function terrainMapPreview(
  key: string,
  sprites: Readonly<Record<string, TerrainMapSprite>>,
) {
  const exact = sprites[key];
  if (exact) return { sprite: exact, changed: new Set<string>() };
  type Key = [string, number, number, [string, ...unknown[]][]];
  const current = JSON.parse(key) as Key;
  const prefix = JSON.stringify(current.slice(0, 3)).slice(0, -1) + ",";
  const baseline = Object.entries(sprites).find(([candidate]) => candidate.startsWith(prefix));
  if (!baseline) return undefined;
  const original = new Map(
    (JSON.parse(baseline[0]) as Key)[3].map((tile) => [tile[0], JSON.stringify(tile)]),
  );
  const changed = new Set(
    current[3]
      .filter((tile) => original.get(tile[0]) !== JSON.stringify(tile))
      .map((tile) => tile[0]),
  );
  return { sprite: baseline[1], changed };
}

export type TerrainTileView = ReturnType<typeof terrainTileView>;
type TerrainGame = Pick<GameView, "mapId" | "cols" | "rows" | "tiles">;

function sameTile(a: TileView, b: TileView) {
  const edges = (x: readonly string[] | undefined, y: readonly string[] | undefined) =>
    x === y || (!!x && !!y && x.length === y.length && x.every((edge, index) => edge === y[index]));
  return (
    a.id === b.id &&
    a.col === b.col &&
    a.row === b.row &&
    a.terrain === b.terrain &&
    a.floor === b.floor &&
    edges(a.connections, b.connections) &&
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
      previous.tiles.every((tile, index) => sameTile(tile, game.tiles[index]!));
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
          [...this.scene!.tiles].map(([id, tile]) => [
            id,
            { ...tile, terrainFrame: artwork.frames[tile.frameKey] },
          ]),
        ),
      };
    }
    this.game = game;
    this.artwork = artwork;
    return this.scene!;
  }
}

type TerrainContext<Image> = Pick<
  CanvasRenderingContext2D,
  | "save"
  | "restore"
  | "beginPath"
  | "moveTo"
  | "lineTo"
  | "closePath"
  | "fill"
  | "clip"
  | "fillStyle"
  | "filter"
> & {
  drawImage(image: Image, x: number, y: number, width: number, height: number): void;
};

/** Identical pixel scale, floors and clipping in build assets and runtime fallback. */
export function paintTerrainTiles<Image>(
  context: TerrainContext<Image>,
  tiles: readonly TerrainMapTile[],
  images: ReadonlyMap<string, Image>,
) {
  for (const tile of tiles) {
    context.save();
    context.beginPath();
    tile.points.split(" ").forEach((point, index) => {
      const [x, y] = point.split(",").map(Number);
      if (index === 0) context.moveTo(x!, y!);
      else context.lineTo(x!, y!);
    });
    context.closePath();
    context.fillStyle = tile.fill;
    context.fill();
    context.clip();
    if (tile.terrainFrame) {
      context.filter = tile.terrain === "forest" ? "brightness(0.93)" : "none";
      context.drawImage(
        images.get(tile.terrainFrame.url)!,
        tile.x - tile.spriteSize / 2,
        tile.y - tile.spriteSize / 2,
        tile.spriteSize,
        tile.spriteSize,
      );
    }
    context.restore();
  }
}

/** Bake only published, isolated tile images; no atlas or mutable game crosses this boundary. */
async function bake(plan: TerrainMapPlan): Promise<Readonly<TerrainMapImage> | undefined> {
  const canvas = document.createElement("canvas");
  const dimensions = terrainMapDimensions(plan);
  canvas.width = dimensions.width;
  canvas.height = dimensions.height;
  const context = canvas.getContext("2d");
  if (!context) return undefined;
  const sources = [
    ...new Set(plan.tiles.flatMap((tile) => (tile.terrainFrame ? [tile.terrainFrame.url] : []))),
  ];
  const decoded = await Promise.all(
    sources.map(
      (source) =>
        new Promise<HTMLImageElement | undefined>((resolve) => {
          const image = new Image();
          image.onload = () => resolve(image);
          image.onerror = () => resolve(undefined);
          image.src = source;
        }),
    ),
  );
  if (decoded.some((image) => !image)) return undefined;
  const images = new Map(sources.map((source, index) => [source, decoded[index]!]));
  context.imageSmoothingEnabled = false;
  context.scale(plan.pixelScale, plan.pixelScale);
  for (let offset = 0; offset < plan.tiles.length; offset += 64) {
    paintTerrainTiles(context, plan.tiles.slice(offset, offset + 64), images);
    // Yield between batches so changed/custom maps do not monopolize the UI thread.
    if (offset + 64 < plan.tiles.length)
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }
  // Publish the completed bitmap directly: no SVG parse, PNG encode or second decode.
  return Object.freeze({
    canvas,
    ...dimensions,
  });
}

/** Retain only the latest map; revisiting game state must not accumulate large bitmaps. */
export class TerrainMapCache {
  private entry:
    | { key: string; artwork: Artwork; image: Promise<Readonly<TerrainMapImage> | undefined> }
    | undefined;

  prepare(key: string, artwork: Artwork, plan: TerrainMapPlan) {
    if (this.entry?.key === key && this.entry.artwork === artwork) return this.entry.image;
    const entry = { key, artwork, image: bake(plan).catch(() => undefined) };
    this.entry = entry;
    void entry.image.then((image) => {
      if (!image && this.entry === entry) this.entry = undefined;
    });
    return entry.image;
  }
}
