import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture";
import { required } from "./helpers";
test("Resolved native definitions cannot be mutated", () => {
  assert.throws(
    () => Object.assign(required(Catalog.unit(catalog, "soldier").attacks[0]), { damage: 99 }),
    TypeError,
  );
  assert.throws(
    () => Object.assign(Catalog.terrain_type(catalog, "plains"), { name: "Changed" }),
    TypeError,
  );
  assert.throws(
    () => Object.assign(required(Catalog.map(catalog, "mirror").terrain[0]), { terrain: "forest" }),
    TypeError,
  );
});
test("Publications expose frozen state and inspection without engine commands", () => {
  const game = board();
  for (const data of [game.state, Native.inspection(game.view)]) {
    for (const command of ["move", "attack", "recruit", "endTurn", "restoreState", "addUnit"])
      assert.equal(command in data, false);
    assert(Object.isFrozen(data));
  }
  for (const value of [
    game.state.gold,
    game.state.log,
    game.state.units,
    game.state.tiles,
    ...game.state.units,
    ...game.state.tiles,
  ])
    assert(Object.isFrozen(value));
  assert.throws(() => Object.assign(required(Native.unit(game.view, "u2")), { hp: 0 }), TypeError);
  assert.throws(
    () => Object.assign(required(Native.tile(game.view, "3,1")), { owner: "enemy" }),
    TypeError,
  );
  assert.equal(Reflect.set(game.state.gold, "player", 0), false);
  assert.equal(Reflect.set(game.state.units, "length", 0), false);
});
test("Queries preserve publication identities and frozen forecasts", () => {
  const game = board();
  const enemy = game.add("soldier", "enemy", "2,3");
  const soldier = required(Native.unit(game.view, "u2"));
  assert.equal(Native.at(game.view, soldier.tile), soldier);
  assert(
    Native.targets(game.view, soldier.id).includes(required(Native.unit(game.view, enemy.id))),
  );
  assert(Object.isFrozen(Native.targets(game.view, soldier.id)));
  assert(Object.isFrozen(Native.neighbors(game.view, soldier.tile)));
  assert(Object.isFrozen(Native.recruit_tiles(game.view, "u1", "soldier")));
  const details = required(Native.attack_details(game.view, soldier.id, enemy.id, "sword"));
  for (const value of [
    details,
    details.attack,
    details.damageDetail,
    details.response,
    details.retaliationDetail,
  ])
    if (value) assert(Object.isFrozen(value));
  assert.throws(() => Object.assign(details.attack, { damage: 99 }), TypeError);
});
test("Reachability queries cannot rewrite published rule answers", () => {
  const game = board();
  const before = structuredClone(game.state);
  const reachable = Native.reachable(game.view, "u3");
  const expected = { ...reachable };
  assert.equal(Reflect.set(reachable, "0,0", 99), false);
  assert.deepEqual(Native.reachable(game.view, "u3"), expected);
  assert.deepEqual(game.state, before);
});
test("Later commands and checkpoint edits cannot rewrite an old publication", () => {
  const game = board();
  const old = game.view,
    before = structuredClone(game.state);
  Native.dispatch(game.battle, { kind: "move", unit: 3, tile: "3,1" });
  game.edit((state) => {
    state.gold.player = 0;
    state.log[0] = "Later event";
    required(state.units.find((unit) => unit.id === "u2")).hp = 1;
    required(state.tiles.find((tile) => tile.id === "3,1")).terrain = "forest";
  });
  assert.deepEqual(Native.state(old), before);
  assert.equal(required(Native.unit(old, "u3")).tile, "1,2");
  assert.equal(required(Native.tile(old, "3,1")).terrain, "village");
});
test("Recruitment uses the same publication and selection as terrain details", () => {
  const game = board();
  const terrain = required(Presentation.terrain_panel(catalog, game.view, undefined, "0,2"));
  assert(terrain.recruitment);
  assert.equal(terrain.recruitment.hint, "New recruits act next turn.");
  assert(terrain.recruitment.options.some((option) => option.enabled));
  assert.equal(
    required(Presentation.terrain_panel(catalog, game.view, "u1", "0,2")).recruitment,
    undefined,
  );
});
