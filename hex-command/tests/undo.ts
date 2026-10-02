import type { UnitType } from "../src/game/types.ts";
import { required } from "./helpers.ts";
import assert from "node:assert/strict";
import { Game } from "../src/game/engine.ts";
import { createMoveUndo } from "../src/game/undo.ts";

const snapshot = (game: Game) => structuredClone({ ...game });
const playerUnit = (game: Game, type: UnitType) =>
  required(
    game.units.find((unit) => unit.side === "player" && unit.type === type),
  );

// Capturing a village must restore its former owner, unit flags and capture log.
for (const owner of [null, "enemy", "player"] as const) {
  const game = new Game();
  const undo = createMoveUndo();
  const scout = playerUnit(game, "scout");
  required(game.tile("3,1")).owner = owner;
  game.log = Array.from({ length: 8 }, (_, i) => `Earlier event ${i}`);
  scout.hp = 5;
  const before = snapshot(game);
  const moved = undo.move(game, scout.id, "3,1");
  assert.equal(moved, scout);
  assert.equal(scout.moved, true);
  assert.equal(required(game.tile("3,1")).owner, "player");
  assert.equal(undo.canUndo(game), true);
  assert.deepEqual(undo.undo(game), { unitId: scout.id, tileId: "1,2" });
  assert.deepEqual(snapshot(game), before);
  assert.ok(game instanceof Game);
  assert.equal(undo.canUndo(game), false);
  // Restored units are fresh references; another capture still works normally.
  undo.move(game, scout.id, "3,1");
  assert.equal(required(game.unit(scout.id)).moved, true);
}

// Independent moves unwind in reverse order; failed moves add no entries.
{
  const game = new Game();
  const undo = createMoveUndo();
  const soldier = playerUnit(game, "soldier");
  const scout = playerUnit(game, "scout");
  const before = snapshot(game);
  undo.move(game, soldier.id, "2,3");
  const afterFirst = snapshot(game);
  undo.move(game, scout.id, "3,1");
  assert.throws(() => undo.move(game, scout.id, "2,2"), /outside/);
  assert.deepEqual(undo.undo(game), { unitId: scout.id, tileId: "1,2" });
  assert.deepEqual(snapshot(game), afterFirst);
  assert.deepEqual(undo.undo(game), { unitId: soldier.id, tileId: "1,3" });
  assert.deepEqual(snapshot(game), before);
  assert.equal(undo.undo(game), null);
  assert.throws(() => undo.move(game, soldier.id, "0,0"), /outside/);
  assert.equal(undo.canUndo(game), false);
}

// Explicit barriers retain the action state and make preceding moves permanent.
for (const action of ["attack", "recruit", "endTurn", "reset"]) {
  const game = new Game();
  const undo = createMoveUndo();
  const scout = playerUnit(game, "scout");
  undo.move(game, scout.id, "3,1");
  if (action === "attack") {
    const target = game.addUnit("soldier", "enemy", "4,1");
    game.attack(scout.id, target.id);
  } else if (action === "recruit") {
    game.recruit(playerUnit(game, "commander").id, "soldier", "0,2");
  } else if (action === "endTurn") {
    game.endTurn();
  }
  undo.clear();
  const after = snapshot(game);
  assert.equal(undo.canUndo(game), false);
  assert.equal(undo.undo(game), null);
  assert.deepEqual(snapshot(game), after);
}

// Stale history cannot carry into a different turn, round, battle or victory.
for (const change of ["enemy", "winner", "round", "game"]) {
  const game = new Game();
  const undo = createMoveUndo();
  undo.move(game, playerUnit(game, "scout").id, "3,1");
  let current = game;
  if (change === "enemy") game.turn = "enemy";
  if (change === "winner") game.winner = "player";
  if (change === "round") game.round++;
  if (change === "game") current = new Game();
  const before = snapshot(current);
  assert.equal(undo.canUndo(current), false);
  assert.equal(undo.undo(current), null);
  assert.deepEqual(snapshot(current), before);
  game.turn = "player";
  game.winner = null;
  game.round = 1;
  assert.equal(undo.canUndo(game), false);
}

{
  const game = new Game();
  const undo = createMoveUndo();
  game.turn = "enemy";
  const scout = required(
    game.units.find((unit) => unit.side === "enemy" && unit.type === "scout"),
  );
  undo.move(game, scout.id, "7,1");
  assert.equal(undo.canUndo(game), false);
  game.turn = "player";
  assert.equal(undo.undo(game), null);
}

console.log(
  "PASS: move undo restores captures and full state, unwinds LIFO, and respects action and turn barriers",
);
