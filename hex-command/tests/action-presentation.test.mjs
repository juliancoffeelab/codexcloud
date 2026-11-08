import { test } from "vitest";
import assert from "node:assert/strict";
import * as Action from "@frontend/Action_presentation";
import * as Session from "@frontend/Session";
import { factory, Native } from "./native-fixture.mjs";
import { isOk } from "./result-fixture.mjs";
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));
async function until(predicate) {
  for (let i = 0; i < 1000; i++) {
    if (predicate()) return;
    await tick();
  }
  throw Error("Action transition did not settle");
}
test("Enemy decisions wait for the actual presentation lease, including the final player publication", async () => {
  const session = await Session.create_async(async () => {}, factory);
  try {
    Session.initialize(session);
    const operation = Session.commands(session).end_turn();
    await until(() => Session.view(session).action);
    let view = Session.view(session),
      revision = Native.revision(view.interaction_game);
    for (let i = 0; i < 10; i++) await tick();
    assert.equal(
      Native.revision(Session.view(session).interaction_game),
      revision,
      "No timer may advance an unacknowledged action",
    );
    assert.equal(isOk(await Session.commands(session).end_turn()), false);
    let actions = 0,
      finalPlayerLease = false;
    while (Session.view(session).settling) {
      view = Session.view(session);
      if (view.action) {
        if (Native.state(view.interaction_game).turn === "player") {
          finalPlayerLease = true;
          assert.equal(view.settling, true);
          assert.equal(isOk(await Session.commands(session).end_turn()), false);
        }
        view.combat_playback?.skip();
        Action.complete(Action.claim(view.action));
        actions++;
        const previous = view.action;
        await until(
          () => Session.view(session).action !== previous || !Session.view(session).settling,
        );
      } else await tick();
      assert(actions < 100, "Finite ordinary Mirror phase");
    }
    assert(isOk(await operation));
    assert(actions > 0);
    assert(finalPlayerLease);
    assert.equal(Native.state(Session.view(session).interaction_game).turn, "player");
  } finally {
    Session.close(session);
  }
});
test("Disposed and replaced tasks cannot be completed twice by stale renderer leases", async () => {
  const task = Action.create(undefined),
    token = Action.token(task),
    lease = Action.claim(token);
  Action.cancel(task, 1);
  Action.complete(lease);
  const result = await Action.finished(task);
  assert.equal(result.TAG, 0);
  assert.equal(result._0, 1);
});
