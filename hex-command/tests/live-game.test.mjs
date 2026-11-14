import "./storage-fixture.mjs";
import { createHeadlessSession } from "./headless-session.mjs";
import * as Setup from "@frontend/Setup";
import { cellKey } from "./hex-fixture.mjs";
import { historicalTiles } from "./legacy-save.mjs";
import assert from "node:assert/strict";
import { test, beforeEach, vi } from "vitest";
import * as Session from "@frontend/Session";
import * as Save from "./save-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
import {
  Native,
  Catalog,
  factory,
  catalog,
  createBattle,
  restoreBattle,
} from "./native-fixture.mjs";
import { Game, developerFixture } from "./game-fixture.mjs";
import { saveBattle, SAVE_KEY } from "./legacy-save.mjs";
import { TestStorage, required } from "./helpers.mjs";
beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));
const state = (session) => Native.state(Session.view(session).game);
const capture = (session) => structuredClone(state(session));
async function controller(clock = async (_milliseconds) => {}) {
  return await createHeadlessSession(clock, factory);
}
async function load(restore = restoreBattle) {
  return Save.load(
    Catalog.raw(catalog),
    restore,
    () => createBattle({ kind: "scenario", map: "mirror" }),
    Native.dispose,
    Native.save,
  );
}
async function write(game, pending = false) {
  await Save.write(Save.create(), Native.save(game), pending);
}
test("Live controller uses native movement/undo and preserves previous UI queries", async () => {
  await saveBattle(developerFixture());
  const session = await controller(),
    commands = Session.commands(session);
  const opening = capture(session),
    previous = Session.view(session).game;
  const actor = required(opening.units.find((unit) => unit.faction === "player"));
  const moves = Native.reachable(previous, actor.id),
    tile = required(Object.keys(moves)[0]);
  commands.choose(cellKey(actor.tile));
  commands.choose(tile);
  assert(Session.view(session).can_undo);
  assert.deepEqual(Native.unit(previous, actor.id)?.tile, actor.tile);
  assert.deepEqual(Native.reachable(previous, actor.id), moves);

  commands.undo();
  assert.deepEqual(capture(session), opening);
  const restored = await load();
  assert.deepEqual(Native.state(Native.view(restored.game)), opening);
  Native.dispose(restored.game);
  Session.close(session);
});
test("A native enemy turn cancelled by map selection never acts on the new battle", async () => {
  let resume;
  const pause = new Promise((resolve) => {
    resume = resolve;
  });
  const session = await controller(() => pause),
    commands = Session.commands(session),
    opening = capture(session);
  const pending = commands.end_turn(),
    checkpoint = await load();
  assert(checkpoint.pending_enemy);
  assert.deepEqual(Native.state(Native.view(checkpoint.game)), opening);
  Native.dispose(checkpoint.game);
  commands.reset(undefined, undefined, "river-valley");
  const replacement = capture(session);
  resume();
  await pending;
  assert.deepEqual(capture(session), replacement);
  Session.close(session);
});
test("Native combat inspection feeds UI damage breakdowns and old saves migrate once", async () => {
  const opening = createBattle({ kind: "scenario", map: "developer" });
  const current = Native.state(Native.view(opening));
  const checkpoint = structuredClone(Native.save(opening));
  Object.assign(checkpoint.state, { mapRevision: 1, cols: 6, rows: 20 });
  checkpoint.state.tiles = checkpoint.state.tiles.filter((tile) => tile.col < 6 && tile.row < 20);
  Object.assign(required(checkpoint.state.tiles[0]), {
    terrain: "road",
    connections: ["se", "ne"],
    deadEnds: ["w"],
    floor: "green-forest",
  });
  required(checkpoint.state.tiles.find((tile) => tile.terrain === "village")).owner = "player";
  checkpoint.state.round = 4;
  checkpoint.state.gold.player = 23;
  required(checkpoint.state.units[0]).hp -= 1;
  checkpoint.seed = 42;
  const historical = { version: 3, checkpoint: historicalTiles(checkpoint), pendingEnemy: false };
  const retained = new Map(checkpoint.state.tiles.map((tile) => [tile.id, tile]));
  const expected = {
    ...checkpoint.state,
    mapRevision: current.mapRevision,
    cols: current.cols,
    rows: current.rows,
    tiles: current.tiles.map((tile) => retained.get(tile.id) ?? structuredClone(tile)),
  };
  await Store.store_save(JSON.stringify(historical));
  const loaded = await load(),
    game = loaded.game;
  assert.equal(loaded.notice, void 0);
  const { tiles: expectedTiles, units: expectedUnits, ...progress } = expected;
  const actual = Native.state(Native.view(game));
  const { tiles: actualTiles, units: actualUnits, ...actualProgress } = actual;
  assert.deepEqual(actualProgress, progress);
  assert.deepEqual(
    actualUnits.map(({ role, ...unit }) => unit),
    expectedUnits.map(({ role, ...unit }) => unit),
  );
  for (const [id, tile] of retained)
    assert.deepEqual(required(actualTiles.find((tile) => tile.id === id)), tile);
  assert.equal(actualTiles.length, expectedTiles.length);
  assert.equal(Native.save(game).seed, 42);
  assert.equal(JSON.parse(await Save.flashText()).version, Save.version);
  assert.deepEqual(Native.tile(Native.view(game), "0,0"), checkpoint.state.tiles[0]);
  assert.deepEqual(
    Native.tile(Native.view(game), "8,2"),
    Native.tile(Native.view(opening), "8,2"),
    "New cells acquire the current authored directional artwork fields",
  );
  Native.dispose(opening);
  const previous = Native.view(game),
    actor = required(Native.state(previous).units.find((unit) => unit.faction === "player"));
  const target = required(Native.targets(previous, actor.id)[0]),
    forecast = required(Native.best_attack(previous, actor.id, target.id));
  Native.dispatch(game, {
    kind: "attack",
    unit: Native.unit_id(actor.id),
    target: Native.unit_id(target.id),
    weapon: forecast.attack.id,
  });
  assert.equal(Native.unit(previous, target.id)?.hp, target.hp);
  assert.equal(
    Native.unit(Native.view(game), target.id)?.hp ?? 0,
    Math.max(0, target.hp - forecast.damage),
  );
  await write(game);
  assert.equal(JSON.parse(await Save.flashText()).version, Save.version);
  const restored = (await load()).game;
  assert.deepEqual(Native.state(Native.view(restored)), Native.state(Native.view(game)));
  Native.dispose(restored);
  Native.dispose(game);
});
test("All campaign dialogue and retry controls dispatch through native hooks", async () => {
  for (const id of ["lantern-road"]) {
    const game = createBattle({ kind: "campaign", id }),
      opening = structuredClone(Native.state(Native.view(game)));
    Native.dispatch(game, { kind: "dismissDialogue" });
    Native.dispatch(game, { kind: "previousDialogue" });
    assert.equal(Native.state(Native.view(game)).campaign?.dialogueIndex, 0);
    Native.dispatch(game, { kind: "skipDialogue" });
    let current = Native.state(Native.view(game));
    assert.equal(current.campaign?.dialogueIndex, current.campaign?.dialogue.length);
    Native.dispatch(game, { kind: "retryMission" });
    assert.deepEqual(Native.state(Native.view(game)), opening);
    Native.dispose(game);
  }
});
test("A failed saved battle remains available for repair while fresh play is available", async () => {
  await saveBattle(new Game("mirror"));
  const rejected = JSON.parse(await Save.flashText());
  const restored = await load(() => {
    throw new Error("Rejected");
  });
  assert(restored.problem);
  assert.equal(Native.state(Native.view(restored.game)).turn, "player");
  assert.equal(localStorage.getItem(SAVE_KEY + "-unrestored"), null);
  assert.equal(await Save.flashText(), JSON.stringify(rejected));
  Native.dispose(restored.game);
});
test("Rejected native campaign data leaves the WASM factory usable", async () => {
  const game = createBattle({ kind: "campaign", id: "lantern-road" });
  await write(game);
  Native.dispose(game);
  const checkpoint = JSON.parse(await Save.flashText());
  const corruptions = [
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = ["Captain Nera"];
    },
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = [];
      checkpoint.checkpoint.state.units[0].experience = 4294967295;
    },
  ];
  for (const corrupt of corruptions) {
    corrupt();
    await Store.store_save(JSON.stringify(checkpoint));
    const restored = await load();
    assert(restored.problem);
    Native.dispose(restored.game);
  }
  const next = createBattle({ kind: "campaign", id: "lantern-road" });
  Native.dispatch(next, { kind: "retryMission" });
  assert.equal(
    Native.state(Native.view(next)).units.find((unit) => unit.role === "leader")?.character,
    "nera",
  );
  Native.dispose(next);
});
test("Live enemy planning finishes a phase and saves the resulting player turn", async () => {
  const session = await controller();
  await Session.commands(session).end_turn();
  const after = capture(session);
  assert.equal(after.turn, "player");
  assert.equal(after.round, 2);
  const restored = await load();
  assert.equal(restored.pending_enemy, false);
  assert.deepEqual(Native.state(Native.view(restored.game)), after);
  Native.dispose(restored.game);
  Session.close(session);
});
test("An invalid native checkpoint remains available for deletion and leaves a playable battle", async () => {
  const rejected = { version: 999, checkpoint: {}, pendingEnemy: false };
  await Store.store_save(JSON.stringify(rejected));
  const restored = await load();
  assert(restored.problem);
  assert.equal(Native.state(Native.view(restored.game)).turn, "player");
  assert.equal(localStorage.getItem(SAVE_KEY + "-unrestored"), null);
  assert.equal(await Save.flashText(), JSON.stringify(rejected));
  Native.dispose(restored.game);
});
test("Reloading an enemy-phase checkpoint rebuilds planning once from the saved seed", async () => {
  let game = createBattle({ kind: "scenario", map: "mirror" }),
    options = { policy: "classic", diplomacy: "diplomatic", recruitmentLimit: null };
  Native.dispatch(game, { kind: "endTurn" });
  Native.enemy_step(game, options);
  await Save.write(
    Save.create(),
    Native.save(game),
    false,
    Setup.of_json({
      player: null,
      first: "ai",
      enemy: { policy: options.policy, diplomacy: options.diplomacy },
    }),
  );
  const phase = structuredClone(Native.state(Native.view(game)));
  assert.equal(phase.turn, "ai");
  const checkpoint = Native.save(game);
  Native.dispose(game);
  game = restoreBattle(checkpoint);
  for (let steps = 0; Native.state(Native.view(game)).turn === "ai"; steps++) {
    assert(steps < 100);
    Native.enemy_step(game, options);
  }
  const expected = structuredClone(Native.state(Native.view(game)));
  Native.dispose(game);
  let release;
  const paused = new Promise((resolve) => {
    release = resolve;
  });
  const session = await controller(() => paused);
  let completed;
  const completion = new Promise((resolve) => {
    completed = resolve;
  });
  const unsubscribe = Session.subscribe(session, (view) => {
    const current = Native.state(view.game);
    if (current.turn === "player" && current.round === 2) completed();
  });
  Session.initialize(session);
  Session.initialize(session);
  assert.deepEqual(capture(session), phase);
  release();
  await completion;
  assert.equal(state(session).round, 2);
  assert.deepEqual(
    capture(session),
    expected,
    "resume must match a fresh restoration of the saved seed and state",
  );
  await Promise.resolve();
  const restored = await load();
  assert.equal(restored.pending_enemy, false);
  assert.deepEqual(Native.state(Native.view(restored.game)), capture(session));
  Native.dispose(restored.game);
  Session.close(session);
  unsubscribe();
});
