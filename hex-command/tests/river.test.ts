import type { Tile, Side, Dialogue } from "../rust/pkg/hex_command";
import { test, afterEach, vi } from "vitest";
import { required, storageWithValue } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.ts";
import { loadBattle, saveBattle } from "./legacy-save.ts";
import oldLayout from "../src/save/migration/legacy/data/river-v1.json";
import oldRoster from "../src/save/migration/legacy/data/river-v1-roster.json";

test("River layout, checkpoint and missing map migration round-trip", async () => {
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
  const river = new Game("river");
  assert.equal(river.tiles.length, 135);
  assert.equal(river.tiles.filter((tile) => tile.terrain === "bridge").length, 1);
  assert.equal(river.snapshot().villages.length, 6);
  river.move("u3", "5,3");
  saveBattle(river, true);
  assert.deepEqual(loadBattle().game.snapshot(), river.snapshot());
  assert.equal(loadBattle().pendingEnemy, true);
  assert.equal(required(loadBattle().game.tile("7,3")).terrain, "bridge");
  const legacy = new Game();
  legacy.move("u3", "3,1");
  saveBattle(legacy);
  const save = JSON.parse(stored) as {
    state: { mapId?: string; mapRevision?: number };
  };
  delete save.state.mapId;
  stored = JSON.stringify(save);
  assert.deepEqual(loadBattle().game.snapshot(), legacy.snapshot());
  save.state.mapId = "unknown";
  stored = JSON.stringify(save);
  assert.equal(loadBattle().game.mapId, "mirror");
});

// Published revision-1 river saves keep their original crossings and progress.
test("Revision-one river saves retain their crossings and progress", async () => {
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
  const oldRiver = new Game("river");
  oldRiver.tiles = structuredClone(oldLayout.tiles) as Tile[];
  oldRiver.units = [];
  oldRiver.nextId = 1;
  for (const unit of oldRoster.units)
    oldRiver.addUnit(unit.type, unit.side as Side, `${unit.col},${unit.row}`);
  oldRiver.move("u3", "4,4");
  required(oldRiver.unit("u2")).tile = "7,1";
  saveBattle(oldRiver);
  const oldSave = JSON.parse(stored) as { state: { mapRevision?: number } };
  delete oldSave.state.mapRevision;
  stored = JSON.stringify(oldSave);
  assert.deepEqual(loadBattle().game.snapshot(), oldRiver.snapshot());
  assert.equal(required(loadBattle().game.tile("7,1")).terrain, "bridge");
});
afterEach(() => {
  vi.unstubAllGlobals();
});
