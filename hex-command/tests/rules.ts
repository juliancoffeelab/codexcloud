import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game, TYPES, distance } from "../src/game/engine.ts";
let g = new Game();
assert.equal(g.tiles.length, 77);
assert.equal(g.units.length, 6);
assert.equal(g.tiles.filter((t) => t.terrain === "village").length, 4);
assert.equal(distance(required(g.tile("0,3")), required(g.tile("10,3"))), 10);
const soldier = required(g.unit("u2"));
assert(!g.reachable(soldier.id).has("0,0"));
assert(!g.reachable(soldier.id).has("0,3"));
assert(g.reachable(soldier.id).has("2,3"));
g.move(soldier.id, "2,3");
assert.throws(() => g.move(soldier.id, "3,3"));
g.endTurn();
assert.throws(() => g.move(soldier.id, "3,3"));
g.endTurn();
assert.equal(soldier.moved, false);
g = new Game();
const scout = required(g.unit("u3"));
scout.tile = "3,2";
required(g.unit("u2")).tile = "3,4";
required(g.tile("4,2")).terrain = "forest";
required(g.tile("4,4")).terrain = "forest";
assert.equal(g.reachable(scout.id).get("4,2"), 1);
assert.equal(g.reachable("u2").get("4,4"), 2);
g = new Game();
g.units = [required(g.unit("u1")), required(g.unit("u4"))];
const archer = g.addUnit("archer", "player", "2,1"),
  enemy = g.addUnit("soldier", "enemy", "3,1");
assert.deepEqual(g.preview(archer, enemy), { damage: 3, retaliation: 0 });
required(g.tile(enemy.tile)).terrain = "forest";
assert.equal(g.damage(archer, enemy), 2);
required(g.tile(enemy.tile)).terrain = "hill";
assert.equal(g.damage(archer, enemy), 2);
g = new Game();
g.units = [required(g.unit("u1")), required(g.unit("u4"))];
const melee = g.addUnit("soldier", "player", "2,1"),
  defender = g.addUnit("scout", "enemy", "3,1");
assert.deepEqual(g.preview(melee, defender), { damage: 4, retaliation: 1 });
defender.hp = 4;
assert.equal(g.preview(melee, defender).retaliation, 0);
g.attack(melee.id, defender.id);
assert.equal(g.unit(defender.id), undefined);
assert.equal(melee.attacked, true);
assert.equal(g.reachable(melee.id).size, 0);
g = new Game();
g.units = [required(g.unit("u1")), required(g.unit("u4"))];
const bow = g.addUnit("archer", "player", "2,1"),
  far = g.addUnit("scout", "enemy", "4,1");
for (const t of g.neighbors(bow.tile).filter((t) => distance(t, required(g.tile(far.tile))) === 1))
  t.terrain = "forest";
assert(!g.canAttackFrom(bow, bow.tile, far));
g.neighbors(bow.tile).filter((t) => distance(t, required(g.tile(far.tile))) === 1)[0]!.terrain =
  "plains";
assert(g.canAttackFrom(bow, bow.tile, far));
g = new Game();
const recruitTile = required(g.recruitTiles("u1", "archer")[0]).id;
const recruit = g.recruit("u1", "archer", recruitTile);
assert.equal(g.gold.player, 4);
assert(recruit.attacked && recruit.moved);
assert.throws(() => g.recruit("u1", "archer", recruitTile));
g.endTurn();
g.endTurn();
assert.equal(recruit.attacked, false);
g = new Game();
required(g.unit("u3")).tile = "2,1";
g.move("u3", "3,1");
assert.equal(required(g.tile("3,1")).owner, "player");
required(g.unit("u3")).hp = 4;
g.endTurn();
g.endTurn();
assert.equal(g.gold.player, 14);
assert.equal(required(g.unit("u3")).hp, 6);
required(g.unit("u6")).tile = "4,1";
g.endTurn();
g.move("u6", "3,2");
g.endTurn();
assert.equal(g.gold.player, 16);
g = new Game();
g.units = [required(g.unit("u1")), required(g.unit("u4"))];
required(g.unit("u1")).tile = "9,3";
required(g.unit("u4")).hp = 3;
g.attack("u1", "u4");
assert.equal(g.winner, "player");
assert.throws(() => g.endTurn());
let battles = 0,
  finished = 0;
for (let n = 0; n < 20; n++) {
  g = new Game();
  for (let round = 0; round < 35 && !g.winner; round++) {
    for (const u of [...g.units].filter((u) => u.side === "player")) {
      if (g.winner) break;
      const targets = g.targets(u.id);
      if (targets.length) {
        g.attack(u.id, required(targets[0]).id);
        continue;
      }
      const goals =
        u.type === "scout"
          ? g.tiles.filter((t) => t.terrain === "village" && t.owner !== "player")
          : g.units.filter((v) => v.side === "enemy").map((v) => required(g.tile(v.tile)));
      const options = [...g.reachable(u.id).keys()];
      options.sort(
        (a, b) =>
          Math.min(...goals.map((t) => distance(required(g.tile(a)), t))) -
          Math.min(...goals.map((t) => distance(required(g.tile(b)), t))),
      );
      if (options.length && goals.length) g.move(u.id, required(options[0]));
      const attacks = g.targets(u.id);
      if (attacks.length) g.attack(u.id, required(attacks[0]).id);
    }
    if (g.winner) break;
    g.endTurn();
    g.enemyRecruit();
    for (const u of [...g.units].filter((u) => u.side === "enemy")) {
      g.enemyAct(u.id);
      if (g.winner) break;
    }
    if (!g.winner) g.endTurn();
    assert.equal(new Set(g.units.map((u) => u.tile)).size, g.units.length);
    assert(g.units.every((u) => u.hp > 0 && u.hp <= TYPES[u.type].hp));
    assert(g.gold.player >= 0 && g.gold.enemy >= 0);
  }
  battles++;
  if (g.winner) finished++;
}
assert(finished > 0);
console.log(
  `PASS: terrain, legal movement, turn ownership, combat, bow blocking, retaliation, recruitment, income/healing, victory, and ${battles} simulated battles (${finished} completed).`,
);
