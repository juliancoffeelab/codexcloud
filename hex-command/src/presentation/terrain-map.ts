import { daylight, lightingKey, type BattlefieldLighting } from "./battlefield-lighting";
import { processTerrain } from "./terrain-process";
import { artFrame, artFrameKey, type Artwork } from "./art";
import { pixelFloorColors } from "./pixel-art";
import { terrainLayout } from "./terrain-layout";
import { battlefieldSpriteSize, hexPoints, tileCenter } from "../ui/geometry";
import type { GameView, TileView } from "../game/view";
import type { FrameImage } from "./art";

export interface TerrainMapTile {
  readonly points: string;
  readonly x: number;
  readonly y: number;
  readonly spriteSize: number;
  readonly spriteX?: number;
  readonly spriteY?: number;
  readonly spriteWidth?: number;
  readonly spriteHeight?: number;
  readonly fill: string;
  readonly terrain: string;
  readonly terrainFrame: Readonly<FrameImage> | undefined;
}

export interface TerrainMapPlan {
  readonly width: number;
  readonly height: number;
  readonly pixelScale: number;
  readonly tiles: readonly TerrainMapTile[];
  readonly lighting?: BattlefieldLighting;
}

export interface TerrainMapSprite extends FrameImage {
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
    fill: (tile.floor && pixelFloorColors[tile.floor]) || "transparent",
    terrainFrame: frame,
  };
}

/** Select each tile independently; no map-wide terrain assembly is permitted. */
export function terrainMapTiles(tiles: readonly TileView[], mapId: string, artwork: Artwork) {
  return tiles.map((tile) => ({
    ...tile,
    ...terrainTileView(tile, mapId, artwork),
  }));
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
        tile.wallCorners,
      ]),
    lightingKey(lighting),
  ]);
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

/** For edits, keep the authored sprite underneath and repaint only changed cells until baking finishes. */
export function terrainMapPreview(
  key: string,
  sprites: Readonly<Record<string, TerrainMapSprite>>,
  baselineKey?: string,
) {
  const exact = sprites[terrainAssetKey(key)];
  if (exact) return { sprite: exact, changed: new Set<string>() };
  if (!baselineKey) return undefined;
  type Key = [string, number, number, [string, ...unknown[]][], string];
  const current = JSON.parse(key) as Key;
  // Both keys come from Rust-owned catalog/state data, never a generated module.
  const baseline = JSON.parse(baselineKey) as Key;
  if (
    JSON.stringify(current.slice(0, 3)) !== JSON.stringify(baseline.slice(0, 3)) ||
    current[4] !== baseline[4]
  )
    return undefined;
  const sprite = sprites[terrainAssetKey(baselineKey)];
  if (!sprite) return undefined;
  const original = new Map(baseline[3].map((tile) => [tile[0], JSON.stringify(tile)]));
  const changed = new Set(
    current[3]
      .filter((tile) => original.get(tile[0]) !== JSON.stringify(tile))
      .map((tile) => tile[0]),
  );
  return { sprite, changed };
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
    edges(a.wallCorners?.map(String), b.wallCorners?.map(String)) &&
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
          terrainMapTiles(game.tiles, game.mapId, artwork).map((tile) => [tile.id, tile]),
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
            {
              ...tile,
              terrainFrame: artwork.frames[tile.frameKey],
            },
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
  | "fillStyle"
  | "filter"
> & {
  drawImage(image: Image, x: number, y: number, width: number, height: number): void;
};

/** Ordinary sprite draw ordering from explicit artwork metadata, without geometry inference. */
export function terrainDrawingOrder<T extends TerrainMapTile>(tiles: readonly T[]): T[] {
  return [...tiles].sort(
    (a, b) => (a.terrainFrame?.z ?? 0) - (b.terrainFrame?.z ?? 0) || a.y - b.y || a.x - b.x,
  );
}

/** Identical pixel scale, floors and clipping in build assets and runtime fallback. */
export function paintTerrainTiles<Image>(
  context: TerrainContext<Image>,
  tiles: readonly TerrainMapTile[],
  images: ReadonlyMap<string, Image>,
) {
  for (const tile of terrainDrawingOrder(tiles)) {
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
    // Source frames already carry the native hex mask. A second vector clip
    // partially erases shared edge pixels and leaves seams between matching rails.
    if (tile.terrainFrame) {
      context.filter = tile.terrain === "forest" ? "brightness(0.93)" : "none";
      context.drawImage(
        images.get(tile.terrainFrame.url)!,
        tile.spriteX ?? tile.x - tile.spriteSize / 2,
        tile.spriteY ?? tile.y - tile.spriteSize / 2,
        tile.spriteWidth ?? tile.spriteSize,
        tile.spriteHeight ?? tile.spriteSize,
      );
    }
    context.filter = "none";
    context.restore();
  }
}

/** Bake only published, isolated tile images; no atlas or mutable game crosses this boundary. */
async function bake(
  plan: TerrainMapPlan,
  signal?: AbortSignal,
): Promise<Readonly<TerrainMapImage> | undefined> {
  const canvas = document.createElement("canvas");
  const dimensions = terrainMapDimensions(plan);
  canvas.width = dimensions.width;
  canvas.height = dimensions.height;
  const context = canvas.getContext("2d");
  if (!context) return undefined;
  const sources = [
    ...new Set(
      plan.tiles.flatMap((tile) => [...(tile.terrainFrame ? [tile.terrainFrame.url] : [])]),
    ),
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
  if (plan.lighting?.time === "night" && signal?.aborted) return undefined;
  const images = new Map(sources.map((source, index) => [source, decoded[index]!]));
  context.imageSmoothingEnabled = false;
  context.scale(plan.pixelScale, plan.pixelScale);
  const ordered = terrainDrawingOrder(plan.tiles);
  for (let offset = 0; offset < ordered.length; offset += 64) {
    if (plan.lighting?.time === "night" && signal?.aborted) return undefined;
    paintTerrainTiles(context, ordered.slice(offset, offset + 64), images);
    // Yield between batches so changed/custom maps do not monopolize the UI thread.
    if (offset + 64 < ordered.length) await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }
  if (
    plan.lighting?.time === "night" &&
    !(await processTerrain(canvas, plan.pixelScale, plan.lighting, signal))
  )
    return undefined;
  // Publish the completed bitmap directly: no SVG parse, PNG encode or second decode.
  return Object.freeze({
    canvas,
    ...dimensions,
  });
}

/** Retain only the latest map; revisiting game state must not accumulate large bitmaps. */
export class TerrainMapCache {
  private entry:
    | {
        key: string;
        artwork: Artwork;
        image: Promise<Readonly<TerrainMapImage> | undefined>;
        abort: AbortController;
      }
    | undefined;

  prepare(key: string, artwork: Artwork, plan: TerrainMapPlan) {
    if (this.entry?.key === key && this.entry.artwork === artwork) return this.entry.image;
    this.entry?.abort.abort();
    const abort = new AbortController();
    const entry = { key, artwork, abort, image: bake(plan, abort.signal).catch(() => undefined) };
    this.entry = entry;
    void entry.image.then((image) => {
      if (!image && this.entry === entry) this.entry = undefined;
    });
    return entry.image;
  }

  dispose() {
    this.entry?.abort.abort();
    this.entry = undefined;
  }
}
