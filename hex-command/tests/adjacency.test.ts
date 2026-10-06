import { test } from "vitest";
import assert from "node:assert/strict";
import { Game, distance } from "../src/game/engine";
import type { Tile } from "../src/game/types";
import { required } from "./helpers";

// Both row parities, edges, and original array order must match hex distance.
for (const game of [new Game(), new Game("river"), new Game("river", 1)]) {
  test(
    "Cached adjacency preserves distance, order and restored references — " +
      JSON.stringify([game.mapId, game.mapRevision]),
    async () => {
      for (const tile of game.tiles) {
        assert.equal(game.tile(tile.id), tile);
        assert.deepEqual(
          game.neighbors(tile.id),
          game.tiles.filter((neighbor) => distance(tile, neighbor) === 1),
        );
      }
      assert.equal(game.tile(null), undefined);
      assert.equal(game.tile("missing"), undefined);
      assert.deepEqual(game.neighbors("missing"), []);

      const original = required(game.tile("2,2"));
      const expected = game.neighbors(original.id);
      game.neighbors(original.id).reverse().pop();
      assert.deepEqual(game.neighbors(original.id), expected, "queries cannot modify cached order");

      const saved = game.captureState();
      game.restoreState(saved);
      assert.notEqual(game.tile(original.id), original, "restore replaces indexed tile references");
      for (const neighbor of game.neighbors(original.id)) {
        assert.equal(
          neighbor,
          game.tile(neighbor.id),
          "restored adjacency uses the restored board",
        );
      }
      const neighbor = required(game.neighbors(original.id)[0]);
      neighbor.terrain = "water";
      assert.equal(
        game.tile(neighbor.id)?.terrain,
        "water",
        "cached neighbors retain live terrain",
      );
    },
  );
}

function routingProbe(cols: number, rows: number): number {
  const game = new Game("river");
  const state = game.captureState();
  state.cols = cols;
  state.rows = rows;
  state.units = [];
  state.tiles = Array.from({ length: cols * rows }, (_, index): Tile => {
    const col = index % cols;
    const row = Math.floor(index / cols);
    return { id: `${col},${row}`, col, row, terrain: "plains", owner: null };
  });
  game.restoreState(state);
  const unit = game.addUnit("soldier", "enemy", "0,0");
  const goal = required(game.tile(`${cols - 1},${rows - 1}`));
  const expectedDistance = distance(required(game.tile(unit.tile)), goal);

  // Reads during routing expose a distance-based board scan without timing noise.
  let coordinateReads = 0;
  let terrainReads = 0;
  for (const tile of game.tiles) {
    for (const key of ["col", "row"] as const) {
      const value = tile[key];
      Object.defineProperty(tile, key, {
        get() {
          coordinateReads++;
          return value;
        },
      });
    }
    const terrain = tile.terrain;
    Object.defineProperty(tile, "terrain", {
      get() {
        terrainReads++;
        return terrain;
      },
    });
  }
  const routes = game.routeDistances(unit, [goal]);
  assert.equal(routes.size, cols * rows);
  assert.equal(routes.get(unit.tile), expectedDistance);
  assert.equal(coordinateReads, 0, "routing must use cached adjacency, not distance scans");
  assert(terrainReads <= 8 * cols * rows);
  return terrainReads;
}

test("Routing terrain reads scale linearly with board size", async () => {
  const small = routingProbe(15, 9);
  const large = routingProbe(30, 18);
  assert(large / small < 5, "four times the board must not require sixteen times the reads");
});

test("Routing fields respect terrain edits and callers cannot alter cached distances", () => {
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "water";
  for (const id of ["1,3", "2,3", "3,3"]) game.tile(id)!.terrain = "plains";
  const unit = game.addUnit("soldier", "player", "1,3");
  const goal = game.tile("3,3")!;
  const routes = game.routeDistances(unit, [goal]);
  assert.equal(routes.get(unit.tile), 2);
  routes.set(unit.tile, 100);
  assert.equal(game.routeDistances(unit, [goal]).get(unit.tile), 2);
  const fork = game.fork();
  fork.tile("2,3")!.terrain = "forest";
  assert.equal(fork.routeDistances(unit, [goal]).get(unit.tile), 3);
  assert.equal(game.routeDistances(unit, [goal]).get(unit.tile), 2);
  game.tile("2,3")!.terrain = "water";
  assert.equal(game.routeDistances(unit, [goal]).has(unit.tile), false);
});
