import {
  pixelFrames,
  pixelSources,
  pixelUnitModels,
  pixelTerrainLayouts,
  pixelFloorColors,
} from "./pixel-art";
import type { TileView } from "../game/view";
import { terrainLayout } from "./terrain-layout";
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
let terrainDefinitions: Record<string, SpriteFrame> | undefined;
const pendingFrames = new Map<string, Promise<boolean>>();

/** Unit/portrait loading never extracts unused terrain or layout combinations. */
export function loadArtwork(): Promise<boolean> {
  if (!loadingArtwork) {
    const definitions = Object.fromEntries(
      Object.entries(pixelFrames)
        .filter(([name]) => !name.startsWith("terrain_"))
        .map(([name, crop]) => [
          name,
          {
            source: `${import.meta.env?.BASE_URL ?? "/"}assets/${pixelSources.units.filename}?v=${pixelSources.units.revision}`,
            crop,
          },
        ]),
    );
    loadingArtwork = loadFrames(definitions).then((ready) => {
      if (!ready) loadingArtwork = undefined;
      return ready;
    });
  }
  return loadingArtwork;
}

async function loadFrames(definitions: Record<string, SpriteFrame>): Promise<boolean> {
  const missing = Object.entries(definitions).filter(
    ([name]) => !artwork.frames[name] && !pendingFrames.has(name),
  );
  if (missing.length) {
    const loading = prepareSpriteImages(missing.map(([, definition]) => definition))
      .then((ready) => {
        if (ready)
          artwork = Object.freeze({
            frames: Object.freeze({
              ...artwork.frames,
              ...Object.fromEntries(
                missing.map(([name, definition]) => [
                  name,
                  Object.freeze(spriteImage(definition)!),
                ]),
              ),
            }),
          });
        return ready;
      })
      .finally(() => {
        for (const [name] of missing) pendingFrames.delete(name);
      });
    for (const [name] of missing) pendingFrames.set(name, loading);
  }
  return (
    await Promise.all(
      Object.keys(definitions).map((name) =>
        artwork.frames[name] ? true : pendingFrames.get(name)!,
      ),
    )
  ).every(Boolean);
}

/** Request only the native terrain frames required by a fallback or hovered tile. */
export function loadTerrainArtwork(tiles: readonly TileView[], mapId: string): Promise<boolean> {
  terrainDefinitions ??= artworkFrames();
  const definitions: Record<string, SpriteFrame> = {};
  for (const tile of tiles) {
    const name = artFrameKey("terrain", terrainLayout(tile, mapId).name, "player", tile.floor);
    const definition = terrainDefinitions[name];
    if (!definition) return Promise.resolve(false);
    definitions[name] = definition;
  }
  return loadFrames(definitions);
}

/** Atlas references stop at this loading boundary. */
export function artworkFrames(): Record<string, SpriteFrame> {
  const frames: Record<string, SpriteFrame> = Object.fromEntries(
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
  for (const [name, layouts] of Object.entries(pixelTerrainLayouts)) {
    layouts.forEach((pieces, index) => {
      for (const floor of [undefined, ...Object.keys(pixelFloorColors)]) {
        const suffix = floor ? `__${floor}` : "";
        const base = frames[`terrain_${name}${suffix}`] ?? frames[`terrain_${name}`]!;
        frames[`terrain_${name}_layout_${index}${suffix}`] = {
          ...base,
          layers: pieces.map((piece) => ({
            ...(frames[`terrain_${piece.name}${suffix}`] ?? frames[`terrain_${piece.name}`]!),
            x: piece.x,
            y: piece.y,
          })),
        };
      }
    });
  }
  return frames;
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
  return artwork.frames[artFrameKey(kind, name, side, floor)];
}

export function artFrameKey(
  kind: SpriteKind,
  name: string,
  side: Side = "player",
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
