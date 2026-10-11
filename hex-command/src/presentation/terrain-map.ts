import type { Artwork } from "./art";
import type { GameView } from "../game/view";
import type { FrameImage } from "./sprite-images";

interface TerrainMapTile {
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
    game.tiles.map((tile) => [
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

/** Bake only published, isolated tile images; no atlas or mutable game crosses this boundary. */
async function bake(plan: TerrainMapPlan): Promise<Readonly<TerrainMapImage> | undefined> {
  const canvas = document.createElement("canvas");
  canvas.width = Math.ceil(plan.width * plan.pixelScale);
  canvas.height = Math.ceil(plan.height * plan.pixelScale);
  const displayWidth = canvas.width / plan.pixelScale;
  const displayHeight = canvas.height / plan.pixelScale;
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
  for (const tile of plan.tiles) {
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
  // Publish the completed bitmap directly: no SVG parse, PNG encode or second decode.
  return Object.freeze({
    canvas,
    width: canvas.width,
    height: canvas.height,
    displayWidth,
    displayHeight,
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
