import "./storage-fixture.mjs";
import { cellKey } from "./hex-fixture.mjs";
import { assertResultError } from "./result-fixture.mjs";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { afterEach, test, vi } from "vitest";
import { fromJSON, toJSON, toArray, mapBindings } from "./json-fixture.mjs";
import { assertionNative } from "./native-adapter.mjs";
import { required, TestStorage } from "./helpers.mjs";

const wasm = await readFile(process.env.HEX_COMMAND_WASM_PATH);
const start = () => fromJSON("Start", { kind: "scenario", map: "mirror" });

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

async function freshBindings() {
  // Poisoning is terminal for one module/page. Each prior test fully disposes
  // its owners before this explicit test-environment reset; product loading
  // never replaces a live module's global registry.
  vi.resetModules();
  vi.stubGlobal("Ocamljs_hex_command", undefined);
  Reflect.deleteProperty(globalThis, "Ocamljs_hex_command");
  const [Domain, Native, Session, Save, Setup, runtime] = await Promise.all([
    import("@frontend/Domain"),
    import("@frontend/Native"),
    import("@frontend/Session"),
    import("@frontend/Save"),
    import("@frontend/Setup"),
    import("@wasm/bindings.runtime.mjs"),
  ]);
  await runtime.initialize(wasm);
  return { Domain, Native: assertionNative(Native), Session, Save, Setup };
}

test("Native entity lookups reject prototype names and derived queries stay empty", async () => {
  const { Native } = await freshBindings();
  const factory = await Native.create_factory();
  try {
    const battle = Native.create(factory, start(), 17);
    const view = Native.view(battle);
    const state = Native.state(view);
    const actor = required(toArray(state.units)[0]);
    const tile = required(toArray(state.tiles)[0]);
    const choices = required(toArray(Native.inspection(view).units)[0]);
    assert.equal(Native.tile(view, tile.id), tile);
    assert.equal(Native.unit(view, actor.id), actor);
    assert.equal(Native.at(view, cellKey(actor.tile)), actor);
    assert.equal(Native.actions(view, choices.unit), choices);

    for (const id of ["constructor", "__proto__", "toString", "missing-entity"]) {
      for (const lookup of [Native.tile, Native.unit, Native.at, Native.actions])
        assert.equal(lookup(view, id), undefined, `${id} must not name an inherited entity`);
      assert.deepEqual(toArray(Native.neighbors(view, id)), []);
      assert.deepEqual(mapBindings("String", Native.reachable(view, id)), []);
      assert.deepEqual(toArray(Native.targets(view, id)), []);
      assert.deepEqual(toArray(Native.attack_options(view, id, actor.id)), []);
      assert.equal(Native.best_attack(view, id, actor.id), undefined);
      assert.equal(Native.attack_details(view, id, actor.id, "missing-weapon"), undefined);
      assert.deepEqual(toArray(Native.promotion_options(view, id)), []);
      assert.deepEqual(toArray(Native.recruit_tiles(view, id, actor.type_)), []);
      assert.equal(Native.movement_cost(view, id, id), Infinity);
    }
  } finally {
    Native.dispose_factory(factory);
  }
});

test.each([
  ["dispatch", "__ocamljs_fn_dispatch_update"],
  ["AI", "__ocamljs_fn_choose_ai_command"],
  ["checkpoint", "__ocamljs_fn_save"],
  ["save", "__ocamljs_fn_save_json"],
  ["opening save", "__ocamljs_fn_save_json"],
])(
  "A trapped %s invalidates all sessions sharing the module exactly once",
  async (phase, target) => {
    const storage = new TestStorage();
    storage.setItem("hex-command-sound-enabled", "false");
    vi.stubGlobal("localStorage", storage);
    let armed = false,
      injected = 0,
      callsAfterTrap = 0,
      flashAtFault,
      latestFlash,
      coherent;
    const reason = `injected ${phase} trap`;
    const instantiate = WebAssembly.instantiate;
    vi.spyOn(WebAssembly, "instantiate").mockImplementation(async (...args) => {
      const result = await Reflect.apply(instantiate, WebAssembly, args);
      const original = result instanceof WebAssembly.Instance ? result : result.instance;
      const exports = original.exports;
      const monitored = Object.fromEntries(
        Object.entries(exports).map(([name, value]) => [
          name,
          typeof value !== "function"
            ? value
            : (...args) => {
                if (injected) callsAfterTrap++;
                if (armed && name === target) {
                  injected++;
                  flashAtFault = latestFlash;
                  coherent = sessions.map(Session.view);
                  throw new WebAssembly.RuntimeError(reason);
                }
                return Reflect.apply(value, exports, args);
              },
        ]),
      );
      return result instanceof WebAssembly.Instance
        ? { exports: monitored }
        : { ...result, instance: { exports: monitored } };
    });
    const { Domain, Native, Session, Save, Setup } = await freshBindings();
    const factories = [],
      sessions = [],
      unsubscribe = [];
    try {
      factories.push(await Native.create_factory());
      factories.push(await Native.create_factory());
      sessions.push(await Session.create_async(() => Promise.resolve(), factories[0]));
      sessions.push(await Session.create_async(() => Promise.resolve(), factories[0]));
      sessions.push(await Session.create_async(() => Promise.resolve(), factories[1]));
      sessions.forEach(Session.initialize);
      await Promise.all(sessions.map(Session.saved));
      const Store = await import("./storage-adapter.mjs");
      [latestFlash] = await Store.read_session();
      const db = await Store.database();
      const proto = Object.getPrototypeOf(db.transaction("state", "readonly").objectStore("state"));
      const put = proto.put;
      vi.spyOn(proto, "put").mockImplementation(function (value, key) {
        if (this.name === "state" && key === "current") latestFlash = value;
        return Reflect.apply(put, this, [value, key]);
      });
      const session = sessions[0];
      const actions = Session.commands(session);
      const initial = Session.view(session).game;
      const before = structuredClone(toJSON("State", Native.state(initial)));
      const actor = required(before.units.find((unit) => unit.id === "u3"));
      const occupied = new Set(before.units.map((unit) => unit.tile));
      const destination = required(
        mapBindings("String", Native.reachable(initial, actor.id))
          .map(([id]) => id)
          .find((id) => id !== cellKey(actor.tile) && !occupied.has(id)),
      );
      if (phase !== "AI") actions.choose(cellKey(actor.tile));

      const failures = sessions.map(() => []);
      sessions.forEach((current, index) => {
        unsubscribe.push(
          Session.subscribe(current, (view) => {
            if (view.failure !== undefined) {
              failures[index].push(view.failure);
              // Whichever owner is notified first, its subscriber closes the
              // other factory while that factory's notifications are pending.
              Native.dispose_factory(factories[index === 2 ? 0 : 1]);
            }
          }),
        );
      });
      armed = true;

      if (phase === "AI") await actions.end_turn().catch(() => {});
      else if (phase === "opening save")
        actions.reset(
          Setup.of_json({
            player: null,
            first: "ai",
            enemy: { policy: "classic", diplomacy: "diplomatic" },
          }),
          undefined,
          "mirror",
        );
      else actions.choose(destination);

      assert.equal(injected, 1);
      assert.match(required(Domain.Wasm.failure()).message, new RegExp(reason));
      assert(flashAtFault, "A last valid checkpoint exists before the fault");
      sessions.forEach((current, index) => {
        const failed = Session.view(current);
        assert.match(required(failed.failure), new RegExp(reason));
        assert.equal(failed.game, coherent[index].game, "Keep the last coherent detached view");
        assert.equal(failed.can_undo, false);
        assert.equal(failed.map_focus.selected, undefined);
        assert.equal(failed.map_focus.target, undefined);
        assert.equal(failures[index].length, 1, "Publish one terminal transition per session");
        assert.throws(() => Session.commands(current).reset(undefined, undefined, "river-valley"));
        assert.throws(() => Session.commands(current).focus(destination));
        Session.commands(current).choose(destination);
        Session.rendered(current, failed.revision);
        assert.equal(Session.view(current), failed, "Later commands cannot replace terminal UI");
        assert.equal(failures[index].length, 1);
      });
      assertResultError(await actions.end_turn(), Session.error_message);
      for (const factory of factories) {
        assert.throws(() => Native.create(factory, start(), 17));
        assert.throws(() => Native.catalog(factory));
      }
      await assert.rejects(Native.create_factory(), new RegExp(reason));
      assert.deepEqual(toJSON("State", Native.state(initial)), before);
      assert.deepEqual(Native.unit(initial, actor.id).tile, actor.tile);
      // AI has legitimately written its pending-enemy flash save before the
      // fault. Preserve that last valid save, rather than the pre-turn baseline.
      assert.equal((await Store.read_session())[0], flashAtFault);
      sessions.forEach(Session.close);
      factories.forEach(Native.dispose_factory);
      assert.equal(
        callsAfterTrap,
        0,
        "Neither error handling nor cleanup may reenter poisoned WASM",
      );
      failures.forEach((events) => assert.equal(events.length, 1));
    } finally {
      unsubscribe.forEach((remove) => remove());
      for (const session of sessions) {
        try {
          Session.close(session);
        } catch {}
      }
      for (const factory of factories) {
        try {
          Native.dispose_factory(factory);
        } catch {}
      }
    }
  },
);
