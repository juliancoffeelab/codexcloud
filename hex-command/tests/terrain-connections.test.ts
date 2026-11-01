import assert from "node:assert/strict";
import { test } from "vitest";
import { MAPS } from "../src/game/maps";
import { pixelSprites, pixelFloorColors } from "../src/presentation/pixel-art";
import {
  TERRAIN_DIRECTIONS,
  CONNECTED_TERRAINS,
  terrainArtNames,
  validTerrainConnections,
} from "../src/presentation/terrain-connections";
import castleBells from "../pixel/art/terrain/castle-bell/variants.json";
import castleWalls from "../pixel/art/terrain/castle-wall/variants.json";
import castleGates from "../pixel/art/terrain/castle-gate/variants.json";
import roadDeadEnds from "../pixel/art/terrain/road/dead-ends.json";
import { createTestFactory } from "./native-fixture";
import { nativeValue } from "../src/game/live-game";
import type { TerrainOverride } from "../src/game/types";

const directions = (mask: number) => TERRAIN_DIRECTIONS.filter((_, bit) => mask & (1 << bit));
function requireFrames(tile: Parameters<typeof terrainArtNames>[0] & { floor?: string }) {
  const name = terrainArtNames(tile)[0]!;
  const frame = `terrain_${name === "grove" ? "sacred_grove" : name}`;
  assert(Object.hasOwn(pixelSprites, frame), `Missing ${frame}`);
  for (const floor of Object.keys(pixelFloorColors)) {
    const key = `${frame}__${floor}`;
    assert(Object.hasOwn(pixelSprites, key), `Missing ${key}`);
  }
}

test("Every supported terrain connection and floor has a native frame", () => {
  for (const terrain of CONNECTED_TERRAINS) {
    assert(!validTerrainConnections({ terrain }));
    assert.throws(() => terrainArtNames({ terrain }), /missing terrain connections/);
    for (let mask = 0; mask < 64; mask++) {
      const tile = { terrain, connections: directions(mask) };
      if (validTerrainConnections(tile)) requireFrames(tile);
    }
  }
  for (const [terrain, variants] of [
    ["castle-wall", castleWalls],
    ["castle-gate", castleGates],
    ["castle-bell", castleBells],
  ] as const) {
    for (const variant of variants)
      requireFrames({
        terrain,
        connections: directions(variant.mask),
        wallCorners: variant.wallCorners,
      });
  }
  for (const [exits, ends] of roadDeadEnds) {
    const tile = {
      terrain: "road" as const,
      connections: directions(exits!),
      deadEnds: directions(ends!),
    };
    assert(validTerrainConnections(tile));
    requireFrames(tile);
  }
});

test("Every authored map calls supported terrain configurations with available artwork", () => {
  for (const map of Object.values(MAPS)) {
    for (const tile of map.terrain) {
      assert(validTerrainConnections(tile), `${map.id} at ${tile.col},${tile.row}`);
      if (tile.floor !== undefined) assert(Object.hasOwn(pixelFloorColors, tile.floor));
      requireFrames(tile);
    }
  }
});

test("Native saves reject unregistered road caps and preserve registered ones", () => {
  const factory = createTestFactory();
  const game = nativeValue(factory.create({ kind: "scenario", map: "mirror" }, 17));
  const checkpoint = nativeValue(game.save());
  for (const [exits, ends] of [...roadDeadEnds, [11, 16], [34, 16], [34, 0], [34, 2]]) {
    const saved = structuredClone(checkpoint);
    const tile = saved.state.tiles.find((tile) => tile.terrain === "plains")!;
    Object.assign(tile, {
      terrain: "road",
      connections: directions(exits!),
      deadEnds: directions(ends!),
    });
    const accepted = validTerrainConnections(tile as TerrainOverride);
    const restored = factory.restore(saved);
    assert.equal(restored.ok, accepted, `${exits}-end-${ends}`);
    if (restored.ok) {
      assert.deepEqual(nativeValue(restored.value.save()).state.tiles, saved.state.tiles);
      restored.value.dispose();
    }
  }
  assert(!validTerrainConnections({ terrain: "road", connections: ["e", "e"] }));
  assert(!validTerrainConnections({ terrain: "village", connections: directions(63) }));
  assert(!validTerrainConnections({ terrain: "ford", connections: ["e", "se"] }));
  assert(!validTerrainConnections({ terrain: "keep", connections: ["e"] }));
  game.dispose();
});
