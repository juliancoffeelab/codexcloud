import { test } from "vitest";
import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "../src/game/engine.ts";

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
      assert.throws(() => game.recruit(commander.id, type, destination), /Cannot recruit/);
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

// Refuge reinforcements and purchases begin as rookies; its authored elite
// patrol remains armored, and a recruit can earn that armor through advancement.
test("Refuge recruits start as rookies and can earn armor", async () => {
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  assert(game.units.some((unit) => unit.side === "enemy" && unit.type === "soldier"));
  assert(game.units.some((unit) => unit.side === "enemy" && unit.type === "warden"));
  game.round = 3;
  game.beginTurn("enemy");
  assert.equal(game.at("9,7")?.type, "swordsman");
  const oldIds = new Set(game.units.map((unit) => unit.id));
  game.gold.enemy = 6;
  game.enemyRecruit();
  const rookie = required(game.units.find((unit) => !oldIds.has(unit.id)));
  assert.equal(rookie.type, "swordsman");
  rookie.experience = 10;
  game.beginTurn("enemy");
  game.enemyAct(rookie.id);
  assert.equal(rookie.type, "soldier");
  assert.equal(rookie.hp, 12);
  assert(rookie.moved && rookie.attacked);
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

// Thorns can harass armored humans in cover, while skeletal armor remains a counter.
test("Thorns harass humans in cover but skeletal armor counters them", async () => {
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const thorncaller = game.addUnit("thorncaller", "player", "2,3");
  const archer = game.addUnit("archer", "player", "2,4");
  for (const [type, exposed, covered] of [
    ["soldier", 3, 2],
    ["warden", 3, 2],
    ["archer", 4, 3],
    ["wolf", 5, 3],
    ["skeleton", 2, 1],
  ] as const) {
    const target = game.addUnit(type, "enemy", "3,3");
    const tile = required(game.tile(target.tile));
    tile.terrain = "plains";
    assert.equal(game.damageBreakdown(thorncaller, target, "thorns").damage, exposed);
    tile.terrain = "forest";
    assert.equal(game.damageBreakdown(thorncaller, target, "thorns").damage, covered);
    assert.equal(game.damage(archer, target, "bow"), covered);
    game.units = game.units.filter((unit) => unit.id !== target.id);
  }
});

// The increased healer price requires another income tick for the complete trio.
test("Coastal purse buys its third complementary role after village income", async () => {
  const game = Game.startCampaign("borrowed-tide", 2, [], 8);
  game.advanceDialogue(true);
  const mira = required(
    game.units.find((unit) => unit.side === "player" && unit.role === "leader"),
  );
  for (const type of ["reefguard", "harpooner"] as const) {
    const tile = required(game.recruitTiles(mira.id, type)[0]);
    game.recruit(mira.id, type, tile.id);
  }
  assert.equal(game.gold.player, 9);
  assert.deepEqual(game.recruitTiles(mira.id, "wavemender"), []);
  game.beginTurn("player");
  assert.equal(game.gold.player, 11);
  game.recruit(mira.id, "wavemender", required(game.recruitTiles(mira.id, "wavemender")[0]).id);
  assert.equal(game.gold.player, 1);
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "plains";
  const guard = game.addUnit("reefguard", "player", "2,3");
  const caster = game.addUnit("wavemender", "player", "2,4");
  const armor = game.addUnit("warden", "enemy", "3,3");
  assert.equal(game.preview(guard, armor, "spear").damage, 2);
  assert.equal(game.preview(caster, armor, "surge").damage, 4);
  assert(TYPES.reefguard.attacks.every((weapon) => weapon.damageType === "pierce"));
  assert(TYPES.tridentguard.attacks.some((weapon) => weapon.damageType === "impact"));
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

test.each(["forest", "grove"] as const)(
  "Woodland racial cover protects melee and ranged targets in %s, including veterans",
  (terrain) => {
    const game = new Game();
    game.units = [];
    const sword = game.addUnit("swordsman", "enemy", "3,3");
    const bow = game.addUnit("bonearcher", "enemy", "4,3");
    for (const type of [
      "wolf",
      "direwolf",
      "boar",
      "greatboar",
      "wildkeeper",
      "grovewarden",
    ] as const) {
      const target = game.addUnit(type, "player", "2,3");
      const tile = required(game.tile(target.tile));
      tile.terrain = "plains";
      const exposedBow = game.damage(bow, target, "bonebow");
      assert.equal(game.damage(sword, target, "sword"), 3);
      tile.terrain = terrain;
      assert.equal(game.damage(sword, target, "sword"), 2);
      assert.equal(game.damage(bow, target, "bonebow"), exposedBow - 2);
      assert.equal(game.cost(target, tile), 1);
      game.units = game.units.filter((unit) => unit !== target);
    }
    for (const type of ["thorncaller", "briarweaver"] as const) {
      const target = game.addUnit(type, "player", "2,3");
      assert.equal(
        game.damage(bow, target, "bonebow"),
        1,
        "Individual concealment stacks with race and terrain cover",
      );
      game.units = game.units.filter((unit) => unit !== target);
    }
  },
);

test.each(["boar", "greatboar"] as const)(
  "%s leads with Impact and can use Pierce as an alternative",
  (type) => {
    const game = new Game();
    game.units = [];
    const attacker = game.addUnit(type, "player", "2,3");
    const target = game.addUnit("skeleton", "enemy", "3,3");
    required(game.tile(target.tile)).terrain = "plains";
    assert.equal(game.damage(attacker, target, "trample"), type === "boar" ? 5 : 6);
    assert.equal(game.damage(attacker, target, "tusks"), type === "boar" ? 2 : 3);
    assert.equal(game.previewDetails(attacker, target).attack.id, "trample");
  },
);

test("Wavemender purchase costs ten gold and promotion retains healing", () => {
  const game = Game.startCampaign("borrowed-tide");
  game.advanceDialogue(true);
  const leader = required(
    game.units.find((unit) => unit.side === "player" && TYPES[unit.type].leader),
  );
  game.gold.player = 9;
  assert.equal(game.recruitTiles(leader.id, "wavemender").length, 0);
  const tile = required(game.recruitPlacementTiles(leader.id, "wavemender")[0]);
  assert.throws(() => game.recruit(leader.id, "wavemender", tile.id), /Cannot recruit/);
  game.gold.player = 10;
  const healer = game.recruit(leader.id, "wavemender", tile.id);
  assert.equal(game.gold.player, 0);
  assert.equal(TYPES[healer.type].healing, 1);
  healer.experience = 10;
  healer.moved = healer.attacked = false;
  game.advanceUnit(healer.id, "tidesage");
  assert.equal(TYPES[healer.type].healing, 2);
});
