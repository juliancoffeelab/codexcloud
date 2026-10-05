import { test, afterEach, vi } from "vitest";
import type { UnitType } from "../src/game/types.ts";
import { required, storageWithValue } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
import { describeChanges } from "../src/controller/feedback.ts";
import { createGameView } from "../src/game/view.ts";
import { arsenalView } from "../src/ui/arsenal-view.ts";
import { attackChoiceViews } from "../src/ui/combat-view.ts";
import assert from "node:assert/strict";
import { Game, TYPES, distance } from "../src/game/engine.ts";
import { loadBattle, saveBattle } from "../src/game/save.ts";
const fixture = (a: UnitType = "soldier", b: UnitType = "soldier", targetTile = "3,3") => {
  const game = new Game();
  for (const tile of game.tiles) {
    tile.terrain = "plains";
    tile.owner = null;
  }
  game.units = [];
  const attacker = game.addUnit(a, "player", "2,3"),
    target = game.addUnit(b, "enemy", targetTile);
  return { game, attacker, target };
};

// Explicit balance outcomes, independent of preview/resolution agreement.
const weapons = [
  ["soldier", "sword"],
  ["soldier", "mace"],
  ["archer", "bow"],
  ["scout", "knife"],
  ["scout", "sling"],
] as const;
for (const [defenderType, expected] of [
  ["soldier", [3, 4, 3, 1, 2]],
  ["commander", [3, 3, 4, 1, 1]],
  ["archer", [4, 3, 4, 2, 1]],
  ["scout", [4, 3, 4, 2, 1]],
] as const) {
  test(
    "Weapon damage matches explicit armor balance — " + JSON.stringify([defenderType, expected]),
    async () => {
      for (const [index, [attackerType, weapon]] of weapons.entries()) {
        const { game, attacker, target } = fixture(attackerType, defenderType);
        assert.equal(game.damage(attacker, target, weapon), expected[index]);
      }
    },
  );
}
for (const terrain of ["forest", "hill"] as const) {
  test("Cover reduces sling damage — " + JSON.stringify([terrain]), async () => {
    const { game, attacker, target } = fixture("scout", "soldier");
    required(game.tile(target.tile)).terrain = terrain;
    assert.equal(game.damage(attacker, target, "sling"), 1);
  });
}
test("Zero damage attacks remain available and consume actions", async () => {
  const { game, attacker, target } = fixture("archer", "soldier");
  required(game.tile(target.tile)).terrain = "hill";
  const published = createGameView(game);
  const choice = required(
    attackChoiceViews(published, attacker, target).find((c) => c.id === "knife"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.remainingHp, target.hp);
  assert.equal(choice.attackCalculation.equation, "2 − 1 (armor) − 1 (cover) = 0");
  assert.deepEqual(game.attack(attacker.id, target.id, "knife"), { damage: 0, retaliation: 4 });
  assert.equal(target.hp, 12);
  assert.equal(attacker.hp, 4);
  assert(attacker.moved && attacker.attacked);
  assert.throws(() => game.attack(attacker.id, target.id, "knife"));
  assert.deepEqual(
    arsenalView(required(published.unit(target.id))).modifiers.map((m) => m.label),
    ["−1", "−1", "+1"],
  );
});
test("Covered retaliation can deal zero damage", async () => {
  const { game, attacker, target } = fixture("commander", "archer");
  required(game.tile(attacker.tile)).terrain = "hill";
  const preview = game.previewDetails(attacker, target, "sword");
  assert.equal(preview.retaliation, 0);
  assert.equal(preview.response?.id, "knife");
  const choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "sword"),
  );
  assert(choice.available);
  assert(choice.retaliationCalculation.description.includes("knife"));
  assert.equal(choice.retaliationCalculation.equation, "2 − 1 (armor) − 1 (cover) = 0");
});
test("Blocked attacks still produce combat feedback", async () => {
  const { game, attacker, target } = fixture("scout", "commander", "4,3");
  required(game.tile(target.tile)).terrain = "hill";
  const before = game.snapshot();
  assert.deepEqual(game.attack(attacker.id, target.id, "sling"), { damage: 0, retaliation: 0 });
  assert.equal(target.hp, 16);
  assert.equal(attacker.hp, 8);
  assert(describeChanges(before, game.snapshot()).attacked, "blocked attacks still play their cue");
});
test("Weapon selection exploits armor and honors range", async () => {
  const { game, attacker, target } = fixture();
  assert.equal(game.damage(attacker, target, "sword"), 3);
  assert.equal(game.damage(attacker, target, "mace"), 4);
  assert.equal(required(game.bestAttack(attacker, target)).id, "mace");
  assert.deepEqual(game.preview(attacker, target, "mace"), {
    damage: 4,
    retaliation: 4,
  });
  required(game.tile(target.tile)).terrain = "hill";
  assert.equal(game.damage(attacker, target, "mace"), 3);
  required(game.tile(target.tile)).terrain = "ford";
  assert.equal(game.damage(attacker, target, "mace"), 5);
  assert.throws(() => game.attack(attacker.id, target.id, "bow"), /cannot reach/);
  assert.equal(attacker.attacked, false);
  assert.equal(target.hp, 12);
});
test("Archers choose between safe bows and melee retaliation", async () => {
  const { game, attacker, target } = fixture("archer", "soldier");
  assert.deepEqual(game.preview(attacker, target, "bow"), {
    damage: 3,
    retaliation: 0,
  });
  assert.deepEqual(game.preview(attacker, target, "knife"), {
    damage: 1,
    retaliation: 4,
  });
  assert.equal(required(game.bestAttack(attacker, target)).id, "bow");
  game.attack(attacker.id, target.id, "knife");
  assert.equal(required(game.lastCombat).sound, "attack-melee");
  assert.equal(attacker.hp, 4);
  assert.equal(target.hp, 11);
  assert.throws(() => game.attack(attacker.id, target.id, "bow"));
});
test("Ranged retaliation and intervening hills constrain attacks", async () => {
  const { game, attacker, target } = fixture("archer", "archer", "4,3");
  assert.deepEqual(game.preview(attacker, target, "bow"), {
    damage: 4,
    retaliation: 4,
  });
  assert.equal(game.canAttackFrom(attacker, attacker.tile, target, "knife"), false);
  assert.throws(() => game.preview(attacker, target, "knife"), /cannot reach/);
  for (const tile of game
    .neighbors(attacker.tile)
    .filter((t) => distance(t, required(game.tile(target.tile))) === 1))
    tile.terrain = "hill";
  assert.equal(game.canAttackFrom(attacker, attacker.tile, target, "bow"), false);
  assert.throws(() => game.attack(attacker.id, target.id, "bow"));
});
test("Scouts avoid lethal ranged replies and kills prevent retaliation", async () => {
  const { game, attacker, target } = fixture("scout", "archer");
  assert.deepEqual(game.preview(attacker, target, "sling"), {
    damage: 1,
    retaliation: 4,
  });
  assert.deepEqual(game.preview(attacker, target, "knife"), {
    damage: 2,
    retaliation: 2,
  });
  attacker.hp = 3;
  assert.equal(
    required(game.bestAttack(attacker, target)).id,
    "knife",
    "avoid a lethal ranged reply",
  );
  target.hp = 1;
  assert.equal(
    game.preview(attacker, target, "sling").retaliation,
    0,
    "defeated defenders never reply",
  );
  assert(game.attackOptions(attacker, target).some((a) => a.id === "sling"));
});
// Lethal forecasts identify the unit that dies, including overkill and retaliation.
test("Lethal forecasts identify casualties including overkill", async () => {
  const { game, attacker, target } = fixture();
  target.hp = 4;
  let choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −4 HP (dies)");
  assert.equal(choice.playerForecast.impactLabel, "Take 0 HP");
  target.hp = 3;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −4 HP (dies)", "overkill is also lethal");
  target.hp = 5;
  attacker.hp = 4;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −4 HP");
  assert.equal(choice.playerForecast.impactLabel, "Take −4 HP (dies)");
});

// Every legal weapon across unit pairs and cover must exactly match resolution.
test("All legal unit/weapon/terrain/range previews match resolution", async () => {
  let exchanges = 0;
  for (const a of Object.keys(TYPES) as UnitType[])
    for (const b of Object.keys(TYPES) as UnitType[])
      for (const terrain of ["plains", "forest", "hill", "ford"] as const)
        for (const range of [1, 2] as const)
          for (const weapon of TYPES[a].attacks) {
            const { game, attacker, target } = fixture(a, b, range === 1 ? "3,3" : "4,3");
            required(game.tile(target.tile)).terrain = terrain;
            if (!game.canAttackFrom(attacker, attacker.tile, target, weapon.id)) continue;
            const preview = game.previewDetails(attacker, target, weapon.id);
            assert(preview.damage >= 0 && Number.isInteger(preview.damage));
            assert(preview.retaliation >= 0 && Number.isInteger(preview.retaliation));
            assert(!preview.response || preview.response.category === weapon.category);
            game.attack(attacker.id, target.id, weapon.id);
            assert.equal(attacker.hp, TYPES[a].hp - preview.retaliation);
            assert.equal(target.hp, TYPES[b].hp - preview.damage);
            assert(attacker.moved && attacker.attacked);
            exchanges++;
          }
});
test("Enemy exploits armor weakness", async () => {
  const { game, attacker, target } = fixture();
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert.equal(required(game.lastCombat).attackId, "mace", "enemy exploits armor weakness");
  assert.equal(attacker.hp, 8);
});
test("Wounded archers reposition rather than accept lethal replies", async () => {
  const { game, attacker, target } = fixture("archer", "archer", "4,3");
  target.hp = 3;
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert(
    required(game.unit(target.id)),
    "enemy should reposition instead of accepting a lethal ranged reply",
  );
  assert.equal(
    game.lastCombat,
    undefined,
    "a wounded bowman withdraws before taking another fight",
  );
  assert.equal(target.hp, 3);
  assert.equal(target.retreating, true);
  assert(distance(required(game.tile(target.tile)), required(game.tile(attacker.tile))) > 2);
});
// Recovery follows a real withdrawal, rather than triggering merely from low HP.
test("Withdrawal recovers only on safe owned villages", async () => {
  const { game, attacker, target } = fixture("archer", "archer", "4,3");
  target.hp = 3;
  game.tile("5,3")!.terrain = "village";
  game.tile("5,3")!.owner = "enemy";
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert.equal(target.tile, "5,3", "a withdrawing bowman prefers a safe healing village");
  assert.equal(target.hp, 3, "withdrawal skips retaliation damage");
  game.beginTurn("enemy");
  assert.equal(target.hp, 5);
  game.enemyAct(target.id);
  assert.equal(target.tile, "5,3", "continue recovering instead of reengaging at half health");
  game.beginTurn("enemy");
  game.enemyAct(target.id);
  assert.equal(target.tile, "5,3", "recovery continues until fully healed");
  game.beginTurn("enemy");
  game.enemyAct(target.id);
  assert.equal(target.retreating, undefined, "recovered troops can rejoin the patrol");
  assert(attacker.hp > 0);
});
test("Surrounded soldiers make a desperate stand", async () => {
  const { game, attacker, target } = fixture("soldier", "soldier");
  target.hp = 1;
  for (const tile of game.neighbors(target.tile)) {
    if (!game.at(tile.id)) game.addUnit("soldier", "player", tile.id);
  }
  game.turn = "enemy";
  assert.equal(game.reachable(target.id).size, 0);
  target.retreating = true;
  game.enemyAct(target.id);
  assert(!game.unit(target.id), "a surrounded soldier can make a desperate stand");
  assert.equal(target.retreating, undefined, "a cornered stand is not a withdrawal");
  assert(
    attacker.hp < TYPES.soldier.hp ||
      game.units.some((unit) => unit.side === "player" && unit.hp < TYPES.soldier.hp),
  );
});

// Actual retaliation determines retreat, across every weapon and target.
test("Full health prevents panic about hypothetical later attacks", async () => {
  const { game, attacker, target } = fixture("soldier", "bonearcher", "4,3");
  attacker.tile = "3,3";
  game.addUnit("soldier", "player", "4,2");
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert.equal(required(game.lastCombat).attackId, "bonebow");
  assert.equal(target.hp, TYPES.bonearcher.hp, "future danger does not cause full-health panic");
  assert.equal(target.tile, "4,3");
  assert.equal(target.retreating, undefined);
});
test("Safe attacks allow immediate reengagement", async () => {
  const { game, attacker, target } = fixture("soldier", "archer");
  target.hp = 2;
  target.retreating = true;
  game.turn = "enemy";
  assert.equal(game.preview(target, attacker, "knife").retaliation, 4);
  assert.equal(game.preview(target, attacker, "bow").retaliation, 0);
  game.enemyAct(target.id);
  assert.equal(required(game.lastCombat).attackId, "bow");
  assert.equal(target.hp, 2);
  assert.equal(target.retreating, undefined, "any safe attack permits immediate reengagement");
});
test("Killing blows avoid retaliation", async () => {
  const { game, attacker, target } = fixture("archer", "soldier");
  attacker.hp = 1;
  target.hp = 1;
  game.turn = "enemy";
  assert.equal(game.preview(target, attacker, "sword").retaliation, 0);
  game.enemyAct(target.id);
  assert(!game.unit(attacker.id), "a killing blow prevents retaliation");
  assert.equal(target.hp, 1);
  assert.equal(target.retreating, undefined);
});
test("Retreat ends without a healing destination", async () => {
  const { game, target } = fixture("soldier", "soldier", "7,3");
  target.hp = 1;
  target.retreating = true;
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert.equal(target.retreating, undefined, "without a healing destination, resume normal play");
  assert.notEqual(target.tile, "7,3", "a stale retreat does not flee indefinitely");
});
for (const unavailable of ["neutral", "player-owned", "occupied", "threatened"] as const) {
  test(
    "Unavailable villages cannot sustain recovery — " + JSON.stringify([unavailable]),
    async () => {
      const { game, target } = fixture("soldier", "soldier", "7,3");
      target.hp = 1;
      target.retreating = true;
      const village = required(game.tile("8,3"));
      village.terrain = "village";
      village.owner =
        unavailable === "neutral" ? null : unavailable === "player-owned" ? "player" : "enemy";
      if (unavailable === "occupied") game.addUnit("soldier", "enemy", village.id);
      if (unavailable === "threatened") game.addUnit("archer", "player", "9,3");
      game.turn = "enemy";
      game.enemyAct(target.id);
      assert.equal(target.retreating, undefined, `${unavailable} village cannot sustain recovery`);
    },
  );
}
test("Cornered combat preserves mediation", async () => {
  const game = Game.startCampaign("ashen-oath", 1);
  game.advanceDialogue(true);
  const target = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "soldier"),
  );
  target.tile = "6,4";
  target.hp = 1;
  target.retreating = true;
  for (const tile of game.neighbors(target.tile)) {
    if (!game.at(tile.id)) game.addUnit("soldier", "player", tile.id);
  }
  game.turn = "enemy";
  assert.equal(game.reachable(target.id).size, 0);
  game.enemyAct(target.id);
  assert(!game.unit(target.id));
  assert.equal(
    game.campaign?.refuge?.escalated,
    false,
    "cornered combat is not a retreating death",
  );
});

// Definitions supply new weapons to old saves without replacing battle progress.
test("Old saves inherit weapon definitions without losing progress", async () => {
  const game = new Game();
  game.move("u3", "3,1");
  let stored = "";
  vi.stubGlobal(
    "localStorage",
    storageWithValue(
      () => stored,
      (value) => {
        stored = value;
      },
    ),
  );
  saveBattle(game);
  const restored = loadBattle().game;
  assert.deepEqual(restored.snapshot(), game.snapshot());
  assert.equal(TYPES[required(restored.unit("u3")).type].attacks.length, 2);
  assert.equal(TYPES[required(restored.unit("u2")).type].damageModifiers.impact, 1);
});

test("Armor equation", async () => {
  assert.equal(
    damageExplanation({
      base: 2,
      modifier: -1,
      modified: 1,
      protection: 0,
      exposure: 0,
      damage: 1,
    }),
    "2 − 1 (armor) = 1",
  );
});
test("Armor and cover equation", async () => {
  assert.equal(
    damageExplanation({
      base: 3,
      modifier: -1,
      modified: 2,
      protection: 1,
      exposure: 0,
      damage: 1,
    }),
    "3 − 1 (armor) − 1 (cover) = 1",
  );
});
test("Zero damage from cover equation", async () => {
  assert.equal(
    damageExplanation({
      base: 1,
      modifier: 0,
      modified: 1,
      protection: 1,
      exposure: 0,
      damage: 0,
    }),
    "1 − 1 (cover) = 0",
  );
});
test("Weakness and exposure equation", async () => {
  assert.equal(
    damageExplanation({
      base: 3,
      modifier: 1,
      modified: 4,
      protection: 0,
      exposure: 1,
      damage: 5,
    }),
    "3 + 1 (weakness) + 1 (shallow water) = 5",
  );
});
test("Negative damage is clamped to zero", async () => {
  assert.equal(
    damageExplanation({
      base: 1,
      modifier: -1,
      modified: 0,
      protection: 1,
      exposure: 0,
      damage: 0,
    }),
    "max(0, 1 − 1 (armor) − 1 (cover)) = 0",
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});
