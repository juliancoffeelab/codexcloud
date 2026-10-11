import assert from "node:assert/strict";
import { test } from "vitest";
import * as Plan from "@frontend/Terrain_plan";
import * as Art from "@frontend/Art";
import { pixelFloorColors } from "../src/presentation/pixel-art";
import roadDeadEnds from "../pixel/art/terrain/road/dead-ends.json";
import { Native, Catalog, factory, catalog, createBattle } from "./native-fixture";
import { required } from "./helpers";
import type { SavedGame, Tile, HexDirection } from "../rust/pkg/hex_command";

const order: HexDirection[] = ["e", "se", "sw", "w", "nw", "ne"];
const directions = (mask: number) => order.filter((_, bit) => mask & (1 << bit));
function requireFrames(tile: Native.Immutable<Tile>, mapId: string) {
  const name = Plan.layout_name(tile, mapId);
  assert(Art.terrain_frame(Art.current(), undefined, name), `Missing ${name}`);
  for (const floor of Object.keys(pixelFloorColors))
    assert(Art.terrain_frame(Art.current(), floor, name), `Missing ${name}/${floor}`);
}
function restore(saved: SavedGame): boolean {
  try {
    const battle = Native.restore(factory, saved);
    Native.dispose(battle);
    return true;
  } catch {
    return false;
  }
}

test("Every native-accepted terrain connection and floor has a corresponding native image", () => {
  const game = createBattle({ kind: "scenario", map: "mirror" });
  const original = structuredClone(Native.save(game)) as SavedGame;
  // Connection legality is independent of armies; keep the native validator
  // fixture focused and avoid computing hundreds of unrelated unit forecasts.
  original.state.units = [];
  const candidate = required(
    original.state.tiles.find(
      (tile) =>
        tile.terrain === "plains" && !original.state.units.some((unit) => unit.tile === tile.id),
    ),
  );
  for (const terrain of ["road", "village", "bridge", "ford"]) {
    requireFrames({ ...candidate, terrain }, original.state.mapId);
    let accepted = 0;
    for (let mask = 1; mask < 64; mask++) {
      const saved = structuredClone(original) as SavedGame;
      const tile = required(saved.state.tiles.find((tile) => tile.id === candidate.id));
      Object.assign(tile, { terrain, connections: directions(mask) });
      if (restore(saved)) {
        accepted++;
        requireFrames(tile, saved.state.mapId);
      }
    }
    assert(accepted > 0, `${terrain} must retain its authored connected configurations`);
  }
});

test("Every authored native map selects available art, including independent floors and deterministic layouts", () => {
  for (const map of Catalog.maps(catalog)) {
    for (const authored of map.terrain) {
      const tile: Native.Immutable<Tile> = {
        ...authored,
        id: `${authored.col},${authored.row}`,
        owner: null,
      };
      if (tile.floor !== undefined) assert(Object.hasOwn(pixelFloorColors, tile.floor));
      requireFrames(tile, map.id);
    }
  }
});

test("Rust saves preserve registered road caps and reject invalid connections", () => {
  const game = createBattle({ kind: "scenario", map: "mirror" });
  const original = Native.save(game);
  const selected = required(
    original.state.tiles.find(
      (tile) =>
        tile.terrain === "plains" && !original.state.units.some((unit) => unit.tile === tile.id),
    ),
  );
  for (const [exits, ends] of roadDeadEnds) {
    const saved = structuredClone(original) as SavedGame;
    const tile = required(saved.state.tiles.find((tile) => tile.id === selected.id));
    Object.assign(tile, {
      terrain: "road",
      connections: directions(required(exits)),
      deadEnds: directions(required(ends)),
    });
    const restored = Native.restore(factory, saved);
    try {
      assert.deepEqual(Native.save(restored).state.tiles, saved.state.tiles);
      requireFrames(tile, saved.state.mapId);
    } finally {
      Native.dispose(restored);
    }
  }
  const invalid: Partial<Tile>[] = [
    ...[
      [11, 16],
      [34, 16],
      [34, 0],
      [34, 2],
    ].map(([exits, ends]) => ({
      terrain: "road",
      connections: directions(required(exits)),
      deadEnds: directions(required(ends)),
    })),
    { terrain: "road", connections: ["e", "e"] },
    { terrain: "village", connections: directions(63) },
    { terrain: "ford", connections: ["e", "se"] },
    { terrain: "keep", connections: ["e"] },
  ];
  for (const override of invalid) {
    const saved = structuredClone(original) as SavedGame;
    Object.assign(required(saved.state.tiles.find((tile) => tile.id === selected.id)), override);
    assert.equal(restore(saved), false, JSON.stringify(override));
  }
});
