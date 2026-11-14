import { cellKey } from "./hex-fixture.mjs";
import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture.mjs";
import { required, setTerrain } from "./helpers.mjs";
function fixture(attackerType = "soldier", defenderType = "soldier", targetTile = "3,3") {
  const game = board();
  game.edit((state) => {
    state.units = [];
    for (const tile of state.tiles) {
      setTerrain(tile, "plains");
      tile.owner = null;
    }
  });
  const attacker = game.add(attackerType, "player", "2,3");
  const defender = game.add(defenderType, "enemy", targetTile);
  return { game, attacker, defender };
}
function choice(game, attacker, defender, weapon) {
  return required(
    Presentation.attacks(
      catalog,
      game.view,
      required(Native.unit(game.view, attacker.id)),
      required(Native.unit(game.view, defender.id)),
    ).find((choice2) => choice2.weapon.id === weapon),
  );
}
test("Zero damage attacks remain available and consume actions", () => {
  const { game, attacker, defender } = fixture("bonearcher", "soldier");
  game.edit(
    (state) =>
      (required(state.tiles.find((tile) => tile.id === cellKey(defender.tile))).terrain = "hill"),
  );
  const selected = choice(game, attacker, defender, "shard");
  assert(selected.result);
  assert.equal(required(selected.result).enemy.remaining_hp, defender.hp);
  const beforeView = game.view;
  Native.dispatch(game.battle, {
    kind: "attack",
    unit: Native.unit_id(attacker.id),
    target: Native.unit_id(defender.id),
    weapon: "shard",
  });
  assert.equal(required(Native.unit(game.view, defender.id)).hp, defender.hp);
  const health = required(Native.unit(game.view, attacker.id)).hp;
  const preview = required(Native.attack_details(beforeView, attacker.id, defender.id, "shard"));
  assert(health >= preview.attackerHealth.minimum && health <= preview.attackerHealth.maximum);
  assert(
    required(Native.unit(game.view, attacker.id)).moved &&
      required(Native.unit(game.view, attacker.id)).attackSpent,
  );
  assert.throws(() =>
    Native.dispatch(game.battle, {
      kind: "attack",
      unit: Native.unit_id(attacker.id),
      target: Native.unit_id(defender.id),
      weapon: "shard",
    }),
  );
});
test("Covered retaliation includes a zero-damage outcome", () => {
  const { game, attacker, defender } = fixture("commander", "bonearcher");
  game.edit(
    (state) =>
      (required(state.tiles.find((tile) => tile.id === cellKey(attacker.tile))).terrain = "hill"),
  );
  const selected = choice(game, attacker, defender, "sword");
  assert(selected.result);
  assert(selected.result.retaliation.description.includes("shard"));
  assert.equal(
    required(Native.attack_details(game.view, attacker.id, defender.id, "sword")).attackerHealth
      .maximum,
    attacker.hp,
  );
});
test("Forecasts distinguish guaranteed kills, uncertain kills and lethal retaliation", () => {
  const { game, attacker, defender } = fixture();
  const preview = required(Native.attack_details(game.view, attacker.id, defender.id, "mace"));
  const { minimum, maximum } = preview.damageDetail;
  assert(minimum > 0 && maximum > minimum);
  const update = (hp, attackerHp) =>
    game.edit((state) => {
      required(state.units.find((unit) => unit.id === defender.id)).hp = hp;
      if (attackerHp !== undefined)
        required(state.units.find((unit) => unit.id === attacker.id)).hp = attackerHp;
    });
  update(minimum);
  assert(
    required(choice(game, attacker, defender, "mace").result).enemy.impact_label.includes("(dies)"),
  );
  assert.equal(
    required(choice(game, attacker, defender, "mace").result).enemy.remaining_class,
    "fatal",
  );
  update(preview.damage);
  const uncertain = required(Native.attack_details(game.view, attacker.id, defender.id, "mace"));
  assert(uncertain.killChance > 0 && uncertain.killChance < 1);
  assert.equal(required(choice(game, attacker, defender, "mace").result).enemy.remaining_class, "");
  assert(uncertain.response);
  update(maximum + 1, 1);
  assert.equal(
    required(choice(game, attacker, defender, "mace").result).player.remaining_class,
    "fatal",
  );
});
test("Restored checkpoints retain the detached battle publication", () => {
  const game = board();
  Native.dispatch(game.battle, { kind: "move", unit: 3, tile: { col: 3, row: 1 } });
  const before = game.state;
  const saved = Native.save(game.battle);
  Native.replace(game.battle, saved);
  assert.deepEqual(game.state, before);
});
for (const [name, detail, expected] of [
  [
    "Armor equation",
    { base: 2, modifier: -1, modified: 1, protection: 0, exposure: 0, damage: 1 },
    "2 \u2212 1 (armor) = 1",
  ],
  [
    "Armor and cover equation",
    { base: 3, modifier: -1, modified: 2, protection: 1, exposure: 0, damage: 1 },
    "3 \u2212 1 (armor) \u2212 1 (cover) = 1",
  ],
  [
    "Zero damage from cover",
    { base: 1, modifier: 0, modified: 1, protection: 1, exposure: 0, damage: 0 },
    "1 \u2212 1 (cover) = 0",
  ],
  [
    "Weakness and shallow water exposure",
    { base: 3, modifier: 1, modified: 4, protection: 0, exposure: 1, damage: 5 },
    "3 + 1 (weakness) + 1 (shallow water) = 5",
  ],
  [
    "Variable rolls retain equal blocked total bounds",
    {
      base: 1,
      modifier: -3,
      modified: -2,
      protection: 0,
      exposure: 0,
      damage: 0,
      rollMin: 0,
      rollMax: 2,
      minimum: 0,
      maximum: 0,
      expected: 0,
    },
    "max(0, 0–2 − 3 (armor)) = 0–0 (average 0.0)",
  ],
  [
    "Negative damage clamped to zero",
    { base: 1, modifier: -1, modified: 0, protection: 1, exposure: 0, damage: 0 },
    "max(0, 1 \u2212 1 (armor) \u2212 1 (cover)) = 0",
  ],
])
  test(name, () =>
    assert.equal(
      Presentation.damage_equation({
        count: 1,
        total: detail.damage,
        rollMin: detail.base,
        rollMax: detail.base,
        minimum: detail.damage,
        maximum: detail.damage,
        expected: detail.damage,
        probabilities: [1],
        ...detail,
        damageType: "blade",
        terrain: "",
      }),
      expected,
    ),
  );

test("Resolved combat publishes detached strikes and restoring its checkpoint replays the same rolls", () => {
  const { game, attacker, defender } = fixture();
  const checkpoint = Native.save(game.battle);
  const attack = {
    kind: "attack",
    unit: Native.unit_id(attacker.id),
    target: Native.unit_id(defender.id),
    weapon: "mace",
  };
  const publication = Native.dispatch(game.battle, attack);
  assert.equal(publication, game.view, "A successful command returns its accepted detached view");
  const combat = required(Native.combat(publication));
  assert(combat.strikes.length > 0);
  assert.equal(combat.units[0].hp, attacker.hp);
  const last = combat.strikes.at(-1);
  assert.deepEqual(last.health, [
    Native.unit(publication, attacker.id)?.hp ?? 0,
    Native.unit(publication, defender.id)?.hp ?? 0,
  ]);
  const resolved = Native.save(game.battle);
  const restored = Native.replace(game.battle, checkpoint);
  assert.equal(restored, game.view);
  assert.equal(Native.combat(restored), undefined);
  Native.dispatch(game.battle, attack);
  assert.deepEqual(Native.combat(game.view), combat);
  assert.deepEqual(Native.save(game.battle), resolved);
  Native.dispatch(game.battle, { kind: "endTurn" });
  assert.equal(Native.combat(game.view), undefined);
  assert.deepEqual(Native.combat(publication), combat);
});
