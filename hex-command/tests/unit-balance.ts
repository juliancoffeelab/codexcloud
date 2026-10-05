import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "../src/game/engine.ts";

// Pathfinder affects forests and hills, while every water crossing keeps its cost.
for (const type of ["scout", "soldier", "archer", "commander"] as const) {
  const game = new Game();
  const unit = game.units.find((unit) => unit.type === type) || game.addUnit(type, "player", "2,3");
  for (const terrain of ["forest", "hill"] as const) {
    assert.equal(game.cost(unit, { terrain }), type === "scout" ? 1 : 2);
  }
  assert.equal(game.cost(unit, { terrain: "water" }), Infinity);
  assert.equal(game.cost(unit, { terrain: "bridge" }), 1);
  assert.equal(
    game.cost(unit, { terrain: "ford" }),
    ["soldier", "commander"].includes(type) ? 3 : 2,
  );
}

// Actual route planning and movement must use the scout's hill discount.
{
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "water";
  required(game.tile("0,0")).terrain = "plains";
  for (let col = 1; col <= 5; col++) required(game.tile(`${col},0`)).terrain = "hill";
  const scout = game.addUnit("scout", "player", "0,0");
  assert.equal(game.reachable(scout.id).get("5,0"), 5);
  game.move(scout.id, "5,0");
  assert.equal(scout.tile, "5,0");
  assert.equal(scout.moved, true);
}

// Prices govern both affordability and the gold spent on a successful recruit.
for (const [type, price] of [
  ["scout", 5],
  ["archer", 8],
  ["swordsman", 6],
  ["spearman", 6],
] as const) {
  const game = new Game();
  const commander = required(
    game.units.find((unit) => unit.side === "player" && unit.type === "commander"),
  );
  assert.equal(TYPES[type].cost, price);
  game.gold.player = price;
  const destination = required(game.recruitTiles(commander.id, type)[0]).id;
  game.gold.player = price - 1;
  assert.deepEqual(game.recruitTiles(commander.id, type), []);
  assert.throws(() => game.recruit(commander.id, type, destination), /Cannot recruit/);
  game.gold.player = price;
  const recruit = game.recruit(commander.id, type, destination);
  assert.equal(game.gold.player, 0);
  assert.equal(recruit.type, type);
  assert.equal(recruit.moved, true);
  assert.equal(recruit.attacked, true);
}
// The quarry's initial purse affords a melee screen and a ranged skeleton together.
{
  const game = Game.startCampaign("ashen-oath");
  game.advanceDialogue(true);
  const orin = required(
    game.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  const startingCompany = game.units.length;
  for (const type of ["skeleton", "bonearcher"] as const) {
    const tile = required(game.recruitTiles(orin.id, type)[0]);
    game.recruit(orin.id, type, tile.id);
  }
  assert.equal(game.units.length, startingCompany + 2);
  assert.equal(game.gold.player, 0);
}

// The woodland purse can also field a runner and ranged support instead of one boar.
{
  const game = Game.startCampaign("green-accord");
  game.advanceDialogue(true);
  const alder = required(
    game.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  for (const type of ["wolf", "thorncaller"] as const) {
    const tile = required(game.recruitTiles(alder.id, type)[0]);
    game.recruit(alder.id, type, tile.id);
  }
  assert.equal(game.gold.player, 0);
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const spear = game.addUnit("spearman", "enemy", "3,3");
  const boar = game.addUnit("boar", "player", "2,3");
  assert.equal(game.preview(spear, boar, "spear").damage, 4);
  assert.equal(game.preview(boar, spear, "tusks").damage, 4);
  assert.equal(
    TYPES.boar.hp,
    13,
    "The front-line body remains sturdy, but spear damage has a counter",
  );
}

// Thorns can harass armored humans in cover, while skeletal armor remains a counter.
{
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const thorncaller = game.addUnit("thorncaller", "player", "2,3");
  const archer = game.addUnit("archer", "player", "2,4");
  for (const [type, exposed, covered] of [
    ["soldier", 2, 1],
    ["warden", 2, 1],
    ["archer", 3, 2],
    ["wolf", 4, 3],
    ["skeleton", 1, 0],
  ] as const) {
    const target = game.addUnit(type, "enemy", "3,3");
    const tile = required(game.tile(target.tile));
    tile.terrain = "plains";
    assert.equal(game.damageBreakdown(thorncaller, target, "thorns").damage, exposed);
    tile.terrain = "forest";
    assert.equal(game.damageBreakdown(thorncaller, target, "thorns").damage, covered);
    assert.equal(game.damage(archer, target, "bow"), covered + 1);
    game.units = game.units.filter((unit) => unit.id !== target.id);
  }
}

// A final coastal purse buys three different roles, rather than only three spear bodies.
{
  const game = Game.startCampaign("borrowed-tide", 2, [], 8);
  game.advanceDialogue(true);
  const mira = required(
    game.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  for (const type of ["reefguard", "harpooner", "wavemender"] as const) {
    const tile = required(game.recruitTiles(mira.id, type)[0]);
    game.recruit(mira.id, type, tile.id);
  }
  assert.equal(game.gold.player, 0);
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const guard = game.addUnit("reefguard", "player", "2,3");
  const caster = game.addUnit("wavemender", "player", "2,4");
  const armor = game.addUnit("warden", "enemy", "3,3");
  assert.equal(game.preview(guard, armor, "spear").damage, 2);
  assert.equal(game.preview(caster, armor, "surge").damage, 4);
  assert(TYPES.reefguard.attacks.every((weapon) => weapon.damageType === "pierce"));
  assert(TYPES.tridentguard.attacks.some((weapon) => weapon.damageType === "impact"));
}

// A woodland screen survives a sword and an arrow; the same exposed caster dies.
for (const terrain of ["forest", "grove", "plains"] as const) {
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  required(game.tile("2,3")).terrain = terrain;
  const caster = game.addUnit("thorncaller", "player", "2,3");
  const soldier = game.addUnit("soldier", "enemy", "3,3");
  const archer = game.addUnit("archer", "enemy", "4,3");
  game.turn = "enemy";
  game.attack(soldier.id, caster.id, "sword");
  game.attack(archer.id, caster.id, "bow");
  assert.equal(game.unit(caster.id)?.hp, terrain === "plains" ? undefined : 1);
  assert.equal(game.cost(caster, { terrain }), 1);
}

// Mobility does not turn the fragile skirmisher into a safe melee armor counter.
{
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const thorncaller = game.addUnit("thorncaller", "player", "2,3");
  const soldier = game.addUnit("soldier", "enemy", "4,3");
  assert.deepEqual(game.preview(thorncaller, soldier, "thorns"), {
    damage: 2,
    retaliation: 0,
  });
  soldier.tile = "3,3";
  assert.deepEqual(game.preview(thorncaller, soldier, "rootstaff"), {
    damage: 3,
    retaliation: 4,
  });
  assert.ok(thorncaller.hp < 2 * game.preview(thorncaller, soldier, "rootstaff").retaliation);
  assert.ok(TYPES.thorncaller.cost < TYPES.archer.cost);
  assert.ok(TYPES.thorncaller.hp < TYPES.archer.hp);
  assert.ok(TYPES.thorncaller.move > TYPES.archer.move);

  // The shared roster gives enemy thorncallers the same weapon rules.
  soldier.side = "player";
  thorncaller.side = "enemy";
  assert.equal(game.preview(thorncaller, soldier, "thorns").damage, 2);
}
console.log(
  "PASS: movement, recruitment prices, Thorncaller ranged matchups, and melee counterplay",
);
