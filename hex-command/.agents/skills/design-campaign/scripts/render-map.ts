/** Offline draft preview; renders supplied data and never runs scenario logic. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { createCanvas, loadImage } from "@napi-rs/canvas";
import { artFrameKey, currentArtwork, type FrameImage } from "../../../../src/presentation/art";
import { pixelSprites } from "../../../../src/presentation/pixel-art";
import {
  paintTerrainTiles,
  terrainDrawingOrder,
  terrainMapTiles,
} from "../../../../src/presentation/terrain-map";
import {
  lightingMatrix,
  paintLights,
  shadePixels,
} from "../../../../src/presentation/battlefield-lighting";
import { paintFog } from "../../../../src/presentation/battlefield-fog";
import {
  initSync,
  previewPresentation,
  type PresentationInput,
  type MapLighting,
} from "../../../../rust/pkg/hex_command";
import { battlefieldSpriteSize, mapDimensions, tileCenter } from "../../../../src/ui/geometry";
import type { TileView } from "../../../../src/game/view";
import type { ArtworkFaction } from "../../../../src/presentation/art";
import type { Side } from "../../../../src/game/types";

interface Position {
  col: number;
  row: number;
}
interface Unit {
  tile?: string;
  col?: number;
  row?: number;
  faction?: ArtworkFaction;
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
  lighting?: MapLighting;
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
  choices?: string[];
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
    grid: { type: "boolean", default: false },
    "grid-color": { type: "string" },
    artwork: { type: "string" },
    "draw-order": { type: "string", default: "native" },
    "order-seed": { type: "string", default: "1" },
    "order-terrain": { type: "string" },
  },
});
if (!values.map || !values.output)
  throw new Error(
    "Usage: render-map.ts --map map.json [--overlay overlay.json] [--grid] [--grid-color red] --output preview.png",
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
const positions = new Set<string>();
for (const tile of map.tiles) {
  if (
    !Number.isInteger(tile.col) ||
    !Number.isInteger(tile.row) ||
    tile.col < 0 ||
    tile.col >= map.cols ||
    tile.row < 0 ||
    tile.row >= map.rows ||
    !tile.id ||
    !/^[a-z0-9-]+$/.test(tile.id) ||
    positions.has(`${tile.col},${tile.row}`) ||
    ids.has(tile.id)
  )
    throw new Error(`Invalid or duplicate tile: ${tile.id}`);
  ids.add(tile.id);
  positions.add(`${tile.col},${tile.row}`);
}
if (ids.size !== map.cols * map.rows) throw new Error("Supply the complete draft tile grid");
function position(point: Position) {
  if (!positions.has(`${point.col},${point.row}`))
    throw new Error(`Position outside map: ${point.col},${point.row}`);
  return tileCenter(point);
}
const root = fileURLToPath(new URL("../../../../", import.meta.url));
// Candidate files replace existing complete frames, never paint onto the map.
const baseArtwork = currentArtwork();
const candidates: Record<string, FrameImage> = values.artwork
  ? JSON.parse(await readFile(resolve(values.artwork), "utf8"))
  : {};
for (const [key, frame] of Object.entries(candidates)) {
  const original = baseArtwork.frames[key];
  if (
    !original ||
    original.width !== frame.width ||
    original.height !== frame.height ||
    original.anchorX !== frame.anchorX ||
    original.anchorY !== frame.anchorY ||
    !Number.isInteger(frame.z) ||
    typeof frame.url !== "string"
  )
    throw new Error(`Candidate must preserve the existing frame geometry: ${key}`);
  candidates[key] = { ...frame, url: resolve(dirname(resolve(values.artwork!)), frame.url) };
}
async function sprite(key: string) {
  if (candidates[key]) return loadImage(candidates[key].url);
  const path = pixelSprites[key];
  if (!path)
    throw new Error(`Missing artwork: ${key}. Run npm run art:build after adding artwork.`);
  return loadImage(resolve(root, "public", path));
}
initSync({ module: await readFile(resolve(root, "rust/pkg/hex_command_bg.wasm")) });
const units = (overlay.units ?? map.units ?? []).map((unit) => {
  const tile = unit.tile
    ? map.tiles.find((tile) => tile.id === unit.tile)
    : map.tiles.find((tile) => tile.col === unit.col && tile.row === unit.row);
  if (!tile) throw new Error("Unit placement is outside the draft map");
  return { ...unit, tile: tile.id, col: tile.col, row: tile.row };
});
const resolved = previewPresentation({
  tiles: map.tiles,
  units: units.map(({ col: _col, row: _row, ...unit }) => unit),
  lighting: map.lighting ?? { time: "day" },
  marks: overlay.marks ?? [],
  choices: overlay.choices ?? [],
  memory: overlay.memory ?? [],
  round: overlay.round ?? 1,
} as PresentationInput);
const lighting = resolved.lighting;
const fog = new Set(resolved.fog);
const artwork = { frames: { ...baseArtwork.frames, ...candidates } };
const terrainDefinitions = JSON.parse(
  await readFile(resolve(root, "rust/data/game/terrain.json"), "utf8"),
) as { id: string; role: import("../../../../rust/pkg/hex_command").TerrainRole }[];
const terrainRoles = Object.fromEntries(
  terrainDefinitions.map((definition) => [definition.id, definition.role]),
);
const tiles = terrainMapTiles(map.tiles, map.mapId, artwork, terrainRoles);
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
const order = values["draw-order"];
if (order === "native") paintTerrainTiles(tc, tiles, frames);
else {
  if (!["reverse-within-z", "shuffle-within-z", "reverse-all", "shuffle-all"].includes(order))
    throw new Error(`Unsupported diagnostic draw order: ${order}`);
  let seed = Number(values["order-seed"]);
  if (!Number.isInteger(seed) || seed <= 0 || seed > 0xffffffff)
    throw new Error("order-seed must be a positive 32-bit integer");
  const ordered = terrainDrawingOrder(tiles);
  const requested = values["order-terrain"]?.split(",");
  if (requested?.some((kind) => !map.tiles.some((tile) => tile.terrain === kind)))
    throw new Error("Every order-terrain entry must name terrain present in the draft");
  const selected = values["order-terrain"]
    ? new Set(
        map.tiles
          .filter((tile) => values["order-terrain"]!.split(",").includes(tile.terrain))
          .map((tile) => tile.id),
      )
    : new Set(ordered.map((tile) => tile.id));
  const targets = ordered.filter((tile) => selected.has(tile.id));
  const levels = new Map<number, typeof ordered>();
  for (const tile of targets) {
    const z = tile.terrainFrame!.z ?? 0;
    if (!levels.has(z)) levels.set(z, []);
    levels.get(z)!.push(tile);
  }
  const groups = order.endsWith("all") ? [targets] : [...levels.values()];
  for (const group of groups) {
    if (order.startsWith("reverse")) group.reverse();
    else
      for (let i = group.length - 1; i > 0; i--) {
        seed ^= seed << 13;
        seed ^= seed >>> 17;
        seed ^= seed << 5;
        const j = (seed >>> 0) % (i + 1);
        [group[i], group[j]] = [group[j]!, group[i]!];
      }
  }
  const reordered = groups.flat();
  let cursor = 0;
  // Exercise the actual sequence without changing unselected scenery. Calling
  // the shared painter one frame at a time prevents its normal sort restoring it.
  for (const tile of ordered)
    paintTerrainTiles(tc, [selected.has(tile.id) ? reordered[cursor++]! : tile], frames);
}
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
// Match Battlefield.svelte: tile order, feet at center + 20, then a separate
// indicator layer above every sprite. Draft placements depict full-health units.
const occupants = new Map(units.map((unit) => [unit.tile, unit]));
const visibleUnits = map.tiles.flatMap((tile) => {
  const unit = occupants.get(tile.id);
  return unit && !(unit.side === "enemy" && fog.has(tile.id)) ? [unit] : [];
});
for (const unit of visibleUnits) {
  const p = position(unit),
    frame = createCanvas(64, 64),
    fc = frame.getContext("2d");
  const palette = unit.faction ?? unit.side;
  const teamColor =
    palette === "passive"
      ? "#efecdc"
      : palette === "green"
        ? "#83b765"
        : unit.side === "player"
          ? "#6fdec0"
          : "#ff9590";
  ctx.beginPath();
  ctx.ellipse(p.x, p.y + 19, 19, 7, 0, 0, Math.PI * 2);
  ctx.fillStyle = teamColor;
  ctx.globalAlpha = 0.5;
  ctx.fill();
  ctx.globalAlpha = 1;
  ctx.strokeStyle = teamColor;
  ctx.lineWidth = 1.8;
  ctx.stroke();
  fc.drawImage(await sprite(artFrameKey("units", unit.type, unit.faction ?? unit.side)), 0, 0);
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
    p.y + 20 - battlefieldSpriteSize,
    battlefieldSpriteSize,
    battlefieldSpriteSize,
  );
}
for (const unit of visibleUnits) {
  const p = position(unit);
  ctx.beginPath();
  ctx.roundRect(p.x - 19, p.y + 27, 38, 4, 2);
  ctx.fillStyle = "#a9d0a5";
  ctx.fill();
}
if (values.grid) {
  ctx.save();
  ctx.strokeStyle = values["grid-color"] ?? "#ff4040";
  ctx.globalAlpha = 0.9;
  ctx.lineWidth = 1 / scale;
  ctx.beginPath();
  const edges = new Set<string>();
  for (const tile of tiles) {
    const points = tile.points
      .split(" ")
      .map((point) => point.split(",").map(Number) as [number, number]);
    for (let index = 0; index < points.length; index++) {
      const a = points[index]!,
        b = points[(index + 1) % points.length]!;
      const key = [a, b]
        .map(([x, y]) => `${x.toFixed(6)},${y.toFixed(6)}`)
        .sort()
        .join("|");
      if (edges.has(key)) continue;
      edges.add(key);
      ctx.moveTo(...a);
      ctx.lineTo(...b);
    }
  }
  ctx.stroke();
  ctx.restore();
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
