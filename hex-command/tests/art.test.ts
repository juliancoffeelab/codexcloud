import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { TYPES } from "./game-fixture";
let art: typeof import("../src/presentation/art");
import {
  pixelFrames,
  pixelSources,
  pixelUnitModels,
  pixelDefaultFloors,
  pixelFloorColors,
  pixelTerrainLayouts,
} from "../src/presentation/pixel-art";
import { Game } from "./game-fixture";
import { createGameView } from "./fixture-view";
import { battlefieldTiles, terrainArtNames, terrainProps } from "../src/ui/battlefield-view";
import { terrainLayout } from "../src/presentation/terrain-layout";
import { hexPoints, tileCenter } from "../src/ui/geometry";
import { mockSpriteCanvas } from "./helpers";

const requested: string[] = [];

test("Adjacent hexes share both edge vertices without gaps on either row parity", () => {
  const vertices = (col: number, row: number) => {
    const center = tileCenter({ col, row });
    return hexPoints(center.x, center.y)
      .split(" ")
      .map((point) => point.split(",").map(Number));
  };
  for (const row of [4, 5]) {
    const odd = row % 2;
    const origin = vertices(4, row);
    for (const [col, nextRow] of [
      [5, row],
      [4 + odd, row + 1],
      [3 + odd, row + 1],
      [3, row],
      [3 + odd, row - 1],
      [4 + odd, row - 1],
    ]) {
      const neighbor = vertices(col!, nextRow!);
      assert.equal(
        origin.filter(([x, y]) =>
          neighbor.some(([nx, ny]) => Math.hypot(x! - nx!, y! - ny!) < 1e-8),
        ).length,
        2,
      );
    }
  }
});
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
  assert.equal(requested.length, 1);
  failNative = false;
  assert.equal(await art.loadArtwork(), true);
  assert.equal(requested.length, 2);
  assert.equal(await art.loadArtwork(), true);
  assert.equal(requested.length, 2);
});

function extraction(sprite: string): {
  source: string;
  coordinates: number[];
  width: number;
  height: number;
  smoothing: boolean;
  draws: { source: string; coordinates: number[] }[];
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
  const { prepareSpriteImages } = await import("../src/presentation/sprite-images");
  const definitions = art.artworkFrames();
  await prepareSpriteImages(Object.values(definitions));
  for (const [frame, source] of Object.entries(frameSources)) {
    const [x, y, width, height] = pixelFrames[frame as keyof typeof pixelFrames];
    assert([x, y, width, height].every(Number.isFinite));
    assert.equal(width, 64);
    assert.equal(height, width);
    assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
    const { spriteImage } = await import("../src/presentation/sprite-images");
    const isolated = imageData(spriteImage(definitions[frame]!)!.url.split(",")[1]!);
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

test("Compositions draw native layers in order, publish atomically and reuse cached images", async () => {
  const { prepareSpriteImages, spriteImage } = await import("../src/presentation/sprite-images");
  const frame = {
    source: "base.png",
    crop: [0, 0, 64, 64] as const,
    layers: [
      { source: "units-pixel.png", crop: [64, 0, 64, 64] as const, x: 3, y: 7 },
      { source: "base.png", crop: [0, 64, 64, 64] as const, x: -2, y: 11 },
    ],
  };
  failNative = true;
  assert.equal(await prepareSpriteImages([frame]), false);
  assert.equal(spriteImage(frame), undefined);
  failNative = false;
  assert.equal(await prepareSpriteImages([frame]), true);
  const ready = spriteImage(frame)!;
  const image = imageData(ready.url.split(",")[1]!);
  assert.equal(image.width, 64);
  assert.equal(image.height, 64);
  assert.equal(image.smoothing, false);
  assert.deepEqual(image.draws, [
    { source: "base.png", coordinates: [0, 0, 64, 64, 0, 0, 64, 64] },
    { source: "units-pixel.png", coordinates: [64, 0, 64, 64, 3, 7, 64, 64] },
    { source: "base.png", coordinates: [0, 64, 64, 64, -2, 11, 64, 64] },
  ]);
  const requests = requested.length;
  assert.equal(await prepareSpriteImages([frame]), true);
  assert.equal(spriteImage(frame), ready);
  assert.equal(requested.length, requests);
});

test("Static terrain frames supply matching edge fills without procedural overlays", async () => {
  await art.loadArtwork();
  const game = new Game("developer");
  await art.loadTerrainArtwork(game.tiles, game.mapId);

  const artwork = art.currentArtwork();
  const tiles = battlefieldTiles(createGameView(game), null, artwork, null);
  for (const name of ["plains", "water", "ford", "bridge"] as const) {
    const tile = game.tiles.find((tile) => tile.terrain === name)!;
    const rendered = art.artSprite(
      artwork,
      "terrain",
      terrainLayout(tile, game.mapId).name,
      0,
      0,
      70,
      70,
    );
    assert.equal((rendered.match(/<image /g) ?? []).length, 1);
    assert(!rendered.includes("<path"), "The bridge is a complete static sprite");
    const isolated = extraction(rendered);
    const [x, y, width, height] = pixelFrames[`terrain_${name}`];
    assert.deepEqual(isolated.coordinates, [x, y, width, height, 0, 0, width, height]);
    assert.equal(isolated.smoothing, false);
    assert.equal(
      tiles.find((tile) => tile.terrain === name)?.fill,
      pixelFloorColors[pixelDefaultFloors[name]],
    );
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

test("Map floors choose independent palettes for directional terrain and matching hex fills", async () => {
  await art.loadArtwork();
  const game = new Game("developer");
  const road = game.tiles.find((tile) => tile.terrain === "plains")!;
  road.terrain = "road";
  road.connections = ["e", "w", "se"];
  road.floor = "green-forest";
  const forest = game.tiles.find((tile) => tile.terrain === "forest")!;
  forest.floor = "snow-forest";
  await art.loadTerrainArtwork(game.tiles, game.mapId);
  const rendered = battlefieldTiles(createGameView(game), null, art.currentArtwork(), null);
  for (const [tile, frame] of [
    [road, "terrain_road_11__green-forest"],
    [forest, "terrain_forest__snow-forest"],
  ] as const) {
    const result = rendered.find((candidate) => candidate.id === tile.id)!;
    assert.equal(result.fill, pixelFloorColors[tile.floor!]);
    const crop = pixelFrames[frame];
    assert.deepEqual(imageData(result.terrainFrame!.url.split(",")[1]!).coordinates, [
      ...crop,
      0,
      0,
      64,
      64,
    ]);
    assert.equal(result.floor, tile.floor);
    if (tile.terrain === "forest") {
      const props = terrainProps(tile, game.mapId);
      const image = imageData(result.terrainFrame!.url.split(",")[1]!);
      assert.equal(image.draws.length, props.length + 1);
      props.forEach((prop, index) => {
        const crop = pixelFrames[`terrain_${prop.name}__${tile.floor}` as keyof typeof pixelFrames];
        assert.deepEqual(image.draws[index + 1]!.coordinates, [...crop, prop.x, prop.y, 64, 64]);
      });
    }
  }
});

test("Prop arrangements stay stable per map tile and vary between locations", () => {
  for (const terrain of ["forest", "hill", "plains", "ford"] as const) {
    const choices = new Set<string>();
    for (let col = 0; col < 16; col++) {
      const tile = { terrain, id: `${col},3` };
      const first = terrainProps(tile, "sample");
      assert.equal(terrainProps(tile, "sample"), first);
      choices.add(JSON.stringify(first));
    }
    assert(choices.size > 1);
  }
  assert.deepEqual(terrainProps({ terrain: "road", id: "1,1" }, "sample"), []);
});

test("Directional props select arrangements for the same orientation as the base frame", () => {
  for (const connections of [
    ["e", "w"],
    ["nw", "se"],
    ["ne", "sw"],
  ] as const) {
    const tile = { terrain: "ford" as const, id: "4,2", connections: [...connections] };
    const frame = terrainArtNames(tile)[0]!;
    const props = terrainProps(tile, "sample");
    assert(pixelTerrainLayouts[frame]!.includes(props));
    assert.equal(terrainProps(tile, "sample"), props);
  }
});

test("Route connections select one complete native terrain frame", async () => {
  await art.loadArtwork();
  assert.deepEqual(terrainArtNames({ terrain: "bridge" }), ["bridge"]);
  assert.deepEqual(terrainArtNames({ terrain: "ford", connections: ["se", "nw"] }), ["ford_nw-se"]);
  assert.deepEqual(terrainArtNames({ terrain: "bridge", connections: ["sw", "ne"] }), [
    "bridge_ne-sw",
  ]);
  assert.deepEqual(terrainArtNames({ terrain: "road", connections: ["w", "e", "se"] }), [
    "road_11",
  ]);
  assert.deepEqual(terrainArtNames({ terrain: "road", connections: ["w", "e"] }), ["road"]);
  assert.deepEqual(terrainArtNames({ terrain: "plains" }), ["plains"]);
  assert.deepEqual(terrainArtNames({ terrain: "village" }), ["village_0"]);
  assert.deepEqual(terrainArtNames({ terrain: "village", connections: ["w", "e"] }), ["village"]);
  assert.deepEqual(terrainArtNames({ terrain: "village", connections: ["w", "e", "se"] }), [
    "village_11",
  ]);
  assert.deepEqual(terrainArtNames({ terrain: "keep" }), ["keep"]);
  assert.deepEqual(
    terrainArtNames({ terrain: "road", connections: ["se", "ne"], deadEnds: ["w"] }),
    ["road_34-end-8"],
  );
  const tiles = [
    { terrain: "bridge" as const, connections: ["ne", "sw"] as const },
    { terrain: "ford" as const, connections: ["nw", "se"] as const },
    { terrain: "road" as const, connections: ["e", "w", "se"] as const },
    { terrain: "village" as const },
    { terrain: "village" as const, connections: ["e", "w", "se"] as const },
    { terrain: "road" as const, connections: ["se", "ne"] as const, deadEnds: ["w"] as const },
  ].map((tile) => ({ id: "1,1", col: 1, row: 1, owner: null, ...tile }));
  await art.loadTerrainArtwork(tiles, "preview");
  for (const tile of tiles) {
    const frame = extraction(
      art.artSprite(
        art.currentArtwork(),
        "terrain",
        terrainLayout(tile, "preview").name,
        0,
        0,
        70,
        70,
      ),
    );
    assert.equal(frame.width, 64);
    assert.equal(frame.height, 64);
    assert.equal(frame.smoothing, false);
  }
});

test("Terrain, troops and commanders share one battlefield pixel scale", async () => {
  assert.equal(await art.loadArtwork(), true);
  const game = new Game("developer");
  await art.loadTerrainArtwork(game.tiles, game.mapId);
  const tiles = battlefieldTiles(createGameView(game), null, art.currentArtwork(), null);
  assert(tiles.some((tile) => tile.occupant));
  for (const tile of tiles) {
    if (!tile.occupant) continue;
    assert.equal(tile.occupant.frame?.width, 64);
    assert.equal(tile.occupant.frame?.height, 64);
    assert.equal(tile.terrainFrame?.width, 64);
    assert.equal(tile.terrainFrame?.height, 64);
    assert.equal(tile.occupant.spriteSize, tile.spriteSize);
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

test("Startup skips terrain; overlapping tile requests prepare only their frames without mutating old snapshots", async () => {
  await art.loadArtwork();
  const units = art.currentArtwork();
  assert(!Object.keys(units.frames).some((name) => name.startsWith("terrain_")));
  assert(requested.every((url) => url.includes("units-pixel.png")));
  const tile = {
    id: "2,3",
    col: 2,
    row: 3,
    terrain: "forest" as const,
    floor: "snow-forest",
    owner: null,
  };
  const first = art.loadTerrainArtwork([tile], "test");
  const second = art.loadTerrainArtwork([tile, tile], "test");
  assert.equal(await first, true);
  assert.equal(await second, true);
  const ready = art.currentArtwork();
  assert.equal(Object.keys(ready.frames).length, Object.keys(units.frames).length + 1);
  assert.equal(Object.keys(units.frames).length, Object.keys(ready.frames).length - 1);
  assert(Object.isFrozen(ready.frames));
  assert(art.artFrame(ready, "terrain", terrainLayout(tile, "test").name, "player", tile.floor));
  assert.equal(requested.filter((url) => url.includes("terrain-pixel.png")).length, 1);
  assert.equal(await art.loadTerrainArtwork([tile], "test"), true);
  assert.equal(art.currentArtwork(), ready);
});
