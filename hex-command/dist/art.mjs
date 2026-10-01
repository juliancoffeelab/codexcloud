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

// Top-face vertices in sheet coordinates: top, upper-right, lower-right,
// bottom, lower-left, upper-left. Exclude the perspective soil skirt.
const terrainFaces = {
  normal: [
    [[258,128],[474,218],[480,354],[258,450],[36,351],[43,217]],
    [[768,133],[985,218],[991,354],[771,449],[548,350],[552,219]],
    [[1280,128],[1498,215],[1503,355],[1284,450],[1055,350],[1062,216]],
    [[258,588],[473,677],[479,818],[259,919],[40,818],[44,677]],
    [[771,585],[987,677],[995,818],[776,919],[551,819],[554,677]],
    [[1280,583],[1499,676],[1504,818],[1286,919],[1055,818],[1061,677]],
  ],
  "low-poly": [
    [[271,100],[490,203],[490,329],[266,428],[43,329],[43,204]],
    [[768,106],[984,203],[984,337],[765,437],[548,337],[548,203]],
    [[1276,107],[1494,203],[1494,334],[1265,437],[1042,334],[1042,203]],
    [[268,568],[490,666],[490,809],[266,913],[46,810],[46,666]],
    [[769,574],[991,672],[991,810],[769,913],[544,810],[544,672]],
    [[1276,573],[1502,674],[1502,809],[1277,914],[1042,810],[1042,674]],
  ],
  pixel: [
    [[268,106],[472,215],[472,345],[268,454],[65,345],[65,215]],
    [[768,106],[970,215],[970,345],[768,454],[566,345],[566,215]],
    [[1268,106],[1470,215],[1470,345],[1268,454],[1066,345],[1066,215]],
    [[268,548],[472,655],[472,785],[268,890],[65,785],[65,655]],
    [[768,548],[970,655],[970,785],[768,890],[566,785],[566,655]],
    [[1268,548],[1470,655],[1470,785],[1268,890],[1066,785],[1066,655]],
  ],
};

// Reproject each face as six affine triangles. The destination is the same
// pointy hex used by the board, rather than a second hex inside a square crop.
function terrainSprite(style, index, x, y, width, height, extraClass) {
  const source = terrainFaces[style][index];
  const center = source.reduce((sum, point) =>
    [sum[0] + point[0] / 6, sum[1] + point[1] / 6], [0, 0]);
  const target = [[35,0],[35 + 35*Math.sqrt(3)/2,17.5],
    [35 + 35*Math.sqrt(3)/2,52.5],[35,70],
    [35 - 35*Math.sqrt(3)/2,52.5],[35 - 35*Math.sqrt(3)/2,17.5]];
  const triangles = source.map((point, i) => {
    const next = source[(i + 1) % 6];
    const u = [point[0]-center[0], point[1]-center[1]];
    const v = [next[0]-center[0], next[1]-center[1]];
    const a = [target[i][0]-35, target[i][1]-35];
    const b = [target[(i+1)%6][0]-35, target[(i+1)%6][1]-35];
    const det = u[0]*v[1]-v[0]*u[1];
    const m = [(a[0]*v[1]-b[0]*u[1])/det,
      (a[1]*v[1]-b[1]*u[1])/det,
      (b[0]*u[0]-a[0]*v[0])/det,
      (b[1]*u[0]-a[1]*v[0])/det];
    m.push(35-m[0]*center[0]-m[2]*center[1],
      35-m[1]*center[0]-m[3]*center[1]);
    // Clip each sheet transform inside the bounded HTML sprite viewport.
    const points = [[35,35],target[i],target[(i+1)%6]];
    // Extend internal seams slightly; the outer board clip stays exact.
    const centroid = points.reduce((sum, p) => [sum[0]+p[0]/3, sum[1]+p[1]/3], [0,0]);
    const expanded = points.map(p => [centroid[0]+(p[0]-centroid[0])*1.02,
      centroid[1]+(p[1]-centroid[1])*1.02]);
    return `<div style="position:absolute;inset:0;clip-path:polygon(${expanded.map(p=>`${p[0]}px ${p[1]}px`).join(',')})"><img src="assets/terrain-${style}.png" alt="" draggable="false" style="position:absolute;max-width:none;width:1536px;height:1024px;transform-origin:0 0;transform:matrix(${m.join(',')});image-rendering:${style === 'pixel' ? 'pixelated' : 'auto'}"/></div>`;
  }).join('');
  return `<svg class="art-sprite ${style === 'pixel' ? 'pixel-art' : ''} ${extraClass}" x="${x}" y="${y}" width="${width}" height="${height}" viewBox="0 0 70 70" preserveAspectRatio="none" overflow="hidden" aria-hidden="true" focusable="false"><foreignObject width="70" height="70"><div xmlns="http://www.w3.org/1999/xhtml" style="position:relative;width:70px;height:70px;overflow:hidden">${triangles}</div></foreignObject></svg>`;
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
