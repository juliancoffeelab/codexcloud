import { pixelSprites, pixelUnitModels, pixelFloorColors, pixelTerrainGeometry } from "./pixel-art";
import type { DialoguePortrait } from "../game/types";

export interface FrameImage {
  readonly url: string;
  readonly width: number;
  readonly height: number;
  readonly anchorX?: number;
  readonly anchorY?: number;
  readonly z?: number;
}
export type ArtworkFaction = string;
export type SpriteKind = "units" | "terrain";
export interface Artwork {
  readonly frames: Readonly<Record<string, Readonly<FrameImage>>>;
}

// Publishing URLs does not fetch images. SVG requests only the frames actually rendered.
const artwork: Artwork = Object.freeze({
  frames: Object.freeze(
    Object.fromEntries(
      Object.entries(pixelSprites).map(([name, path]) => [
        name,
        Object.freeze({
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
  if (!image) return "";
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
  return renderSprite(
    artFrame(artwork, kind, name, side, floor),
    x,
    y,
    width,
    height,
    extraClass,
    kind === "units" ? side : undefined,
  );
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
  if (kind === "units") name = pixelUnitModels[name] ?? name;
  const frameName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  let key = `${kind === "terrain" ? "terrain" : side}_${frameName}`;
  if (kind === "terrain" && floor && pixelFloorColors[floor]) {
    key += `__${floor}`;
  }
  return key;
}
