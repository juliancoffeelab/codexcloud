import assert from "node:assert/strict";
import { test, vi } from "vitest";
import type { State, EngineView, SavedGame } from "../rust/pkg/hex_command";
const controlled = vi.hoisted(() => ({ bridge: undefined as any, load: () => Promise.resolve() }));
vi.mock("@wasm/hex_command", async (importOriginal) => ({
  ...(await importOriginal<object>()),
  default: () => controlled.load(),
  NativeEngineFactory: class {
    constructor() {
      return controlled.bridge;
    }
  },
}));
import * as Native from "@frontend/Native";

async function fixture() {
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
  let nextId = 1,
    nextView: EngineView = { revision: 0, state: structuredClone(state) };
  const requests: { id: number; revision: number }[] = [],
    releases: number[] = [];
  let frees = 0,
    stale = false,
    incoherent = false,
    failClose = false,
    saved: SavedGame | undefined;
  controlled.load = () => Promise.resolve();
  controlled.bridge = {
    catalog: () => ({
      ok: true,
      value: { maps: [], campaigns: [], units: [], terrain: [], races: [] },
    }),
    create: () => ({ ok: true, value: { id: nextId++, view: nextView } }),
    restore: (input: SavedGame) => {
      saved = input;
      nextView = { revision: 0, state: structuredClone(input.state) };
      return { ok: true, value: { id: nextId++, view: nextView } };
    },
    inspect: () => ({
      ok: true,
      value: {
        revision: nextView.revision + (incoherent ? 1 : 0),
        units: [],
        recruitment: {},
        costs: {},
        neighbors: {},
        income: { player: 0, enemy: 0 },
      },
    }),
    dispatch: (id: number, revision: number) => {
      requests.push({ id, revision });
      if (stale)
        return {
          ok: false,
          error: { code: "staleRevision", expected: revision, actual: revision + 1 },
        };
      nextView = {
        revision: revision + 1,
        state: { ...structuredClone(state), round: state.round + 1 },
      };
      return {
        ok: true,
        value: {
          view: nextView,
          effects: { addedUnit: null, damage: null, retaliation: null, winner: null, events: [] },
        },
      };
    },
    chooseAiCommand: (_id: number, revision: number) => ({
      ok: true,
      value: { revision, command: { kind: "endTurn" } },
    }),
    save: () => ({ ok: true, value: { seed: 42, state: structuredClone(nextView.state) } }),
    disposeBattle: (id: number) => {
      releases.push(id);
      if (failClose && id === 1) throw new Error("release failed");
      return { ok: true, value: null };
    },
    free: () => {
      frees++;
    },
  };
  const factory = await Native.create_factory(),
    battle = Native.create(factory, { kind: "scenario", map: "mirror" }, 42);
  return {
    factory,
    battle,
    state,
    requests,
    releases,
    frees: () => frees,
    saved: () => saved,
    stale: () => {
      stale = true;
    },
    incoherent: () => {
      incoherent = true;
    },
    failClose: () => {
      failClose = true;
    },
  };
}

test("Native owns frozen Rust snapshots without duplicating publications", async () => {
  const { factory, battle, state, saved } = await fixture();
  const publication = Native.view(battle),
    view = Native.state(publication);
  assert(Object.isFrozen(view.units[0]));
  assert(Object.isFrozen(view.gold));
  assert.throws(() => Object.assign(view.gold, { player: 99 }), TypeError);
  state.gold.player = 8;
  assert.equal(view.gold.player, 3);
  assert.equal("lastCombat" in view, false);
  assert.equal("experience" in view.units[0]!, false);
  const checkpoint = structuredClone(Native.save(battle)) as SavedGame;
  checkpoint.state.lastCombat = null;
  Native.restore(factory, checkpoint);
  assert.equal(
    saved(),
    checkpoint,
    "Rust owns transfer decoding; OCaml avoids another full input copy",
  );
  assert.equal(Native.state(Native.view(battle)).lastCombat, undefined);
  Native.dispose_factory(factory);
});

test("Native forwards current revisions and rejects stale work without changing old views", async () => {
  const { factory, battle, requests, stale } = await fixture();
  const before = Native.view(battle);
  Native.enemy_step(battle, { policy: "mixed", diplomacy: "restrained", recruitmentLimit: 2 });
  assert.equal(Native.revision(Native.view(battle)), 1);
  assert.equal(Native.revision(before), 0);
  assert.equal(Native.state(before).round, 1);
  const current = Native.view(battle);
  stale();
  assert.throws(() => Native.dispatch(battle, { kind: "endTurn" }), /staleRevision/);
  assert.equal(Native.view(battle), current);
  assert.deepEqual(requests.at(-1), { id: 1, revision: 1 });
  Native.dispose_factory(factory);
});

test("Native releases all handles and frees the factory even when one cleanup fails", async () => {
  const { factory, battle, releases, frees, failClose } = await fixture();
  const second = Native.create(factory, { kind: "scenario", map: "mirror" }, 7);
  failClose();
  assert.throws(() => Native.dispose_factory(factory));
  Native.dispose_factory(factory);
  Native.dispose(battle);
  assert.deepEqual(releases.sort(), [1, 2]);
  assert.equal(frees(), 1);
  assert.throws(() => Native.view(second), /disposed/);
  assert.throws(() => Native.catalog(factory), /disposed/);
});

test("Native rejects invalid seeds and propagates asynchronous module failure", async () => {
  const { factory } = await fixture();
  for (const seed of [-1, 2 ** 32, 0.5, NaN])
    assert.throws(
      () => Native.create(factory, { kind: "scenario", map: "mirror" }, seed),
      /uint32/,
    );
  controlled.load = () => Promise.reject(new Error("Missing WASM module"));
  await assert.rejects(Native.create_factory(), /Missing WASM module/);
  Native.dispose_factory(factory);
});

test("Incoherent accepted publications release their handle without corrupting detached views", async () => {
  const { factory, battle, releases, incoherent } = await fixture();
  const old = Native.view(battle);
  incoherent();
  assert.throws(
    () => Native.dispatch(battle, { kind: "endTurn" }),
    /Incoherent native publication/,
  );
  assert.deepEqual(releases, [1]);
  assert.equal(Native.state(old).round, 1);
  assert.throws(() => Native.view(battle), /disposed/);
  Native.dispose_factory(factory);
});
