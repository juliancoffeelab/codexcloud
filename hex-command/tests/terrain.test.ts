import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, factory } from "./presentation-fixture";
import * as Save from "@frontend/Save";
import { required } from "./helpers";
test("Legacy saved armies retain progress while terrain migrates", () => {
  const game = board();
  game.edit((state) => {
    required(state.units.find((unit) => unit.id === "u3")).tile = "4,2";
    required(state.units.find((unit) => unit.id === "u6")).tile = "6,0";
    required(state.units.find((unit) => unit.id === "u3")).hp = 5;
    required(state.units.find((unit) => unit.id === "u3")).moved = true;
    required(state.tiles.find((tile) => tile.id === "3,1")).owner = "player";
    state.round = 4;
    state.gold.player = 23;
    state.log.push("Ongoing battle.");
  });
  const { tiles, cols, rows, lastCombat, ...state } = structuredClone(game.state);
  const villages = tiles
    .filter((tile) => tile.terrain === "village")
    .map(({ id, owner }) => ({ id, owner }));
  const sites = tiles
    .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
    .map(({ id, owner }) => ({ id, owner }));
  const [migrated] = Save.migrate(Catalog.raw(catalog), {
    version: 1,
    campaignDeploymentFormat: "named-waves-v1",
    pendingEnemy: true,
    state: { ...state, villages, sites },
  });
  const restored = Native.restore(factory, migrated.checkpoint);
  try {
    assert.deepEqual(Native.state(Native.view(restored)), game.state);
    assert.equal(migrated.pending_enemy, true);
    assert.equal(required(Native.tile(Native.view(restored), "4,2")).terrain, "plains");
    assert.equal(required(Native.tile(Native.view(restored), "6,0")).terrain, "hill");
  } finally {
    Native.dispose(restored);
  }
});
