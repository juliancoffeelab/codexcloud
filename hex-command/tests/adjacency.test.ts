import { test } from "vitest";
import assert from "node:assert/strict";
import { Game, distance } from "./game-fixture";
import type { Tile } from "../src/game/types";
import { required } from "./helpers";

// Both row parities, edges, and original array order must match hex distance.
for (const [map, revision] of [
  ["mirror", undefined],
  ["river", undefined],
  ["river", 1],
] as const) {
  test(
    "Cached adjacency preserves distance, order and restored references — " +
      JSON.stringify([map, revision]),
    async () => {
      const game = new Game(map, revision);
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
