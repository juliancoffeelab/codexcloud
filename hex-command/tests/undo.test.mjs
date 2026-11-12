import "./storage-fixture.mjs";
import { createHeadlessSession } from "./headless-session.mjs";
import { cellKey } from "./hex-fixture.mjs";
import { afterEach, beforeEach, test, vi } from "vitest";
import assert from "node:assert/strict";
import * as Session from "@frontend/Session";
import * as Save from "./save-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
import { Native, Catalog, catalog, factory, createBattle } from "./native-fixture.mjs";
import { required, TestStorage } from "./helpers.mjs";
const sessions = /* @__PURE__ */ new Set();
const resumes = [];
beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));
afterEach(() => {
  for (const session of sessions) Session.close(session);
  sessions.clear();
  for (const resume of resumes.splice(0)) resume();
});
function checkpoint() {
  return structuredClone(Native.save(createBattle({ kind: "scenario", map: "mirror" })));
}
async function create(saved = checkpoint()) {
  await Promise.all([...sessions].map(Session.saved));
  await Save.write(Save.create(), saved, false);
  const session = await createHeadlessSession(
    () => new Promise((resolve) => resumes.push(resolve)),
    factory,
  );
  sessions.add(session);
  return session;
}
const state = (session) => Native.state(Session.view(session).interaction_game);
const player = (session, type) =>
  required(state(session).units.find((unit) => unit.side === "player" && unit.type === type));
function move(session, type, destination) {
  const unit = player(session, type);
  const actions = Session.commands(session);
  actions.choose(cellKey(unit.tile));
  actions.choose(destination);
  return unit.id;
}
for (const owner of [null, "enemy", "player"])
  test(`Undo restores captured village and the full state: ${owner}`, async () => {
    const saved = checkpoint();
    required(saved.state.tiles.find((tile) => tile.id === "3,1")).owner = owner;
    saved.state.log = Array.from({ length: 8 }, (_, index) => `Earlier event ${index}`);
    required(saved.state.units.find((unit) => unit.side === "player" && unit.type === "scout")).hp =
      5;
    const session = await create(saved);
    const before = structuredClone(state(session));
    const scout = move(session, "scout", "3,1");
    assert.equal(Native.unit(Session.view(session).interaction_game, scout).moved, true);
    assert.equal(Native.tile(Session.view(session).interaction_game, "3,1").owner, "player");
    assert.equal(Session.view(session).can_undo, true);
    Session.commands(session).undo();
    assert.deepEqual(state(session), before);
    assert.equal(Session.view(session).map_focus.selected, scout);
    assert.equal(Session.view(session).focused, "1,2");
    assert.equal(Session.view(session).can_undo, false);
    Session.commands(session).choose("3,1");
    assert.equal(Native.unit(Session.view(session).interaction_game, scout).moved, true);
  });
test("Multiple moves unwind in reverse order; rejected moves add no checkpoints", async () => {
  const session = await create();
  const before = structuredClone(state(session));
  const soldier = move(session, "soldier", "2,3");
  const afterFirst = structuredClone(state(session));
  const scout = move(session, "scout", "3,1");
  Session.commands(session).choose("2,2");
  Session.commands(session).undo();
  assert.equal(Session.view(session).map_focus.selected, scout);
  assert.deepEqual(state(session), afterFirst);
  Session.commands(session).undo();
  assert.equal(Session.view(session).map_focus.selected, soldier);
  assert.deepEqual(state(session), before);
  Session.commands(session).undo();
  assert.deepEqual(state(session), before);
  assert.equal(Session.view(session).can_undo, false);
  Session.commands(session).choose("0,0");
  assert.equal(Session.view(session).can_undo, false);
});
for (const action of ["attack", "recruit", "endTurn", "reset"])
  test(`Non-move action is an undo barrier: ${action}`, async () => {
    const saved = checkpoint();
    if (action === "attack")
      saved.state.units.push({
        id: `u${saved.state.nextId++}`,
        type: "soldier",
        side: "enemy",
        tile: { col: 4, row: 1 },
        hp: Catalog.unit(catalog, "soldier").hp,
        moved: false,
        attacked: false,
        experience: 0,
      });
    const session = await create(saved);
    move(session, "scout", "3,1");
    const commands = Session.commands(session);
    if (action === "attack") {
      commands.choose("4,1");
      commands.attack("knife");
      assert(Native.at(Session.view(session).interaction_game, "3,1").attacked);
    }
    if (action === "recruit") {
      commands.recruit("swordsman", "0,2");
      assert(Native.at(Session.view(session).interaction_game, "0,2"));
    }
    if (action === "endTurn") void commands.end_turn();
    if (action === "reset") commands.reset(undefined, undefined, "river-valley");
    const after = structuredClone(state(session));
    assert.equal(Session.view(session).can_undo, false);
    commands.undo();
    assert.deepEqual(state(session), after);
  });
test("Undo history stays within its session and never carries into another battle", async () => {
  const first = await create();
  move(first, "scout", "3,1");
  const second = await create();
  assert.equal(Session.view(second).can_undo, false);
  Session.commands(second).undo();
  assert.equal(cellKey(player(second, "scout").tile), "1,2");
  assert.equal(Session.view(first).can_undo, true);
  Session.commands(first).reset(undefined, undefined, "mirror");
  assert.equal(Session.view(first).can_undo, false);
  Session.commands(first).undo();
  assert.equal(cellKey(player(first, "scout").tile), "1,2");
});
test("A victorious attack clears move history", async () => {
  const saved = checkpoint();
  const commander = required(
    saved.state.units.find(
      (unit) => unit.side === "enemy" && Catalog.unit(catalog, unit.type).family === "commander",
    ),
  );
  commander.tile = { col: 4, row: 1 };
  commander.hp = 1;
  const session = await create(saved);
  move(session, "scout", "3,1");
  Session.commands(session).choose("4,1");
  Session.commands(session).attack("knife");
  assert.equal(state(session).winner, "player");
  assert.equal(Session.view(session).can_undo, false);
  const after = structuredClone(state(session));
  Session.commands(session).undo();
  assert.deepEqual(state(session), after);
});
test("Enemy inspection cannot create player move history", async () => {
  const saved = checkpoint();
  saved.state.turn = "enemy";
  const session = await create(saved);
  const before = structuredClone(state(session));
  const scout = required(
    before.units.find((unit) => unit.side === "enemy" && unit.type === "scout"),
  );
  Session.commands(session).choose(cellKey(scout.tile));
  Session.commands(session).choose("7,1");
  assert.equal(Session.view(session).can_undo, false);
  Session.commands(session).undo();
  assert.deepEqual(state(session), before);
});
test("A closed session rejects undo commands and leaves its final snapshot intact", async () => {
  const session = await create();
  move(session, "scout", "3,1");
  const final = Session.view(session);
  Session.close(session);
  assert.throws(() => Session.commands(session).undo(), sessionException(/closed/));
  assert.equal(Session.view(session), final);
});

function sessionException(pattern) {
  return (error) => {
    assert.equal(error.MEL_EXN_ID, Session.Failed);
    assert.match(Session.error_message(error._1), pattern);
    return true;
  };
}
