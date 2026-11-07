import { test, afterEach, vi } from "vitest";
import { required, storageWithValue } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.ts";
import { loadBattle, saveBattle } from "./legacy-save.ts";

// Existing v1 saves store battle progress rather than terrain. Updating the map
// must preserve an ongoing battle, including units on changed ground.
test("Saved armies retain progress while terrain migrates", async () => {
  const game = new Game();
  let stored = "";
  vi.stubGlobal(
    "localStorage",
    storageWithValue(
      () => stored,
      (value) => {
        stored = value;
      },
    ),
  );
  required(game.unit("u3")).tile = { col: 4, row: 2 }; // Former forest, now plains.
  required(game.unit("u6")).tile = { col: 6, row: 0 }; // Former plains, now hills.
  required(game.unit("u3")).hp = 5;
  required(game.unit("u3")).moved = true;
  required(game.tile("3,1")).owner = "player";
  game.round = 4;
  game.gold.player = 23;
  game.record("Ongoing battle.");
  await saveBattle(game, true);
  const loaded = await loadBattle();
  assert.deepEqual(loaded.game.snapshot(), game.snapshot());
  assert.equal(loaded.pendingEnemy, true);
  assert.equal(loaded.game.nextId, game.nextId);
  assert.deepEqual(loaded.game.log, game.log);
  assert.equal(required(loaded.game.tile("4,2")).terrain, "plains");
  assert.equal(required(loaded.game.tile("6,0")).terrain, "hill");
});

afterEach(() => {
  vi.unstubAllGlobals();
});
