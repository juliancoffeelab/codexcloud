import { test, afterEach, vi } from "vitest";
import { required, storageWithValue } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.ts";
import { loadBattle, saveBattle } from "./legacy-save.ts";

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
  const oldRiver = new Game("river", 1);
  oldRiver.move("u3", "4,4");
  required(oldRiver.unit("u2")).tile = "7,1";
  saveBattle(oldRiver);
  const oldSave = JSON.parse(stored) as { state: { mapRevision?: number } };
  delete oldSave.state.mapRevision;
  stored = JSON.stringify(oldSave);
  assert.deepEqual(loadBattle().game.snapshot(), oldRiver.snapshot());
  assert.equal(required(loadBattle().game.tile("7,1")).terrain, "bridge");
});
test("Ford entry costs constrain heavy troops", async () => {
  const fordGame = new Game("river");
  required(fordGame.unit("u2")).tile = "4,7";
  assert.equal(fordGame.reachable("u2").get("5,7"), 3);
  assert.equal(fordGame.reachable("u2").has("6,7"), false);
  assert.equal(fordGame.reachable("u2").has("9,7"), false);
  assert.equal(fordGame.cost(required(fordGame.unit("u3")), required(fordGame.tile("5,7"))), 2);
  fordGame.move("u2", "5,7");
  assert.equal(required(fordGame.unit("u2")).tile, "5,7");
  for (const type of ["commander", "soldier", "archer", "scout"] as const) {
    assert.equal(
      fordGame.cost({ type }, required(fordGame.tile("5,7"))),
      ["soldier", "commander"].includes(type) ? 3 : 2,
    );
  }
});
// Bridge hexes permit ordinary movement while adjacent water stays forbidden.
test("Bridges allow ordinary movement and adjacent water remains blocked", async () => {
  const river = new Game("river");
  required(river.unit("u2")).tile = "6,3";
  assert.ok(river.reachable("u2").has("7,3"));
  assert.ok(!river.reachable("u2").has("8,2"));
  river.move("u2", "7,3");
  assert.equal(required(river.unit("u2")).tile, "7,3");
});
test("Exposure also applies to retaliation", async () => {
  const exposed = new Game("river");
  required(exposed.tile("6,7")).terrain = "plains";
  exposed.units = [];
  const wader = exposed.addUnit("soldier", "player", "5,7");
  const defender = exposed.addUnit("soldier", "enemy", "6,7");
  assert.deepEqual(exposed.preview(wader, defender), {
    damage: 4,
    retaliation: 5,
  });
  exposed.attack(wader.id, defender.id);
  assert.equal(wader.hp, 7);
  assert.equal(defender.hp, 8);
});

afterEach(() => {
  vi.unstubAllGlobals();
});
