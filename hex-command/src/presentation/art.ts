import { campaignSheets } from "./campaign-art";
import type { Side } from "../game/types";

const unitOrder = ["soldier", "archer", "scout", "commander"];
const terrainOrder = ["plains", "forest", "hill", "water", "village", "keep"];
const storageKey = "hex-command-art-style";

export const ART_STYLES = {
  normal: { label: "Normal" },
  "low-poly": { label: "Low-poly" },
  pixel: { label: "Pixel" },
};

export type ArtStyle = keyof typeof ART_STYLES;
export type SpriteKind = "units" | "terrain";

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
    const loading = Promise.all(
      [
        assetURL(style, "units"),
        assetURL(style, "terrain"),
        shallowAssetURL(style),
        ...campaignAssetURLs(style),
      ].map(
        (url) =>
          new Promise<boolean>((resolve) => {
            const image = new Image();
            image.onload = () => resolve(true);
            image.onerror = () => resolve(false);
            image.src = url;
          }),
      ),
    ).then((results) => {
      const ready = results.every(Boolean);
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
  pixel: {
    width: 1536,
    height: 1024,
    revision: "94af10625f5f",
    frames: [
      {
        x: 61.0,
        y: 50.0,
        size: 414,
      },
      {
        x: 561.0,
        y: 50.0,
        size: 414,
      },
      {
        x: 1061.0,
        y: 50.0,
        size: 414,
      },
      {
        x: 61.0,
        y: 537.0,
        size: 414,
      },
      {
        x: 561.0,
        y: 537.0,
        size: 414,
      },
      {
        x: 1061.0,
        y: 537.0,
        size: 414,
      },
    ],
  },
};
const unitSheets = {
  normal: {
    width: 1774,
    height: 887,
    revision: "baba41eaaf62",
    frames: [
      {
        x: 0,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 1330.5,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 0,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 1330.5,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
    ],
  },
  "low-poly": {
    width: 1774,
    height: 887,
    revision: "28004172c750",
    frames: [
      {
        x: 0,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 1330.5,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 0,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 1330.5,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
    ],
  },
  pixel: {
    width: 1774,
    height: 887,
    revision: "cd2af5ff67fe",
    frames: [
      {
        x: 0,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 0.0,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 0.0,
        width: 457,
        height: 443.5,
      },
      {
        x: 1344,
        y: 0.0,
        width: 430,
        height: 443.5,
      },
      {
        x: 0,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 443.5,
        y: 443.5,
        width: 443.5,
        height: 443.5,
      },
      {
        x: 887,
        y: 443.5,
        width: 457,
        height: 443.5,
      },
      {
        x: 1344,
        y: 443.5,
        width: 430,
        height: 443.5,
      },
    ],
  },
};
function assetURL(style: ArtStyle, kind: SpriteKind) {
  return `${import.meta.env?.BASE_URL ?? "/"}assets/${kind}-${style}.png?v=${(kind === "terrain" ? terrainSheets : unitSheets)[style].revision}`;
}
const shallowRevisions = {
  normal: "e00d7da98abd",
  "low-poly": "467b3db16f62",
  pixel: "8843342bc280",
};
function shallowAssetURL(style: ArtStyle) {
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
  sources?: Record<CampaignSourceKey, CampaignSource>;
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
function campaignAssetURLs(style: ArtStyle) {
  const sheet: CampaignSourceMetadata = campaignSheets[style];
  return [
    ...new Set(
      [
        ...(["player", "enemy", "terrain"] as const).map((key) => campaignSource(style, key)),
        ...Object.values(sheet.frameSources ?? {}),
      ].map(campaignAssetURL),
    ),
  ];
}

/** Crop unit sheets through the same HTML image path for every artwork family. */
function unitImage(
  url: string,
  sheet: { width: number; height: number },
  cropX: number,
  cropY: number,
  cropWidth: number,
  cropHeight: number,
  clip?: ClipPolygon,
) {
  const clipping = clip
    ? `;clip-path:polygon(${clip.map(([x, y]) => `${((100 * x) / cropWidth).toFixed(4)}% ${((100 * y) / cropHeight).toFixed(4)}%`).join(",")})`
    : "";
  // The HTML sheet is much larger than its SVG frame. Explicit native bounds and
  // paint containment keep mobile compositors from painting outside that crop.
  return `<foreignObject width="${cropWidth}" height="${cropHeight}" overflow="hidden"><div xmlns="http://www.w3.org/1999/xhtml" style="position:relative;width:${cropWidth}px;height:${cropHeight}px;overflow:hidden;contain:paint${clipping}"><img src="${url}" alt="" draggable="false" style="position:absolute;display:block;max-width:none;width:${sheet.width}px;height:${sheet.height}px;left:${-cropX}px;top:${-cropY}px"/></div></foreignObject>`;
}

/** Draw the terrain sheet in SVG coordinates; the sprite viewport crops the frame. */
function sheetImage(
  url: string,
  sheet: { width: number; height: number },
  cropX: number,
  cropY: number,
) {
  return `<image href="${url}" x="${-cropX}" y="${-cropY}" width="${sheet.width}" height="${sheet.height}" preserveAspectRatio="none"/>`;
}

export interface SpriteInfo {
  /** Actual raster source, including its cache revision. */
  source: string;
  /** Native source-sheet dimensions, not the displayed sprite size. */
  width: number;
  height: number;
  crop: { x: number; y: number; width: number; height: number };
  composite?: string;
}
interface ResolvedSprite extends SpriteInfo {
  family: "campaign" | "units" | "terrain" | "bridge";
  clip?: ClipPolygon;
}

/** One source/crop lookup for both rendering and the developer inspector. */
function resolveSprite(
  style: ArtStyle,
  kind: SpriteKind,
  name: string,
  side: Side,
): ResolvedSprite | null {
  if (!isArtStyle(style)) return null;
  const campaignName =
    name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
  const campaignKey = kind === "terrain" ? `terrain_${campaignName}` : `${side}_${campaignName}`;
  const campaignSheet = campaignSheets[style];
  const frame = campaignSheet.frames[campaignKey as keyof typeof campaignSheet.frames];
  if (frame) {
    const [x, y, width, height] = frame;
    const source = campaignSource(style, kind === "terrain" ? "terrain" : side, campaignKey);
    const metadata: CampaignSourceMetadata = campaignSheet;
    return {
      family: "campaign",
      source: campaignAssetURL(source),
      width: source.width,
      height: source.height,
      crop: { x, y, width, height },
      clip: metadata.clips?.[campaignKey],
    };
  }
  const isUnit = kind === "units";
  const specialTerrain = !isUnit && (name === "ford" || name === "bridge");
  const index = specialTerrain ? 3 : (isUnit ? unitOrder : terrainOrder).indexOf(name);
  if (index < 0) return null;
  if (isUnit) {
    const sheet = unitSheets[style];
    return {
      family: "units",
      source: assetURL(style, "units"),
      width: sheet.width,
      height: sheet.height,
      crop: { ...sheet.frames[index + (side === "enemy" ? 4 : 0)]! },
    };
  }
  const sheet = terrainSheets[style];
  const { x, y, size } = sheet.frames[index]!;
  return {
    family: name === "bridge" ? "bridge" : "terrain",
    source: name === "ford" ? shallowAssetURL(style) : assetURL(style, "terrain"),
    width: sheet.width,
    height: sheet.height,
    crop: { x, y, width: size, height: size },
    ...(name === "bridge" ? { composite: "Water raster with an SVG stone crossing" } : {}),
  };
}

export function artSpriteInfo(
  style: ArtStyle,
  kind: SpriteKind,
  name: string,
  side: Side = "player",
): SpriteInfo | null {
  const sprite = resolveSprite(style, kind, name, side);
  if (!sprite) return null;
  const { source, width, height, crop, composite } = sprite;
  return { source, width, height, crop, ...(composite ? { composite } : {}) };
}

function terrainSprite(
  style: ArtStyle,
  sprite: SpriteInfo,
  x: number,
  y: number,
  width: number,
  height: number,
  extraClass: string,
) {
  const { crop } = sprite;
  return `<svg class="art-sprite ${style === "pixel" ? "pixel-art" : ""} ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${crop.width} ${crop.height}" preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true" focusable="false">${sheetImage(sprite.source, sprite, crop.x, crop.y)}</svg>`;
}

// Render a region of the original sheet directly. Keep the generated PNGs intact.
export function artSprite(
  style: ArtStyle,
  kind: SpriteKind,
  name: string,
  x: number,
  y: number,
  width: number,
  height: number,
  extraClass = "",
  side: Side = "player",
) {
  const sprite = resolveSprite(style, kind, name, side);
  if (!sprite) return "";
  const { source, crop, family } = sprite;
  const { x: cropX, y: cropY, width: cropWidth, height: cropHeight } = crop;
  if (family === "campaign") {
    const image =
      kind === "units"
        ? unitImage(source, sprite, cropX, cropY, cropWidth, cropHeight, sprite.clip)
        : sheetImage(source, sprite, cropX, cropY);
    return `<svg class="art-sprite ${style === "pixel" ? "pixel-art" : ""} ${extraClass}" data-faction="${side}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${cropWidth} ${cropHeight}" preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true" focusable="false">${image}</svg>`;
  }
  if (family === "bridge") {
    // Reuse the river's water palette beneath a simple stone crossing.
    const water = terrainSprite(style, sprite, 0, 0, 70, 70, "");
    const joints = [10, 22, 34, 46, 58].map((at) => `<path d="M${at} 24v22"/>`).join("");
    return `<svg class="art-sprite ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 70 70" aria-hidden="true" focusable="false">${water}<path d="M0 26h70v23H0z" fill="#3e443b" opacity=".45"/><path d="M0 23h70v22H0z" fill="#a6a18a" stroke="#686e58" stroke-width="1"/><g stroke="#777c67" stroke-width="1" opacity=".65">${joints}</g><path d="M0 23h70M0 45h70" stroke="#6a705a" stroke-width="5"/><path d="M0 21h70M0 43h70" stroke="#b6b09a" stroke-width="3"/></svg>`;
  }
  if (family === "terrain") return terrainSprite(style, sprite, x, y, width, height, extraClass);
  // Preserve HTML cropping for half-pixel unit frames; SVG sheet scaling can bleed adjacent rows.
  return `<svg class="art-sprite ${style === "pixel" ? "pixel-art" : ""} ${extraClass}" data-faction="${side}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${cropWidth} ${cropHeight}" preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true" focusable="false">${unitImage(source, sprite, cropX, cropY, cropWidth, cropHeight)}</svg>`;
}
