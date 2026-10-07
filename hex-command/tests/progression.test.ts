import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { describeChanges } from "../src/controller/feedback";
import { Game, RACES, TERRAIN, TYPES } from "./game-fixture";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { goalProgress } from "../src/game/campaign-progress";
import { UNIT_TYPES, type UnitType } from "../src/game/types";
import { movementCost } from "../src/save/migration/legacy/movement";
import { createGameView } from "./fixture-view";
import { saveBattle, loadBattle, SAVE_KEY } from "./legacy-save";
import { required, TestStorage } from "./helpers";

test("Unit enum and definitions cover the same roster", async () => {
  assert.deepEqual(Object.keys(TYPES).sort(), [...UNIT_TYPES].sort());
});
for (const type of UNIT_TYPES) {
  test(
    "Every unit belongs to an acyclic race-preserving progression ladder — " +
      JSON.stringify([type]),
    async () => {
      const definition = TYPES[type];
      assert.equal(
        definition.damage,
        definition.attacks[0].damage,
        `${type} primary damage cannot drift from its weapon`,
      );
      assert.equal(
        definition.range,
        Math.max(...definition.attacks.map((attack) => attack.range)),
        `${type} range summarizes all available weapons`,
      );
      assert(definition.family && TYPES[definition.family]);
      const family = TYPES[definition.family];
      assert((family.advancesTo ?? []).length, `${type} belongs to a progression ladder`);
      const seen = new Set<UnitType>();
      const visit = (next: UnitType) => {
        assert(!seen.has(next), `No cyclic advancement at ${next}`);
        seen.add(next);
        for (const advanced of TYPES[next].advancesTo ?? []) {
          assert(TYPES[advanced].level! > TYPES[next].level!);
          assert.equal(TYPES[advanced].race, definition.race);
          visit(advanced);
        }
      };
      visit(definition.family);
    },
  );
}
// Access precedence and additive costs are independent of numeric Infinity sentinels.
test("Terrain, race and unit access compose with additive costs", async () => {
  assert.equal(TERRAIN.water.cost, 1);
  assert.equal(TERRAIN.water.passable, false);
  assert.equal(movementCost(TERRAIN.water, "water", RACES.human, TYPES.archer), Infinity);
  assert.equal(movementCost(TERRAIN.water, "water", RACES.merfolk, TYPES.harpooner), 1);
  assert.equal(
    movementCost(TERRAIN.water, "water", RACES.merfolk, {
      terrainModifiers: { water: { passable: false } },
    }),
    Infinity,
  );
  assert.equal(
    movementCost(TERRAIN.water, "water", RACES.human, {
      terrainModifiers: { water: { passable: true, cost: 1 } },
    }),
    2,
  );
  assert.equal(
    movementCost(TERRAIN.forest, "forest", RACES.merfolk, {
      terrainModifiers: { forest: { cost: -1 } },
    }),
    2,
  );
});

test("Combat XP and promotion preserve wounds, identity and immutable publications", async () => {
  const game = new Game();
  game.units = [];
  const attacker = game.addUnit("swordsman", "player", "3,3");
  const defender = game.addUnit("archer", "enemy", "4,3");
  attacker.hp = 6;
  defender.hp = 3;
  game.attack(attacker.id, defender.id, "sword");
  assert.equal(attacker.experience, 6);
  assert.deepEqual(game.promotionOptions(attacker.id), []);
  attacker.experience = 10;
  attacker.moved = attacker.attacked = false;
  const before = createGameView(game);
  const beforePromotion = game.snapshot();
  assert.deepEqual(before.promotionOptions(attacker.id), ["soldier"]);
  game.advanceUnit(attacker.id, "soldier");
  const feedback = describeChanges(beforePromotion, game.snapshot());
  assert(feedback.advanced);
  assert.equal(feedback.attacked, false, "Promotion must not replay the prior combat cue");
  assert.equal(attacker.hp, 8, "Promotion preserves all four wounds");
  assert.equal(attacker.id, before.units[0]!.id);
  assert.equal(attacker.experience, 0);
  assert(attacker.moved && attacker.attacked);
  assert.equal(
    before.unit(attacker.id)?.type,
    "swordsman",
    "Published progression remains immutable",
  );
  assert(TYPES.soldier.attacks.some((attack) => attack.damageType === "impact"));
  assert.equal(TYPES.swordsman.attacks.length, 1);
  assert.throws(() => game.advanceUnit(attacker.id, "soldier"), /IllegalAdvance/);
});

test("Zero damage attacks cannot farm XP", async () => {
  const zero = new Game();
  zero.units = [];
  const scout = zero.addUnit("scout", "player", "3,3");
  const heavy = zero.addUnit("soldier", "enemy", "4,3");
  required(zero.tile(heavy.tile)).terrain = "hill";
  zero.attack(scout.id, heavy.id, "knife");
  assert.equal(scout.experience, 0, "Zero damage never farms XP");
});

test("Veteran and named companion progression carries forward and retry restores incoming XP", async () => {
  const campaign = Game.startCampaign("borrowed-tide");
  campaign.advanceDialogue(true);
  const leader = required(
    campaign.units.find((unit) => unit.role === "leader" && unit.side === "player"),
  );
  assert(campaign.recruitTiles(leader.id, "reefguard").some((tile) => tile.terrain === "water"));
  const recruit = required(
    campaign.units.find((unit) => unit.type === "reefguard" && unit.side === "player"),
  );
  recruit.experience = 10;
  campaign.advanceUnit(recruit.id, "tridentguard");
  leader.experience = 18;
  campaign.advanceUnit(leader.id, "tideseer");
  const pilot = required(campaign.units.find((unit) => unit.name === "Edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  assert.equal(
    next.units.find((unit) => unit.role === "leader" && unit.side === "player")?.type,
    "tideseer",
  );
  assert.equal(next.units.filter((unit) => unit.name === "Edda").length, 1);
  assert.equal(next.units.find((unit) => unit.name === "Edda")?.experience, 7);
  assert.equal(
    next.units.find((unit) => unit.type === "tridentguard" && unit.side === "player")?.experience,
    0,
  );
  required(next.units.find((unit) => unit.name === "Edda")).experience = 10;
  assert.equal(
    next.retryCampaignMission().units.find((unit) => unit.name === "Edda")?.experience,
    7,
    "Retry restores incoming XP",
  );
});
test("Enemy capture immediately resets evacuation defense", async () => {
  const contested = Game.startCampaign("borrowed-tide", 1);
  contested.advanceDialogue(true);
  contested.round = 6;
  const edda = required(contested.units.find((unit) => unit.name === "Edda"));
  edda.tile = "4,4";
  contested.move(edda.id, "5,4");
  contested.endTurn();
  contested.endTurn();
  assert.equal(contested.campaign!.holds!["5,4"]!.heldTurns, 1);
  contested.move(edda.id, "5,3");
  const raider = contested.addUnit("swordsman", "enemy", "6,4");
  contested.endTurn();
  contested.move(raider.id, "5,4");
  assert.equal(contested.campaign!.holds!["5,4"]!.heldTurns, 0);
  contested.endTurn();
  assert.equal(contested.winner, null);
  saveBattle(contested);
  assert.equal(loadBattle().game.campaign!.holds!["5,4"]!.heldTurns, 0);
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
  recruit.experience = 10;
  campaign.advanceUnit(recruit.id, "tridentguard");
  leader.experience = 18;
  campaign.advanceUnit(leader.id, "tideseer");
  const pilot = required(campaign.units.find((unit) => unit.name === "Edda"));
  pilot.experience = 7;
  campaign.winner = "player";
  const next = campaign.nextCampaignMission([recruit.id]);
  saveBattle(next);
  assert.equal(loadBattle().game.campaign?.id, "borrowed-tide");
  assert.deepEqual(loadBattle().game.units, next.units);
  const saved = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
  saved.state.units[0].experience = -1;
  localStorage.setItem(SAVE_KEY, JSON.stringify(saved));
  assert.equal(loadBattle().game.mapId, "mirror");
});
test("Promoted skirmish leaders survive reload", async () => {
  const classic = new Game();
  const commander = required(classic.unit("u1"));
  commander.experience = 18;
  classic.advanceUnit(commander.id, "marshal");
  saveBattle(classic);
  assert.equal(
    loadBattle().game.unit("u1")?.type,
    "marshal",
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
