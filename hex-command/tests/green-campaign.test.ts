import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture";
import { campaignMission } from "../src/game/campaign-progress";
import { loadBattle, saveBattle } from "./legacy-save";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

const companion = (game: Game) =>
  required(game.units.find((unit) => unit.side === "player" && unit.character === "lark"));

// Dain's death changes this debrief and future missions, but retry rewinds current-stage deaths.
test("Companion deaths persist but retry restores incoming companions", () => {
  const game = Game.startCampaign("green-accord", 1);
  game.units = game.units.filter((unit) => unit.character !== "dain");
  game.round = 4;
  game.gold.player = 23;
  saveBattle(game);
  const restored = loadBattle().game;
  assert.equal(restored.mapId, game.mapId);
  assert.equal(restored.round, 4);
  assert.equal(restored.gold.player, 23);
  assert.deepEqual(restored.units, game.units);
  assert(restored.campaign?.flags["dain-dead"]?.at(-1)?.state);
  assert(restored.retryCampaignMission().units.some((unit) => unit.character === "dain"));
  saveBattle(restored);
  assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
});

// A legacy victory cannot resurrect Lark; surviving Lark keeps the already earned victory.
for (const mission of [0, 2]) {
  test("Legacy victory cannot resurrect Lark — " + JSON.stringify([mission]), async () => {
    const game = Game.startCampaign("green-accord", mission);
    game.winner = "player";
    game.campaign!.outcome = "Mission complete. The company can rest and regroup.";
    game.campaign!.completed = mission === 2;
    game.round = 9;
    game.gold.player = 31;
    saveBattle(game);
    assert.equal(loadBattle().game.winner, "player");
    game.units = game.units.filter((unit) => unit.character !== "lark");
    saveBattle(game);
    const restored = loadBattle().game;
    assert.equal(restored.mapId, game.mapId);
    assert.equal(restored.winner, "enemy");
    assert.equal(restored.campaign?.completed, false);
    assert.equal(
      restored.campaign?.outcome,
      "Lark fell. The company cannot continue without them.",
    );
    assert.equal(restored.round, 9);
    assert.equal(restored.gold.player, 31);
    assert.deepEqual(restored.units, game.units);
    assert.equal(
      restored.campaign?.dialogue[restored.campaign.dialogueIndex]?.text,
      restored.campaign?.outcome,
    );
    assert.throws(() => restored.nextCampaignMission([]), /NoNextMission/);
    assert.equal(companion(restored.retryCampaignMission()).character, "lark");
    saveBattle(restored);
    assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
  });
}

beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  vi.unstubAllGlobals();
});
