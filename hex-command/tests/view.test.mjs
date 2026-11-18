import { cellKey } from "./hex-fixture.mjs";
import * as NativeCatalog from "@frontend/Catalog";
import { test } from "vitest";
import assert from "node:assert/strict";
import { board, catalog, Catalog, Native, Presentation } from "./presentation-fixture.mjs";
import { required, setTerrain } from "./helpers.mjs";
import * as NativeBindings from "@frontend/Native";
import { fromJSON, toArray } from "./json-fixture.mjs";
test("Tile identity stays raw while gameplay roles stay finite through publication and JSON restore", () => {
  const game = board();
  const verify = () => {
    const native = toArray(NativeBindings.state(game.view).tiles);
    const json = game.state.tiles;
    for (let i = 0; i < native.length; i++) {
      assert.equal(native[i].terrain, json[i].terrain);
      const role = NativeCatalog.terrain_role(catalog, native[i].terrain);
      assert.notEqual(typeof role, "string");
      assert.equal(
        role,
        fromJSON("TerrainRole", Catalog.terrain_type(catalog, json[i].terrain).role),
      );
    }
  };
  verify();
  const saved = Native.save(game.battle);
  game.edit(() => {});
  assert.deepEqual(Native.save(game.battle), saved);
  verify();
});
test("Publications expose detached state and inspection without engine commands", () => {
  const game = board();
  for (const data of [game.state, Native.inspection(game.view)]) {
    for (const command of ["move", "attack", "recruit", "endTurn", "restoreState", "addUnit"])
      assert.equal(command in data, false);
  }
});
test("Queries preserve publication identities", () => {
  const game = board();
  const enemy = game.add("soldier", "enemy", "2,3");
  const soldier = required(Native.unit(game.view, "u2"));
  assert.equal(Native.at(game.view, cellKey(soldier.tile)), soldier);
  assert(
    Native.targets(game.view, soldier.id).includes(required(Native.unit(game.view, enemy.id))),
  );
});
test("Reachability dictionary copies leave native rule answers unchanged", () => {
  const game = board();
  const before = structuredClone(game.state);
  const reachable = Native.reachable(game.view, "u3");
  const expected = { ...reachable };
  assert.equal(Reflect.set(reachable, "0,0", 99), true);
  assert.deepEqual(Native.reachable(game.view, "u3"), expected);
  assert.deepEqual(game.state, before);
});
test("Later commands and checkpoint edits cannot rewrite an old publication", () => {
  const game = board();
  const old = game.view,
    before = structuredClone(game.state);
  Native.dispatch(game.battle, { kind: "move", unit: 3, tile: { col: 3, row: 1 } });
  game.edit((state) => {
    state.gold.player = 0;
    state.log[0] = "Later event";
    required(state.units.find((unit) => unit.id === "u2")).hp = 1;
    setTerrain(required(state.tiles.find((tile) => tile.id === "3,1")), "forest");
  });
  assert.deepEqual(Native.state(old), before);
  assert.equal(cellKey(required(Native.unit(old, "u3")).tile), "1,2");
  assert.equal(required(Native.tile(old, "3,1")).terrain, "village");
});
test("Indexed queries match native inspection and remain attached to their publication", () => {
  const game = board(),
    old = game.view,
    inspection = Native.inspection(old);
  const check = (view, source) => {
    for (const unit of source.units)
      assert.deepEqual(Native.unit_details(view, unit.unit), unit.details);
    const tiles = Native.state(view).tiles;
    for (const origin of tiles) {
      const offsets =
        origin.row % 2 === 0
          ? [
              [-1, -1],
              [0, -1],
              [-1, 0],
              [1, 0],
              [-1, 1],
              [0, 1],
            ]
          : [
              [0, -1],
              [1, -1],
              [-1, 0],
              [1, 0],
              [0, 1],
              [1, 1],
            ];
      const neighbors = tiles.filter((tile) =>
        offsets.some(
          ([col, row]) => tile.col === origin.col + col && tile.row === origin.row + row,
        ),
      );
      assert.deepEqual(
        Native.neighbors(view, origin.id).map((tile) => tile.id),
        neighbors.map((tile) => tile.id),
      );
    }
    for (const [side, kinds] of Object.entries(source.recruitment))
      assert.deepEqual(Native.recruitment_options(view, side), kinds);
    for (const unit of source.units)
      for (const [kind, tiles] of Object.entries(unit.recruitment))
        assert.deepEqual(
          Native.recruit_tiles(view, unit.unit, kind).map((tile) => tile.id),
          tiles.map(cellKey),
        );
    assert.deepEqual(Native.neighbors(view, "missing"), []);
    assert.deepEqual(Native.recruit_tiles(view, "missing", "missing"), []);
    for (const key of ["constructor", "__proto__"]) {
      assert.deepEqual(Native.neighbors(view, key), []);
      assert.deepEqual(Native.recruit_tiles(view, key, "soldier"), []);
      assert.deepEqual(Native.recruit_tiles(view, "u1", key), []);
    }
  };
  Native.dispatch(game.battle, { kind: "move", unit: 3, tile: { col: 3, row: 1 } });
  check(game.view, Native.inspection(game.view));
  // The retained publication's indexes are first forced after a newer view exists.
  check(old, inspection);
});
test("Recruitment uses the same publication and selection as terrain details", () => {
  const game = board();
  const terrain = required(Presentation.terrain_panel(catalog, game.view, void 0, "0,2"));
  assert(terrain.recruitment);
  assert.equal(terrain.recruitment.hint, "New recruits act next turn.");
  assert(terrain.recruitment.options.some((option) => option.enabled));
  assert.equal(
    required(Presentation.terrain_panel(catalog, game.view, "u1", "0,2")).recruitment,
    void 0,
  );
});
