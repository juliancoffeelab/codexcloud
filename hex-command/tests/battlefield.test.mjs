import { cellKey } from "./hex-fixture.mjs";
import { test, vi } from "vitest";
import assert from "node:assert/strict";
import { board, campaign, catalog, Native, Presentation } from "./presentation-fixture.mjs";
import { required } from "./helpers.mjs";
import * as Motion from "@frontend/Motion";
import * as Domain from "@frontend/Domain";

const tiles = (view, selected, inspected) =>
  Presentation.battlefield(catalog, view, selected, inspected);
test("Selection and attack targets have accessible labels and health", () => {
  const game = board();
  const enemy = game.add("archer", "enemy", "2,3");
  const display = tiles(game.view, "u2");
  assert.equal(display.length, game.state.tiles.length);
  const soldier = required(display.find((entry) => entry.tile.id === "1,3"));
  assert(soldier.classes.includes("selected"));
  assert.equal(soldier.occupant, required(Native.unit(game.view, "u2")));
  const target = required(display.find((entry) => entry.tile.id === cellKey(enemy.tile)));
  assert(target.classes.includes("target"));
  assert(target.label.includes("attack available"));
  assert.equal(target.health_width, 38);
  assert(!target.label.includes("Objective:"));
});
test("Recruitment highlights only inspected eligible ground", () => {
  const game = board();
  const display = tiles(game.view, void 0, "0,2");
  assert(
    required(display.find((entry) => entry.tile.id === "0,2")).classes.includes("recruitable"),
  );
  assert(!display.some((entry) => /selected|reachable|target/.test(entry.classes)));
  assert(!tiles(game.view, void 0, "0,3").some((entry) => entry.classes.includes("recruitable")));
});
test("Moving preserves tile identities and leaves the old publication unchanged", () => {
  const game = board();
  const oldView = game.view;
  const before = tiles(oldView, "u2");
  Native.dispatch(game.battle, { kind: "move", unit: 2, tile: { col: 2, row: 2 } });
  const after = tiles(game.view, "u2");
  assert.equal(required(after.find((entry) => entry.tile.id === "1,3")).occupant, void 0);
  assert.equal(
    required(required(after.find((entry) => entry.tile.id === "2,2")).occupant).id,
    "u2",
  );
  assert.deepEqual(
    after.map((entry) => entry.tile),
    before.map((entry) => entry.tile),
  );
  assert.equal(cellKey(required(Native.unit(oldView, "u2")).tile), "1,3");
});
for (const id of ["lantern-road"]) {
  test(`Campaign objectives and named companions have accessible descriptions: ${id}`, () => {
    const game = campaign(id);
    const display = tiles(game.view);
    for (const [index, status] of Native.inspection(game.view).objectives.goals.entries()) {
      if (!status.visible) continue;
      for (const id of status.activeTiles) {
        const tile = required(display.find((entry) => entry.tile.id === cellKey(id)));
        assert(tile.classes.includes("objective"));
        assert(tile.label.includes(`Objective: ${status.progress.label}`));
      }
    }
    for (const unit of game.state.units.filter((unit2) => unit2.name)) {
      assert(
        required(display.find((entry) => entry.tile.id === cellKey(unit.tile))).label.includes(
          required(unit.name),
        ),
      );
    }
  });
}
test("Motion forwards a typed presentation event to the native battlefield without mutating DOM leaves", () => {
  const events = [];
  vi.stubGlobal(
    "CustomEvent",
    class {
      constructor(type, options) {
        this.type = type;
        this.detail = options.detail;
      }
    },
  );
  vi.stubGlobal("document", {
    querySelector: (id) =>
      id === "#map[data-phaser-battlefield]"
        ? { dispatchEvent: (event) => events.push(event) }
        : null,
  });
  try {
    for (const kind of ["select", "move", "recruit", "attack", "capture"])
      Motion.animate_tile(undefined, "1,3", kind);
    assert.deepEqual(
      events.map((e) => [e.type, e.detail.tile, e.detail.kind]),
      ["select", "move", "recruit", "attack", "capture"].map((kind) => [
        "battlefieldmotion",
        "1,3",
        kind,
      ]),
    );
  } finally {
    vi.unstubAllGlobals();
  }
});
