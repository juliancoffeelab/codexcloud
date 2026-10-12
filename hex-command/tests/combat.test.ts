import { test, afterEach, vi } from "vitest";
import type { UnitType } from "../src/game/types.ts";
import { required, storageWithValue } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
import { describeChanges } from "../src/controller/feedback.ts";
import { createGameView } from "./fixture-view.ts";
import { arsenalView } from "../src/ui/arsenal-view.ts";
import { attackChoiceViews } from "../src/ui/combat-view.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "./game-fixture.ts";
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

test("Zero damage attacks remain available and consume actions", async () => {
  const { game, attacker, target } = fixture("bonearcher", "soldier");
  required(game.tile(target.tile)).terrain = "hill";
  const published = createGameView(game);
  const choice = required(
    attackChoiceViews(published, attacker, target).find((c) => c.id === "shard"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.remainingHp, target.hp);
  assert.equal(choice.attackCalculation.equation, "2 − 1 (armor) − 1 (cover) = 0");
  game.attack(attacker.id, target.id, "shard");
  assert.equal(target.hp, 24);
  assert.equal(attacker.hp, 8);
  assert(attacker.moved && attacker.attacked);
  assert.throws(() => game.attack(attacker.id, target.id, "shard"));
  assert.deepEqual(
    arsenalView(required(published.unit(target.id))).modifiers.map((m) => m.label),
    ["−1", "−1", "+1"],
  );
});
test("Covered retaliation can deal zero damage", async () => {
  const { game, attacker, target } = fixture("commander", "bonearcher");
  required(game.tile(attacker.tile)).terrain = "hill";
  const preview = game.previewDetails(attacker, target, "sword");
  assert.equal(preview.retaliation, 0);
  assert.equal(preview.response?.id, "shard");
  const choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "sword"),
  );
  assert(choice.available);
  assert(choice.retaliationCalculation.description.includes("shard"));
  assert.equal(choice.retaliationCalculation.equation, "2 − 1 (armor) − 1 (cover) = 0");
});
test("Skeletal mage impact spell publishes armor weakness and resolves at range", () => {
  const { game, attacker, target } = fixture("skeleton-impact-caster", "soldier", "4,3");
  const view = createGameView(game);
  const choice = required(
    attackChoiceViews(view, attacker, target).find((c) => c.id === "boneburst"),
  );
  assert(choice.available);
  assert.equal(choice.attackCalculation.equation, "10 + 1 (weakness) = 11");
  const hp = target.hp;
  game.attack(attacker.id, target.id, "boneburst");
  assert.equal(target.hp, hp - 11);
  assert.equal(attacker.hp, TYPES[attacker.type].hp);
  assert.equal(view.unit(target.id)?.hp, hp, "The earlier publication remains immutable");
});
for (const [type, weapon, equation, damage] of [
  ["reefguard", "tail", "4 + 1 (weakness) = 5", 5],
  ["merfolk-maceguard", "mace", "6 + 1 (weakness) = 7", 7],
] as const) {
  test(`${type} impact attack publishes and resolves skeletal weakness`, () => {
    const { game, attacker, target } = fixture(type, "skeleton");
    const choice = required(
      attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === weapon),
    );
    assert(choice.available);
    assert.equal(choice.attackCalculation.equation, equation);
    const hp = target.hp;
    game.attack(attacker.id, target.id, weapon);
    assert.equal(target.hp, hp - damage);
    assert(attacker.hp < TYPES[type].hp, "Melee counters still allow retaliation");
  });
}
test("Blocked attacks still produce combat feedback", async () => {
  const { game, attacker, target } = fixture("bonearcher", "commander");
  required(game.tile(target.tile)).terrain = "hill";
  const before = game.snapshot();
  game.attack(attacker.id, target.id, "shard");
  assert.equal(target.hp, 32);
  assert.equal(attacker.hp, 8);
  assert(describeChanges(before, game.snapshot()).attacked, "blocked attacks still play their cue");
});
// Lethal forecasts identify the unit that dies, including overkill and retaliation.
test("Lethal forecasts identify casualties including overkill", async () => {
  const { game, attacker, target } = fixture();
  target.hp = 4;
  let choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −7 HP (dies)");
  assert.equal(choice.playerForecast.impactLabel, "Take 0 HP");
  target.hp = 3;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −7 HP (dies)", "overkill is also lethal");
  target.hp = 8;
  attacker.hp = 7;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, "Deal −7 HP");
  assert.equal(choice.playerForecast.impactLabel, "Take −7 HP (dies)");
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
