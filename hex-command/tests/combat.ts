import type { UnitType } from "../src/game/types.ts";
import { required, storageWithValue } from "./helpers.ts";
import { damageExplanation } from "../src/presentation/combat.ts";
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
{
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
}
{
  const { game, attacker, target } = fixture("archer", "soldier");
  assert.deepEqual(game.preview(attacker, target, "bow"), {
    damage: 2,
    retaliation: 0,
  });
  assert.deepEqual(game.preview(attacker, target, "knife"), {
    damage: 2,
    retaliation: 4,
  });
  assert.equal(required(game.bestAttack(attacker, target)).id, "bow");
  game.attack(attacker.id, target.id, "knife");
  assert.equal(required(game.lastCombat).sound, "attack-melee");
  assert.equal(attacker.hp, 4);
  assert.equal(target.hp, 10);
  assert.throws(() => game.attack(attacker.id, target.id, "bow"));
}
{
  const { game, attacker, target } = fixture("archer", "archer", "4,3");
  assert.deepEqual(game.preview(attacker, target, "bow"), {
    damage: 3,
    retaliation: 3,
  });
  assert.equal(game.canAttackFrom(attacker, attacker.tile, target, "knife"), false);
  assert.throws(() => game.preview(attacker, target, "knife"), /cannot reach/);
  for (const tile of game
    .neighbors(attacker.tile)
    .filter((t) => distance(t, required(game.tile(target.tile))) === 1))
    tile.terrain = "hill";
  assert.equal(game.canAttackFrom(attacker, attacker.tile, target, "bow"), false);
  assert.throws(() => game.attack(attacker.id, target.id, "bow"));
}
{
  const { game, attacker, target } = fixture("scout", "archer");
  assert.deepEqual(game.preview(attacker, target, "sling"), {
    damage: 1,
    retaliation: 3,
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
}
// Every legal weapon across unit pairs and cover must exactly match resolution.
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
          assert(preview.damage >= 1 && Number.isInteger(preview.damage));
          assert(preview.retaliation >= 0 && Number.isInteger(preview.retaliation));
          assert(!preview.response || preview.response.category === weapon.category);
          game.attack(attacker.id, target.id, weapon.id);
          assert.equal(attacker.hp, TYPES[a].hp - preview.retaliation);
          assert.equal(target.hp, TYPES[b].hp - preview.damage);
          assert(attacker.moved && attacker.attacked);
          exchanges++;
        }
{
  const { game, attacker, target } = fixture();
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert.equal(required(game.lastCombat).attackId, "mace", "enemy exploits armor weakness");
  assert.equal(attacker.hp, 8);
}
{
  const { game, attacker, target } = fixture("archer", "archer", "4,3");
  target.hp = 3;
  game.turn = "enemy";
  game.enemyAct(target.id);
  assert(
    required(game.unit(target.id)),
    "enemy should reposition instead of accepting a lethal ranged reply",
  );
  assert.equal(required(game.lastCombat).attackId, "knife");
  assert.equal(target.hp, 1);
}
// Definitions supply new weapons to old saves without replacing battle progress.
{
  const game = new Game();
  game.move("u3", "3,1");
  let stored = "";
  globalThis.localStorage = storageWithValue(
    () => stored,
    (value) => {
      stored = value;
    },
  );
  saveBattle(game);
  const restored = loadBattle().game;
  assert.deepEqual(restored.snapshot(), game.snapshot());
  assert.equal(TYPES[required(restored.unit("u3")).type].attacks.length, 2);
  assert.equal(TYPES[required(restored.unit("u2")).type].resistances.impact, -25);
}
console.log(
  `PASS: weapon choices, armor, matching replies, range/blocking, deaths, AI and old saves; ${exchanges} exact preview/resolution exchanges`,
);

assert.equal(
  damageExplanation({
    base: 2,
    resistance: 25,
    resisted: 2,
    protection: 0,
    exposure: 0,
    damage: 2,
  }),
  "round(2 × 75%) = 2",
);
assert.equal(
  damageExplanation({
    base: 3,
    resistance: 25,
    resisted: 2,
    protection: 1,
    exposure: 0,
    damage: 1,
  }),
  "round(3 × 75%) − 1 (cover) = 1",
);
assert.equal(
  damageExplanation({
    base: 1,
    resistance: 0,
    resisted: 1,
    protection: 1,
    exposure: 0,
    damage: 1,
  }),
  "max(1, round(1 × 100%) − 1 (cover)) = 1",
);
assert.equal(
  damageExplanation({
    base: 3,
    resistance: -25,
    resisted: 4,
    protection: 0,
    exposure: 1,
    damage: 5,
  }),
  "round(3 × 125%) + 1 (shallow water) = 5",
);
console.log("PASS: numeric equations preserve armor rounding, cover, minimum damage and exposure");
