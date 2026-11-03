import { test } from "vitest";
import assert from "node:assert/strict";
import { currentArtwork } from "../src/presentation/art";
import { Game } from "./game-fixture";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { required } from "./helpers";

test("Selection and attack targets have accessible labels and health", async () => {
  const game = new Game();
  const enemy = game.addUnit("archer", "enemy", "hex-0408-edbae05560");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  assert.equal(selected.length, opening.tiles.length);
  const soldier = required(selected.find((tile) => tile.id === "hex-0056-2e6c8414fc"));
  assert(soldier.classes.includes("selected"));
  assert.equal(soldier.occupant?.unit, opening.unit("u2"));
  const target = required(selected.find((tile) => tile.id === enemy.tile));
  assert(target.classes.includes("target"));
  assert(target.label.includes("attack available"));
  assert.equal(target.occupant?.healthWidth, 38);
});

test("Recruitment highlights only legal destinations", async () => {
  const game = new Game();
  game.addUnit("archer", "enemy", "hex-0408-edbae05560");
  const opening = game.publishedView();

  const recruited = battlefieldTiles(opening, null, currentArtwork(), "hex-0013-23f615dae8");
  assert(
    required(recruited.find((tile) => tile.id === "hex-0013-23f615dae8")).classes.includes(
      "recruitable",
    ),
  );
  assert(!recruited.some((tile) => /selected|reachable|target/.test(tile.classes)));
});

test("Moving units preserves stable terrain and SVG layers", async () => {
  const game = new Game();
  game.addUnit("archer", "enemy", "hex-0408-edbae05560");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  game.move("u2", "hex-0397-b042f4a1af");
  const moved = battlefieldTiles(game.publishedView(), "u2", currentArtwork(), null);
  assert.equal(required(moved.find((tile) => tile.id === "hex-0056-2e6c8414fc")).occupant, null);
  assert.equal(
    required(moved.find((tile) => tile.id === "hex-0397-b042f4a1af")).occupant?.unit.id,
    "u2",
  );
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
  const enemy = game.addUnit("archer", "enemy", "hex-0408-edbae05560");
  const opening = game.publishedView();
  const selected = battlefieldTiles(opening, "u2", currentArtwork(), null);
  const target = required(selected.find((tile) => tile.id === enemy.tile));
  assert(!target.label.includes("Objective:"), "Ordinary skirmish tiles have no goal label");
});

// Sealed stones leave the map individually; territory that can be lost stays visible.
test("Completed seal markers disappear while territory remains visible", async () => {
  const game = Game.startCampaign("ashen-oath", 2);
  game.campaign!.marks.push("sealed:hex-0941-278b487380");
  const tiles = battlefieldTiles(game.publishedView(), null, currentArtwork(), null);
  assert(
    !required(tiles.find((tile) => tile.id === "hex-0941-278b487380")).classes.includes(
      "objective",
    ),
  );
  assert(
    required(tiles.find((tile) => tile.id === "hex-0957-c94afbbfa1")).classes.includes("objective"),
  );
  const escort = Game.startCampaign("green-accord", 2);
  required(escort.tile("hex-0955-5859c768ff")).owner = "player";
  assert(
    required(
      battlefieldTiles(escort.publishedView(), null, currentArtwork(), null).find(
        (tile) => tile.id === "hex-0955-5859c768ff",
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
  const groundNode = feedbackNode("data-tile", "hex-0056-2e6c8414fc");
  const unitNode = feedbackNode("data-unit-tile", "hex-0056-2e6c8414fc", true);
  const indicatorNode = feedbackNode("data-indicator-tile", "hex-0056-2e6c8414fc");
  const emptyNode = feedbackNode("data-unit-tile", "hex-0397-b042f4a1af");
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

    animateTile("hex-0056-2e6c8414fc", "select");
    assert(groundNode.classes.has("motion-select"));
    assert.equal(unitNode.classes.size, 0);
    for (const kind of ["move", "recruit", "attack"] as const) {
      animateTile("hex-0056-2e6c8414fc", kind);
      assert.deepEqual([...unitNode.classes], [`motion-${kind}`]);
    }
    animateTile("hex-0056-2e6c8414fc", "capture");
    assert(indicatorNode.classes.has("motion-capture"));
    animateTile("hex-0397-b042f4a1af", "move");
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
