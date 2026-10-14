import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { describeChanges } from "../src/controller/feedback";
import { Game, TYPES } from "./game-fixture";
import { CAMPAIGNS } from "../src/save/migrations/legacy/catalog";
import { saveBattle, loadBattle, SAVE_KEY } from "./legacy-save";
import { required, TestStorage } from "./helpers";

test("Combat XP and promotion preserve wounds, identity and immutable publications", async () => {
  const game = new Game();
  game.units = [];
  const attacker = game.addUnit("swordsman", "player", "3,3");
  const defender = game.addUnit("archer", "enemy", "4,3");
  const promotion = required(TYPES.swordsman.advancesTo?.[0]);
  const threshold = required(TYPES.swordsman.experienceNeeded);
  attacker.hp = TYPES.swordsman.hp - 4;
  defender.hp = 1;
  game.attack(attacker.id, defender.id, "sword");
  assert.equal(attacker.experience, Math.min(threshold, 6 * (TYPES[defender.type].level ?? 1)));
  attacker.experience = threshold - 1;
  assert.deepEqual(game.promotionOptions(attacker.id), []);
  attacker.experience = threshold;
  attacker.moved = attacker.attacked = false;
  const before = game.publishedView();
  const beforePromotion = game.snapshot();
  assert(before.promotionOptions(attacker.id).includes(promotion));
  game.advanceUnit(attacker.id, promotion);
  const feedback = describeChanges(beforePromotion, game.snapshot());
  assert(feedback.advanced);
  assert.equal(feedback.attacked, false, "Promotion must not replay the prior combat cue");
  assert.equal(attacker.hp, TYPES[promotion].hp - 4, "Promotion preserves all four wounds");
  assert.equal(attacker.id, before.units[0]!.id);
  assert.equal(attacker.experience, 0);
  assert(attacker.moved && attacker.attacked);
  assert.equal(
    before.unit(attacker.id)?.type,
    "swordsman",
    "Published progression remains immutable",
  );
  assert.throws(() => game.advanceUnit(attacker.id, promotion), /IllegalAdvance/);
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
  recruit.experience = required(TYPES[recruit.type].experienceNeeded);
  campaign.advanceUnit(recruit.id, required(TYPES[recruit.type].advancesTo?.[0]));
  leader.experience = required(TYPES[leader.type].experienceNeeded);
  campaign.advanceUnit(leader.id, required(TYPES[leader.type].advancesTo?.[0]));
  const pilot = required(campaign.units.find((unit) => unit.name === "Edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  saveBattle(next);
  assert.equal(loadBattle().game.campaign?.id, "borrowed-tide");
  assert.deepEqual(loadBattle().game.units, next.units);
  const saved = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
  saved.checkpoint.state.units[0].experience = -1;
  localStorage.setItem(SAVE_KEY, JSON.stringify(saved));
  assert.equal(loadBattle().game.mapId, "mirror");
});
test("Promoted skirmish leaders survive reload", async () => {
  const classic = new Game();
  const commander = required(classic.unit("u1"));
  const promotion = required(TYPES[commander.type].advancesTo?.[0]);
  commander.experience = required(TYPES[commander.type].experienceNeeded);
  classic.advanceUnit(commander.id, promotion);
  saveBattle(classic);
  assert.equal(
    loadBattle().game.unit("u1")?.type,
    promotion,
    "Promoted leaders survive reload outside campaigns",
  );
});
for (const [chapter, oldName, officerName] of [
  [0, "Captain Vale", "Postmaster Harl"],
  [1, "Factor Senn", "Foreman Cale"],
] as const) {
  test(
    "Legacy coastal officers migrate names without resetting progress — " +
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
      oldCoast.campaign!.dialogue = [
        required(CAMPAIGNS["borrowed-tide"].missions[chapter]!.legacyDialogue?.[0]),
      ];
      saveBattle(oldCoast);
      const migrated = loadBattle().game;
      assert.equal(migrated.campaign?.id, "borrowed-tide");
      assert.deepEqual(migrated.unit(officer.id), { ...officer, name: officerName });
      assert.equal(migrated.campaign!.dialogue[0]?.speaker, officerName);
    },
  );
}

beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage());
});

afterEach(() => {
  vi.unstubAllGlobals();
});
