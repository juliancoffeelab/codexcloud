import { cellKey } from "./hex-fixture.mjs";
import * as Store from "./storage-adapter.mjs";
import { historicalTiles } from "./legacy-save.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import { required, TestStorage } from "./helpers.mjs";
import assert from "node:assert/strict";
import { pixelSprites } from "../src/presentation/pixel-art.mjs";
const { Game, TYPES, developerFixture } = await import("./game-fixture.mjs");
const { Native, Catalog, catalog, restoreBattle } = await import("./native-fixture.mjs");
test("Map revisions preserve native history and expose one latest river-valley choice", async () => {
  const history = Catalog.raw(catalog).maps.filter((map) => map.id === "river-valley");
  assert(history.length > 1);
  const latest = history.at(-1);
  assert.deepEqual(Catalog.map(catalog, "river-valley"), latest);
  for (const maps of [
    Catalog.maps(catalog),
    Catalog.scenario_maps(catalog),
    Catalog.battle_maps(catalog),
  ])
    assert.deepEqual(
      maps.filter((map) => map.id === "river-valley"),
      [latest],
    );
  assert.equal(
    Catalog.raw(catalog).maps.filter((map) => map.id === "river-valley").length,
    history.length,
  );
});
test("Developer showcase contains both factions' playable units and expansion space", async () => {
  const game = new Game("developer");
  const actors = new Set(game.units.map((unit) => `${unit.side}:${unit.type}`));
  for (const type of Object.keys(TYPES))
    for (const side of ["player", "enemy"])
      assert(actors.has(`${side}:${type}`), `${side} ${type}`);
  const occupied = new Set(game.units.map((unit) => unit.tile));
  const freeGround = game.tiles.filter(
    (tile) => tile.terrain === "plains" && !occupied.has(tile.id),
  ).length;
  const artworkTypes = Object.keys(pixelSprites).filter((frame) =>
    frame.startsWith("player_"),
  ).length;
  assert(freeGround >= artworkTypes * 2 - game.units.length);
});
test("Developer units use ordinary movement, combat and turns", async () => {
  const developerBattle = restoreBattle(developerFixture(["soldier"]).checkpoint());
  const initial = Native.state(Native.view(developerBattle));
  const soldier = required(
    initial.units.find((unit) => unit.type === "soldier" && unit.side === "player"),
  );
  const enemySoldier = required(
    initial.units.find((unit) => unit.type === "soldier" && unit.side === "enemy"),
  );
  Native.dispatch(developerBattle, {
    kind: "move",
    unit: Native.unit_id(soldier.id),
    tile: { col: 4, row: 2 },
  });
  assert.equal(cellKey(Native.unit(Native.view(developerBattle), soldier.id).tile), "4,2");
  assert(
    Native.targets(Native.view(developerBattle), soldier.id).some(
      (unit) => unit.id === enemySoldier.id,
    ),
  );
  Native.dispatch(developerBattle, {
    kind: "attack",
    unit: Native.unit_id(soldier.id),
    target: Native.unit_id(enemySoldier.id),
    weapon: "sword",
  });
  assert(
    Native.unit(Native.view(developerBattle), enemySoldier.id).hp <
      Catalog.unit(catalog, "soldier").hp,
  );
  assert(Native.unit(Native.view(developerBattle), soldier.id).attacked);
  Native.dispatch(developerBattle, { kind: "endTurn" });
  assert.equal(Native.state(Native.view(developerBattle)).turn, "enemy");
  assert.equal(Native.state(Native.view(developerBattle)).winner, null);
});
const { loadBattle, saveBattle, SAVE_KEY, SAVE_VERSION } = await import("./legacy-save.mjs");
test("Developer saves remain noncampaign and round-trip", async () => {
  const showcase = developerFixture();
  await saveBattle(showcase);
  const loaded = await loadBattle();
  assert.deepEqual(loaded.game.snapshot(), showcase.snapshot());
  assert.equal(loaded.pendingEnemy, false);
  assert.equal(loaded.game.campaign, void 0);
});
for (const [type, destination] of [["scout", "5,1"]]) {
  test(
    "Moved developer sites persist ownership \u2014 " + JSON.stringify([type, destination]),
    async () => {
      const game = developerFixture([type]);
      const unit = required(
        game.units.find((unit2) => unit2.side === "player" && unit2.type === type),
      );
      game.move(unit.id, destination);
      assert.equal(required(game.tile(destination)).owner, "player");
      await saveBattle(game);
      assert.deepEqual((await loadBattle()).game.snapshot(), game.snapshot());
      assert.equal(required((await loadBattle()).game.tile(destination)).owner, "player");
    },
  );
}
test("Skirmish site ownership follows its terrain", async () => {
  for (const id of ["mirror", "river-valley"]) {
    const game = new Game(id);
    const sites = game.tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
      .map(({ id, owner }) => ({ id, owner }));
    assert.deepEqual(game.snapshot().sites, sites.length ? sites : undefined);
  }
});
function validDeveloperSave() {
  const showcase = developerFixture();
  const validSave = {
    version: SAVE_VERSION,
    pendingEnemy: false,
    state: { ...showcase.snapshot(), nextId: showcase.nextId, log: showcase.log },
  };
  return validSave;
}
test("Legacy developer saves gain neutral sites without resetting units", async () => {
  const validSave = validDeveloperSave();
  const legacySave = historicalTiles(validSave);
  delete legacySave.state.sites;
  await Store.store_save(JSON.stringify(legacySave));
  const legacy = (await loadBattle()).game;
  assert.equal(legacy.mapId, "developer");
  assert.deepEqual(legacy.units, validSave.state.units);
  assert(required(legacy.snapshot().sites).every((site) => site.owner === null));
});
test("Campaign saves still require site ownership", async () => {
  const campaign = Game.startCampaign("green-accord");
  const missingCampaignSites = {
    version: SAVE_VERSION,
    pendingEnemy: false,
    state: { ...campaign.snapshot(), nextId: campaign.nextId, log: campaign.log },
  };
  delete missingCampaignSites.state.sites;
  await Store.store_save(JSON.stringify(missingCampaignSites));
  assert.equal((await loadBattle()).game.mapId, "mirror", "Campaign sites remain mandatory");
});
const invalidCases = [
  [
    "duplicate unit position",
    (save) => {
      save.state.units[1].tile = save.state.units[0].tile;
    },
  ],
  [
    "unit on impassable water",
    (save) => {
      save.state.units[0].tile = { col: 3, row: 0 };
    },
  ],
  [
    "missing enemy commander",
    (save) => {
      save.state.units = save.state.units.filter(
        (unit) => !(unit.side === "enemy" && required(TYPES[unit.type]).family === "commander"),
      );
    },
  ],
  [
    "campaign attached to developer map",
    (save) => {
      save.state.campaign = Game.startCampaign("green-accord").campaign;
    },
  ],
  [
    "campaign map without campaign state",
    (save) => {
      save.state.mapId = "green-border";
    },
  ],
  [
    "missing site",
    (save) => {
      required(save.state.sites).pop();
    },
  ],
  [
    "duplicate site",
    (save) => {
      const sites = required(save.state.sites);
      sites[1].id = sites[0].id;
    },
  ],
  [
    "site on plains",
    (save) => {
      required(save.state.sites)[0].id = "2,2";
    },
  ],
  [
    "invalid site owner",
    (save) => {
      Object.assign(required(required(save.state.sites)[0]), { owner: "neutral" });
    },
  ],
];
test.each(invalidCases)("Invalid developer save is rejected: %s", async (_label, invalidate) => {
  const invalid = validDeveloperSave();
  invalidate(invalid);
  await Store.store_save(JSON.stringify(invalid));
  assert.equal((await loadBattle()).game.mapId, "mirror");
});
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});
afterEach(() => {
  vi.unstubAllGlobals();
});
