import { test, afterEach, vi } from "vitest";
import type { UnitType } from "../src/game/types.ts";
import { required, storageWithValue } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
import { describeChanges } from "../src/controller/feedback.ts";
import { createGameView } from "./fixture-view.ts";
import { arsenalView } from "../src/ui/arsenal-view.ts";
import { attackChoiceViews } from "../src/ui/combat-view.ts";
import assert from "node:assert/strict";
import { Game, TYPES, distance } from "./game-fixture.ts";
import { loadBattle, saveBattle } from "./legacy-save.ts";
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
  game.attack(attacker.id, target.id, "knife");
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
  game.attack(attacker.id, target.id, "sling");
  assert.equal(target.hp, 16);
  assert.equal(attacker.hp, 8);
  assert(describeChanges(before, game.snapshot()).attacked, "blocked attacks still play their cue");
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

// Representative native exchanges cover armor, cover, ranged replies and damage types.
test.each([
  ["soldier", "bonearcher", "mace", "plains", "3,3"],
  ["archer", "soldier", "bow", "forest", "4,3"],
  ["scout", "archer", "sling", "plains", "4,3"],
  ["thorncaller", "spearman", "thorns", "grove", "3,3"],
  ["boar", "skeleton", "trample", "plains", "3,3"],
  ["harpooner", "swordsman", "harpoon", "ford", "3,3"],
] as const)(
  "Native %s attacking %s with %s on %s matches its preview",
  (a, b, weapon, terrain, tile) => {
    const { game, attacker, target } = fixture(a, b, tile);
    required(game.tile(target.tile)).terrain = terrain;
    const preview = game.previewDetails(attacker, target, weapon);
    game.attack(attacker.id, target.id, weapon);
    assert.equal(attacker.hp, TYPES[a].hp - preview.retaliation);
    assert.equal(target.hp, TYPES[b].hp - preview.damage);
    assert(attacker.moved && attacker.attacked);
  },
);

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
