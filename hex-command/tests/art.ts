import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { inflateSync } from "node:zlib";
import { TERRAIN, TYPES } from "../src/game/engine";
import {
  artSprite,
  dialoguePortrait,
  loadArtStyle,
  artworkForStyle,
} from "../src/presentation/art";
import { campaignSheets } from "../src/presentation/campaign-art";
import { pixelSources } from "../src/presentation/pixel-art";
import { Game } from "../src/game/engine";
import { createGameView } from "../src/game/view";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { mockSpriteCanvas } from "./helpers";

const restoreCanvas = mockSpriteCanvas();
const originalImage = Object.getOwnPropertyDescriptor(globalThis, "Image");
const requested: string[] = [];
let failNative = true;
class TestImage {
  onload?: () => void;
  onerror?: () => void;
  set src(url: string) {
    requested.push(url);
    Object.defineProperty(this, "src", { value: url });
    queueMicrotask(() => {
      if (failNative && url.includes("units-pixel.png")) this.onerror?.();
      else this.onload?.();
    });
  }
}
Object.defineProperty(globalThis, "Image", { configurable: true, value: TestImage });
assert.equal(
  artSprite(artworkForStyle("pixel"), "units", "thorncaller", 0, 0, 90, 120),
  "",
  "An unprepared frame must never fall back to a full atlas",
);
const firstLoad = loadArtStyle("pixel");
assert.equal(loadArtStyle("pixel"), firstLoad);
assert.equal(await firstLoad, false);
assert.equal(requested.length, 2);
failNative = false;
assert.equal(await loadArtStyle("pixel"), true);
assert.equal(requested.length, 4);
assert.equal(await loadArtStyle("pixel"), true);
assert.equal(requested.length, 4);
assert.equal(await loadArtStyle("normal"), true);
assert.equal(await loadArtStyle("low-poly"), true);

function extraction(sprite: string): {
  source: string;
  coordinates: number[];
  width: number;
  height: number;
  smoothing: boolean;
} {
  const encoded = /href="data:image\/png;base64,([^"]+)"/.exec(sprite)?.[1];
  assert(encoded, "Renderer receives only a standalone frame image");
  assert(!sprite.includes("foreignObject"));
  assert(!sprite.includes('overflow="hidden"'));
  assert(!sprite.includes('x="-'));
  return JSON.parse(Buffer.from(encoded, "base64").toString());
}

const terrainSamples = battlefieldTiles(
  createGameView(new Game("developer")),
  null,
  artworkForStyle("pixel"),
  null,
);

const frameSources = Object.fromEntries(
  Object.keys(campaignSheets.pixel.frames).map((key) => [
    key,
    key.startsWith("terrain_") ? pixelSources.terrain : pixelSources.units,
  ]),
);
for (const source of new Set(Object.values(frameSources))) {
  const png = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
  assert.equal(png.subarray(1, 4).toString(), "PNG");
  assert.equal(png.readUInt32BE(16), source.width);
  assert.equal(png.readUInt32BE(20), source.height);
  assert.equal(png[25], 6, "Focused pixel sources retain RGBA transparency");
  assert.equal(createHash("sha256").update(png).digest("hex").slice(0, 12), source.revision);
}
for (const [frame, source] of Object.entries(frameSources)) {
  const [x, y, width, height] =
    campaignSheets.pixel.frames[frame as keyof typeof campaignSheets.pixel.frames];
  assert([x, y, width, height].every(Number.isFinite));
  assert(width > 0 && height > 0);
  assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
  const kind = frame.startsWith("terrain_") ? "terrain" : "units";
  const name = frame.replace(/^(player|enemy|terrain)_/, "");
  const side = frame.startsWith("enemy_") ? "enemy" : "player";
  const sprite =
    frame === "portrait_worker"
      ? dialoguePortrait(artworkForStyle("pixel"), "worker", 0, 0, 60, 70)
      : artSprite(artworkForStyle("pixel"), kind, name, 0, 0, 60, 70, "", side);
  const isolated = extraction(sprite);
  assert(isolated.source.includes(`${source.filename}?v=${source.revision}`));
  assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
  assert.equal(isolated.width, width);
  assert.equal(isolated.height, height);
  assert.equal(isolated.smoothing, false);
}
// Check the pixels served to the ordinary renderer, not just manifest hashes.
const matrixFrames: string[] = [];
for (const kind of ["units", "terrain", "portraits"] as const) {
  const source = pixelSources[kind === "portraits" ? "units" : kind];
  const atlas = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
  const chunks: Buffer[] = [];
  for (let offset = 8; offset < atlas.length;) {
    const length = atlas.readUInt32BE(offset);
    if (atlas.toString("ascii", offset + 4, offset + 8) === "IDAT") {
      chunks.push(atlas.subarray(offset + 8, offset + 8 + length));
    }
    offset += length + 12;
  }
  const pixels = inflateSync(Buffer.concat(chunks));
  const stride = source.width * 4 + 1;
  assert.equal(pixels.length, stride * source.height);
  for (let y = 0; y < source.height; y++) {
    assert.equal(pixels[y * stride], 0, "Generated PNG scanlines use filter zero");
  }
  const root = new URL(`../art/pixel/${kind}/`, import.meta.url);
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const directory = new URL(`${entry.name}/`, root);
    const localPalette = new URL("palette.json", directory);
    const palette: Record<string, number[]> = JSON.parse(
      readFileSync(
        existsSync(localPalette)
          ? localPalette
          : new URL("../art/pixel/palette.json", import.meta.url),
        "utf8",
      ),
    );
    for (const filename of readdirSync(directory).filter((name) => name.endsWith(".txt"))) {
      const side = filename.slice(0, -4);
      assert(
        (kind === "terrain"
          ? ["tile"]
          : kind === "portraits"
            ? ["portrait"]
            : ["player", "enemy"]
        ).includes(side),
        `Unmapped matrix: ${entry.name}/${filename}`,
      );
      const frameName =
        entry.name === "bonearcher"
          ? "bone_archer"
          : entry.name === "grove"
            ? "sacred_grove"
            : entry.name;
      const key = `${kind === "terrain" ? "terrain" : side}_${frameName}`;
      matrixFrames.push(key);
      assert(Object.hasOwn(frameSources, key), `Matrix missing from public assets: ${key}`);
      const [cropX, cropY, width, height] =
        campaignSheets.pixel.frames[key as keyof typeof campaignSheets.pixel.frames];
      const rows = readFileSync(new URL(filename, directory), "utf8").trimEnd().split("\n");
      assert.equal(height, 48);
      assert.equal(width, 48);
      assert.equal(rows.length, height);
      if (kind === "terrain") {
        // Check against the editable source palette, independently of the
        // generated metadata used by the renderer.
        const groundSymbol = ["water", "bridge", "ford"].includes(entry.name) ? "W" : "G";
        const ground = palette[groundSymbol]!;
        assert.equal(ground[3], 255);
        const expected = `#${ground
          .slice(0, 3)
          .map((channel) => channel.toString(16).padStart(2, "0"))
          .join("")}`;
        const sample = terrainSamples.find((tile) => tile.terrain === entry.name);
        assert(sample, `Missing terrain sample: ${entry.name}`);
        assert.equal(
          sample.fill,
          expected,
          `${entry.name} must not expose a mismatched edge colour`,
        );
      }
      for (let y = 0; y < height; y++) {
        assert.equal(rows[y]!.length, width);
        for (let x = 0; x < width; x++) {
          const offset = (cropY + y) * stride + 1 + (cropX + x) * 4;
          assert.deepEqual(
            [...pixels.subarray(offset, offset + 4)],
            palette[rows[y]![x]!],
            `${key} pixel (${x}, ${y}) must match its matrix material`,
          );
        }
      }
    }
  }
}
const frameName = (name: string) =>
  name === "bonearcher" ? "bone_archer" : name === "grove" ? "sacred_grove" : name;
const expectedFrames = [
  ...Object.keys(TYPES).flatMap((name) =>
    ["player", "enemy"].map((side) => `${side}_${frameName(name)}`),
  ),
  ...Object.keys(TERRAIN).map((name) => `terrain_${frameName(name)}`),
  "portrait_worker",
].sort();
assert.deepEqual(
  matrixFrames.sort(),
  expectedFrames,
  "Every game unit and terrain has an editable matrix",
);
assert.deepEqual(Object.keys(frameSources).sort(), expectedFrames);
assert.deepEqual(
  readdirSync(new URL("../public/assets/", import.meta.url))
    .filter((name) => /pixel.*\.png$/.test(name))
    .sort(),
  ["terrain-pixel.png", "units-pixel.png"],
  "Pixel art is served from one unit atlas and one terrain atlas",
);

for (const style of ["pixel", "normal", "low-poly"] as const) {
  for (const side of ["player", "enemy"] as const) {
    for (const name of Object.keys(TYPES)) {
      extraction(artSprite(artworkForStyle(style), "units", name, 0, 0, 90, 120, "", side));
    }
  }
  for (const name of Object.keys(TERRAIN)) {
    extraction(artSprite(artworkForStyle(style), "terrain", name, 0, 0, 70, 70));
  }
  extraction(dialoguePortrait(artworkForStyle(style), "worker", 0, 0, 90, 120));
}
const lark = extraction(dialoguePortrait(artworkForStyle("pixel"), "thorncaller", 0, 0, 90, 120));
const [lx, ly] = campaignSheets.pixel.frames.player_thorncaller;
assert.deepEqual(
  lark.coordinates,
  [lx, ly, 48, 48, 0, 0, 48, 48],
  "Tall dialogue layout receives only Lark's 48×48 pixels",
);
assert.equal(lark.width, 48);
assert.equal(lark.height, 48);

if (originalImage) Object.defineProperty(globalThis, "Image", originalImage);
else Reflect.deleteProperty(globalThis, "Image");
restoreCanvas();
console.log(
  "PASS: isolated frame extraction for every style, faithful pixel atlases, and loading/retry publication",
);
