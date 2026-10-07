import { test } from "vitest";
import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "./game-fixture.ts";

// Pathfinder affects forests and hills, while every water crossing keeps its cost.
for (const type of ["scout", "soldier", "archer", "commander"] as const) {
  test(
    "Pathfinder discounts woods and hills without discounting water — " + JSON.stringify([type]),
    async () => {
      const game = new Game();
      const unit =
        game.units.find((unit) => unit.type === type) || game.addUnit(type, "player", "2,3");
      for (const terrain of ["forest", "hill"] as const) {
        assert.equal(game.cost(unit, { terrain }), type === "scout" ? 1 : 2);
      }
      assert.equal(game.cost(unit, { terrain: "water" }), Infinity);
      assert.equal(game.cost(unit, { terrain: "bridge" }), 1);
      assert.equal(
        game.cost(unit, { terrain: "ford" }),
        ["soldier", "commander"].includes(type) ? 3 : 2,
      );
    },
  );
}

// Actual route planning and movement must use the scout's hill discount.
test("Scout hill discount applies to real routes", async () => {
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
});

// Prices govern both affordability and the gold spent on a successful recruit.
for (const [type, price] of [
  ["scout", 5],
  ["archer", 7],
  ["swordsman", 6],
  ["spearman", 6],
] as const) {
  test(
    "Recruit prices control affordability and gold spent — " + JSON.stringify([type, price]),
    async () => {
      const game = new Game();
      const commander = required(
        game.units.find((unit) => unit.side === "player" && unit.type === "commander"),
      );
      assert.equal(TYPES[type].cost, price);
      game.gold.player = price;
      const destination = required(game.recruitTiles(commander.id, type)[0]).id;
      game.gold.player = price - 1;
      assert.deepEqual(game.recruitTiles(commander.id, type), []);
      assert.throws(() => game.recruit(commander.id, type, destination), /IllegalRecruit/);
      game.gold.player = price;
      const recruit = game.recruit(commander.id, type, destination);
      assert.equal(game.gold.player, 0);
      assert.equal(recruit.type, type);
      assert.equal(recruit.moved, true);
      assert.equal(recruit.attacked, true);
    },
  );
}
// The quarry's initial purse affords a melee screen and a ranged skeleton together.
test("Quarry purse buys a melee screen and a ranged skeleton", async () => {
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
});

// The woodland purse can also field a runner and ranged support instead of one boar.
test("Woodland purse supports a runner and ranged support", async () => {
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
});

// Light Impact support survives a rookie sword and bow while its screen withdraws;
// a heavy infantry sword still makes that exposed position fatal.
for (const support of ["wavemender", "adept"] as const)
  for (const infantry of ["swordsman", "soldier"] as const) {
    test(
      "Support survives rookie attacks but remains vulnerable to heavy infantry — " +
        JSON.stringify([support, infantry]),
      async () => {
        const game = new Game();
        game.units = [];
        for (const tile of game.tiles) tile.terrain = "plains";
        const healer = game.addUnit(support, "player", "2,3");
        const sword = game.addUnit(infantry, "enemy", "3,3");
        const archer = game.addUnit("archer", "enemy", "4,3");
        game.turn = "enemy";
        game.attack(sword.id, healer.id, "sword");
        game.attack(archer.id, healer.id, "bow");
        assert.equal(game.unit(healer.id)?.hp, infantry === "swordsman" ? 1 : undefined);
      },
    );
  }

// The bow can survive one sword plus an Impact spell, but two spells still
// kill it: its extra HP does not remove the undead damage-type weakness.
for (const second of ["swordsman", "adept"] as const) {
  test(
    "Undead ranged units retain their impact weakness — " + JSON.stringify([second]),
    async () => {
      const game = new Game();
      game.units = [];
      for (const tile of game.tiles) tile.terrain = "plains";
      const bow = game.addUnit("bonearcher", "player", "2,3");
      const caster = game.addUnit("adept", "enemy", "4,3");
      const other = game.addUnit(second, "enemy", "3,3");
      game.turn = "enemy";
      game.attack(caster.id, bow.id, "spark");
      game.attack(other.id, bow.id, second === "adept" ? "spark" : "sword");
      assert.equal(game.unit(bow.id)?.hp, second === "swordsman" ? 1 : undefined);
    },
  );
}

// A woodland screen survives a sword and an arrow; the same exposed caster dies.
for (const terrain of ["forest", "grove", "plains"] as const) {
  test("Woodland cover protects a fragile caster — " + JSON.stringify([terrain]), async () => {
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
    assert.equal(game.unit(caster.id)?.hp, terrain === "plains" ? undefined : 3);
    assert.equal(game.cost(caster, { terrain }), 1);
  });
}

// Mobility does not turn the fragile skirmisher into a safe melee armor counter.
test("Mobile skirmishers remain vulnerable in melee", async () => {
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const thorncaller = game.addUnit("thorncaller", "player", "2,3");
  const soldier = game.addUnit("soldier", "enemy", "4,3");
  assert.deepEqual(game.preview(thorncaller, soldier, "thorns"), {
    damage: 3,
    retaliation: 0,
  });
  soldier.tile = "3,3";
  assert.deepEqual(game.preview(thorncaller, soldier, "rootstaff"), {
    damage: 3,
    retaliation: 4,
  });
  assert.ok(thorncaller.hp < 2 * game.preview(thorncaller, soldier, "rootstaff").retaliation);
  assert.ok(TYPES.thorncaller.cost <= TYPES.archer.cost);
  assert.ok(TYPES.thorncaller.hp < TYPES.archer.hp);
  assert.ok(TYPES.thorncaller.move > TYPES.archer.move);

  // The shared roster gives enemy thorncallers the same weapon rules.
  soldier.side = "player";
  thorncaller.side = "enemy";
  assert.equal(game.preview(thorncaller, soldier, "thorns").damage, 3);
});
