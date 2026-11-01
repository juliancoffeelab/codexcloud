import { test } from "vitest";
import type { UnitType } from "../src/game/types.ts";
import { required } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
import { attackChoiceViews } from "../src/ui/combat-view.ts";
import assert from "node:assert/strict";
import { Game, TYPES } from "./game-fixture.ts";
const fixture = (a: UnitType = "soldier", b: UnitType = "soldier", targetTile = "3,3") => {
  const game = new Game();
  for (const tile of game.tiles) {
    tile.terrain = "plains";
    delete tile.connections;
    tile.owner = null;
  }
  game.units = [];
  const attacker = game.addUnit(a, "player", "2,3"),
    target = game.addUnit(b, "enemy", targetTile);
  return { game, attacker, target };
};

test("Drain forecasts show expected healing and seeded combat replays exactly", () => {
  const vampire = required(
    (Object.keys(TYPES) as UnitType[]).find((type) =>
      TYPES[type].attacks.some((weapon) => weapon.drain),
    ),
  );
  const { game, attacker, target } = fixture(vampire, "soldier");
  attacker.hp = 5;
  const weapon = required(TYPES[vampire].attacks.find((weapon) => weapon.drain));
  const preview = game.previewDetails(attacker, target, weapon.id);
  assert((preview.healing ?? 0) > 0);
  const checkpoint = game.captureState();
  game.attack(attacker.id, target.id, weapon.id);
  const result = game.captureState();
  game.restoreState(checkpoint);
  game.attack(attacker.id, target.id, weapon.id);
  assert.deepEqual(game.captureState(), result);
});

test("Thorn forecasts show separate reflection even on a killing hit", () => {
  const thornType = required(
    (Object.keys(TYPES) as UnitType[]).find((type) => (TYPES[type].thorns ?? 0) > 0),
  );
  const { game, attacker, target } = fixture("soldier", thornType);
  target.hp = 1;
  const weapon = required(
    TYPES[attacker.type].attacks.find((attack) => attack.category === "melee"),
  );
  const preview = game.previewDetails(attacker, target, weapon.id);
  const choice = required(
    attackChoiceViews(game.publishedView(), attacker, target).find(
      (choice) => choice.id === weapon.id,
    ),
  );
  assert(choice.available);
  assert.equal(preview.thornDamage, required(TYPES[thornType].thorns));
  assert.equal(preview.response, null);
  assert(choice.retaliationCalculation.equation.includes(`${preview.thornDamage} (thorns:`));
  game.attack(attacker.id, target.id, weapon.id);
  assert.equal(game.unit(attacker.id)?.hp ?? 0, choice.playerForecast.remainingHp);
  assert.equal(game.unit(target.id), undefined);
});

test("Zero damage attacks remain available and consume actions", async () => {
  const { game, attacker, target } = fixture("bonearcher", "soldier");
  required(game.tile(target.tile)).terrain = "hill";
  const published = game.publishedView();
  const choice = required(
    attackChoiceViews(published, attacker, target).find((c) => c.id === "shard"),
  );
  assert(choice.available);
  assert.equal(choice.enemyForecast.remainingHp, target.hp);
  const targetHp = target.hp;
  const attackerHp = attacker.hp;
  const preview = game.previewDetails(attacker, target, "shard");
  game.attack(attacker.id, target.id, "shard");
  assert.equal(target.hp, targetHp);
  assert(attacker.hp >= Math.max(0, attackerHp - preview.retaliationMax));
  assert(attacker.hp <= Math.max(0, attackerHp - preview.retaliationMin));
  assert(attacker.moved && attacker.attacked);
  assert.throws(() => game.attack(attacker.id, target.id, "shard"));
});
test("Covered retaliation includes a zero-damage outcome", async () => {
  const { game, attacker, target } = fixture("commander", "bonearcher");
  required(game.tile(attacker.tile)).terrain = "hill";
  const preview = game.previewDetails(attacker, target, "sword");
  assert.equal(preview.retaliationMin, 0);
  assert.equal(preview.response?.id, "shard");
  const choice = required(
    attackChoiceViews(game.publishedView(), attacker, target).find((c) => c.id === "sword"),
  );
  assert(choice.available);
  assert(choice.retaliationCalculation.description.includes("shard"));
  assert.equal(preview.attackerHealth.maximum, attacker.hp);
});
test("Forecasts distinguish guaranteed kills, uncertain kills and lethal retaliation", () => {
  const { game, attacker, target } = fixture();
  const preview = game.previewDetails(attacker, target, "mace");
  const minimum = required(preview.damageDetail.minimum);
  const maximum = required(preview.damageDetail.maximum);
  assert(minimum > 0 && maximum > minimum);
  const choice = () => {
    const view = required(
      attackChoiceViews(game.publishedView(), attacker, target).find((c) => c.id === "mace"),
    );
    assert(view.available);
    return view;
  };
  target.hp = minimum;
  assert(choice().enemyForecast.impactLabel.includes("(dies)"));
  assert.equal(choice().enemyForecast.remainingClass, "fatal");
  target.hp = preview.damage;
  const uncertain = game.previewDetails(attacker, target, "mace");
  assert(uncertain.killChance > 0 && uncertain.killChance < 1);
  assert.equal(choice().enemyForecast.remainingClass, "");
  assert(uncertain.response);
  target.hp = maximum + 1;
  attacker.hp = 1;
  assert(choice().playerForecast.impactLabel.includes("(dies)"));
  assert.equal(choice().playerForecast.remainingClass, "fatal");
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

test("Counted damage descriptions use native per-hit and total metadata", () => {
  assert.equal(
    damageExplanation({
      base: 2,
      modifier: -1,
      modified: 1,
      protection: 0,
      exposure: 0,
      damage: 1,
      count: 3,
      total: 3,
    }),
    "(2 − 1 (armor)) × 3 = 3",
  );
});
