import { test } from "vitest";
import assert from "node:assert/strict";
import { ActionPresentation, claimAction } from "../src/controller/action-presentation";
import type { GameView } from "../src/game/view";

test("A renderer detach does not complete an action; only its current claimant can finish", async () => {
  const action = new ActionPresentation({} as GameView);
  let settled = false;
  void action.finished.then(() => (settled = true));
  const old = claimAction(action.token);
  old.finishMotion("move:unit");
  old.detach();
  old.finishMotion("spawn:unit");
  old.complete();
  await Promise.resolve();
  assert.equal(settled, false);
  const current = claimAction(action.token);
  assert.equal(current.motionDone("move:unit"), true);
  assert.equal(current.motionDone("spawn:unit"), false);
  old.complete();
  await Promise.resolve();
  assert.equal(settled, false);
  current.complete();
  assert.deepEqual(await action.finished, { kind: "completed" });
});

test("Session cancellation is distinguishable from successful presentation", async () => {
  const action = new ActionPresentation({} as GameView);
  const renderer = claimAction(action.token);
  action.cancel("session-replaced");
  renderer.complete();
  assert.deepEqual(await action.finished, { kind: "cancelled", reason: "session-replaced" });
  assert.equal("complete" in action.token, false);
  assert.equal("cancel" in action.token, false);
});
