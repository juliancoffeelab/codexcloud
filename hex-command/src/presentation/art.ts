import { campaignSheets } from "./campaign-art";
import { pixelFrames, pixelSources } from "./pixel-art";
import { styledSheets } from "./styled-art";
import type { DialoguePortrait, Side } from "../game/types";
import {
  prepareSpriteImages,
  spriteImage,
  type SpriteFrame,
  type FrameImage,
} from "./sprite-images";

const terrainOrder = ["plains", "forest", "hill", "water", "village", "keep"];
const storageKey = "hex-command-art-style";

export const ART_STYLES = {
  normal: { label: "Normal" },
  "low-poly": { label: "Low-poly" },
  pixel: { label: "Pixel" },
};

export type ArtStyle = keyof typeof ART_STYLES;
export type SpriteKind = "units" | "terrain";
export interface Artwork {
  readonly style: ArtStyle;
  readonly frames: Readonly<Record<string, Readonly<FrameImage>>>;
}
const artwork = new Map<ArtStyle, Artwork>();
/** The controller publishes this immutable resource snapshot with BattleView. */
export function artworkForStyle(style: ArtStyle): Artwork {
  let published = artwork.get(style);
  if (!published) {
    published = Object.freeze({ style, frames: Object.freeze({}) });
    artwork.set(style, published);
  }
  return published;
}

function isArtStyle(style: unknown): style is ArtStyle {
  return typeof style === "string" && Object.hasOwn(ART_STYLES, style);
}

export function savedArtStyle(): ArtStyle {
  try {
    const value = localStorage.getItem(storageKey);
    return isArtStyle(value) ? value : "normal";
  } catch {
    return "normal";
  }
}

export function saveArtStyle(style: string) {
  if (!isArtStyle(style)) return;
  try {
    localStorage.setItem(storageKey, style);
  } catch {}
}

const loadedStyles = new Map<ArtStyle, Promise<boolean>>();
export function loadArtStyle(style: string): Promise<boolean> {
  if (!isArtStyle(style)) return Promise.resolve(false);
  if (!loadedStyles.has(style)) {
    const definitions = styleFrames(style);
    const loading = prepareSpriteImages(Object.values(definitions)).then((ready) => {
      if (ready) {
        const frames = Object.fromEntries(
          Object.entries(definitions).map(([key, frame]) => [
            key,
            Object.freeze(spriteImage(frame)!),
          ]),
        );
        artwork.set(style, Object.freeze({ style, frames: Object.freeze(frames) }));
      }
      if (!ready) loadedStyles.delete(style);
      return ready;
    });
    loadedStyles.set(style, loading);
  }
  return loadedStyles.get(style)!;
}

// Terrain sheets are drawn overhead with regular pointy hexes. Each frame
// is a square around the hex: cropping and uniform scaling preserve the art.
const terrainSheets = {
  normal: {
    width: 1536,
    height: 1024,
    revision: "bc9be76e5658",
    frames: [
      {
        x: 76.0,
        y: 37.0,
        size: 424,
      },
      {
        x: 556.0,
        y: 37.0,
        size: 424,
      },
      {
        x: 1036.0,
        y: 37.0,
        size: 424,
      },
      {
        x: 76.0,
        y: 518.0,
        size: 424,
      },
      {
        x: 556.0,
        y: 518.0,
        size: 424,
      },
      {
        x: 1036.0,
        y: 518.0,
        size: 424,
      },
    ],
  },
  "low-poly": {
    width: 1536,
    height: 1024,
    revision: "ffabf1089a96",
    frames: [
      {
        x: 41.0,
        y: 26.0,
        size: 440,
      },
      {
        x: 548.0,
        y: 26.0,
        size: 440,
      },
      {
        x: 1055.0,
        y: 26.0,
        size: 440,
      },
      {
        x: 41.0,
        y: 526.0,
        size: 440,
      },
      {
        x: 548.0,
        y: 526.0,
        size: 440,
      },
      {
        x: 1055.0,
        y: 526.0,
        size: 440,
      },
    ],
  },
};
function assetURL(style: ArtStyle, kind: SpriteKind) {
  if (style === "pixel") return campaignAssetURL(pixelSources[kind]);
  const revision =
    kind === "terrain" ? terrainSheets[style].revision : styledSheets[style].source.revision;
  return `${import.meta.env?.BASE_URL ?? "/"}assets/${kind}-${style}.png?v=${revision}`;
}
const shallowRevisions = {
  normal: "e00d7da98abd",
  "low-poly": "467b3db16f62",
};
function shallowAssetURL(style: ArtStyle) {
  if (style === "pixel") return assetURL(style, "terrain");
  return `${import.meta.env?.BASE_URL ?? "/"}assets/shallow-${style}.png?v=${shallowRevisions[style]}`;
}
interface CampaignSource {
  filename: string;
  width: number;
  height: number;
  revision: string;
}
type CampaignSourceKey = Side | "terrain";
type ClipPolygon = readonly (readonly [number, number])[];
interface CampaignSourceMetadata {
  width: number;
  height: number;
  revision: string;
  sources?: Partial<Record<CampaignSourceKey, CampaignSource>>;
  frameSources?: Readonly<Record<string, CampaignSource>>;
  clips?: Readonly<Record<string, ClipPolygon>>;
}
function campaignSource(style: ArtStyle, key: CampaignSourceKey, frame?: string): CampaignSource {
  const sheet: CampaignSourceMetadata = campaignSheets[style];
  return (
    (frame ? sheet.frameSources?.[frame] : undefined) ??
    sheet.sources?.[key] ?? {
      filename: `campaign-${style}.png`,
      width: sheet.width,
      height: sheet.height,
      revision: sheet.revision,
    }
  );
}
function campaignAssetURL(source: CampaignSource) {
  return `${import.meta.env?.BASE_URL ?? "/"}assets/${source.filename}?v=${source.revision}`;
}
function frameFor(
  style: ArtStyle,
  kind: SpriteKind,
  name: string,
  side: Side,
): SpriteFrame | undefined {
  const campaignName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  const frameKey = kind === "terrain" ? `terrain_${campaignName}` : `${side}_${campaignName}`;
  const sheet = campaignSheets[style];
  const crop = sheet.frames[frameKey as keyof typeof sheet.frames];
  if (crop) {
    const metadata: CampaignSourceMetadata = sheet;
    return {
      source: campaignAssetURL(
        campaignSource(style, kind === "terrain" ? "terrain" : side, frameKey),
      ),
      crop,
      pixel: style === "pixel",
      mask: kind === "units" ? metadata.clips?.[frameKey] : undefined,
    };
  }
  if (style === "pixel") return undefined;
  if (kind === "units") {
    const styled = styledSheets[style];
    const crop = styled.frames[frameKey as keyof typeof styled.frames];
    return crop ? { source: campaignAssetURL(styled.source), crop, pixel: false } : undefined;
  }
  const index = name === "ford" ? 3 : terrainOrder.indexOf(name);
  if (index < 0) return undefined;
  const { x, y, size } = terrainSheets[style].frames[index]!;
  return {
    source: name === "ford" ? shallowAssetURL(style) : assetURL(style, kind),
    crop: [x, y, size, size],
    pixel: false,
  };
}

function workerFrame(style: ArtStyle): SpriteFrame {
  if (style !== "pixel")
    return {
      source: campaignAssetURL(styledSheets[style].source),
      crop: styledSheets[style].frames.portrait_worker,
      pixel: false,
    };
  return {
    source: campaignAssetURL(pixelSources.units),
    crop: pixelFrames.portrait_worker,
    pixel: true,
  };
}

/** Enumerate every frame once; atlas references stop at this loading boundary. */
function styleFrames(style: ArtStyle): Record<string, SpriteFrame> {
  const frames: Record<string, SpriteFrame> = { portrait_worker: workerFrame(style) };
  for (const [key, crop] of Object.entries(campaignSheets[style].frames)) {
    if (key === "portrait_worker") continue;
    const kind = key.startsWith("terrain_") ? "terrain" : "units";
    const side = key.startsWith("enemy_") ? "enemy" : "player";
    const metadata: CampaignSourceMetadata = campaignSheets[style];
    frames[key] = {
      source: campaignAssetURL(campaignSource(style, kind === "terrain" ? "terrain" : side, key)),
      crop,
      pixel: style === "pixel",
      mask: kind === "units" ? metadata.clips?.[key] : undefined,
    };
  }
  if (style !== "pixel") {
    const styled = styledSheets[style];
    for (const [key, crop] of Object.entries(styled.frames)) {
      frames[key] = { source: campaignAssetURL(styled.source), crop, pixel: false };
    }
  }
  for (const name of [...terrainOrder, "ford"]) {
    const frame = frameFor(style, "terrain", name, "player");
    if (frame) frames[`terrain_${name}`] = frame;
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
  return `<svg class="art-sprite ${image.pixel ? "pixel-art" : ""} ${extraClass}" ${side ? `data-faction="${side}"` : ""} x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${image.width} ${image.height}" aria-hidden="true" focusable="false"><image href="${image.url}" x="0" y="0" width="${image.width}" height="${image.height}" preserveAspectRatio="none"/></svg>`;
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
): string {
  if (kind === "terrain" && name === "bridge" && artwork.style !== "pixel") {
    const water = artSprite(artwork, "terrain", "water", 0, 0, 70, 70);
    const joints = [10, 22, 34, 46, 58].map((at) => `<path d="M${at} 24v22"/>`).join("");
    return `<svg class="art-sprite ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 70 70" aria-hidden="true" focusable="false">${water}<path d="M0 26h70v23H0z" fill="#3e443b" opacity=".45"/><path d="M0 23h70v22H0z" fill="#a6a18a" stroke="#686e58" stroke-width="1"/><g stroke="#777c67" stroke-width="1" opacity=".65">${joints}</g><path d="M0 23h70M0 45h70" stroke="#6a705a" stroke-width="5"/><path d="M0 21h70M0 43h70" stroke="#b6b09a" stroke-width="3"/></svg>`;
  }
  const frameName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  const key = `${kind === "terrain" ? "terrain" : side}_${frameName}`;
  return renderSprite(
    artwork.frames[key],
    x,
    y,
    width,
    height,
    extraClass,
    kind === "units" ? side : undefined,
  );
}
