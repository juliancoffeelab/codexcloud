import { test } from "vitest";
import type { UnitType } from "../src/game/types.ts";
import { required } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
import { describeChanges } from "../src/controller/feedback.ts";
import { createGameView } from "./fixture-view.ts";
import { attackChoiceViews } from "../src/ui/combat-view.ts";
import assert from "node:assert/strict";
import { Game } from "./game-fixture.ts";
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
  const targetHp = target.hp;
  game.attack(attacker.id, target.id, "shard");
  assert.equal(target.hp, targetHp);
  assert.equal(attacker.hp, choice.playerForecast.remainingHp);
  assert(attacker.moved);
  assert.equal(attacker.remainingAttacks?.shard, 0);
  assert.throws(() => game.attack(attacker.id, target.id, "shard"));
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
  assert.equal(choice.playerForecast.remainingHp, attacker.hp);
});
test("Blocked strikes with unused attacks still produce combat feedback", () => {
  const { game, attacker } = fixture();
  const before = game.snapshot();
  const prior = required(before.units.find((unit) => unit.id === attacker.id));
  // Synthetic published budgets isolate feedback from current weapon statistics.
  prior.remainingAttacks = { strike: 3 };
  const after = structuredClone(before);
  const actor = required(after.units.find((unit) => unit.id === attacker.id));
  actor.remainingAttacks = { strike: 2 };
  actor.moved = true;
  assert(!actor.attacked);
  const feedback = describeChanges(before, after);
  assert(feedback.attacked, "zero-damage strikes still play their cue");
  assert.equal(feedback.attacker?.id, attacker.id);
});
// Lethal forecasts identify the unit that dies, including overkill and retaliation.
test("Lethal forecasts identify casualties including overkill", async () => {
  const { game, attacker, target } = fixture();
  const damage = game.previewDetails(attacker, target, "mace").damage;
  target.hp = damage;
  let choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, `Deal −${damage} HP (dies)`);
  assert.equal(choice.playerForecast.impactLabel, "Take 0 HP");
  target.hp = damage - 1;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(
    choice.enemyForecast.impactLabel,
    `Deal −${damage} HP (dies)`,
    "overkill is also lethal",
  );
  target.hp = damage + 1;
  const retaliation = game.previewDetails(attacker, target, "mace").retaliation;
  attacker.hp = retaliation;
  choice = required(
    attackChoiceViews(createGameView(game), attacker, target).find((c) => c.id === "mace"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.impactLabel, `Deal −${damage} HP`);
  assert.equal(choice.playerForecast.impactLabel, `Take −${retaliation} HP (dies)`);
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
