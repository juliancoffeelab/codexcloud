import assert from "node:assert/strict";
import { Game } from "../src/game/engine";
import { createGameView } from "../src/game/view";
import { battlefieldTiles } from "../src/ui/battlefield-view";
import { required } from "./helpers";

const game = new Game();
const enemy = game.addUnit("archer", "enemy", "2,3");
const opening = createGameView(game);
const selected = battlefieldTiles(opening, "u2", "pixel", null);
assert.equal(selected.length, opening.tiles.length);
const soldier = required(selected.find((tile) => tile.id === "1,3"));
assert(soldier.classes.includes("selected"));
assert.equal(soldier.occupant?.unit, opening.unit("u2"));
const target = required(selected.find((tile) => tile.id === enemy.tile));
assert(target.classes.includes("target"));
assert(target.label.includes("attack available"));
assert.equal(target.occupant?.healthWidth, 38);

const recruited = battlefieldTiles(opening, null, "pixel", "0,2");
assert(required(recruited.find((tile) => tile.id === "0,2")).classes.includes("recruitable"));
assert(!recruited.some((tile) => /selected|reachable|target/.test(tile.classes)));

game.move("u2", "2,2");
const moved = battlefieldTiles(createGameView(game), "u2", "pixel", null);
assert.equal(required(moved.find((tile) => tile.id === "1,3")).occupant, null);
assert.equal(required(moved.find((tile) => tile.id === "2,2")).occupant?.unit.id, "u2");
for (let index = 0; index < selected.length; index++) {
  const before = required(selected[index]);
  const after = required(moved[index]);
  assert.equal(after.id, before.id);
  assert.equal(after.clipId, before.clipId);
  assert.equal(after.points, before.points);
  assert.equal(after.terrainArtwork, before.terrainArtwork);
}
const alternate = battlefieldTiles(opening, "u2", "normal", null);
assert.notEqual(required(alternate[0]).terrainArtwork, required(selected[0]).terrainArtwork);
assert.equal(required(alternate[0]).points, required(selected[0]).points);

console.log("PASS: battlefield selection, targets, recruitment, movement and stable SVG layers");
