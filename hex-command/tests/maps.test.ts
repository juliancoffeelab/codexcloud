import { test, beforeEach, afterEach, vi } from "vitest";
import { required, TestStorage } from "./helpers.ts";
import assert from "node:assert/strict";
import { pixelSprites } from "../src/presentation/pixel-art";

const { Game, TYPES, developerFixture } = await import("./game-fixture.ts");

test("Developer map shows every playable type and reserves space for the pixel collection", () => {
  const game = new Game("developer");
  for (const type of Object.keys(TYPES))
    for (const side of ["player", "enemy"])
      assert(
        game.units.some((unit) => unit.type === type && unit.side === side),
        `${side} ${type}`,
      );
  const artworkTypes = Object.keys(pixelSprites).filter((frame) =>
    frame.startsWith("player_"),
  ).length;
  const occupied = new Set(game.units.map((unit) => unit.tile));
  const freeGround = game.tiles.filter(
    (tile) => tile.terrain === "plains" && !occupied.has(tile.id),
  ).length;
  assert(
    freeGround >= artworkTypes * 2 - game.units.length,
    "Room for both factions' complete artwork",
  );
});

// These are ordinary units on an ordinary map: movement, combat and turns still work.
test("Developer units use ordinary movement, combat and turns", async () => {
  const developerBattle = developerFixture(["soldier"]);
  const soldier = required(
    developerBattle.units.find((unit) => unit.type === "soldier" && unit.side === "player"),
  );
  const enemySoldier = required(
    developerBattle.units.find((unit) => unit.type === "soldier" && unit.side === "enemy"),
  );
  developerBattle.move(soldier.id, "4,2");
  assert.equal(soldier.tile, "4,2");
  assert(developerBattle.targets(soldier.id).includes(enemySoldier));
  developerBattle.attack(soldier.id, enemySoldier.id, "sword");
  assert(enemySoldier.hp < TYPES.soldier.hp);
  assert(soldier.attacked);
  developerBattle.endTurn();
  assert.equal(developerBattle.turn, "enemy");
  assert.equal(developerBattle.winner, null);
});

// Developer snapshots remain noncampaign saves without relaxing ordinary validation.
const { loadBattle, saveBattle, SAVE_KEY, SAVE_VERSION } = await import("./legacy-save.ts");

test("Developer saves remain noncampaign and round-trip", async () => {
  const showcase = developerFixture();
  saveBattle(showcase);
  const loaded = loadBattle();
  assert.deepEqual(loaded.game.snapshot(), showcase.snapshot());
  assert.equal(loaded.pendingEnemy, false);
  assert.equal(loaded.game.campaign, undefined);
});
test("Moved developer sites persist ownership", async () => {
  const game = developerFixture(["scout"]);
  const unit = required(game.units.find((unit) => unit.side === "player" && unit.type === "scout"));
  const destination = "5,1";
  game.move(unit.id, destination);
  assert.equal(required(game.tile(destination)).owner, "player");
  saveBattle(game);
  assert.deepEqual(loadBattle().game.snapshot(), game.snapshot());
  assert.equal(required(loadBattle().game.tile(destination)).owner, "player");
});
test("Ordinary skirmishes do not publish site ownership overrides", async () => {
  assert.equal(new Game("mirror").snapshot().sites, undefined);
  assert.equal(new Game("mirror").snapshot().sites, undefined);
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
  const units = structuredClone(validSave.state.units);
  const legacySave = structuredClone(validSave);
  delete legacySave.state.sites;
  localStorage.setItem(SAVE_KEY, JSON.stringify(legacySave));
  const legacy = loadBattle().game;
  assert.equal(legacy.mapId, "developer");
  assert.deepEqual(legacy.units, units);
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
        (unit) => !(unit.side === "enemy" && TYPES[unit.type].family === "commander"),
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
