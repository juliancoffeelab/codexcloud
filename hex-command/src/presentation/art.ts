import { pixelSprites, pixelUnitModels, pixelFloorColors, pixelTerrainGeometry } from "./pixel-art";
import type { DialoguePortrait } from "../game/types";

export interface FrameImage {
  readonly key?: string;
  readonly url: string;
  readonly width: number;
  readonly height: number;
  readonly anchorX?: number;
  readonly anchorY?: number;
  readonly z?: number;
}
export type ArtworkFaction = string;
export type SpriteKind = "units" | "terrain" | "items";
export interface Artwork {
  readonly frames: Readonly<Record<string, Readonly<FrameImage>>>;
}

// Immutable URLs are shared by rendering and preparation. Image bytes stay in asset files.
const artwork: Artwork = Object.freeze({
  frames: Object.freeze(
    Object.fromEntries(
      Object.entries(pixelSprites).map(([name, path]) => [
        name,
        Object.freeze({
          key: name,
          url: `${import.meta.env?.BASE_URL ?? "/"}${path}`,
          width: 64,
          height: 64,
          ...Object.entries(pixelTerrainGeometry).find(([kind]) =>
            name.startsWith(`terrain_${kind}_`),
          )?.[1],
        }),
      ]),
    ),
  ),
});

export function currentArtwork(): Artwork {
  return artwork;
}

/** Only complete images reach SVG. No offsets, sheet sizes, masks or crop viewports. */
function renderSprite(
  image: FrameImage | undefined,
  x: number,
  y: number,
  width: number,
  height: number,
  extraClass = "",
  side?: ArtworkFaction,
): string {
  if (!image) throw new Error("Missing requested artwork frame");
  return `<svg class="art-sprite pixel-art ${extraClass}" ${side ? `data-faction="${side}"` : ""} x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${image.width} ${image.height}" aria-hidden="true" focusable="false"><image href="${image.url}" x="0" y="0" width="${image.width}" height="${image.height}" preserveAspectRatio="none"/></svg>`;
}

export function dialoguePortrait(
  artwork: Artwork,
  portrait: DialoguePortrait,
  x: number,
  y: number,
  width: number,
  height: number,
) {
  if (portrait !== "worker") return artSprite(artwork, "units", portrait, x, y, width, height);
  return renderSprite(artwork.frames.portrait_worker, x, y, width, height);
}

export function artSprite(
  artwork: Artwork,
  kind: SpriteKind,
  name: string,
  x: number,
  y: number,
  width: number,
  height: number,
  extraClass = "",
  side: ArtworkFaction = "player",
  floor?: string,
): string {
  const key = artFrameKey(kind, name, side, floor);
  const frame = artwork.frames[key];
  if (!frame) throw new Error(`Missing artwork frame: ${key}`);
  return renderSprite(frame, x, y, width, height, extraClass, kind === "units" ? side : undefined);
}

/** Resolve an isolated image from the published resource snapshot. */
export function artFrame(
  artwork: Artwork,
  kind: SpriteKind,
  name: string,
  side: ArtworkFaction = "player",
  floor?: string,
): Readonly<FrameImage> | undefined {
  return artwork.frames[artFrameKey(kind, name, side, floor)];
}

export function artFrameKey(
  kind: SpriteKind,
  name: string,
  side: ArtworkFaction = "player",
  floor?: string,
): string {
  if (kind === "items") return `terrain_prop_items_${name}`;
  if (kind === "units") name = pixelUnitModels[name] ?? name;
  const frameName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  let key = `${kind === "terrain" ? "terrain" : side}_${frameName}`;
  if (kind === "terrain" && floor && pixelFloorColors[floor]) {
    key += `__${floor}`;
  }
  return key;
}

/** Items always have artwork. The same resolver serves HUD, ground piles and preparation. */
export function itemFrame(artwork: Artwork, id: string): Readonly<FrameImage> {
  const frame = artFrame(artwork, "items", id);
  if (!frame) throw new Error(`Missing item artwork: ${id}`);
  return frame;
}
