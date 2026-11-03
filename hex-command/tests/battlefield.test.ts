import { test } from "vitest";
import assert from "node:assert/strict";
import { currentArtwork } from "../src/presentation/art";
import { Game } from "./game-fixture";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { required } from "./helpers";

test("Selection and attack targets have accessible labels and health", async () => {
  const game = new Game();
  const enemy = game.addUnit("archer", "enemy", "2,3");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  assert.equal(selected.length, opening.tiles.length);
  const soldier = required(selected.find((tile) => tile.id === "1,3"));
  assert(soldier.classes.includes("selected"));
  assert.equal(soldier.occupant?.unit, opening.unit("u2"));
  const target = required(selected.find((tile) => tile.id === enemy.tile));
  assert(target.classes.includes("target"));
  assert(target.label.includes("attack available"));
  assert.equal(target.occupant?.healthWidth, 38);
});

test("Recruitment highlights only legal destinations", async () => {
  const game = new Game();
  game.addUnit("archer", "enemy", "2,3");
  const opening = game.publishedView();

  const recruited = battlefieldTiles(opening, null, currentArtwork(), "0,2");
  assert(required(recruited.find((tile) => tile.id === "0,2")).classes.includes("recruitable"));
  assert(!recruited.some((tile) => /selected|reachable|target/.test(tile.classes)));
});

test("Moving units preserves stable terrain and SVG layers", async () => {
  const game = new Game();
  game.addUnit("archer", "enemy", "2,3");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  game.move("u2", "2,2");
  const moved = battlefieldTiles(game.publishedView(), "u2", currentArtwork(), null);
  assert.equal(required(moved.find((tile) => tile.id === "1,3")).occupant, null);
  assert.equal(required(moved.find((tile) => tile.id === "2,2")).occupant?.unit.id, "u2");
  for (let index = 0; index < selected.length; index++) {
    const before = required(selected[index]);
    const after = required(moved[index]);
    assert.equal(after.id, before.id);
    assert.equal(after.clipId, before.clipId);
    assert.equal(after.points, before.points);
    assert.equal(after.terrainFrame, before.terrainFrame);
  }
});

// Objective destinations and named companions convey the same information without color.
test("Campaign objectives and named companions have accessible descriptions", () => {
  const campaign = Game.startCampaign("green-accord");
  const tiles = battlefieldTiles(campaign.publishedView(), null, currentArtwork(), null);
  for (const { progress, activeTiles } of required(campaign.publishedView().objectives).goals) {
    for (const id of activeTiles) {
      const destination = required(tiles.find((tile) => tile.id === id));
      assert(destination.classes.includes("objective"));
      assert(destination.label.includes(`Objective: ${progress.label}`));
    }
  }
  for (const unit of campaign.publishedView().units.filter((unit) => unit.character)) {
    assert(required(tiles.find((tile) => tile.id === unit.tile)).label.includes(unit.name!));
  }
});
test("Ordinary skirmish labels have no objective", async () => {
  const game = new Game();
  const enemy = game.addUnit("archer", "enemy", "2,3");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  const target = required(selected.find((tile) => tile.id === enemy.tile));
  assert(!target.label.includes("Objective:"), "Ordinary skirmish tiles have no goal label");
});

// Sealed stones leave the map individually; territory that can be lost stays visible.
test("Completed seal markers disappear while territory remains visible", async () => {
  const game = Game.startCampaign("ashen-oath", 2);
  game.campaign!.marks.push("sealed:7,2");
  const tiles = battlefieldTiles(game.publishedView(), null, currentArtwork(), null);
  assert(!required(tiles.find((tile) => tile.id === "7,2")).classes.includes("objective"));
  assert(required(tiles.find((tile) => tile.id === "7,6")).classes.includes("objective"));
  const escort = Game.startCampaign("green-accord", 2);
  required(escort.tile("7,4")).owner = "player";
  assert(
    required(
      battlefieldTiles(escort.publishedView(), null, currentArtwork(), null).find(
        (tile) => tile.id === "7,4",
      ),
    ).classes.includes("objective"),
  );
  escort.winner = "player";
  assert(
    !battlefieldTiles(escort.publishedView(), null, currentArtwork(), null).some((tile) =>
      tile.classes.includes("objective"),
    ),
  );
});

// The three paint layers retain separate feedback targets; overlays never need
// the keyboard-focusable ground node to contain the unit artwork.
const { animateTile } = await import("../src/presentation/motion");
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
test("Selection, unit motion and capture target independent paint layers", async () => {
  const groundNode = feedbackNode("data-tile", "1,3");
  const unitNode = feedbackNode("data-unit-tile", "1,3", true);
  const indicatorNode = feedbackNode("data-indicator-tile", "1,3");
  const emptyNode = feedbackNode("data-unit-tile", "2,2");
  const feedbackLayers = new Map([
    ["#map [data-tile]", [groundNode]],
    ["#map [data-unit-tile]", [unitNode, emptyNode]],
    ["#map [data-indicator-tile]", [indicatorNode]],
  ]);
  const originalDocument = Object.getOwnPropertyDescriptor(globalThis, "document");
  const originalTimeout = globalThis.setTimeout;
  const originalClearTimeout = globalThis.clearTimeout;
  const cleanup: (() => void)[] = [];
  try {
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      value: { querySelectorAll: (selector: string) => feedbackLayers.get(selector) ?? [] },
    });
    globalThis.setTimeout = ((callback: () => void) => {
      cleanup.push(callback);
      return cleanup.length;
    }) as unknown as typeof setTimeout;
    globalThis.clearTimeout = (() => undefined) as typeof clearTimeout;

    animateTile("1,3", "select");
    assert(groundNode.classes.has("motion-select"));
    assert.equal(unitNode.classes.size, 0);
    for (const kind of ["move", "recruit", "attack"] as const) {
      animateTile("1,3", kind);
      assert.deepEqual([...unitNode.classes], [`motion-${kind}`]);
    }
    animateTile("1,3", "capture");
    assert(indicatorNode.classes.has("motion-capture"));
    animateTile("2,2", "move");
    assert.equal(emptyNode.classes.size, 0);
    for (const finish of cleanup) finish();
    assert.equal(groundNode.classes.size + unitNode.classes.size + indicatorNode.classes.size, 0);
  } finally {
    if (originalDocument) Object.defineProperty(globalThis, "document", originalDocument);
    else Reflect.deleteProperty(globalThis, "document");
    globalThis.setTimeout = originalTimeout;
    globalThis.clearTimeout = originalClearTimeout;
  }
});
