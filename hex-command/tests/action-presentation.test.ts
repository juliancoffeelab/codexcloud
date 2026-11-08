import { test } from "vitest";
import assert from "node:assert/strict";
import { ActionPresentation, claimAction } from "../src/controller/action-presentation";

test("Action completion is explicit and settles once", async () => {
  const action = new ActionPresentation();
  let settled = false;
  void action.finished.then(() => (settled = true));
  await Promise.resolve();
  assert.equal(settled, false);
  const renderer = claimAction(action.token);
  renderer.complete();
  action.fail(new Error("Late failure"));
  assert.deepEqual(await action.finished, { kind: "completed" });
});

test("Session cancellation is distinguishable from successful presentation", async () => {
  const action = new ActionPresentation();
  const renderer = claimAction(action.token);
  action.cancel("session-replaced");
  renderer.complete();
  assert.deepEqual(await action.finished, { kind: "cancelled", reason: "session-replaced" });
  assert.equal("complete" in action.token, false);
  assert.equal("cancel" in action.token, false);
});
