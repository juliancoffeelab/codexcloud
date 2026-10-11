import { test, beforeEach, afterEach, vi } from "vitest";
import { required, TestStorage } from "./helpers.ts";
import assert from "node:assert/strict";

const { Game, TYPES } = await import("./game-fixture.ts");
const { Native, Catalog, catalog, createBattle } = await import("./native-fixture.ts");

// These are ordinary units on an ordinary map: movement, combat and turns still work.
test("Developer units use ordinary movement, combat and turns", async () => {
  const developerBattle = createBattle({ kind: "scenario", map: "developer" });
  const initial = Native.state(Native.view(developerBattle));
  const soldier = required(
    initial.units.find((unit) => unit.type === "soldier" && unit.side === "player"),
  );
  const enemySoldier = required(
    initial.units.find((unit) => unit.type === "soldier" && unit.side === "enemy"),
  );
  Native.dispatch(developerBattle, { kind: "move", unit: Native.unit_id(soldier.id), tile: "4,2" });
  assert.equal(Native.unit(Native.view(developerBattle), soldier.id)!.tile, "4,2");
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
    Native.unit(Native.view(developerBattle), enemySoldier.id)!.hp <
      Catalog.unit(catalog, "soldier").hp,
  );
  assert(Native.unit(Native.view(developerBattle), soldier.id)!.attacked);
  Native.dispatch(developerBattle, { kind: "endTurn" });
  assert.equal(Native.state(Native.view(developerBattle)).turn, "enemy");
  assert.equal(Native.state(Native.view(developerBattle)).winner, null);
});

// Developer snapshots remain noncampaign saves without relaxing ordinary validation.
const { loadBattle, saveBattle, SAVE_KEY, SAVE_VERSION } = await import("./legacy-save.ts");

test("Developer saves remain noncampaign and round-trip", async () => {
  const showcase = new Game("developer");
  saveBattle(showcase);
  const loaded = loadBattle();
  assert.deepEqual(loaded.game.snapshot(), showcase.snapshot());
  assert.equal(loaded.pendingEnemy, false);
  assert.equal(loaded.game.campaign, undefined);
});
for (const [type, destination] of [
  ["scout", "5,1"],
  ["soldier", "3,1"],
  ["soldier", "4,1"],
] as const) {
  test(
    "Moved developer sites persist ownership — " + JSON.stringify([type, destination]),
    async () => {
      const game = new Game("developer");
      const unit = required(
        game.units.find((unit) => unit.side === "player" && unit.type === type),
      );
      game.move(unit.id, destination);
      assert.equal(required(game.tile(destination)).owner, "player");
      saveBattle(game);
      assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
      assert.equal(required(loadBattle().game.tile(destination)).owner, "player");
    },
  );
}
test("Ordinary skirmishes do not publish site ownership overrides", async () => {
  assert.equal(new Game("mirror").snapshot().sites, undefined);
  assert.equal(new Game("river").snapshot().sites, undefined);
});
function validDeveloperSave() {
  const showcase = new Game("developer");
  const validSave = {
    version: SAVE_VERSION,
    pendingEnemy: false,
    state: { ...showcase.snapshot(), nextId: showcase.nextId, log: showcase.log },
  };
  return validSave;
}
test("Legacy developer saves gain neutral sites without resetting units", async () => {
  const showcase = new Game("developer");
  const validSave = validDeveloperSave();
  const legacySave = structuredClone(validSave);
  delete legacySave.state.sites;
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacySave));
  const legacy = loadBattle().game;
  assert.equal(legacy.mapId, "developer");
  assert.deepEqual(legacy.units, showcase.units);
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
  localStorage.setItem(SAVE_KEY, JSON.stringify(missingCampaignSites));
  assert.equal(loadBattle().game.mapId, "mirror", "Campaign sites remain mandatory");
});
const invalidCases: [string, (save: ReturnType<typeof validDeveloperSave>) => void][] = [
  [
    "duplicate unit position",
    (save) => {
      save.state.units[1]!.tile = save.state.units[0]!.tile;
    },
  ],
  [
    "unit on impassable water",
    (save) => {
      save.state.units[0]!.tile = "3,0";
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
      sites[1]!.id = sites[0]!.id;
    },
  ],
  [
    "site on plains",
    (save) => {
      required(save.state.sites)[0]!.id = "2,2";
    },
  ],
  [
    "invalid site owner",
    (save) => {
      Object.assign(required(required(save.state.sites)[0]), { owner: "neutral" });
    },
  ],
];
test.each(invalidCases)("Invalid developer save is rejected: %s", (_label, invalidate) => {
  const invalid = validDeveloperSave();
  invalidate(invalid);
  localStorage.setItem(SAVE_KEY, JSON.stringify(invalid));
  assert.equal(loadBattle().game.mapId, "mirror");
});

beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});

afterEach(() => {
  vi.unstubAllGlobals();
});
