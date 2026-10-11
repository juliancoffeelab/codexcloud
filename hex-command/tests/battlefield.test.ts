import { test } from "vitest";
import assert from "node:assert/strict";
import { board, campaign, catalog, Native, Presentation, Story } from "./presentation-fixture";
import { required } from "./helpers";
import * as Motion from "@frontend/Motion";

const tiles = (view: Native.view, selected?: string, inspected?: string) =>
  Presentation.battlefield(catalog, view, selected, inspected);
test("Selection and attack targets have accessible labels and health", () => {
  const game = board();
  const enemy = game.add("archer", "enemy", "2,3");
  const display = tiles(game.view, "u2");
  assert.equal(display.length, game.state.tiles.length);
  const soldier = required(display.find((entry) => entry.tile.id === "1,3"));
  assert(soldier.classes.includes("selected"));
  assert.equal(soldier.occupant, required(Native.unit(game.view, "u2")));
  const target = required(display.find((entry) => entry.tile.id === enemy.tile));
  assert(target.classes.includes("target"));
  assert(target.label.includes("attack available"));
  assert.equal(target.health_width, 38);
  assert(!target.label.includes("Objective:"));
});
test("Recruitment highlights only inspected eligible ground", () => {
  const game = board();
  const display = tiles(game.view, undefined, "0,2");
  assert(
    required(display.find((entry) => entry.tile.id === "0,2")).classes.includes("recruitable"),
  );
  assert(!display.some((entry) => /selected|reachable|target/.test(entry.classes)));
  assert(
    !tiles(game.view, undefined, "0,3").some((entry) => entry.classes.includes("recruitable")),
  );
});
test("Moving preserves tile identities and leaves the old publication unchanged", () => {
  const game = board();
  const oldView = game.view;
  const before = tiles(oldView, "u2");
  Native.dispatch(game.battle, { kind: "move", unit: 2, tile: "2,2" });
  const after = tiles(game.view, "u2");
  assert.equal(required(after.find((entry) => entry.tile.id === "1,3")).occupant, undefined);
  assert.equal(
    required(required(after.find((entry) => entry.tile.id === "2,2")).occupant).id,
    "u2",
  );
  assert.deepEqual(
    after.map((entry) => entry.tile),
    before.map((entry) => entry.tile),
  );
  assert.equal(required(Native.unit(oldView, "u2")).tile, "1,3");
});
for (const id of ["lantern-road", "green-accord", "ashen-oath"]) {
  test(`Campaign objectives and named companions have accessible descriptions: ${id}`, () => {
    const game = campaign(id);
    const display = tiles(game.view);
    for (const goal of Story.mission(catalog, required(game.state.campaign)).source.goals) {
      const progress = Story.progress(catalog, game.view, goal);
      for (const id of progress.tiles) {
        const tile = required(display.find((entry) => entry.tile.id === id));
        assert(tile.classes.includes("objective"));
        assert(tile.label.includes(`Objective: ${progress.label}`));
      }
    }
    for (const unit of game.state.units.filter((unit) => unit.name)) {
      assert(
        required(display.find((entry) => entry.tile.id === unit.tile)).label.includes(
          required(unit.name),
        ),
      );
    }
  });
}
test("Completed seal markers disappear while reversible territory remains visible", () => {
  const game = campaign("lantern-road", 2);
  game.edit((state) => state.campaign!.marks.push("sealed:9,1"));
  const display = tiles(game.view);
  assert(!required(display.find((entry) => entry.tile.id === "9,1")).classes.includes("objective"));
  assert(required(display.find((entry) => entry.tile.id === "11,4")).classes.includes("objective"));
  const escort = campaign("green-accord", 2);
  escort.edit(
    (state) => (required(state.tiles.find((tile) => tile.id === "7,4")).owner = "player"),
  );
  assert(
    required(tiles(escort.view).find((entry) => entry.tile.id === "7,4")).classes.includes(
      "objective",
    ),
  );
  escort.edit((state) => (state.winner = "player"));
  assert(!tiles(escort.view).some((entry) => entry.classes.includes("objective")));
});

function feedbackNode(attribute: string, tileId: string, occupied = false) {
  const classes = new Set<string>();
  return {
    classes,
    classList: {
      add: (name: string) => classes.add(name),
      remove: (name: string) => classes.delete(name),
    },
    getAttribute: (name: string) => (name === attribute ? tileId : null),
    getBoundingClientRect: () => ({}),
    querySelector: (selector: string) => (occupied && selector === ".unit-art" ? {} : null),
  };
}
test("Selection, unit motion and capture target independent paint layers", () => {
  const ground = feedbackNode("data-tile", "1,3"),
    unit = feedbackNode("data-unit-tile", "1,3", true),
    indicator = feedbackNode("data-indicator-tile", "1,3"),
    empty = feedbackNode("data-unit-tile", "2,2");
  const layers = new Map([
    ["#map [data-tile]", [ground]],
    ["#map [data-unit-tile]", [unit, empty]],
    ["#map [data-indicator-tile]", [indicator]],
  ]);
  const document = Object.getOwnPropertyDescriptor(globalThis, "document");
  const timeout = globalThis.setTimeout,
    clear = globalThis.clearTimeout;
  const finishes: (() => void)[] = [];
  try {
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      value: { querySelectorAll: (selector: string) => layers.get(selector) ?? [] },
    });
    globalThis.setTimeout = ((callback: () => void) => {
      finishes.push(callback);
      return finishes.length;
    }) as unknown as typeof setTimeout;
    globalThis.clearTimeout = (() => {}) as unknown as typeof clearTimeout;
    Motion.animate_tile("1,3", "select");
    assert(ground.classes.has("motion-select"));
    assert.equal(unit.classes.size, 0);
    for (const kind of ["move", "recruit", "attack"]) {
      Motion.animate_tile("1,3", kind);
      assert.deepEqual([...unit.classes], [`motion-${kind}`]);
    }
    Motion.animate_tile("1,3", "capture");
    assert(indicator.classes.has("motion-capture"));
    Motion.animate_tile("2,2", "move");
    assert.equal(empty.classes.size, 0);
    for (const finish of finishes) finish();
    assert.equal(ground.classes.size + unit.classes.size + indicator.classes.size, 0);
  } finally {
    if (document) Object.defineProperty(globalThis, "document", document);
    else Reflect.deleteProperty(globalThis, "document");
    globalThis.setTimeout = timeout;
    globalThis.clearTimeout = clear;
  }
});
