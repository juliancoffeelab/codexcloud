import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "./game-fixture";
let art: typeof import("../src/presentation/art");
import {
  pixelFrames,
  pixelSources,
  pixelUnitModels,
  pixelTerrainColors,
} from "../src/presentation/pixel-art";
import { Game } from "./game-fixture";
import { createGameView } from "./fixture-view";
import { battlefieldTiles, terrainArtNames } from "../src/ui/battlefield-view";
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
    art.artSprite(art.currentArtwork(), "units", "thorncaller", 0, 0, 90, 120),
    "",
    "An unprepared frame must never fall back to a full atlas",
  );
  const firstLoad = art.loadArtwork();
  assert.equal(art.loadArtwork(), firstLoad);
  assert.equal(await firstLoad, false);
  assert.equal(requested.length, 2);
  failNative = false;
  assert.equal(await art.loadArtwork(), true);
  assert.equal(requested.length, 4);
  assert.equal(await art.loadArtwork(), true);
  assert.equal(requested.length, 4);
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
  await art.loadArtwork();
  for (const [frame, source] of Object.entries(frameSources)) {
    const [x, y, width, height] = pixelFrames[frame as keyof typeof pixelFrames];
    assert([x, y, width, height].every(Number.isFinite));
    assert.equal(width, 64);
    assert.equal(height, width);
    assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
    const isolated = imageData(art.currentArtwork().frames[frame]!.url.split(",")[1]!);
    assert(isolated.source.includes(`${source.filename}?v=${source.revision}`));
    assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    assert.equal(isolated.width, width);
    assert.equal(isolated.height, height);
    assert.equal(isolated.smoothing, false);
  }
});
test("Game units and artwork aliases route to their isolated frames", async () => {
  await art.loadArtwork();
  for (const type of Object.keys(TYPES)) {
    const model = pixelUnitModels[type] ?? (type === "bonearcher" ? "bone_archer" : type);
    for (const side of ["player", "enemy"] as const) {
      const result = extraction(
        art.artSprite(art.currentArtwork(), "units", type, 0, 0, 64, 64, "", side),
      );
      const crop = pixelFrames[`${side}_${model}` as keyof typeof pixelFrames];
      assert.deepEqual(result.coordinates, [...crop, 0, 0, crop[2], crop[3]]);
    }
  }
});
test("Static terrain frames supply matching edge fills without procedural overlays", async () => {
  await art.loadArtwork();
  const artwork = art.currentArtwork();
  const tiles = battlefieldTiles(createGameView(new Game("developer")), null, artwork, null);
  for (const name of ["plains", "water", "ford", "bridge"] as const) {
    const rendered = art.artSprite(artwork, "terrain", name, 0, 0, 70, 70);
    assert.equal((rendered.match(/<image /g) ?? []).length, 1);
    assert(!rendered.includes("<path"), "The bridge is a complete static sprite");
    const isolated = extraction(rendered);
    const [x, y, width, height] = pixelFrames[`terrain_${name}`];
    assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    assert.equal(isolated.smoothing, false);
    assert.equal(tiles.find((tile) => tile.terrain === name)?.fill, pixelTerrainColors[name]);
  }
});
test("Tall dialogue keeps a character frame at its native 64×64 size", async () => {
  await art.loadArtwork();
  const lark = extraction(art.dialoguePortrait(art.currentArtwork(), "thorncaller", 0, 0, 90, 120));
  const [lx, ly] = pixelFrames.player_thorncaller;
  assert.deepEqual(
    lark.coordinates,
    [lx, ly, 64, 64, 0, 0, 64, 64],
    "Tall dialogue layout receives only the native character frame",
  );
  assert.equal(lark.width, 64);
  assert.equal(lark.height, 64);
});

test("Route connections select native crossings and transparent road overlays", async () => {
  await art.loadArtwork();
  assert.deepEqual(terrainArtNames({ terrain: "bridge" }), ["bridge"]);
  assert.deepEqual(terrainArtNames({ terrain: "ford", connections: ["se", "nw"] }), ["ford_nw-se"]);
  assert.deepEqual(terrainArtNames({ terrain: "bridge", connections: ["sw", "ne"] }), [
    "bridge_ne-sw",
  ]);
  assert.deepEqual(terrainArtNames({ terrain: "plains", connections: ["w", "e", "se"] }), [
    "plains",
    "road_11",
  ]);
  for (const name of ["bridge_ne-sw", "ford_nw-se", "road_11"]) {
    const frame = extraction(art.artSprite(art.currentArtwork(), "terrain", name, 0, 0, 70, 70));
    assert.equal(frame.width, 64);
    assert.equal(frame.height, 64);
    assert.equal(frame.smoothing, false);
  }
});

test("Terrain, troops and commanders share one battlefield pixel scale", async () => {
  assert.equal(await art.loadArtwork(), true);
  const tiles = battlefieldTiles(
    createGameView(new Game("developer")),
    null,
    art.currentArtwork(),
    null,
  );
  assert(tiles.some((tile) => tile.occupant));
  for (const tile of tiles) {
    if (!tile.occupant) continue;
    const dimensions = (sprite: string) =>
      ["width", "height"].map((attribute) => {
        const match = new RegExp(`${attribute}="([^"]+)"`).exec(sprite);
        assert(match);
        return match[1];
      });
    assert.deepEqual(dimensions(tile.occupant.artwork), dimensions(tile.terrainArtwork));
  }
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
