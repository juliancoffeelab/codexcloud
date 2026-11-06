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

test("Moving units preserves stable terrain and interaction geometry", async () => {
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

// Achievements stay complete; ongoing territory requirements can become unmet again.
test("Objective markers distinguish achievements from current ownership", async () => {
  const game = Game.startCampaign("ashen-oath", 2);
  game.campaign!.scripts!.completed.push("sealed-hex-0941-278b487380");
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
    !required(
      battlefieldTiles(escort.publishedView(), null, currentArtwork(), null).find(
        (tile) => tile.id === "hex-0955-5859c768ff",
      ),
    ).classes.includes("objective"),
  );
  required(escort.tile("hex-0955-5859c768ff")).owner = "enemy";
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

// Controller feedback crosses the presentation boundary as data; unit visuals
// no longer need a corresponding DOM element.
const { animateTile } = await import("../src/presentation/motion");
test("Battlefield feedback sends tile effects to the GPU surface", () => {
  const originalDocument = Object.getOwnPropertyDescriptor(globalThis, "document");
  const events: CustomEvent[] = [];
  try {
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      value: {
        querySelector: (selector: string) =>
          selector === "#map[data-gpu-battlefield]"
            ? { dispatchEvent: (event: CustomEvent) => events.push(event) }
            : null,
      },
    });
    for (const kind of ["select", "move", "recruit", "attack", "capture"] as const)
      animateTile("hex-0056-2e6c8414fc", kind);
    assert.deepEqual(
      events.map((event) => [event.type, event.detail]),
      ["select", "move", "recruit", "attack", "capture"].map((kind) => [
        "battlefieldmotion",
        { tile: "hex-0056-2e6c8414fc", kind },
      ]),
    );
  } finally {
    if (originalDocument) Object.defineProperty(globalThis, "document", originalDocument);
    else Reflect.deleteProperty(globalThis, "document");
  }
});
