import type { Artwork } from "./art";
import type { GameView } from "../game/view";
import type { FrameImage } from "./sprite-images";

interface TerrainMapTile {
  readonly points: string;
  readonly clipId: string;
  readonly fill: string;
  readonly terrain: string;
  readonly terrainArtwork: string;
}

export interface TerrainMapPlan {
  readonly width: number;
  readonly height: number;
  readonly pixelScale: number;
  readonly tiles: readonly TerrainMapTile[];
}

export interface TerrainMapImage extends FrameImage {
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
  const definitions = plan.tiles
    .map((tile) => `<clipPath id="${tile.clipId}"><polygon points="${tile.points}"/></clipPath>`)
    .join("");
  const ground = plan.tiles
    .map(
      (tile) =>
        `<g class="terrain-${tile.terrain}"><polygon points="${tile.points}" fill="${tile.fill}"/><g clip-path="url(#${tile.clipId})">${tile.terrainArtwork}</g></g>`,
    )
    .join("");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${canvas.width}" height="${canvas.height}" viewBox="0 0 ${displayWidth} ${displayHeight}"><style>image{image-rendering:pixelated}.terrain-forest .terrain-art{filter:brightness(0.93)}</style><defs>${definitions}</defs>${ground}</svg>`;
  const image = new Image();
  const ready = await new Promise<boolean>((resolve) => {
    image.onload = () => resolve(true);
    image.onerror = () => resolve(false);
    image.src = "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
  });
  if (!ready) return undefined;
  const context = canvas.getContext("2d");
  if (!context) return undefined;
  context.imageSmoothingEnabled = false;
  context.drawImage(image, 0, 0);
  return Object.freeze({
    url: canvas.toDataURL("image/png"),
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
