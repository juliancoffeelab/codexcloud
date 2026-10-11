import assert from "node:assert/strict";
import { test, beforeEach, vi } from "vitest";
import * as Session from "@frontend/Session";
import * as Save from "@frontend/Save";
import { Native, Catalog, factory, catalog, createBattle, restoreBattle } from "./native-fixture";
import { Game } from "./game-fixture";
import { saveBattle, SAVE_KEY } from "./legacy-save";
import { TestStorage, required } from "./helpers";
import type { State } from "../rust/pkg/hex_command";

beforeEach(() => vi.stubGlobal("localStorage", new TestStorage()));
const state = (session: Session.t): Native.Immutable<State> =>
  Native.state(Session.view(session).game);
const capture = (session: Session.t) => structuredClone(state(session)) as State;
function controller(clock = async (_milliseconds: number) => {}) {
  return Session.create(clock, factory);
}
function load(restore = restoreBattle) {
  return Save.load(
    Catalog.raw(catalog),
    restore,
    () => createBattle({ kind: "scenario", map: "mirror" }),
    Native.dispose,
    Native.save,
  );
}
function write(game: Native.battle, pending = false) {
  Save.write(Save.create(), Native.save(game), pending);
}

// Every scenario uses the same compiled OCaml owner and actual generated WASM.
test("Live controller uses native movement/undo and keeps previous UI queries immutable", () => {
  const session = controller(),
    commands = Session.commands(session);
  commands.reset("developer");
  const opening = capture(session),
    previous = Session.view(session).game;
  const actor = required(opening.units.find((unit) => unit.side === "player"));
  const moves = Native.reachable(previous, actor.id),
    tile = required(Object.keys(moves)[0]);
  commands.choose(actor.tile);
  commands.choose(tile);
  assert(Session.view(session).can_undo);
  assert.equal(Native.unit(previous, actor.id)?.tile, actor.tile);
  assert.deepEqual(Native.reachable(previous, actor.id), moves);
  assert(Object.isFrozen(Native.neighbors(previous, actor.tile)));
  assert.throws(() => Object.assign(Native.state(previous).gold, { player: 99 }), TypeError);
  commands.undo();
  assert.deepEqual(capture(session), opening);
  const restored = load();
  assert.deepEqual(Native.state(Native.view(restored.game)), opening);
  Native.dispose(restored.game);
  Session.close(session);
});

test("A native enemy turn cancelled by map selection never acts on the new battle", async () => {
  let resume!: () => void;
  const pause = new Promise<void>((resolve) => {
    resume = resolve;
  });
  const session = controller(() => pause),
    commands = Session.commands(session),
    opening = capture(session);
  const pending = commands.end_turn(),
    checkpoint = load();
  assert(checkpoint.pending_enemy);
  assert.deepEqual(Native.state(Native.view(checkpoint.game)), opening);
  Native.dispose(checkpoint.game);
  commands.reset("river");
  const replacement = capture(session);
  resume();
  await pending;
  assert.deepEqual(capture(session), replacement);
  Session.close(session);
});

test("Native combat inspection feeds UI damage breakdowns and old saves migrate once", () => {
  const old = new Game("developer");
  saveBattle(old);
  const game = load().game;
  assert.deepEqual(Native.state(Native.view(game)), old.captureState());
  const previous = Native.view(game),
    actor = required(Native.state(previous).units.find((unit) => unit.side === "player"));
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
  write(game);
  assert.equal(JSON.parse(localStorage.getItem(SAVE_KEY)!).version, 3);
  const restored = load().game;
  assert.deepEqual(Native.state(Native.view(restored)), Native.state(Native.view(game)));
  Native.dispose(restored);
  Native.dispose(game);
});

test("All campaign dialogue and retry controls dispatch through native hooks", () => {
  for (const id of ["lantern-road", "green-accord", "ashen-oath", "borrowed-tide"]) {
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

test("Supported legacy woodland revisions migrate without losing campaign progress", () => {
  const old = Game.startCampaign("green-accord");
  old.mapRevision = 1;
  old.round = 3;
  old.gold.player = 19;
  saveBattle(old);
  const restored = load(),
    current = Native.state(Native.view(restored.game));
  assert.equal(restored.notice, undefined);
  assert.equal(current.mapRevision, 2);
  assert.equal(current.round, 3);
  assert.equal(current.gold.player, 19);
  assert.deepEqual(current.units, old.units);
  write(restored.game);
  const reloaded = load();
  assert.deepEqual(Native.state(Native.view(reloaded.game)), current);
  Native.dispose(reloaded.game);
  Native.dispose(restored.game);
});

test("A failed legacy import is backed up and cannot prevent startup", () => {
  saveBattle(new Game("mirror"));
  const rejected = JSON.parse(localStorage.getItem(SAVE_KEY)!);
  const restored = load(() => {
    throw new Error("Rejected");
  });
  assert(restored.notice);
  assert.equal(Native.state(Native.view(restored.game)).turn, "player");
  assert.deepEqual(JSON.parse(localStorage.getItem(SAVE_KEY + "-unrestored")!), rejected);
  Native.dispose(restored.game);
});

test("Rejected native campaign data leaves the WASM factory usable", () => {
  const game = createBattle({ kind: "campaign", id: "lantern-road" });
  write(game);
  Native.dispose(game);
  const checkpoint = JSON.parse(localStorage.getItem(SAVE_KEY)!);
  const corruptions = [
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = ["Captain Nera"];
    },
    () => {
      checkpoint.checkpoint.state.campaign.fallenCompanions = [];
      checkpoint.checkpoint.state.units[0].experience = 0xffffffff;
    },
  ];
  for (const corrupt of corruptions) {
    corrupt();
    localStorage.setItem(SAVE_KEY, JSON.stringify(checkpoint));
    const restored = load();
    assert(restored.notice);
    Native.dispose(restored.game);
  }
  const next = createBattle({ kind: "campaign", id: "lantern-road" });
  Native.dispatch(next, { kind: "retryMission" });
  assert.equal(
    Native.state(Native.view(next)).units.find((unit) => unit.role === "leader")?.name,
    "Captain Nera",
  );
  Native.dispose(next);
});

test("Live enemy planning finishes a phase and saves the resulting player turn", async () => {
  const session = controller();
  await Session.commands(session).end_turn();
  const after = capture(session);
  assert.equal(after.turn, "player");
  assert.equal(after.round, 2);
  const restored = load();
  assert.equal(restored.pending_enemy, false);
  assert.deepEqual(Native.state(Native.view(restored.game)), after);
  Native.dispose(restored.game);
  Session.close(session);
});

test("An invalid native checkpoint preserves the old save and leaves a playable battle", () => {
  const rejected = { version: 999, checkpoint: {}, pendingEnemy: false };
  localStorage.setItem(SAVE_KEY, JSON.stringify(rejected));
  const restored = load();
  assert(restored.notice);
  assert.equal(Native.state(Native.view(restored.game)).turn, "player");
  assert.deepEqual(JSON.parse(localStorage.getItem(SAVE_KEY + "-unrestored")!), rejected);
  Native.dispose(restored.game);
});

test("Reloading an exact enemy-phase checkpoint resumes the saved planner once", async () => {
  vi.stubGlobal(
    "Image",
    class {
      onerror: (() => void) | null = null;
      set src(_source: string) {
        queueMicrotask(() => this.onerror?.());
      }
    },
  );
  const game = createBattle({ kind: "scenario", map: "mirror" }),
    options = { policy: "classic", diplomacy: "restrained", recruitmentLimit: 2 } as const;
  Native.dispatch(game, { kind: "endTurn" });
  Native.enemy_step(game, options);
  write(game);
  const phase = structuredClone(Native.state(Native.view(game)));
  assert.equal(phase.turn, "enemy");
  for (let steps = 0; Native.state(Native.view(game)).turn === "enemy"; steps++) {
    assert(steps < 100);
    Native.enemy_step(game, options);
  }
  const expected = structuredClone(Native.state(Native.view(game)));
  Native.dispose(game);
  let release!: () => void;
  const paused = new Promise<void>((resolve) => {
    release = resolve;
  });
  const session = controller(() => paused);
  let completed!: () => void;
  const completion = new Promise<void>((resolve) => {
    completed = resolve;
  });
  const unsubscribe = Session.subscribe(session, (view: any) => {
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
    "resume must preserve the saved planner continuation",
  );
  await Promise.resolve();
  const restored = load();
  assert.equal(restored.pending_enemy, false);
  assert.deepEqual(Native.state(Native.view(restored.game)), capture(session));
  Native.dispose(restored.game);
  Session.close(session);
  unsubscribe();
});
