import assert from "node:assert/strict";
import { Game, distance } from "../src/game/engine";
import type { Tile } from "../src/game/types";
import { required } from "./helpers";

// Both row parities, edges, and original array order must match hex distance.
for (const game of [new Game(), new Game("river"), new Game("river", 1)]) {
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
    assert.equal(neighbor, game.tile(neighbor.id), "restored adjacency uses the restored board");
  }
  const neighbor = required(game.neighbors(original.id)[0]);
  neighbor.terrain = "water";
  assert.equal(game.tile(neighbor.id)?.terrain, "water", "cached neighbors retain live terrain");
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
  }
  let neighborsVisited = 0;
  const neighbors = game.neighbors.bind(game);
  game.neighbors = (id) => {
    const result = neighbors(id);
    neighborsVisited += result.length;
    return result;
  };
  const routes = game.routeDistances(unit, [goal]);
  assert.equal(routes.size, cols * rows);
  assert.equal(routes.get(unit.tile), expectedDistance);
  assert.equal(coordinateReads, 0, "routing must use cached adjacency, not distance scans");
  assert(neighborsVisited <= 6 * cols * rows);
  return neighborsVisited;
}

const small = routingProbe(15, 9);
const large = routingProbe(30, 18);
assert(large / small < 5, "four times the board must not require sixteen times the neighbors");
console.log(
  `PASS: indexed adjacency, restore, ordering and routing; ${small} → ${large} neighbors`,
);
