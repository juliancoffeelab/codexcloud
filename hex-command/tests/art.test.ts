import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { inflateSync } from "node:zlib";
import { TERRAIN, TYPES } from "./game-fixture";
let art: typeof import("../src/presentation/art");
import { campaignSheets } from "../src/presentation/campaign-art";
import { pixelSources } from "../src/presentation/pixel-art";
import {
  styledSheets,
  styledTerrainSheets,
  styledTerrainColors,
} from "../src/presentation/styled-art";
import { Game } from "./game-fixture";
import { createGameView } from "./fixture-view";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { mockSpriteCanvas } from "./helpers";

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

test("Unprepared frames stay blank; concurrent loads deduplicate and failures retry", async () => {
  failNative = true;
  assert.equal(
    art.artSprite(art.artworkForStyle("pixel"), "units", "thorncaller", 0, 0, 90, 120),
    "",
    "An unprepared frame must never fall back to a full atlas",
  );
  const firstLoad = art.loadArtStyle("pixel");
  assert.equal(art.loadArtStyle("pixel"), firstLoad);
  assert.equal(await firstLoad, false);
  assert.equal(requested.length, 2);
  failNative = false;
  assert.equal(await art.loadArtStyle("pixel"), true);
  assert.equal(requested.length, 4);
  assert.equal(await art.loadArtStyle("pixel"), true);
  assert.equal(requested.length, 4);
  assert.equal(await art.loadArtStyle("normal"), true);
  assert.equal(await art.loadArtStyle("low-poly"), true);
});

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

const frameSources = Object.fromEntries(
  Object.keys(campaignSheets.pixel.frames).map((key) => [
    key,
    key.startsWith("terrain_") ? pixelSources.terrain : pixelSources.units,
  ]),
);
test("Public pixel PNG dimensions, alpha and hashes match their manifests", async () => {
  for (const source of new Set(Object.values(frameSources))) {
    const png = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
    assert.equal(png.subarray(1, 4).toString(), "PNG");
    assert.equal(png.readUInt32BE(16), source.width);
    assert.equal(png.readUInt32BE(20), source.height);
    assert.equal(png[25], 6, "Focused pixel sources retain RGBA transparency");
    assert.equal(createHash("sha256").update(png).digest("hex").slice(0, 12), source.revision);
  }
});
test("Every pixel frame is extracted into a standalone unsmoothed image", async () => {
  await art.loadArtStyle("pixel");
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
        ? art.dialoguePortrait(art.artworkForStyle("pixel"), "worker", 0, 0, 60, 70)
        : art.artSprite(art.artworkForStyle("pixel"), kind, name, 0, 0, 60, 70, "", side);
    const isolated = extraction(sprite);
    assert(isolated.source.includes(`${source.filename}?v=${source.revision}`));
    assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    assert.equal(isolated.width, width);
    assert.equal(isolated.height, height);
    assert.equal(isolated.smoothing, false);
  }
});
// Check the pixels served to the ordinary renderer, not just manifest hashes.
test("Every unit, terrain and portrait pixel matches its editable matrix", async () => {
  await art.loadArtStyle("pixel");
  const terrainSamples = battlefieldTiles(
    createGameView(new Game("developer")),
    null,
    art.artworkForStyle("pixel"),
    null,
  );
  const matrixFrames: string[] = [];
  const factions: Record<string, Record<string, number[]>> = JSON.parse(
    readFileSync(new URL("../art/pixel/factions.json", import.meta.url), "utf8"),
  );
  for (const palette of Object.values(factions)) {
    assert.deepEqual(Object.keys(palette).sort(), ["*", "^"]);
  }
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
        const matrixKind = filename.slice(0, -4);
        assert(
          (kind === "terrain"
            ? ["tile"]
            : kind === "portraits"
              ? ["portrait"]
              : ["sprite"]
          ).includes(matrixKind),
          `Unmapped matrix: ${entry.name}/${filename}`,
        );
        for (const side of kind === "units" ? ["player", "enemy"] : [matrixKind]) {
          const colours = kind === "units" ? { ...palette, ...factions[side] } : palette;
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
          assert.equal(width, rows[0]!.length);
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
                colours[rows[y]![x]!],
                `${key} pixel (${x}, ${y}) must match its matrix material`,
              );
            }
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
  for (const frame of expectedFrames) {
    assert(matrixFrames.includes(frame), `Game frame has no editable matrix: ${frame}`);
  }
  assert.deepEqual(
    Object.keys(frameSources).sort(),
    matrixFrames.sort(),
    "Every artwork source is packed, including designs awaiting gameplay registration",
  );
  assert.deepEqual(
    readdirSync(new URL("../public/assets/", import.meta.url))
      .filter((name) => /pixel.*\.png$/.test(name))
      .sort(),
    ["terrain-pixel.png", "units-pixel.png"],
    "Pixel art is served from one unit atlas and one terrain atlas",
  );
});

for (const style of ["pixel", "normal", "low-poly"] as const) {
  test(
    "Every artwork style isolates unit, terrain and portrait frames — " + JSON.stringify([style]),
    async () => {
      await art.loadArtStyle(style);
      for (const side of ["player", "enemy"] as const) {
        for (const name of Object.keys(TYPES)) {
          extraction(
            art.artSprite(art.artworkForStyle(style), "units", name, 0, 0, 90, 120, "", side),
          );
        }
      }
      for (const name of Object.keys(TERRAIN)) {
        extraction(art.artSprite(art.artworkForStyle(style), "terrain", name, 0, 0, 70, 70));
      }
      extraction(art.dialoguePortrait(art.artworkForStyle(style), "worker", 0, 0, 90, 120));
    },
  );
}
for (const style of ["normal", "low-poly"] as const) {
  test("Complete styled roster uses its own atlas without pixel fallbacks — " + style, async () => {
    await art.loadArtStyle(style);
    const source = styledSheets[style].source;
    const png = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
    assert.equal(png.readUInt32BE(16), source.width);
    assert.equal(png.readUInt32BE(20), source.height);
    assert.equal(createHash("sha256").update(png).digest("hex").slice(0, 12), source.revision);
    const frameName = (name: string) => (name === "bonearcher" ? "bone_archer" : name);
    const expected = Object.keys(TYPES).flatMap((type) =>
      ["enemy", "player"].map((side) => `${side}_${frameName(type)}`),
    );
    assert.deepEqual(
      Object.keys(styledSheets[style].frames).sort(),
      [...expected, "portrait_worker"].sort(),
    );
    for (const key of [...expected, "portrait_worker"]) {
      const frame = art.artworkForStyle(style).frames[key];
      assert(frame, `${style} is missing ${key}`);
      assert.equal(frame.pixel, false, `${style}/${key} must not use pixel artwork`);
      const unit = key.replace(/^(player|enemy)_/, "");
      const rendered =
        key === "portrait_worker"
          ? art.dialoguePortrait(art.artworkForStyle(style), "worker", 0, 0, 56, 56)
          : art.artSprite(
              art.artworkForStyle(style),
              "units",
              unit,
              0,
              0,
              56,
              56,
              "",
              key.startsWith("enemy_") ? "enemy" : "player",
            );
      const isolated = extraction(rendered);
      assert.equal(isolated.smoothing, true);
      assert(isolated.source.includes(`/assets/${source.filename}?v=${source.revision}`));
      const [x, y, width, height] =
        styledSheets[style].frames[key as keyof typeof styledSheets.normal.frames];
      assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
      assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    }
    assert(
      !requested.some((url) => url.includes("pixel.png")),
      "A styled roster must not request the pixel atlas",
    );
  });
}
for (const style of ["normal", "low-poly"] as const) {
  test(
    "Static styled surfaces and bridge use complete isolated terrain frames — " + style,
    async () => {
      await art.loadArtStyle(style);
      const sheet = styledTerrainSheets[style];
      const png = readFileSync(
        new URL(`../public/assets/${sheet.source.filename}`, import.meta.url),
      );
      assert.equal(png.readUInt32BE(16), sheet.source.width);
      assert.equal(png.readUInt32BE(20), sheet.source.height);
      assert.equal(
        createHash("sha256").update(png).digest("hex").slice(0, 12),
        sheet.source.revision,
      );
      assert.deepEqual(
        Object.keys(sheet.frames).sort(),
        Object.keys(TERRAIN)
          .map((name) => `terrain_${name === "grove" ? "sacred_grove" : name}`)
          .sort(),
      );
      const samples = battlefieldTiles(
        createGameView(new Game("developer")),
        null,
        art.artworkForStyle(style),
        null,
      );
      const authoredColors = JSON.parse(
        readFileSync(new URL("../art/styled/terrain-colors.json", import.meta.url), "utf8"),
      );
      for (const name of ["plains", "water", "ford", "bridge"] as const) {
        const rendered = art.artSprite(art.artworkForStyle(style), "terrain", name, 0, 0, 70, 70);
        assert.equal((rendered.match(/<image /g) ?? []).length, 1);
        assert(!rendered.includes("<path"), "The bridge is a complete static sprite");
        const isolated = extraction(rendered);
        const [x, y, width, height] = sheet.frames[`terrain_${name}`];
        assert.equal(width, height, "A full square texture covers the exact board hex");
        assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
        assert(
          isolated.source.includes(`/assets/${sheet.source.filename}?v=${sheet.source.revision}`),
        );
        assert.equal(isolated.smoothing, true);
        assert.equal(styledTerrainColors[style][name], authoredColors[style][name]);
        assert.equal(
          samples.find((tile) => tile.terrain === name)?.fill,
          authoredColors[style][name],
        );
      }
    },
  );
}
test("Tall Lark dialogue uses exactly one 48×48 frame", async () => {
  await art.loadArtStyle("pixel");
  const lark = extraction(
    art.dialoguePortrait(art.artworkForStyle("pixel"), "thorncaller", 0, 0, 90, 120),
  );
  const [lx, ly] = campaignSheets.pixel.frames.player_thorncaller;
  assert.deepEqual(
    lark.coordinates,
    [lx, ly, 48, 48, 0, 0, 48, 48],
    "Tall dialogue layout receives only Lark's 48×48 pixels",
  );
  assert.equal(lark.width, 48);
  assert.equal(lark.height, 48);
});

let restoreCanvas: () => void;
beforeEach(async () => {
  vi.resetModules();
  art = await import("../src/presentation/art");
  requested.length = 0;
  failNative = false;
  vi.stubGlobal("Image", TestImage);
  restoreCanvas = mockSpriteCanvas();
});
afterEach(() => {
  restoreCanvas();
});

afterEach(() => {
  vi.unstubAllGlobals();
});
