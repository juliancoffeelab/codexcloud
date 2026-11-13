import "./storage-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.mjs";
import { loadBattle, saveBattle } from "./legacy-save.mjs";
import { saveBattle as saveCurrentBattle } from "./persistence-adapter.mjs";
import { required, TestStorage } from "./helpers.mjs";
const storage = new TestStorage();
const companion = (game) =>
  required(game.units.find((unit) => unit.side === "player" && unit.character === "lark"));
test("Companion deaths persist but retry restores incoming companions", async () => {
  const game = Game.startCampaign("green-accord", 1);
  game.units = game.units.filter((unit) => unit.character !== "dain");
  game.round = 4;
  game.gold.player = 23;
  await saveBattle(game);
  const restored = (await loadBattle()).game;
  assert.equal(restored.mapId, game.mapId);
  assert.equal(restored.round, 4);
  assert.equal(restored.gold.player, 23);
  assert.deepEqual(restored.units, game.units);
  assert(restored.campaign?.flags["dain-dead"]?.at(-1)?.state);
  assert(restored.retryCampaignMission().units.some((unit) => unit.character === "dain"));
  await saveCurrentBattle(restored);
  assert.deepEqual((await loadBattle()).game.snapshot(), restored.snapshot());
});
for (const mission of [0, 2]) {
  test("Legacy victory cannot resurrect Lark \u2014 " + JSON.stringify([mission]), async () => {
    const game = Game.startCampaign("green-accord", mission);
    game.winner = "playerVictory";
    game.campaign.outcome = "Mission complete. The company can rest and regroup.";
    game.campaign.completed = mission === 2;
    game.round = 9;
    game.gold.player = 31;
    await saveBattle(game);
    assert.equal((await loadBattle()).game.winner, "playerVictory");
    game.units = game.units.filter((unit) => unit.character !== "lark");
    await saveBattle(game);
    const restored = (await loadBattle()).game;
    assert.equal(restored.mapId, game.mapId);
    assert.equal(restored.winner, "playerDefeat");
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
    // Restored state may contain current objectives absent from the frozen v1 catalog.
    await saveCurrentBattle(restored);
    assert.deepEqual((await loadBattle()).game.snapshot(), restored.snapshot());
  });
}
beforeEach(() => {
  storage.clear();
  vi.stubGlobal("localStorage", storage);
});
afterEach(() => {
  vi.unstubAllGlobals();
});
