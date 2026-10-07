import assert from "node:assert/strict";
import { test } from "vitest";
import {
  createNativeEngineFactory,
  loadNativeEngineFactory,
  type NativeBridge,
} from "../src/game/native-engine";
import type { EngineResult, EngineView, SavedGame, State } from "../rust/pkg/hex_command";

function value<T>(result: EngineResult<T>): T {
  assert(result.ok, JSON.stringify(result));
  return result.value;
}

function fixture() {
  const state: State = {
    mapId: "mirror",
    mapRevision: 1,
    cols: 1,
    rows: 1,
    round: 1,
    turn: "player",
    gold: { player: 3, enemy: 3 },
    winner: null,
    nextId: 2,
    log: ["Ready"],
    tiles: [{ id: "0,0", col: 0, row: 0, terrain: "keep", owner: "player" }],
    units: [
      {
        id: "u1",
        type: "commander",
        side: "player",
        tile: "0,0",
        hp: 30,
        moved: false,
        attacked: false,
      },
    ],
  };
  let nextId = 1;
  let nextView: EngineView = { revision: 0, state };
  const requests: { id: number; revision: number }[] = [];
  const releases: number[] = [];
  let frees = 0;
  let stale = false;
  let saved: SavedGame | undefined;
  const bridge: NativeBridge = {
    catalog: () => ({
      ok: true,
      value: { maps: [], campaigns: [], units: [], terrain: [], races: [] },
    }),
    create: () => ({ ok: true, value: { id: nextId++, view: nextView } }),
    restore: (input) => {
      saved = input;
      return { ok: true, value: { id: nextId++, view: { revision: 0, state: input.state } } };
    },
    view: (id, revision) => {
      requests.push({ id, revision });
      return { ok: true, value: nextView };
    },
    reachable: () => ({ ok: true, value: [{ tile: "0,0", cost: 0 }] }),
    inspect: () => ({
      ok: true,
      value: {
        revision: nextView.revision,
        units: [],
        recruitment: {},
        costs: {},
        neighbors: {},
        income: { player: 0, enemy: 0 },
      },
    }),
    previewAttack: () => ({
      ok: false,
      error: { code: "command", error: "IllegalAttack" },
    }),
    dispatch: (id, revision) => {
      requests.push({ id, revision });
      if (stale) {
        return {
          ok: false,
          error: { code: "staleRevision", expected: revision, actual: revision + 1 },
        };
      }
      nextView = { revision: revision + 1, state: { ...state, round: state.round + 1 } };
      return {
        ok: true,
        value: {
          view: nextView,
          effects: { addedUnit: null, damage: null, retaliation: null, winner: null, events: [] },
        },
      };
    },
    chooseAiCommand: (_id, revision) => ({
      ok: true,
      value: { revision, command: { kind: "endTurn" } },
    }),
    save: () => ({ ok: true, value: { seed: 42, state } }),
    disposeBattle: (id) => {
      releases.push(id);
      return { ok: true, value: null };
    },
    free: () => {
      frees++;
    },
  };
  const factory = createNativeEngineFactory(bridge);
  const engine = value(factory.create({ kind: "scenario", map: "mirror" }, 42));
  return {
    factory,
    engine,
    state,
    requests,
    releases,
    frees: () => frees,
    saved: () => saved,
    stale: () => {
      stale = true;
    },
  };
}

test("Native snapshots detach and freeze data while preserving absent and null fields", () => {
  const { engine, factory, state, saved } = fixture();
  const view = value(engine.view());
  assert(Object.isFrozen(view.state.units[0]));
  assert(Object.isFrozen(view.state.gold));
  assert.throws(() => Object.assign(view.state.gold, { player: 99 }), TypeError);
  state.gold.player = 8;
  assert.equal(view.state.gold.player, 3);
  assert.equal("lastCombat" in view.state, false);
  assert.equal("experience" in value(engine.view()).state.units[0]!, false);
  state.lastCombat = null;
  const checkpoint = value(engine.save());
  assert.equal(checkpoint.state.lastCombat, null);
  // The fixture owns this independent transfer object; the adapter must clone input.
  const input = structuredClone(checkpoint) as SavedGame;
  value(factory.restore(input));
  assert.notEqual(saved()?.state, input.state);
  assert.equal(saved()?.state.lastCombat, null);
  assert.equal(Object.isFrozen(state), false);
});

test("Native adapter forwards revisions and keeps old publications after a stale decision", () => {
  const { engine, requests, stale } = fixture();
  const before = value(engine.view());
  const decision = value(
    engine.chooseAiCommand({
      policy: "mixed",
      diplomacy: "restrained",
      recruitmentLimit: 2,
    }),
  );
  value(engine.dispatch(decision.command, decision.revision));
  assert.equal(value(engine.view()).revision, 1);
  assert.equal(before.revision, 0);
  assert.equal(before.state.round, 1);
  stale();
  assert.deepEqual(engine.dispatch(decision.command, decision.revision), {
    ok: false,
    error: { code: "staleRevision", expected: 0, actual: 1 },
  });
  value(engine.view());
  assert.deepEqual(requests.at(-1), { id: 1, revision: 1 });
  assert.deepEqual(engine.previewAttack({ unit: 1, target: 99, weapon: "sword" }), {
    ok: false,
    error: { code: "command", error: "IllegalAttack" },
  });
});

test("Native factory disposes all battle handles once and invalidates their operations", () => {
  const { engine, factory, releases, frees } = fixture();
  const second = value(factory.create({ kind: "scenario", map: "mirror" }, 7));
  value(engine.dispose());
  value(engine.dispose());
  assert.deepEqual(engine.save(), { ok: false, error: { code: "disposed" } });
  value(factory.dispose());
  value(factory.dispose());
  assert.deepEqual(releases, [1, 2]);
  assert.equal(frees(), 1);
  assert.deepEqual(second.view(), { ok: false, error: { code: "disposed" } });
  assert.deepEqual(factory.catalog(), { ok: false, error: { code: "disposed" } });
});

test("Native loading and seed errors return tagged failures", async () => {
  const { factory } = fixture();
  assert.equal(factory.create({ kind: "scenario", map: "mirror" }, -1).ok, false);
  assert.equal(factory.create({ kind: "scenario", map: "mirror" }, 2 ** 32).ok, false);
  assert.deepEqual(
    await loadNativeEngineFactory(async () => {
      throw new Error("Missing WASM module");
    }),
    { ok: false, error: { code: "invalidRequest", message: "Missing WASM module" } },
  );
});
