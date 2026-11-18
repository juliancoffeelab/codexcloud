import "./storage-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import * as Presentation from "./presentation-adapter.mjs";
import * as Catalog from "./catalog-adapter.mjs";
import { catalog, Native } from "./native-fixture.mjs";
import { Game } from "./game-fixture.mjs";
import { createGameView } from "./fixture-view.mjs";
import * as Store from "./storage-adapter.mjs";
import { saveBattle, loadBattle, createTestFactory } from "./persistence-adapter.mjs";
import { required, TestStorage } from "./helpers.mjs";
test("Resolved native promotions reference existing authored units", async () => {
  for (const rule of Catalog.units(catalog)) {
    for (const target of rule.promotionIds ?? [])
      assert.equal(Catalog.unit(catalog, target).id, target);
  }
});
test("Combat XP and promotion preserve wounds, identity and immutable publications", async () => {
  const game = new Game();
  game.units = [];
  const attacker = game.addUnit("swordsman", "player", "3,3");
  const defender = game.addUnit("archer", "enemy", "4,3");
  attacker.hp = game.profile(attacker).hp - 4;
  defender.hp = 3;
  game.attack(attacker.id, defender.id, "sword");
  assert.equal(attacker.experience, 6);
  assert.deepEqual(game.promotionOptions(attacker.id), []);
  attacker.experience = 10;
  attacker.moved = attacker.attackSpent = false;
  const before = createGameView(game);
  const beforePromotion = game.captureState();
  assert.deepEqual(
    game.promotionOptions(attacker.id),
    Catalog.unit(catalog, attacker.type).promotionIds,
  );
  game.advanceUnit(attacker.id, "soldier");
  const feedback = Presentation.changes(beforePromotion, game.captureState());
  assert(feedback.advanced);
  assert.equal(feedback.attacked, false, "Promotion must not replay the prior combat cue");
  assert.equal(attacker.hp, game.profile(attacker).hp - 4, "Promotion preserves all four wounds");
  assert.equal(attacker.id, beforePromotion.units[0].id);
  assert.equal(attacker.experience, 0);
  assert(attacker.moved && attacker.attackSpent);
  assert.equal(
    Native.unit(before, attacker.id)?.type,
    "swordsman",
    "Published progression remains immutable",
  );
  assert.throws(() => game.advanceUnit(attacker.id, "soldier"), /IllegalAdvance/);
});
test("Progressed campaign units persist and invalid XP is rejected", async () => {
  const campaign = Game.startCampaign("lantern-road");
  campaign.advanceDialogue(true);
  const leader = required(
    campaign.units.find((unit) => unit.role === "leader" && unit.faction === "player"),
  );
  const recruit = campaign.addUnit("swordsman", "player", "1,1");
  recruit.experience = campaign.profile(recruit).experienceNeeded;
  campaign.advanceUnit(recruit.id, "soldier");
  leader.experience = campaign.profile(leader).experienceNeeded;
  campaign.advanceUnit(leader.id, "soldier");
  const pilot = required(campaign.units.find((unit) => unit.character === "ivo"));
  pilot.experience = 7;
  campaign.winner = "playerVictory";
  const next = campaign.nextCampaignMission([recruit.id]);
  await saveBattle(next);
  assert.equal((await loadBattle(createTestFactory())).game.campaign?.id, "lantern-road");
  assert.deepEqual((await loadBattle(createTestFactory())).game.units, next.units);
  const saved = JSON.parse(required((await Store.read_session())[0]));
  saved.checkpoint.state.units[0].experience = -1;
  await Store.store_save(JSON.stringify(saved));
  assert.equal((await loadBattle(createTestFactory())).game.mapId, "mirror");
});
test("Promoted skirmish leaders survive reload", async () => {
  const classic = new Game();
  const commander = required(classic.unit("u1"));
  commander.experience = classic.profile(commander).experienceNeeded;
  classic.advanceUnit(commander.id, "marshal");
  await saveBattle(classic);
  assert.equal(
    (await loadBattle(createTestFactory())).game.unit("u1")?.type,
    "marshal",
    "Promoted leaders survive reload outside campaigns",
  );
});
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});
afterEach(() => {
  vi.unstubAllGlobals();
});
