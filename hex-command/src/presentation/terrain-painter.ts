import type { FrameImage } from "./art";
import type { BattlefieldLighting } from "./battlefield-lighting";

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
  readonly role: import("../../rust/pkg/hex_command").TerrainRole;
  readonly terrainFrame: Readonly<FrameImage> | undefined;
}

export interface TerrainMapPlan {
  readonly width: number;
  readonly height: number;
  readonly pixelScale: number;
  readonly tiles: readonly TerrainMapTile[];
  readonly lighting?: BattlefieldLighting;
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

/** Identical pixel scale, floors and clipping in offline assets and the rendering worker. */
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
      context.filter = tile.role === "forest" ? "brightness(0.93)" : "none";
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
