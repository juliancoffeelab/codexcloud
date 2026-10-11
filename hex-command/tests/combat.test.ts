import type { PublicUnit } from "../rust/pkg/hex_command";
import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture";
import { required } from "./helpers";
function fixture(attackerType = "soldier", defenderType = "soldier", targetTile = "3,3") {
  const game = board();
  game.edit((state) => {
    state.units = [];
    for (const tile of state.tiles) {
      tile.terrain = "plains";
      tile.owner = null;
    }
  });
  const attacker = game.add(attackerType, "player", "2,3");
  const defender = game.add(defenderType, "enemy", targetTile);
  return { game, attacker, defender };
}
function choice(
  game: ReturnType<typeof board>,
  attacker: Native.Immutable<PublicUnit>,
  defender: Native.Immutable<PublicUnit>,
  weapon: string,
) {
  return required(
    Presentation.attacks(
      catalog,
      game.view,
      required(Native.unit(game.view, attacker.id)),
      required(Native.unit(game.view, defender.id)),
    ).find((choice) => choice.weapon.id === weapon),
  );
}
test("Zero damage attacks remain available and consume actions", () => {
  const { game, attacker, defender } = fixture("bonearcher", "soldier");
  game.edit(
    (state) => (required(state.tiles.find((tile) => tile.id === defender.tile)).terrain = "hill"),
  );
  const selected = choice(game, attacker, defender, "shard");
  assert(selected.result);
  assert.equal(required(selected.result).enemy.remaining_hp, defender.hp);
  assert.equal(selected.result.attack.equation, "2 − 1 (armor) − 1 (cover) = 0");
  Native.dispatch(game.battle, {
    kind: "attack",
    unit: Native.unit_id(attacker.id),
    target: Native.unit_id(defender.id),
    weapon: "shard",
  });
  assert.equal(required(Native.unit(game.view, defender.id)).hp, 24);
  assert.equal(required(Native.unit(game.view, attacker.id)).hp, 8);
  assert(
    required(Native.unit(game.view, attacker.id)).moved &&
      required(Native.unit(game.view, attacker.id)).attacked,
  );
  assert.throws(() =>
    Native.dispatch(game.battle, {
      kind: "attack",
      unit: Native.unit_id(attacker.id),
      target: Native.unit_id(defender.id),
      weapon: "shard",
    }),
  );
  assert.deepEqual(
    Presentation.armor(catalog, defender).map((modifier) => modifier.label),
    ["−1", "−1", "+1"],
  );
});
test("Covered retaliation can deal zero damage", () => {
  const { game, attacker, defender } = fixture("commander", "bonearcher");
  game.edit(
    (state) => (required(state.tiles.find((tile) => tile.id === attacker.tile)).terrain = "hill"),
  );
  const selected = choice(game, attacker, defender, "sword");
  assert(selected.result);
  assert(selected.result.retaliation.description.includes("shard"));
  assert.equal(selected.result.retaliation.equation, "2 − 1 (armor) − 1 (cover) = 0");
});
test("Blocked attacks still produce combat feedback", () => {
  const { game, attacker, defender } = fixture("bonearcher", "commander");
  game.edit(
    (state) => (required(state.tiles.find((tile) => tile.id === defender.tile)).terrain = "hill"),
  );
  const before = game.state;
  Native.dispatch(game.battle, {
    kind: "attack",
    unit: Native.unit_id(attacker.id),
    target: Native.unit_id(defender.id),
    weapon: "shard",
  });
  assert.equal(required(Native.unit(game.view, defender.id)).hp, 32);
  assert.equal(required(Native.unit(game.view, attacker.id)).hp, 8);
  assert(Presentation.changes(before, game.state).attacked);
});
test("Lethal forecasts identify casualties, overkill and retaliation", () => {
  const { game, attacker, defender } = fixture();
  for (const hp of [4, 3]) {
    game.edit((state) => (required(state.units.find((unit) => unit.id === defender.id)).hp = hp));
    const selected = choice(game, attacker, defender, "mace");
    assert.equal(required(selected.result).enemy.impact_label, "Deal −7 HP (dies)");
    assert.equal(required(selected.result).player.impact_label, "Take 0 HP");
  }
  game.edit((state) => {
    required(state.units.find((unit) => unit.id === defender.id)).hp = 8;
    required(state.units.find((unit) => unit.id === attacker.id)).hp = 7;
  });
  const selected = choice(game, attacker, defender, "mace");
  assert.equal(required(selected.result).enemy.impact_label, "Deal −7 HP");
  assert.equal(required(selected.result).player.impact_label, "Take −7 HP (dies)");
});
test("Restored checkpoints retain progress while inheriting current weapons", () => {
  const game = board();
  Native.dispatch(game.battle, { kind: "move", unit: 3, tile: "3,1" });
  const before = game.state;
  const saved = Native.save(game.battle);
  Native.replace(game.battle, saved);
  assert.deepEqual(game.state, before);
  assert.equal(
    Catalog.unit(catalog, required(Native.unit(game.view, "u3")).type).attacks.length,
    2,
  );
  assert.equal(
    Catalog.unit(catalog, required(Native.unit(game.view, "u2")).type).damageModifiers.impact,
    1,
  );
});
for (const [name, detail, expected] of [
  [
    "Armor equation",
    { base: 2, modifier: -1, modified: 1, protection: 0, exposure: 0, damage: 1 },
    "2 − 1 (armor) = 1",
  ],
  [
    "Armor and cover equation",
    { base: 3, modifier: -1, modified: 2, protection: 1, exposure: 0, damage: 1 },
    "3 − 1 (armor) − 1 (cover) = 1",
  ],
  [
    "Zero damage from cover",
    { base: 1, modifier: 0, modified: 1, protection: 1, exposure: 0, damage: 0 },
    "1 − 1 (cover) = 0",
  ],
  [
    "Weakness and shallow water exposure",
    { base: 3, modifier: 1, modified: 4, protection: 0, exposure: 1, damage: 5 },
    "3 + 1 (weakness) + 1 (shallow water) = 5",
  ],
  [
    "Negative damage clamped to zero",
    { base: 1, modifier: -1, modified: 0, protection: 1, exposure: 0, damage: 0 },
    "max(0, 1 − 1 (armor) − 1 (cover)) = 0",
  ],
] as const)
  test(name, () =>
    assert.equal(
      Presentation.damage_equation({ ...detail, damageType: "blade", terrain: "" }),
      expected,
    ),
  );
