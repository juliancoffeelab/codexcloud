import "./storage-fixture.mjs";
import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import * as Presentation from "./presentation-adapter.mjs";
import * as Catalog from "./catalog-adapter.mjs";
import { catalog, Native } from "./native-fixture.mjs";
import history from "../packages/save-migrations/src/legacy/data/stages.json";
import { Game } from "./game-fixture.mjs";
import { createGameView } from "./fixture-view.mjs";
import * as Store from "./storage-adapter.mjs";
import { saveBattle, loadBattle, SAVE_KEY } from "./legacy-save.mjs";
import { required, TestStorage } from "./helpers.mjs";
test("Resolved native promotions reference existing authored units", async () => {
  for (const rule of Catalog.units(catalog)) {
    for (const target of rule.advancesTo ?? [])
      assert.equal(Catalog.unit(catalog, target).id, target);
  }
});
test("Combat XP and promotion preserve wounds, identity and immutable publications", async () => {
  const game = new Game();
  game.units = [];
  const attacker = game.addUnit("swordsman", "player", "3,3");
  const defender = game.addUnit("archer", "enemy", "4,3");
  attacker.hp = Catalog.unit(catalog, "swordsman").hp - 4;
  defender.hp = 3;
  game.attack(attacker.id, defender.id, "sword");
  assert.equal(attacker.experience, 6);
  assert.deepEqual(game.promotionOptions(attacker.id), []);
  attacker.experience = 10;
  attacker.moved = attacker.attacked = false;
  const before = createGameView(game);
  const beforePromotion = game.captureState();
  assert.deepEqual(
    game.promotionOptions(attacker.id),
    Catalog.unit(catalog, attacker.type).advancesTo,
  );
  game.advanceUnit(attacker.id, "soldier");
  const feedback = Presentation.changes(beforePromotion, game.captureState());
  assert(feedback.advanced);
  assert.equal(feedback.attacked, false, "Promotion must not replay the prior combat cue");
  assert.equal(
    attacker.hp,
    Catalog.unit(catalog, "soldier").hp - 4,
    "Promotion preserves all four wounds",
  );
  assert.equal(attacker.id, beforePromotion.units[0].id);
  assert.equal(attacker.experience, 0);
  assert(attacker.moved && attacker.attacked);
  assert.equal(
    Native.unit(before, attacker.id)?.type,
    "swordsman",
    "Published progression remains immutable",
  );
  assert.throws(() => game.advanceUnit(attacker.id, "soldier"), /IllegalAdvance/);
});
test("Progressed campaign units persist and invalid XP is rejected", async () => {
  const campaign = Game.startCampaign("borrowed-tide");
  campaign.advanceDialogue(true);
  const leader = required(
    campaign.units.find((unit) => unit.role === "leader" && unit.side === "player"),
  );
  const recruit = required(
    campaign.units.find((unit) => unit.type === "reefguard" && unit.side === "player"),
  );
  recruit.experience = Catalog.unit(catalog, recruit.type).experienceNeeded;
  campaign.advanceUnit(recruit.id, "tridentguard");
  leader.experience = Catalog.unit(catalog, leader.type).experienceNeeded;
  campaign.advanceUnit(leader.id, "tideseer");
  const pilot = required(campaign.units.find((unit) => unit.character === "edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  await saveBattle(next);
  assert.equal((await loadBattle()).game.campaign?.id, "borrowed-tide");
  assert.deepEqual((await loadBattle()).game.units, next.units);
  const saved = JSON.parse(required((await Store.read_session())[0]));
  saved.state.units[0].experience = -1;
  await Store.store_save(JSON.stringify(saved));
  assert.equal((await loadBattle()).game.mapId, "mirror");
});
test("Promoted skirmish leaders survive reload", async () => {
  const classic = new Game();
  const commander = required(classic.unit("u1"));
  commander.experience = Catalog.unit(catalog, commander.type).experienceNeeded;
  classic.advanceUnit(commander.id, "marshal");
  await saveBattle(classic);
  assert.equal(
    (await loadBattle()).game.unit("u1")?.type,
    "marshal",
    "Promoted leaders survive reload outside campaigns",
  );
});
for (const [chapter, oldName, officerName] of [
  [0, "Captain Vale", "Postmaster Harl"],
  [1, "Factor Senn", "Foreman Cale"],
]) {
  test(
    "Legacy coastal officers migrate names without resetting progress \u2014 " +
      JSON.stringify([chapter, oldName, officerName]),
    async () => {
      const oldCoast = Game.startCampaign("borrowed-tide", chapter);
      const officer = required(
        oldCoast.units.find((unit) => unit.side === "enemy" && unit.role === "leader"),
      );
      officer.name = oldName;
      officer.hp = 5;
      officer.experience = 7;
      officer.moved = officer.attacked = true;
      oldCoast.campaign.dialogue = [
        required(
          required(history[Catalog.campaign(catalog, "borrowed-tide").missions[chapter].map.id])
            .dialogue[0],
        ),
      ];
      await saveBattle(oldCoast);
      const migrated = (await loadBattle()).game;
      assert.equal(migrated.campaign?.id, "borrowed-tide");
      const { name: _oldName, ...expectedOfficer } = officer;
      assert.deepEqual(migrated.unit(officer.id), expectedOfficer);
      assert.equal(Native.campaign(migrated.publishedView()).dialogue[0]?.speaker, officerName);
    },
  );
}
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});
afterEach(() => {
  vi.unstubAllGlobals();
});
