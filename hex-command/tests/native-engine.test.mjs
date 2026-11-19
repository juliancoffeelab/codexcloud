import assert from "node:assert/strict";
import { test, vi } from "vitest";
const controlled = vi.hoisted(() => ({
  handle: void 0,
  json: void 0,
  load: () => Promise.resolve(),
}));
vi.mock("@wasm/bindings.runtime.mjs", async (importOriginal) => {
  const source = await importOriginal();
  return {
    ...source,
    initialize: async () => {
      await controlled.load();
    },
  };
});
vi.mock("@frontend/Domain", async (original) => {
  const source = await original();
  return {
    ...source,
    Wasm: {
      ...source.Wasm,
      load: () => controlled.load().then(() => controlled.json.ok(undefined)),
    },
    NativeEngineFactory: {
      ...source.NativeEngineFactory,
      create: () => controlled.json.ok(controlled.handle),
      catalog: (b) => b.catalog(),
      geometry: (b, ...args) => b.topology(...args),
      new_battle: (b, ...args) => b.create(...args),
      restore: (b, ...args) => b.restore(...args),
      inspect: (b, ...args) => b.inspect(...args),
      dispatch_update: (b, ...args) => b.dispatch(...args),
      choose_ai_command: (b, ...args) => b.chooseAiCommand(...args),
      save: (b, ...args) => b.save(...args),
      save_json: (b, ...args) => b.saveJson(...args),
      dispose_battle: (b, ...args) => b.disposeBattle(...args),
      dispose: (b) => b.free(),
    },
  };
});
import { fromJSON, toJSON, toArray, listToArray } from "./json-fixture.mjs";
import * as RawNative from "@frontend/Native";
import { assertionNative } from "./native-adapter.mjs";
const Native = assertionNative(RawNative);
import * as Catalog from "@frontend/Catalog";
async function fixture() {
  const Json = await import("@frontend/Fixture_json");
  controlled.json = Json;
  const state = fromJSON("State", {
    lighting: { time: "day" },
    mapModifiers: {
      sight: {
        range: 0,
        unlimited: false,
        sightModifiers: { lit: 0, unlit: 0, darkness: 0 },
        penetrates: [],
      },
      damageModifiers: { blade: 0, pierce: 0, impact: 0 },
      terrainModifiers: {},
    },
    conditionMemory: [],
    armies: null,
    lastCombat: null,
    campaign: null,
    combatRng: 17,
    visionGroups: [],
    factions: [],
    groundItems: [],
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
    tiles: [
      {
        effects: [],
        perception: { illumination: "lit", sight: null, remember: false, concealment: [] },
        id: "0,0",
        col: 0,
        row: 0,
        terrain: "keep",
        owner: "player",
      },
    ],
    units: [
      {
        items: [],
        experience: 0,
        id: "u1",
        type: "commander",
        side: "player",
        tile: { col: 0, row: 0 },
        hp: 30,
        moved: false,
        attackSpent: false,
        faction: "player",
        revealedTo: [],
        perceptionEffects: [],
      },
    ],
  });
  let nextId = 1,
    nextView = {
      revision: 0,
      state: structuredClone(state),
      presentation: Json.of_battlefield_presentation({
        lighting: { unlit_tiles: [], lights: [] },
        fog: [],
        visible_units: ["u1"],
        unit_appearances: [],
      }),
    };
  const requests = [],
    releases = [],
    staticCalls = { topology: [] };
  let frees = 0,
    stale = false,
    incoherent = false,
    incoherentBase = false,
    failClose = false,
    saved;
  controlled.load = () => Promise.resolve();
  controlled.handle = {
    catalog: () =>
      Json.of_catalog_view({
        maps: [],
        campaigns: [],
        abilities: [],
        environment_effects: [],
        items: [],
        units: [],
        terrain: [],
        races: [],
      }),
    create: () => Json.ok({ id: nextId++, view: nextView }),
    restore: (input) => {
      saved = input;
      nextView = { ...nextView, revision: 0, state: structuredClone(input.state) };
      return Json.ok({ id: nextId++, view: nextView });
    },
    inspect: () =>
      Json.ok(
        Json.of_battle_inspection({
          revision: nextView.revision + (incoherent ? 1 : 0),
          evacuated_details: {},
          recruit_details: {},
          tiles: [],
          units: [],
          speakers: [],
          recruitment_prices: {},
          recruitment: { player: [], enemy: [] },
          costs: {},
          neighbors: {},
          income: { player: 0, enemy: 0 },
        }),
      ),
    topology: (id, revision) => {
      staticCalls.topology.push({
        id,
        revision,
        map: nextView.state.map_id,
        mapRevision: nextView.state.map_revision,
      });
      return Json.ok(
        Json.of_board_inspection({
          costs: {},
          neighbors: {},
        }),
      );
    },
    dispatch: (id, revision) => {
      requests.push({ id, revision });
      if (stale)
        return Json.error(
          Json.of_engine_error({ code: "staleRevision", expected: revision, actual: revision + 1 }),
        );
      nextView = {
        ...nextView,
        revision: revision + 1,
        state: { ...structuredClone(state), round: state.round + 1 },
      };
      const frame = structuredClone(nextView.state);
      delete frame.tiles;
      return Json.ok({
        base_revision: incoherentBase ? revision + 1 : revision,
        revision: nextView.revision,
        state: frame,
        terrain: Json.of_terrain_update({ kind: "keep" }),
        presentation: nextView.presentation,
        combat: undefined,
      });
    },
    chooseAiCommand: (_id, revision) =>
      Json.ok(Json.of_ai_decision({ revision, command: { kind: "endTurn" } })),
    save: () => Json.ok({ seed: 42, state: structuredClone(nextView.state) }),
    saveJson: () =>
      Json.ok(JSON.stringify(Json.to_saved_game({ seed: 42, state: nextView.state }))),
    disposeBattle: (id) => {
      releases.push(id);
      if (failClose && id === 1) throw new Error("release failed");
      return Json.ok();
    },
    free: () => {
      frees++;
    },
  };
  const factory = await Native.create_factory(),
    battle = Native.create(factory, fromJSON("Start", { kind: "scenario", map: "mirror" }), 42);
  return {
    factory,
    battle,
    state,
    requests,
    releases,
    staticCalls,
    frees: () => frees,
    saved: () => saved,
    stale: () => {
      stale = true;
    },
    incoherent_update: () => {
      incoherentBase = true;
    },
    incoherent: () => {
      incoherent = true;
    },
    failClose: () => {
      failClose = true;
    },
  };
}
test("Native exposes detached Rust snapshots without duplicating publications", async () => {
  const { factory, battle, state, saved } = await fixture();
  const publication = Native.view(battle),
    view = Native.state(publication);

  const adjacent = Native.neighbors(publication, "0,0");
  const expectedAdjacent = adjacent.slice();
  adjacent.push({ id: "caller-owned-array" });
  assert.deepEqual(Native.neighbors(publication, "0,0"), expectedAdjacent);

  state.gold.player = 8;
  assert.equal(Native.gold(view.gold, "player"), 3);
  assert.equal(toJSON("State", view).lastCombat, null);
  assert.equal(toArray(view.units)[0].experience, 0);
  const checkpoint = structuredClone(Native.save(battle));
  checkpoint.state.last_combat = undefined;
  Native.restore(factory, checkpoint);
  assert.equal(
    saved(),
    checkpoint,
    "Rust owns transfer decoding; OCaml avoids another full input copy",
  );
  assert.equal(toJSON("State", Native.state(Native.view(battle))).lastCombat, null);
  Native.dispose_factory(factory);
});
test("Static rule data decodes once per factory and topology once per map revision", async () => {
  const { factory, battle, staticCalls } = await fixture();
  const otherFactories = [];
  try {
    assert.equal(staticCalls.topology.length, 1);
    const old = Native.view(battle);
    Native.dispatch(battle, fromJSON("Command", { kind: "endTurn" }));
    assert.equal(
      Native.state(Native.view(battle)).tiles,
      Native.state(old).tiles,
      "A keep update shares the unchanged terrain array",
    );
    const checkpoint = Native.save(battle);
    Native.replace(battle, checkpoint);
    // A reset/recreation on the same map does not decode static rules again.
    Native.create(factory, fromJSON("Start", { kind: "scenario", map: "mirror" }), 17);
    assert.equal(staticCalls.topology.length, 1);
    assert.equal(Native.state(old).round, 1);

    const revised = structuredClone(checkpoint);
    revised.state.map_revision++;
    Native.restore(factory, revised);
    Native.restore(factory, revised);
    assert.equal(
      staticCalls.topology.length,
      2,
      "Each distinct map revision has one cached topology",
    );

    const independent = await Native.create_factory();
    otherFactories.push(independent);
    Native.create(independent, fromJSON("Start", { kind: "scenario", map: "mirror" }), 17);
    assert.equal(staticCalls.topology.length, 3, "Equal map keys cannot share across factories");
  } finally {
    otherFactories.forEach(Native.dispose_factory);
    Native.dispose_factory(factory);
  }
});
test("Explicit dictionary fixtures preserve prototype keys and Unicode through nested aliases", () => {
  const geometry = {
    neighbors: Object.fromEntries([
      ["__proto__", [{ col: 0, row: 1 }]],
      ["constructor", []],
    ]),
  };
  const native = fromJSON("BoardInspection", geometry);
  assert.deepEqual(toJSON("BoardInspection", native), geometry);
});
test("Native forwards current revisions and rejects stale work without changing old views", async () => {
  const { factory, battle, requests, stale } = await fixture();
  const before = Native.view(battle);
  Native.enemy_step(
    battle,
    fromJSON("AiOptions", { policy: "classic", diplomacy: "diplomatic", recruitmentLimit: 2 }),
  );
  assert.equal(Native.revision(Native.view(battle)), 1);
  assert.equal(Native.revision(before), 0);
  assert.equal(Native.state(before).round, 1);
  const current = Native.view(battle);
  stale();
  assert.throws(
    () => Native.dispatch(battle, fromJSON("Command", { kind: "endTurn" })),
    /staleRevision/,
  );
  assert.equal(Native.view(battle), current);
  assert.deepEqual(requests.at(-1), { id: 1, revision: 1 });
  Native.dispose_factory(factory);
});
test("Native releases all handles and frees the factory even when one cleanup fails", async () => {
  const { factory, battle, releases, frees, failClose } = await fixture();
  const second = Native.create(factory, fromJSON("Start", { kind: "scenario", map: "mirror" }), 7);
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
      () => Native.create(factory, fromJSON("Start", { kind: "scenario", map: "mirror" }), seed),
      /uint32/,
    );
  controlled.load = () => Promise.reject(new Error("Missing WASM module"));
  await assert.rejects(Native.create_factory(), /Missing WASM module/);
  Native.dispose_factory(factory);
});
test("Incoherent accepted publications release their handle without corrupting detached views", async () => {
  const { factory, battle, releases, incoherent } = await fixture();
  const old = Native.view(battle),
    failures = [];
  Native.on_failure(battle, (reason) => {
    failures.push(reason);
    assert.deepEqual(releases, [1], "Failure notification follows native handle release");
    assert.throws(() => Native.view(battle), /disposed/);
  });
  incoherent();
  assert.throws(
    () => Native.dispatch(battle, fromJSON("Command", { kind: "endTurn" })),
    /Incoherent native publication/,
  );
  assert.deepEqual(releases, [1]);
  assert.equal(Native.state(old).round, 1);
  assert.throws(() => Native.view(battle), /disposed/);
  assert.equal(failures.length, 1);
  assert.match(failures[0], /Incoherent native publication/);
  Native.dispose_factory(factory);
  assert.equal(failures.length, 1, "Further cleanup cannot notify the released owner again");
});
test("Catalog indexes detached metadata supplied through its exported constructor", async () => {
  const { factory, state } = await fixture();
  const source = {
    abilities: [],
    environment_effects: [],
    items: [],
    units: [],
    terrain: [],
    races: [],
    campaigns: [],
    maps: [
      {
        id: "small",
        revision: 1,
        name: "Small map",
        description: "One owned tile",
        tags: ["human"],
        fixed_armies: false,
        lighting: { time: "day" },
        presentation: {
          lighting: { unlit_tiles: [], lights: [] },
          fog: [],
          visible_units: [],
          unit_appearances: [],
        },
        cols: 1,
        rows: 1,
        units: [],
        terrain: [
          {
            effects: [],
            perception: { illumination: "lit", sight: null, remember: false, concealment: [] },
            id: "0,0",
            col: 0,
            row: 0,
            terrain: "road",
            owner: null,
            connections: ["e", "w"],
          },
        ],
      },
    ],
  };
  const map = source.maps[0];
  map.state = {
    ...toJSON("State", state),
    mapId: map.id,
    cols: map.cols,
    rows: map.rows,
    units: map.units,
    tiles: map.terrain,
  };
  delete map.lighting;
  delete map.cols;
  delete map.rows;
  delete map.units;
  delete map.terrain;
  const input = fromJSON("CatalogView", structuredClone(source));

  const metadata = Catalog.of_view(input);
  const selected = Catalog.map(metadata, "small"),
    tile = toArray(selected.state.tiles)[0];
  assert(tile);
  assert.equal(
    Catalog.raw(metadata),
    input,
    "Publication indexes the detached source without another copy",
  );
  assert.equal(listToArray(Catalog.maps(metadata))[0], selected);

  assert.equal(Catalog.map(metadata, "small").name, "Small map");
  assert.deepEqual(tile.connections, fromJSON("Tile", source.maps[0].state.tiles[0]).connections);
  Native.dispose_factory(factory);
});

test("An accepted update with the wrong base revision closes its owner and preserves retained data", async () => {
  const { factory, battle, incoherent_update } = await fixture();
  const before = Native.view(battle),
    failures = [];
  Native.on_failure(battle, (reason) => failures.push(reason));
  incoherent_update();
  assert.throws(
    () => Native.dispatch(battle, fromJSON("Command", { kind: "endTurn" })),
    /update revision/,
  );
  assert.equal(failures.length, 1);
  assert.equal(Native.state(before).round, 1);

  assert.throws(() => Native.view(battle), /disposed/);
  Native.dispose_factory(factory);
});
