/** Offline draft preview; renders supplied data and never runs scenario logic. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { createCanvas, loadImage } from "@napi-rs/canvas";
import { artFrameKey, currentArtwork } from "../../../../src/presentation/art";
import { pixelSprites } from "../../../../src/presentation/pixel-art";
import { paintTerrainTiles, terrainTileView } from "../../../../src/presentation/terrain-map";
import {
  daylight,
  lightingMatrix,
  paintLights,
  shadePixels,
  type BattlefieldLighting,
} from "../../../../src/presentation/battlefield-lighting";
import { paintFog } from "../../../../src/presentation/battlefield-fog";
import {
  initSync,
  previewPresentation,
  type PresentationInput,
} from "../../../../rust/pkg/hex_command";
import { battlefieldSpriteSize, mapDimensions, tileCenter } from "../../../../src/ui/geometry";
import type { TileView } from "../../../../src/game/view";
import type { Side } from "../../../../src/game/types";

interface Position {
  col: number;
  row: number;
}
interface Unit extends Position {
  type: string;
  side: Side;
}
interface DraftMap {
  mapId: string;
  name?: string;
  cols: number;
  rows: number;
  tiles: TileView[];
  units?: Unit[];
  lighting?: BattlefieldLighting;
}
interface Annotation extends Position {
  text?: string;
  color?: string;
  dy?: number;
  ring?: boolean;
}
interface Overlay {
  title?: string;
  notes?: string[];
  marks?: string[];
  memory?: string[];
  round?: number;
  units?: Unit[];
  annotations?: Annotation[];
  arrows?: { from: Position; to: Position; color?: string }[];
}
const { values } = parseArgs({
  options: {
    map: { type: "string" },
    overlay: { type: "string" },
    output: { type: "string" },
  },
});
if (!values.map || !values.output)
  throw new Error(
    "Usage: render-map.ts --map map.json [--overlay overlay.json] --output preview.png",
  );
const map: DraftMap = JSON.parse(await readFile(resolve(values.map), "utf8"));
const overlay: Overlay = values.overlay
  ? JSON.parse(await readFile(resolve(values.overlay), "utf8"))
  : {};
if (
  !map.mapId ||
  !Number.isInteger(map.cols) ||
  map.cols <= 0 ||
  !Number.isInteger(map.rows) ||
  map.rows <= 0 ||
  !Array.isArray(map.tiles)
)
  throw new Error("Expected a mapId, positive integer cols/rows and tiles array");
const ids = new Set<string>();
for (const tile of map.tiles) {
  if (
    !Number.isInteger(tile.col) ||
    !Number.isInteger(tile.row) ||
    tile.col < 0 ||
    tile.col >= map.cols ||
    tile.row < 0 ||
    tile.row >= map.rows ||
    tile.id !== `${tile.col},${tile.row}` ||
    ids.has(tile.id)
  )
    throw new Error(`Invalid or duplicate tile: ${tile.id}`);
  ids.add(tile.id);
}
if (ids.size !== map.cols * map.rows) throw new Error("Supply the complete draft tile grid");
function position(point: Position) {
  if (!ids.has(`${point.col},${point.row}`))
    throw new Error(`Position outside map: ${point.col},${point.row}`);
  return tileCenter(point);
}
const root = fileURLToPath(new URL("../../../../", import.meta.url));
async function sprite(key: string) {
  const path = pixelSprites[key];
  if (!path)
    throw new Error(`Missing artwork: ${key}. Run npm run art:build after adding artwork.`);
  return loadImage(resolve(root, "public", path));
}
initSync({ module: await readFile(resolve(root, "rust/pkg/hex_command_bg.wasm")) });
const units = overlay.units ?? map.units ?? [];
const resolved = previewPresentation({
  tiles: map.tiles,
  units,
  lighting: map.lighting ?? daylight,
  marks: overlay.marks ?? [],
  memory: overlay.memory ?? [],
  round: overlay.round ?? 1,
} as PresentationInput);
const lighting = resolved.lighting;
const fog = new Set(resolved.fog);
const artwork = currentArtwork();
const tiles = map.tiles.map((tile) => ({ ...tile, ...terrainTileView(tile, map.mapId, artwork) }));
const frames = new Map<string, Awaited<ReturnType<typeof loadImage>>>();
for (const tile of tiles) {
  if (!tile.terrainFrame) throw new Error(`Missing terrain frame: ${tile.id}`);
  if (!frames.has(tile.terrainFrame.url))
    frames.set(tile.terrainFrame.url, await sprite(tile.frameKey));
}
const scale = 64 / battlefieldSpriteSize;
const size = mapDimensions(map);
const width = Math.ceil(size.width * scale),
  height = Math.ceil(size.height * scale);
const margin = 30,
  header = 74,
  footer = 30 + 22 * (overlay.notes?.length ?? 0);
const terrain = createCanvas(width, height),
  tc = terrain.getContext("2d");
tc.scale(scale, scale);
tc.imageSmoothingEnabled = false;
paintTerrainTiles(tc, tiles, frames);
const pixels = tc.getImageData(0, 0, width, height);
shadePixels(pixels.data, width, 0, height, scale, lighting);
tc.putImageData(pixels, 0, 0);
paintLights(tc, lighting);
const canvas = createCanvas(width + margin * 2, height + header + footer);
const ctx = canvas.getContext("2d");
ctx.fillStyle = "#101827";
ctx.fillRect(0, 0, canvas.width, canvas.height);
ctx.fillStyle = "#ede8d6";
ctx.font = "bold 24px sans-serif";
ctx.fillText(`${overlay.title ?? map.name ?? map.mapId} · Map draft`, margin, 31);
ctx.fillStyle = "#aab8c5";
ctx.font = "14px sans-serif";
ctx.fillText(
  "Static design preview · placements and arrows do not execute scenario mechanics",
  margin,
  55,
);
ctx.drawImage(terrain, margin, header);
ctx.save();
ctx.translate(margin, header);
ctx.scale(scale, scale);
ctx.imageSmoothingEnabled = false;
paintFog(
  ctx,
  map.tiles.filter((tile) => fog.has(tile.id)),
);
for (const unit of units) {
  if (unit.side === "enemy" && fog.has(`${unit.col},${unit.row}`)) continue;
  const p = position(unit),
    frame = createCanvas(64, 64),
    fc = frame.getContext("2d");
  fc.drawImage(await sprite(artFrameKey("units", unit.type, unit.side)), 0, 0);
  const data = fc.getImageData(0, 0, 64, 64),
    matrix = lightingMatrix(p.x, p.y, lighting);
  for (let i = 0; i < data.data.length; i += 4) {
    if (!data.data[i + 3]) continue;
    const rgb = Array.from(data.data.slice(i, i + 3));
    for (let channel = 0; channel < 3; channel++)
      data.data[i + channel] = rgb.reduce(
        (sum, value, source) => sum + value * matrix[channel * 5 + source]!,
        matrix[channel * 5 + 4]! * 255,
      );
  }
  fc.putImageData(data, 0, 0);
  ctx.drawImage(
    frame,
    p.x - battlefieldSpriteSize / 2,
    p.y - battlefieldSpriteSize / 2,
    battlefieldSpriteSize,
    battlefieldSpriteSize,
  );
}
for (const arrow of overlay.arrows ?? []) {
  const from = position(arrow.from),
    to = position(arrow.to),
    angle = Math.atan2(to.y - from.y, to.x - from.x);
  ctx.strokeStyle = arrow.color ?? "#dfb5bb";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(from.x, from.y);
  ctx.lineTo(to.x, to.y);
  for (const offset of [-0.5, 0.5]) {
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(to.x - 12 * Math.cos(angle + offset), to.y - 12 * Math.sin(angle + offset));
  }
  ctx.stroke();
}
for (const annotation of overlay.annotations ?? []) {
  const p = position(annotation),
    color = annotation.color ?? "#c3cbd3";
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  if (annotation.ring) {
    ctx.setLineDash([5, 5]);
    ctx.beginPath();
    ctx.arc(p.x, p.y, 32, 0, Math.PI * 2);
    ctx.stroke();
    ctx.setLineDash([]);
  }
  if (annotation.text) {
    ctx.font = "bold 14px sans-serif";
    ctx.textAlign = "center";
    ctx.lineWidth = 4;
    ctx.strokeStyle = "#101827";
    ctx.strokeText(annotation.text, p.x, p.y + (annotation.dy ?? -40));
    ctx.fillStyle = color;
    ctx.fillText(annotation.text, p.x, p.y + (annotation.dy ?? -40));
  }
}
ctx.restore();
ctx.fillStyle = "#c0cbd4";
ctx.font = "14px sans-serif";
for (const [i, note] of (overlay.notes ?? []).entries())
  ctx.fillText(note, margin, header + height + 24 + i * 22);
const output = resolve(values.output);
await mkdir(dirname(output), { recursive: true });
await writeFile(output, canvas.toBuffer("image/png"));
console.log(output);
