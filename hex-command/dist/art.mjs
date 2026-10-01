const unitOrder = ["soldier", "archer", "scout", "commander"];
const terrainOrder = ["plains", "forest", "hill", "water", "village", "keep"];
const storageKey = "hex-command-art-style";

export const ART_STYLES = {
  normal: { label: "Normal", units: { width: 2073, height: 758 } },
  "low-poly": { label: "Low-poly", units: { width: 2073, height: 759 } },
  pixel: { label: "Pixel", units: { width: 2172, height: 724 } },
  classic: { label: "Classic" },
};

export function savedArtStyle() {
  try {
    const value = localStorage.getItem(storageKey);
    return Object.hasOwn(ART_STYLES, value) ? value : "normal";
  } catch {
    return "normal";
  }
}

export function saveArtStyle(style) {
  if (!Object.hasOwn(ART_STYLES, style)) return;
  try {
    localStorage.setItem(storageKey, style);
  } catch {}
}

const loadedStyles = new Map();
export function loadArtStyle(style) {
  if (!Object.hasOwn(ART_STYLES, style)) return Promise.resolve(false);
  if (style === "classic") return Promise.resolve(true);
  if (!loadedStyles.has(style)) {
    const loading = Promise.all(
      ["units", "terrain"].map(
        (kind) =>
          new Promise((resolve) => {
            const image = new Image();
            image.onload = () => resolve(true);
            image.onerror = () => resolve(false);
            image.src = assetURL(style, kind);
          }),
      ),
    ).then((results) => {
      const ready = results.every(Boolean);
      if (!ready) loadedStyles.delete(style);
      return ready;
    });
    loadedStyles.set(style, loading);
  }
  return loadedStyles.get(style);
}

// Terrain sheets are drawn overhead with regular pointy hexes. Each frame
// is a square around the hex: cropping and uniform scaling preserve the art.
const terrainSheets = {
  "normal": {
    "width": 1536,
    "height": 1024,
    "revision": "aa703bdf9fe2",
    "frames": [
      {
        "x": 76.0,
        "y": 37.0,
        "size": 424
      },
      {
        "x": 556.0,
        "y": 37.0,
        "size": 424
      },
      {
        "x": 1036.0,
        "y": 37.0,
        "size": 424
      },
      {
        "x": 76.0,
        "y": 518.0,
        "size": 424
      },
      {
        "x": 556.0,
        "y": 518.0,
        "size": 424
      },
      {
        "x": 1036.0,
        "y": 518.0,
        "size": 424
      }
    ]
  },
  "low-poly": {
    "width": 1536,
    "height": 1024,
    "revision": "e7bf4ed38750",
    "frames": [
      {
        "x": 41.0,
        "y": 26.0,
        "size": 440
      },
      {
        "x": 548.0,
        "y": 26.0,
        "size": 440
      },
      {
        "x": 1055.0,
        "y": 26.0,
        "size": 440
      },
      {
        "x": 41.0,
        "y": 526.0,
        "size": 440
      },
      {
        "x": 548.0,
        "y": 526.0,
        "size": 440
      },
      {
        "x": 1055.0,
        "y": 526.0,
        "size": 440
      }
    ]
  },
  "pixel": {
    "width": 1536,
    "height": 1024,
    "revision": "9c2b16ea35a2",
    "frames": [
      {
        "x": 61.0,
        "y": 50.0,
        "size": 414
      },
      {
        "x": 561.0,
        "y": 50.0,
        "size": 414
      },
      {
        "x": 1061.0,
        "y": 50.0,
        "size": 414
      },
      {
        "x": 61.0,
        "y": 537.0,
        "size": 414
      },
      {
        "x": 561.0,
        "y": 537.0,
        "size": 414
      },
      {
        "x": 1061.0,
        "y": 537.0,
        "size": 414
      }
    ]
  }
};
function assetURL(style, kind) {
  return `assets/${kind}-${style}.png${kind === "terrain" ? `?v=${terrainSheets[style].revision}` : ""}`;
}
function terrainSprite(style, index, x, y, width, height, extraClass) {
  const sheet = terrainSheets[style];
  const { x: cropX, y: cropY, size } = sheet.frames[index];
  return `<svg class="art-sprite ${style === "pixel" ? "pixel-art" : ""} ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${size} ${size}" preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true" focusable="false"><foreignObject width="${size}" height="${size}"><div xmlns="http://www.w3.org/1999/xhtml" style="position:relative;width:100%;height:100%;overflow:hidden"><img src="${assetURL(style, "terrain")}" alt="" draggable="false" style="position:absolute;max-width:none;width:${sheet.width}px;height:${sheet.height}px;left:${-cropX}px;top:${-cropY}px"/></div></foreignObject></svg>`;
}

// Render a region of the original sheet directly. Keep the generated PNGs intact.
export function artSprite(
  style,
  kind,
  name,
  x,
  y,
  width,
  height,
  extraClass = "",
) {
  if (!Object.hasOwn(ART_STYLES, style) || style === "classic") return "";
  const isUnit = kind === "units";
  const order = isUnit ? unitOrder : terrainOrder;
  const index = order.indexOf(name);
  if (index < 0) return "";
  if (!isUnit) return terrainSprite(style, index, x, y, width, height, extraClass);
  const sheet = isUnit
    ? ART_STYLES[style].units
    : { width: 1536, height: 1024 };
  const columns = isUnit ? 4 : 3;
  const cropWidth = sheet.width / columns;
  const cropHeight = isUnit ? sheet.height : sheet.height / 2;
  const cropX = (index % columns) * cropWidth;
  const cropY = isUnit ? 0 : Math.floor(index / columns) * cropHeight;
  // Bound the sheet inside a fixed-size HTML viewport so its offscreen pixels
  // cannot enlarge the parent hex's SVG geometry or focus/selection bounds.
  return `<svg class="art-sprite ${style === "pixel" ? "pixel-art" : ""} ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 ${cropWidth} ${cropHeight}" preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true" focusable="false"><foreignObject width="${cropWidth}" height="${cropHeight}"><div xmlns="http://www.w3.org/1999/xhtml" style="position:relative;width:100%;height:100%;overflow:hidden"><img src="assets/${kind}-${style}.png" alt="" draggable="false" style="position:absolute;max-width:none;width:${sheet.width}px;height:${sheet.height}px;left:${-cropX}px;top:${-cropY}px"/></div></foreignObject></svg>`;
}
