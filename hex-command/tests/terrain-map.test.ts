import assert from "node:assert/strict";
import { test, vi } from "vitest";
import {
  TerrainMapCache,
  terrainMapKey,
  type TerrainMapPlan,
} from "../src/presentation/terrain-map";
import { mockSpriteCanvas } from "./helpers";
import type { Artwork } from "../src/presentation/art";

test("Terrain cache keys ignore ownership but include every terrain input", () => {
  const tile = { id: "0,0", col: 0, row: 0, terrain: "road" as const, owner: null };
  const game = { mapId: "mirror" as const, cols: 1, rows: 1, tiles: [tile] };
  const key = terrainMapKey(game);
  assert.equal(terrainMapKey({ ...game, tiles: [{ ...tile, owner: "player" }] }), key);
  for (const changed of [
    { ...game, mapId: "river" as const },
    { ...game, cols: 2 },
    { ...game, rows: 2 },
    { ...game, tiles: [{ ...tile, col: 1 }] },
    { ...game, tiles: [{ ...tile, row: 1 }] },
    { ...game, tiles: [{ ...tile, id: "1,0" }] },
    { ...game, tiles: [{ ...tile, terrain: "forest" as const }] },
    { ...game, tiles: [{ ...tile, floor: "snow-forest" }] },
    { ...game, tiles: [{ ...tile, connections: ["e" as const, "w" as const] }] },
    { ...game, tiles: [{ ...tile, deadEnds: ["e" as const] }] },
  ])
    assert.notEqual(terrainMapKey(changed), key);
});

test("Map baking reuses one native bitmap, retries failure and keeps overlapping requests separate", async () => {
  class ControlledImage {
    static requests: ControlledImage[] = [];
    onload?: () => void;
    onerror?: () => void;
    src = "";
    constructor() {
      ControlledImage.requests.push(this);
    }
  }
  vi.stubGlobal("Image", ControlledImage);
  const restoreCanvas = mockSpriteCanvas();
  try {
    const artwork: Artwork = { frames: {} };
    const plan: TerrainMapPlan = {
      width: 140,
      height: 70,
      pixelScale: 64 / 70,
      tiles: [
        {
          points: "0,0 64,0 32,64",
          clipId: "sample",
          fill: "#123456",
          terrain: "forest",
          terrainArtwork: '<svg><image href="data:image/png;base64,AA=="/></svg>',
        },
      ],
    };
    const cache = new TerrainMapCache();
    const first = cache.prepare("first", artwork, plan);
    assert.equal(cache.prepare("first", artwork, plan), first);
    assert.equal(ControlledImage.requests.length, 1);
    const second = cache.prepare("second", artwork, plan);
    ControlledImage.requests[0]!.onload?.();
    const baked = (await first)!;
    assert(Object.isFrozen(baked));
    assert.equal(baked.width, 128);
    assert.equal(baked.height, 64);
    assert.equal(baked.displayWidth, 140);
    assert.equal(baked.displayHeight, 70);
    const painted = JSON.parse(Buffer.from(baked.url.split(",")[1]!, "base64").toString());
    assert.equal(painted.smoothing, false);
    const svg = decodeURIComponent(painted.source.split(",").slice(1).join(","));
    assert(svg.includes('clip-path="url(#sample)"'));
    assert(svg.includes(plan.tiles[0]!.terrainArtwork));
    assert(svg.includes('viewBox="0 0 140 70"'));
    assert.equal(
      cache.prepare("second", artwork, plan),
      second,
      "An older completion cannot replace the newer entry",
    );
    ControlledImage.requests[1]!.onerror?.();
    assert.equal(await second, undefined);
    const retry = cache.prepare("second", artwork, plan);
    assert.notEqual(retry, second);
    ControlledImage.requests[2]!.onload?.();
    await retry;
    assert.equal(cache.prepare("second", artwork, plan), retry);
    const newArtwork = cache.prepare("second", { frames: {} }, plan);
    assert.notEqual(newArtwork, retry);
    ControlledImage.requests[3]!.onload?.();
    await newArtwork;
  } finally {
    restoreCanvas();
    vi.unstubAllGlobals();
  }
});
