import { pixelFrames, pixelSources, pixelUnitModels } from "./pixel-art";
import type { DialoguePortrait, Side } from "../game/types";
import {
  prepareSpriteImages,
  spriteImage,
  type SpriteFrame,
  type FrameImage,
} from "./sprite-images";

export type SpriteKind = "units" | "terrain";
export interface Artwork {
  readonly frames: Readonly<Record<string, Readonly<FrameImage>>>;
}
let artwork: Artwork = Object.freeze({ frames: Object.freeze({}) });
/** The controller publishes this immutable resource snapshot with BattleView. */
export function currentArtwork(): Artwork {
  return artwork;
}

let loadingArtwork: Promise<boolean> | undefined;
export function loadArtwork(): Promise<boolean> {
  if (!loadingArtwork) {
    const definitions = artworkFrames();
    loadingArtwork = prepareSpriteImages(Object.values(definitions)).then((ready) => {
      if (ready) {
        const frames = Object.fromEntries(
          Object.entries(definitions).map(([key, frame]) => [
            key,
            Object.freeze(spriteImage(frame)!),
          ]),
        );
        artwork = Object.freeze({ frames: Object.freeze(frames) });
      } else {
        loadingArtwork = undefined;
      }
      return ready;
    });
  }
  return loadingArtwork;
}

/** Atlas references stop at this loading boundary. */
function artworkFrames(): Record<string, SpriteFrame> {
  return Object.fromEntries(
    Object.entries(pixelFrames).map(([key, crop]) => {
      const source = key.startsWith("terrain_") ? pixelSources.terrain : pixelSources.units;
      return [
        key,
        {
          source: `${import.meta.env?.BASE_URL ?? "/"}assets/${source.filename}?v=${source.revision}`,
          crop,
        },
      ];
    }),
  );
}

/** Only complete images reach SVG. No offsets, sheet sizes, masks or crop viewports. */
function renderSprite(
  image: FrameImage | undefined,
  x: number,
  y: number,
  width: number,
  height: number,
  extraClass = "",
  side?: Side,
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
  side: Side = "player",
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
  side: Side = "player",
  floor?: string,
): Readonly<FrameImage> | undefined {
  if (kind === "units") name = pixelUnitModels[name] ?? name;
  const frameName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  let key = `${kind === "terrain" ? "terrain" : side}_${frameName}`;
  if (kind === "terrain" && floor && artwork.frames[`${key}__${floor}`]) {
    key += `__${floor}`;
  }
  return artwork.frames[key];
}
