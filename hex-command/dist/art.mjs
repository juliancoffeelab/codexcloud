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
            image.src = `assets/${kind}-${style}.png`;
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
