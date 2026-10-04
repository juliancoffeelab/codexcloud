import assert from "node:assert/strict";
import { Game, RACES, TERRAIN, TYPES } from "../src/game/engine";
import { UNIT_TYPES, type UnitType } from "../src/game/types";
import { movementCost } from "../src/game/movement";
import { createGameView } from "../src/game/view";
import { saveBattle, loadBattle, SAVE_KEY } from "../src/game/save";
import { required, TestStorage } from "./helpers";

assert.deepEqual(Object.keys(TYPES).sort(), [...UNIT_TYPES].sort());
for (const type of UNIT_TYPES) {
  const definition = TYPES[type];
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
}
// Access precedence and additive costs are independent of numeric Infinity sentinels.
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
assert.deepEqual(before.promotionOptions(attacker.id), ["soldier"]);
game.advanceUnit(attacker.id, "soldier");
assert.equal(attacker.hp, 9, "Promotion preserves all three wounds");
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
assert.throws(() => game.advanceUnit(attacker.id, "soldier"), /not available/);

const zero = new Game();
zero.units = [];
const scout = zero.addUnit("scout", "player", "3,3");
const heavy = zero.addUnit("soldier", "enemy", "4,3");
required(zero.tile(heavy.tile)).terrain = "hill";
zero.attack(scout.id, heavy.id, "knife");
assert.equal(scout.experience, 0, "Zero damage never farms XP");

// Race armor remains separate from inherited environmental damage.
const terrainGame = new Game();
terrainGame.units = [];
const bow = terrainGame.addUnit("archer", "player", "3,3");
const human = terrainGame.addUnit("swordsman", "enemy", "4,3");
const swimmer = terrainGame.addUnit("harpooner", "enemy", "4,4");
for (const terrain of ["forest", "grove", "hill"] as const) {
  required(terrainGame.tile(swimmer.tile)).terrain = terrain;
  assert.equal(terrainGame.damage(bow, swimmer, "bow"), 4);
  assert.equal(terrainGame.cost(swimmer, { terrain }), 3);
}
for (const terrain of ["ford", "marsh"] as const) {
  required(terrainGame.tile(human.tile)).terrain = terrain;
  required(terrainGame.tile(swimmer.tile)).terrain = terrain;
  assert.equal(terrainGame.damage(bow, human, "bow"), 4);
  assert.equal(terrainGame.damage(bow, swimmer, "bow"), terrain === "marsh" ? 2 : 3);
}
assert.equal(terrainGame.cost(swimmer, { terrain: "plains" }), 1);

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

Object.defineProperty(globalThis, "localStorage", { configurable: true, value: new TestStorage() });
saveBattle(next);
assert.equal(loadBattle().game.campaign?.id, "borrowed-tide");
assert.deepEqual(loadBattle().game.units, next.units);
const saved = JSON.parse(required(localStorage.getItem(SAVE_KEY)));
saved.state.units[0].experience = -1;
localStorage.setItem(SAVE_KEY, JSON.stringify(saved));
assert.equal(loadBattle().game.mapId, "mirror");
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
console.log(
  "PASS: finite race/unit access, terrain weaknesses, every advancement ladder, wounds/actions/XP, veteran checkpoints and saved promoted leaders",
);
