import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "./game-fixture";
let art: typeof import("@frontend/Art");
import { pixelFrames, pixelSources, pixelUnitModels } from "../src/presentation/pixel-art";
import {
  styledSheets,
  styledTerrainSheets,
  styledTerrainColors,
} from "../src/presentation/styled-art";
import { mockSpriteCanvas } from "./helpers";

function style(name: string) {
  const value = art.style_of_string(name);
  assert.notEqual(value, undefined);
  return value!;
}
function forStyle(name: string) {
  return art.artwork(style(name));
}
function load(name: string) {
  return art.load(style(name));
}
function sprite(
  artwork: import("@frontend/Art").Artwork,
  kind: string,
  name: string,
  x: number,
  y: number,
  width: number,
  height: number,
  className = "",
  side = "player",
) {
  return art.sprite(artwork, kind, name, x, y, width, height, className, side, undefined);
}

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
  const unpublished = forStyle("pixel");
  assert.equal(
    sprite(forStyle("pixel"), "units", "thorncaller", 0, 0, 90, 120),
    "",
    "An unprepared frame must never fall back to a full atlas",
  );
  const firstLoad = load("pixel");
  assert.equal(load("pixel"), firstLoad);
  assert.equal(await firstLoad, false);
  assert.equal(forStyle("pixel"), unpublished, "Failed decoding cannot publish partial artwork");
  assert.equal(art.find(unpublished, "player_thorncaller"), undefined);
  assert.equal(requested.length, 2);
  failNative = false;
  assert.equal(await load("pixel"), true);
  assert.notEqual(forStyle("pixel"), unpublished);
  assert.equal(Object.isFrozen(forStyle("pixel")), true);
  assert.equal(Object.isFrozen(art.find(forStyle("pixel"), "player_thorncaller")), true);
  assert.equal(requested.length, 4);
  assert.equal(await load("pixel"), true);
  assert.equal(requested.length, 4);
  assert.equal(await load("normal"), true);
  assert.equal(await load("low-poly"), true);
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
  return imageData(encoded);
}

function imageData(encoded: string): ReturnType<typeof extraction> {
  return JSON.parse(Buffer.from(encoded, "base64").toString());
}

const frameSources = Object.fromEntries(
  Object.keys(pixelFrames).map((key) => [
    key,
    key.startsWith("terrain_") ? pixelSources.terrain : pixelSources.units,
  ]),
);
test("Every pixel frame is extracted into a standalone unsmoothed image", async () => {
  await load("pixel");
  for (const [frame, source] of Object.entries(frameSources)) {
    const [x, y, width, height] = pixelFrames[frame as keyof typeof pixelFrames];
    assert([x, y, width, height].every(Number.isFinite));
    assert.equal(width, frame.startsWith("terrain_") ? 48 : 64);
    assert.equal(height, width);
    assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
    const isolated = imageData(art.find(forStyle("pixel"), frame)!.url.split(",")[1]!);
    assert(isolated.source.includes(`${source.filename}?v=${source.revision}`));
    assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    assert.equal(isolated.width, width);
    assert.equal(isolated.height, height);
    assert.equal(isolated.smoothing, false);
  }
});
test("Artwork aliases route game units to their isolated frames", async () => {
  await load("pixel");
  for (const [type, model] of Object.entries(pixelUnitModels)) {
    for (const side of ["player", "enemy"] as const) {
      const result = extraction(sprite(forStyle("pixel"), "units", type, 0, 0, 64, 64, "", side));
      const crop = pixelFrames[`${side}_${model}` as keyof typeof pixelFrames];
      assert.deepEqual(result.coordinates, [...crop, 0, 0, crop[2], crop[3]]);
    }
  }
});
for (const style of ["normal", "low-poly"] as const) {
  test("Complete styled roster uses its own atlas without pixel fallbacks — " + style, async () => {
    await load(style);
    const source = styledSheets[style].source;
    const frameName = (name: string) => (name === "bonearcher" ? "bone_archer" : name);
    const expected = Object.keys(TYPES).flatMap((type) =>
      ["enemy", "player"].map((side) => `${side}_${frameName(type)}`),
    );
    for (const key of [...expected, "portrait_worker"]) {
      const frame = art.find(forStyle(style), key);
      assert(frame, `${style} is missing ${key}`);
      assert.equal(frame.pixel, false, `${style}/${key} must not use pixel artwork`);
      const unit = key.replace(/^(player|enemy)_/, "");
      const rendered =
        key === "portrait_worker"
          ? art.portrait(forStyle(style), "worker", 0, 0, 56, 56)
          : sprite(
              forStyle(style),
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
      await load(style);
      const sheet = styledTerrainSheets[style];
      for (const name of ["plains", "water", "ford", "bridge"] as const) {
        const rendered = sprite(forStyle(style), "terrain", name, 0, 0, 70, 70);
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
        assert.equal(art.color(forStyle(style), name), styledTerrainColors[style][name]);
      }
    },
  );
}
test("Tall dialogue keeps a character frame at its native 64×64 size", async () => {
  await load("pixel");
  const lark = extraction(art.portrait(forStyle("pixel"), "thorncaller", 0, 0, 90, 120));
  const [lx, ly] = pixelFrames.player_thorncaller;
  assert.deepEqual(
    lark.coordinates,
    [lx, ly, 64, 64, 0, 0, 64, 64],
    "Tall dialogue layout receives only the native character frame",
  );
  assert.equal(lark.width, 64);
  assert.equal(lark.height, 64);
});

let restoreCanvas: () => void;
beforeEach(async () => {
  vi.resetModules();
  art = await import("@frontend/Art");
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
