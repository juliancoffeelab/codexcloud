import assert from "node:assert/strict";
import { BattleController } from "../src/controller/battle";
import { SAVE_KEY } from "../src/game/save";
import { TestStorage } from "./helpers";

const storage = new TestStorage();
globalThis.localStorage = storage;

// Rejected replacement must not spend undo history or cancel an enemy turn.
{
  const controller = new BattleController();
  controller.move("u3", "3,1");
  const before = controller.snapshot();
  const saved = storage.getItem(SAVE_KEY);
  assert.throws(() => controller.reset("unknown"), /Unknown battlefield/);
  assert.deepEqual(controller.snapshot(), before);
  assert.equal(storage.getItem(SAVE_KEY), saved);
  assert(controller.canUndo());
  controller.undoMove();
  const turn = controller.endTurn();
  assert.throws(() => controller.reset("unknown"), /Unknown battlefield/);
  await turn;
  assert.equal(controller.snapshot().round, 2);
  assert.equal(controller.snapshot().turn, "player");
  controller.dispose();
}

// An old unmounted session must never save over its replacement after an await.
{
  storage.clear();
  const previous = new BattleController();
  const pending = previous.endTurn();
  previous.dispose();
  const replacement = new BattleController();
  replacement.reset("river");
  const saved = storage.getItem(SAVE_KEY);
  await pending;
  assert.equal(storage.getItem(SAVE_KEY), saved);
  assert.equal(replacement.snapshot().mapId, "river");
  assert.equal(replacement.snapshot().round, 1);
  assert.throws(() => previous.reset(), /session has closed/);
  assert.throws(() => previous.move("u3", "3,1"), /session has closed/);
  replacement.dispose();
}
console.log(
  "PASS: rejected resets preserve actions/undo; disposed turns cannot overwrite a replacement save",
);
