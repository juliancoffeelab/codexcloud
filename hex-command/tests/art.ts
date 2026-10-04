import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { inflateSync } from "node:zlib";
import { artSprite, loadArtStyle } from "../src/presentation/art";
import { campaignSheets } from "../src/presentation/campaign-art";
import { pixelSources } from "../src/presentation/pixel-art";

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
  assert(
    artSprite("pixel", kind, name, 0, 0, 60, 70, "", side).includes(
      `${source.filename}?v=${source.revision}`,
    ),
    `${frame} must use its shared public atlas in the regular game renderer`,
  );
}
const nativeURL = `${pixelSources.units.filename}?v=${pixelSources.units.revision}`;
// Verify every source matrix reaches the public atlas with its exact palette,
// including single-faction units. This checks pixels, not just manifest hashes.
const atlas = readFileSync(
  new URL(`../public/assets/${pixelSources.units.filename}`, import.meta.url),
);
const chunks: Buffer[] = [];
for (let offset = 8; offset < atlas.length;) {
  const length = atlas.readUInt32BE(offset);
  if (atlas.toString("ascii", offset + 4, offset + 8) === "IDAT") {
    chunks.push(atlas.subarray(offset + 8, offset + 8 + length));
  }
  offset += length + 12;
}
const pixels = inflateSync(Buffer.concat(chunks));
const stride = pixelSources.units.width * 4 + 1;
assert.equal(pixels.length, stride * pixelSources.units.height);
for (let y = 0; y < pixelSources.units.height; y++) {
  assert.equal(pixels[y * stride], 0, "Generated native PNG scanlines use filter zero");
}
const unitsRoot = new URL("../art/pixel/units/", import.meta.url);
const matrixFrames: string[] = [];
for (const entry of readdirSync(unitsRoot, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const directory = new URL(`${entry.name}/`, unitsRoot);
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
    assert(["player", "enemy"].includes(side), `Unmapped matrix: ${entry.name}/${filename}`);
    const frameName = entry.name === "bonearcher" ? "bone_archer" : entry.name;
    const key = `${side}_${frameName}`;
    matrixFrames.push(key);
    assert(Object.hasOwn(frameSources, key), `Matrix missing from public assets: ${key}`);
    const [cropX, cropY, width, height] =
      campaignSheets.pixel.frames[key as keyof typeof campaignSheets.pixel.frames];
    const rows = readFileSync(new URL(filename, directory), "utf8").trimEnd().split("\n");
    assert.equal(rows.length, height);
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
const legacy = JSON.parse(
  readFileSync(new URL("../art/pixel/legacy/frames.json", import.meta.url), "utf8"),
) as { units: Record<string, unknown>; terrain: Record<string, unknown> };
assert.deepEqual(
  Object.keys(frameSources).sort(),
  [
    ...new Set([...Object.keys(legacy.units), ...Object.keys(legacy.terrain), ...matrixFrames]),
  ].sort(),
);
assert.deepEqual(
  readdirSync(new URL("../public/assets/", import.meta.url))
    .filter((name) => /pixel.*\.png$/.test(name))
    .sort(),
  ["terrain-pixel.png", "units-pixel.png"],
  "Pixel art is served from one unit atlas and one terrain atlas",
);
assert(
  artSprite("pixel", "units", "scout", 0, 0, 60, 70).includes(nativeURL),
  "Player scout uses its existing matrix",
);
assert(
  artSprite("pixel", "units", "scout", 0, 0, 60, 70, "", "enemy").includes(nativeURL),
  "Enemy scout keeps its old artwork inside the shared unit atlas",
);
for (const side of ["player", "enemy"] as const) {
  for (const name of ["adept", "bonearcher", "boar", "necromancer", "skeleton", "wolf"] as const) {
    const key = `${side}_${name === "bonearcher" ? "bone_archer" : name}` as const;
    assert.equal(frameSources[key], pixelSources.units);
    const sprite = artSprite("pixel", "units", name, 0, 0, 60, 70, "", side);
    assert(sprite.includes(nativeURL));
    assert(
      sprite.includes(`width:${pixelSources.units.width}px;height:${pixelSources.units.height}px`),
    );
    assert(sprite.includes('class="art-sprite pixel-art'));
    assert(sprite.includes('viewBox="0 0 48 48"'));
  }
  for (const name of ["skeleton", "wolf", "adept", "bonearcher", "boar", "necromancer"]) {
    for (const style of ["normal", "low-poly"] as const) {
      const unchanged = artSprite(style, "units", name, 0, 0, 60, 70, "", side);
      assert(unchanged.includes(`campaign-${style}-${side}.png`));
      assert(!unchanged.includes("campaign-pixel-"));
    }
  }
  assert(artSprite("pixel", "units", "warden", 0, 0, 60, 70, "", side).includes(nativeURL));
  assert(artSprite("pixel", "units", "soldier", 0, 0, 60, 70, "", side).includes(nativeURL));
  for (const style of ["normal", "low-poly"] as const) {
    assert(
      artSprite(style, "units", "scout", 0, 0, 60, 70, "", side).includes(`units-${style}.png`),
    );
  }
}
assert(
  artSprite("pixel", "terrain", "marsh", 0, 0, 70, 70).includes(
    `${pixelSources.terrain.filename}?v=${pixelSources.terrain.revision}`,
  ),
);

const originalImage = Object.getOwnPropertyDescriptor(globalThis, "Image");
const requested: string[] = [];
let failNative = true;
class TestImage {
  onload?: () => void;
  onerror?: () => void;
  set src(url: string) {
    requested.push(url);
    queueMicrotask(() => {
      if (failNative && url.includes(nativeURL)) this.onerror?.();
      else this.onload?.();
    });
  }
}
try {
  Object.defineProperty(globalThis, "Image", { configurable: true, value: TestImage });
  const firstLoad = loadArtStyle("pixel");
  assert.equal(loadArtStyle("pixel"), firstLoad, "Concurrent requests share the pending load");
  assert.equal(await firstLoad, false, "Missing unit atlas rejects ready state");
  assert.equal(requested.length, 2, "Each shared campaign source is preloaded only once");
  assert.equal(requested.filter((url) => url.includes(nativeURL)).length, 1);
  assert.equal(requested.filter((url) => url.includes("campaign-pixel-undead.png")).length, 0);
  failNative = false;
  assert.equal(await loadArtStyle("pixel"), true, "A failed unit atlas load can be retried");
  assert.equal(requested.length, 4);
  assert.equal(requested.filter((url) => url.includes(nativeURL)).length, 2);
  assert.equal(await loadArtStyle("pixel"), true);
  assert.equal(requested.length, 4, "Ready style keeps its cached load");
} finally {
  if (originalImage) Object.defineProperty(globalThis, "Image", originalImage);
  else Reflect.deleteProperty(globalThis, "Image");
}
console.log(
  "PASS: native pixel frames, source hashes/crops, preserved art and preload retry/cache",
);
