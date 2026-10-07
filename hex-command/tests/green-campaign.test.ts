import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { Game } from "./game-fixture";
import { campaignMission } from "../src/game/campaign-progress";
import { loadBattle, saveBattle } from "./legacy-save";
import { required, TestStorage } from "./helpers";

const storage = new TestStorage();

const companion = (game: Game) =>
  required(game.units.find((unit) => unit.side === "player" && unit.name === "Lark"));

// Dain's death changes this debrief and future missions, but retry rewinds current-stage deaths.
for (const mission of [0, 1, 2]) {
  test(
    "Dain deaths persist but retry restores incoming companions — " + JSON.stringify([mission]),
    async () => {
      const game = Game.startCampaign("green-accord", mission);
      game.units = game.units.filter((unit) => unit.name !== "Captain Dain");
      game.round = 4;
      game.gold.player = 23;
      saveBattle(game);
      const restored = loadBattle().game;
      assert.equal(restored.mapId, game.mapId);
      assert.equal(restored.round, 4);
      assert.equal(restored.gold.player, 23);
      assert.deepEqual(restored.units, game.units);
      assert(restored.campaign?.marks.includes("dain-fell"));
      assert(
        !campaignMission(restored.campaign!).victory.some((line) => line.speaker.includes("Dain")),
      );
      assert(restored.retryCampaignMission().units.some((unit) => unit.name === "Captain Dain"));
      saveBattle(restored);
      assert.deepEqual(loadBattle().game.snapshot(), restored.snapshot());
    },
  );
}

// A legacy victory cannot resurrect Lark; surviving Lark keeps the already earned victory.
for (const mission of [0, 1, 2]) {
  test("Legacy victory cannot resurrect Lark — " + JSON.stringify([mission]), async () => {
    const game = Game.startCampaign("green-accord", mission);
    game.winner = "player";
    game.campaign!.outcome = "Mission complete. The company can rest and regroup.";
    game.campaign!.completed = mission === 2;
    game.round = 9;
    game.gold.player = 31;
    saveBattle(game);
    assert.equal(loadBattle().game.winner, "player");
    game.units = game.units.filter((unit) => unit.name !== "Lark");
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
    assert.equal(companion(restored.retryCampaignMission()).name, "Lark");
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
