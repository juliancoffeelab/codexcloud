import type { TerrainType } from "../game/types";
import type { Artwork, FrameImage } from "./art";
import { terrainPortraitKey } from "./terrain-portraits";

type PreviewTile = { readonly terrain: TerrainType };

export function terrainPreviewFrame(tile: PreviewTile, artwork: Artwork): Readonly<FrameImage> {
  const key = terrainPortraitKey(tile.terrain);
  const source = artwork.frames[key];
  if (!source) throw new Error(`Missing terrain preview: ${key}`);
  return source;
}
