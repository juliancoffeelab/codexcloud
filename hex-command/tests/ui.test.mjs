import "./storage-fixture.mjs";
import { cellKey } from "./hex-fixture.mjs";
// @vitest-environment jsdom
import assert from "node:assert/strict";
import { afterEach, beforeEach, test, vi } from "vitest";
import React from "preact/compat";
import { options as preactOptions } from "preact";
import { act } from "preact/test-utils";
import { createRoot } from "preact/compat/client";
import * as RawApp from "@frontend/App";
import * as Boot from "@frontend/Phaser_boot";
import * as Action from "@frontend/Action_presentation";
let renderer, audio;
import * as Sound from "@frontend/Phaser_sound";
const App = { ...RawApp, make: (props) => RawApp.make({ ...props, renderer }) };
import * as Motion from "@frontend/Motion";
import * as Shell from "@frontend/Shell";
import * as LoadingProgress from "@frontend/Loading_progress";
import * as BootProcess from "@frontend/Boot_process";
import * as BrowserDownload from "@frontend/Browser_download";
import * as BattlefieldWindow from "@frontend/Battlefield_window";
import * as RawApplication from "@frontend/Application";
import { requireResult } from "./result-fixture.mjs";
const Application = {
  ...RawApplication,
  mount: (...args) =>
    RawApplication.mount(...args).then((result) => requireResult(result, (message) => message)),
};
import * as ErrorBoundary from "@frontend/Error_boundary";
import * as Session from "@frontend/Session";
import * as Preview from "@frontend/Terrain_preview";
import * as Plan from "./plan-adapter.mjs";
import * as Viewport from "@frontend/Viewport";
import * as Art from "@frontend/Art";
import * as Save from "./save-adapter.mjs";
import * as Store from "./storage-adapter.mjs";
import * as Domain from "@frontend/Domain";
import * as FixtureJson from "@frontend/Fixture_json";
import { factory, Native, Catalog, createBattle } from "./native-fixture.mjs";
import { required, setTerrain } from "./helpers.mjs";
// These are HUD/session tests. Real Phaser WebGL and renderer teardown are
// exercised in the opt-in browser probes; jsdom has no canvas implementation.
vi.mock("@frontend/Phaser_owner", () => ({
  prepare: () => ({ closed: false }),
  dispose: (owner) => {
    owner.closed = true;
  },
  mount_battlefield: (_owner, options) => {
    const canvas = document.createElement("canvas");
    options.parent.appendChild(canvas);
    return { options, canvas };
  },
  present_battlefield: (runtime, view, reduced) => {
    const identity = `${view.session_version}:${Native.state(view.game).mapId}`;
    if (runtime.identity !== identity) {
      runtime.identity = identity;
      runtime.options.progress({
        identity,
        phase: "ready",
        loaded: 1,
        total: 1,
        audio_loaded: 0,
        audio_total: 0,
        error: undefined,
      });
    }
    if (reduced) view.combat_playback?.skip();
    if (view.action) queueMicrotask(() => Action.complete(Action.claim(view.action)));
  },
  dispose_battlefield: (runtime) => runtime.canvas.remove(),
  pulse_battlefield: () => {},
  retry_battlefield: () => {},
  commander_battlefield: () => {},
  retry_audio: () => {},
}));
vi.mock("@frontend/Motion", async (importOriginal) => ({
  ...(await importOriginal()),
  animate_tile: () => {},
  animate_turn: () => {},
  reduced_motion: vi.fn(() => false),
  subscribe_reduced: () => () => {},
}));
const { terrainManifest, rendering } = vi.hoisted(() => ({
  rendering: { failed: false, workers: [] },
  terrainManifest: {},
}));
vi.mock("virtual:terrain-prebuilt", () => ({ default: terrainManifest }));
let root, host, session;
let restoreDialogs;
let restoreCanvasTransfer;
let restoreScroll;
let applicationCleanup;
beforeEach(() => {
  audio = Sound.create();
  renderer = Boot.start(audio);
  const prototype = HTMLDialogElement.prototype;
  const previous = ["showModal", "close"].map((name) => [
    name,
    Object.getOwnPropertyDescriptor(prototype, name),
  ]);
  Object.defineProperties(prototype, {
    showModal: {
      configurable: true,
      value() {
        this.setAttribute("open", "");
      },
    },
    close: {
      configurable: true,
      value() {
        this.removeAttribute("open");
        this.dispatchEvent(new Event("close"));
      },
    },
  });
  const oldScroll = Object.getOwnPropertyDescriptor(Element.prototype, "scrollIntoView");
  Object.defineProperty(Element.prototype, "scrollIntoView", { configurable: true, value() {} });
  restoreScroll = () => {
    if (oldScroll) Object.defineProperty(Element.prototype, "scrollIntoView", oldScroll);
    else Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  };
  restoreDialogs = () => {
    for (const [name, descriptor] of previous) {
      if (descriptor) Object.defineProperty(prototype, name, descriptor);
      else Reflect.deleteProperty(prototype, name);
    }
  };
  rendering.failed = false;
  rendering.workers.length = 0;
  localStorage.clear();
  Reflect.deleteProperty(document, "modelContext");
  for (const key of Object.keys(terrainManifest)) delete terrainManifest[key];
  const canvasPrototype = HTMLCanvasElement.prototype;
  const previousTransfer = Object.getOwnPropertyDescriptor(
    canvasPrototype,
    "transferControlToOffscreen",
  );
  Object.defineProperty(canvasPrototype, "transferControlToOffscreen", {
    configurable: true,
    value: vi.fn(() => ({})),
  });
  restoreCanvasTransfer = () => {
    if (previousTransfer)
      Object.defineProperty(canvasPrototype, "transferControlToOffscreen", previousTransfer);
    else Reflect.deleteProperty(canvasPrototype, "transferControlToOffscreen");
  };
  vi.stubGlobal(
    "Image",
    class {
      constructor() {
        const image = document.createElement("img");
        image.decode = () => Promise.resolve();
        return image;
      }
    },
  );
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
});
afterEach(async () => {
  vi.useRealTimers();
  await act(async () => {
    root?.unmount();
  });
  applicationCleanup?.();
  Boot.close(renderer);
  Reflect.deleteProperty(document, "modelContext");
  if (session) Session.close(session);
  Sound.close(audio);
  host.remove();
  root = void 0;
  session = void 0;
  applicationCleanup = void 0;
  restoreDialogs();
  restoreCanvasTransfer();
  restoreScroll();
});
function element(selector) {
  return required(host.querySelector(selector));
}
async function click(selector) {
  await act(async () => {
    element(selector).dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}
test("The HUD keeps one native canvas through a session replacement and disposes it on unmount", async () => {
  session = await Session.create_with_audio(async () => {}, audio, factory);
  await act(async () =>
    root.render(React.createElement(App.make, { session, onMount: () => () => {} })),
  );
  const canvas = element("#map canvas");
  await act(async () => Session.commands(session).reset(undefined, undefined, "mirror"));
  assert.equal(element("#map canvas"), canvas);
  await act(async () => root.unmount());
  root = undefined;
  assert.equal(canvas.isConnected, false);
});
test("The real embedding mounts native presentation, browser tools and reusable HUD menus", async () => {
  const tools = new Map(),
    removed = [];
  Object.defineProperty(document, "modelContext", {
    configurable: true,
    value: {
      registerTool: (tool) => tools.set(tool.name, tool),
      unregisterTool: (name) => removed.push(name),
    },
  });
  await act(async () => {
    applicationCleanup = await Application.mount(
      required(root),
      host,
      () => true,
      (error) => {
        throw error;
      },
    );
  });
  assert.equal(tools.size, 11);
  assert.equal((await tools.get("get_battle_state").execute()).mapId, "mirror");
  assert.equal(element("#map").getAttribute("data-phaser-battlefield"), "");
  assert.equal(
    host.querySelector(".new-game-dialog .setup-map-preview"),
    null,
    "Closed map choices do not construct hidden previews",
  );
  await click("#settings-button");
  await click("#new-game");
  assert.ok(element(".new-game-dialog .ng-choice"));
  assert.equal(host.querySelector(".choice-dialog"), null, "New Game has no nested picker dialogs");
  await click(".ng-modes button:nth-child(2)");
  assert.ok(element(".new-game-dialog .setup-map-preview polygon"));
  await click(".ng-choice");
  const strip = element('.ng-strip[aria-label="Your race"]');
  const choices = [...strip.querySelectorAll('[role="radio"]')];
  await act(async () => {
    choices[0].focus();
    choices[0].dispatchEvent(new KeyboardEvent("keydown", { key: "End", bubbles: true }));
  });
  assert.equal(document.activeElement, choices.at(-1));
  assert.equal(choices.at(-1).getAttribute("aria-checked"), "true");
  await click(".ng-start-footer button:first-child");
  await click(".ng-browse-footer > button");
  assert.equal(
    host.querySelector(".new-game-dialog .setup-map-preview"),
    null,
    "Closing releases preview nodes",
  );
  await click("#new-game");
  await act(async () => element(".new-game-dialog").dispatchEvent(new Event("close")));
  assert.ok(
    host.querySelector(".new-game-dialog .ng-choice"),
    "A stale close event cannot clear a reopened dialog",
  );
  await act(async () => element(".new-game-dialog").close());
  assert.equal(host.querySelector(".new-game-dialog .ng-choice"), null);
  await click("#settings-button");
  await click("#new-game");
  await click(".ng-choice");
  assert.ok(element(".ng-campaign-info"));
  await act(async () => element(".new-game-dialog").close());
  assert.equal(host.querySelector(".ng-campaign-info"), null);
  assert.equal(host.querySelector(".new-game-dialog .setup-map-preview"), null);
  const cleanup = required(applicationCleanup);
  assert.doesNotThrow(cleanup);
  assert.doesNotThrow(cleanup);
  assert.deepEqual(removed.sort(), [...tools.keys()].sort());
});
test("Startup and live controls share their structure, and inspection panels open and dismiss coherently", async () => {
  const stopMap = vi.fn(),
    stopAttack = vi.fn();
  const revealMap = vi.spyOn(Viewport, "reveal_map").mockReturnValue(stopMap);
  const revealAttack = vi.spyOn(Viewport, "reveal_attack").mockReturnValue(stopAttack);
  await act(async () => {
    required(root).render(React.createElement(Shell.make, { failure: void 0 }));
  });
  const structure = () =>
    Array.from(element("header").querySelectorAll("*"), (node) => [
      node.tagName,
      node.id,
      node.getAttribute("class"),
      node.getAttribute("aria-label"),
      node.getAttribute("d"),
    ]);
  const frameStructure = () =>
    [".map-section", "#map", ".battlefield-clip"].map((selector) => {
      const node = element(selector);
      return [
        node.tagName,
        [...node.classList].filter((name) => name !== "map-ready"),
        node.getAttribute("data-phaser-battlefield"),
      ];
    });
  const loading = structure(),
    loadingFrame = frameStructure();
  assert.equal(element("#gold").textContent?.trim(), "\u2014");
  assert.equal(element("#settings-button").disabled, true);
  assert.equal(host.querySelector("aside"), null);
  assert.equal(revealMap.mock.calls.length, 0);
  session = await Session.create_with_audio(async () => {}, audio, factory);
  const mounted = session;
  await act(async () => {
    required(root).render(
      React.createElement(App.make, {
        session: mounted,
        onMount: () => () => {},
      }),
    );
  });
  assert.deepEqual(structure(), loading);
  assert.deepEqual(frameStructure(), loadingFrame);
  assert.equal(element("#end-turn").disabled, element("#map").getAttribute("aria-busy") === "true");
  assert.equal(element("#settings-button").disabled, false);
  assert(element(".workspace").classList.contains("map-only"));
  assert.equal(host.querySelector("aside"), null);
  const unit = required(
    Native.state(Session.view(mounted).game).units.find((unit2) => unit2.side === "player"),
  );
  await act(async () => Session.commands(mounted).choose(cellKey(unit.tile)));
  assert.equal(element("aside").getAttribute("aria-label"), "Unit and terrain details");
  await click("#status");
  assert.equal(
    host.querySelector("aside"),
    null,
    "Click-away clears selection and inspection together",
  );
  await act(async () => Session.commands(mounted).choose(cellKey(unit.tile)));
  await act(async () => {
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
  });
  assert.equal(host.querySelector("aside"), null);
});
test("Terrain details use one complete canonical image while battlefield directions and floors retain their authored images", async () => {
  const battle = createBattle({ kind: "scenario", map: "mirror" });
  const checkpoint = structuredClone(Native.save(battle));
  const tiles = checkpoint.state.tiles
    .filter(
      (tile) =>
        tile.terrain === "plains" &&
        !checkpoint.state.units.some((unit) => cellKey(unit.tile) === tile.id),
    )
    .slice(0, 2);
  const first = required(tiles[0]),
    second = required(tiles[1]);
  delete first.floor;
  delete first.deadEnds;
  delete second.deadEnds;
  Object.assign(first, { terrain: "road", connections: ["e", "w"] });
  Object.assign(second, { terrain: "road", connections: ["ne", "sw"], floor: "snow-forest" });
  await Save.write(Save.create(), checkpoint, false);
  session = await Session.create_with_audio(async () => {}, audio, factory);
  const mounted = session,
    published = Session.view(mounted);
  assert.equal(required(Native.tile(published.game, second.id)).floor, "snow-forest");
  await act(async () => {
    required(root).render(
      React.createElement(App.make, {
        session: mounted,
        onMount: () => () => {},
      }),
    );
  });
  const scenes = tiles.map(
    (tile) =>
      Plan.prepare(
        checkpoint.state.mapId,
        checkpoint.state.cols,
        checkpoint.state.rows,
        [tile],
        published.artwork,
      ).tiles[0],
  );
  assert.notEqual(
    scenes[0].frame.url,
    scenes[1].frame.url,
    "Worker frame plans retain authored directions and floors",
  );
  const canonical = required(Art.terrain_portrait(published.artwork, "road"));
  for (const tile of tiles) {
    await act(async () => Session.commands(mounted).choose(tile.id));
    assert.equal(element("#terrain-panel .preview-image").getAttribute("aria-hidden"), "true");
    assert.equal(Art.terrain_portrait(published.artwork, tile.terrain), canonical);
  }
});
test("An accepted native command with failed inspection replaces battle controls with reload and preserves saved bytes", async () => {
  session = await Session.create_with_audio(async () => {}, audio, factory);
  const mounted = session;
  await act(async () => {
    required(root).render(
      React.createElement(App.make, {
        session: mounted,
        onMount: () => {
          Session.initialize(mounted);
          return () => {};
        },
      }),
    );
  });
  const game = Session.view(mounted).game,
    opening = Native.state(game);
  const unit = required(
    opening.units.find(
      (unit2) =>
        unit2.side === "player" &&
        Object.keys(Native.reachable(game, unit2.id)).some(
          (id) => id !== cellKey(unit2.tile) && !Native.at(game, id),
        ),
    ),
  );
  const destination = required(
    Object.keys(Native.reachable(game, unit.id)).find(
      (id) => id !== cellKey(unit.tile) && !Native.at(game, id),
    ),
  );
  await act(async () => Session.commands(mounted).choose(cellKey(unit.tile)));
  const saved = required(await Save.flashText());
  const originalDispatch = Domain.NativeEngineFactory.dispatch_update;
  vi.spyOn(Domain.NativeEngineFactory, "dispatch_update").mockImplementation((...args) => {
    const result = originalDispatch(...args);
    return { ...result, _0: { ...result._0, inspection: undefined } };
  });
  vi.spyOn(Domain.NativeEngineFactory, "inspect").mockImplementation(() => {
    throw new Error("Inspection unavailable");
  });
  await assert.doesNotReject(() => act(async () => Session.commands(mounted).choose(destination)));
  assert.match(element(".startup-failure").textContent ?? "", /Reloading keeps your saved battle/);
  assert.equal(element(".startup-failure button").textContent?.trim(), "Reload game");
  assert.match(element(".startup-failure pre").textContent ?? "", /Inspection unavailable/);
  assert.equal(element("#end-turn").disabled, true);
  assert.equal(element("#settings-button").disabled, true);
  assert.equal(host.querySelector("#map canvas"), null);
  assert.equal(element("#map").getAttribute("aria-busy"), "false");
  assert.equal(await Save.flashText(), saved);
  assert.equal(
    Session.view(mounted).game,
    game,
    "The last detached coherent view remains available",
  );
  assert.deepEqual(required(Native.unit(Session.view(mounted).game, unit.id)).tile, unit.tile);
  await act(async () => {
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    element(".startup-failure p").dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
  assert.equal(await Save.flashText(), saved);
});
test("Embedding cleanup frees its factory even when battle disposal fails", async () => {
  await act(async () => {
    applicationCleanup = await Application.mount(
      required(root),
      host,
      () => true,
      (error) => {
        throw error;
      },
    );
  });
  const dispose = vi
    .spyOn(Domain.NativeEngineFactory, "dispose_battle")
    .mockImplementationOnce(() => {
      throw new Error("Battle cleanup failed");
    });
  const free = vi.spyOn(Domain.NativeEngineFactory, "dispose");
  assert.throws(required(applicationCleanup), /Battle cleanup failed/);
  assert.equal(dispose.mock.calls.length, 1);
  assert.equal(free.mock.calls.length, 1);
  assert.doesNotThrow(required(applicationCleanup));
  assert.equal(free.mock.calls.length, 1);
});

for (const phase of ["render", "layout effect"]) {
  test(`The Preact boundary reports a descendant ${phase} failure`, async () => {
    const error = new Error(`controlled ${phase} fault`),
      received = [];
    function Failing() {
      React.useLayoutEffect(() => {
        if (phase === "layout effect") throw error;
      }, []);
      if (phase === "render") throw error;
      return React.createElement("span", null, "mounted");
    }
    await act(async () => {
      root.render(
        React.createElement(ErrorBoundary.make, {
          onError: (failure) => received.push(failure),
          render: () => React.createElement(Failing),
        }),
      );
    });
    assert.deepEqual(received, [error]);
    assert.equal(host.textContent, "");
    await act(async () => root.render(React.createElement(Shell.make, { failure: "Recovered" })));
    assert.match(host.textContent, /Recovered/);
    assert.equal(host.querySelector(".startup-failure button").textContent, "Reload game");
  });
}

test.each(["complete", "cancel", "reduced", "no-op"])(
  "Combat replay keeps live state coherent during %s",
  async (mode) => {
    vi.mocked(Motion.reduced_motion).mockReturnValue(mode === "reduced");
    try {
      const battle = createBattle({ kind: "scenario", map: "mirror" });
      const checkpoint = structuredClone(Native.save(battle));
      checkpoint.state.units = [
        {
          experience: 0,
          id: "u1",
          type: "soldier",
          side: "player",
          tile: { col: 2, row: 3 },
          hp: Catalog.unit(Native.catalog(factory), "soldier").hp,
          moved: false,
          attackSpent: false,
          faction: "player",
          revealedTo: [],
          perceptionEffects: [],
        },
        {
          experience: 0,
          id: "u2",
          type: "soldier",
          side: "ai",
          tile: { col: 3, row: 3 },
          hp: Catalog.unit(Native.catalog(factory), "soldier").hp,
          moved: false,
          attackSpent: false,
          faction: "enemy",
          revealedTo: [],
          perceptionEffects: [],
        },
      ];
      checkpoint.state.nextId = 3;
      for (const tile of checkpoint.state.tiles) {
        setTerrain(tile, "plains");
        tile.owner = null;
      }
      await Save.write(Save.create(), checkpoint, false);
      session = await Session.create_with_audio(async () => {}, audio, factory);
      vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "Date"] });
      const mounted = session;
      const Panels = await import("@frontend/Panels");
      applicationCleanup = Session.subscribe(mounted, (view) =>
        root.render(
          React.createElement(Panels.Unit.make, {
            view: { catalog: view.catalog, artwork: view.artwork, game: view.interaction_game },
            hud: view.hud,
            actions:
              mode === "no-op"
                ? { ...Session.commands(mounted), attack: () => {} }
                : Session.commands(mounted),
            onNewBattle: () => {},
          }),
        ),
      );
      await act(async () => {
        Session.commands(mounted).choose("2,3");
        Session.commands(mounted).choose("3,3");
      });
      const hpBefore = [...host.querySelectorAll(".health-numbers b")].map(
        (node) => node.textContent,
      );
      const before = Session.view(mounted);
      await click('[data-attack="mace"]');
      if (mode === "no-op") {
        assert.equal(Session.view(mounted), before, "The no-op publishes no replacement model");
        assert.equal(element("#unit-panel").classList.contains("resolving"), false);
        assert.equal(element('[data-attack="mace"]').disabled, false);
        assert.equal(host.querySelector(".exchange-log"), null);
        await act(async () => vi.runAllTimers());
        assert.equal(element('[data-attack="mace"]').disabled, false);
        return;
      }
      const combat = required(Native.combat(Session.view(mounted).interaction_game));
      assert(combat.strikes.length > 0);
      if (mode === "reduced") {
        await act(async () => Session.view(mounted).combat_playback.skip());
        assert(host.querySelector(".exchange-totals"));
        assert.deepEqual(
          [...host.querySelectorAll(".health-numbers b")].map((node) => Number(node.textContent)),
          combat.strikes.at(-1).health,
        );
        return;
      }
      assert.deepEqual(
        [...host.querySelectorAll(".health-numbers b")].map((node) => node.textContent),
        hpBefore,
      );
      assert.equal(host.querySelectorAll(".exchange-event").length, 0);
      assert(element("#unit-panel").classList.contains("resolving"));
      await act(async () => vi.advanceTimersByTime(199));
      assert.equal(host.querySelectorAll(".exchange-event").length, 0);
      await act(async () => Session.view(mounted).combat_playback.advance(0));
      assert(host.querySelectorAll(".exchange-event").length > 0);
      assert.deepEqual(
        [...host.querySelectorAll(".health-numbers b")].map((node) => Number(node.textContent)),
        combat.strikes[0].health,
      );
      await click(".exchange-inspect");
      assert.equal(element(".exchange-inspect").getAttribute("aria-expanded"), "true");
      if (mode === "complete") {
        await act(async () => {
          for (let i = 1; i < combat.strikes.length; i++)
            Session.view(mounted).combat_playback.advance(i);
          Session.view(mounted).combat_playback.finish();
        });
        assert(host.querySelector(".exchange-totals"));
        assert.deepEqual(
          [...host.querySelectorAll(".health-numbers b")].map((node) => Number(node.textContent)),
          combat.strikes.at(-1).health,
        );
        await act(async () => root.unmount());
        root = undefined;
        await act(async () => vi.runAllTimers());
        return;
      }
      await act(async () => Session.commands(mounted).choose("0,0"));
      assert.equal(host.querySelector(".exchange-log"), null);
      await act(async () => vi.runAllTimers());
      assert.equal(host.querySelector(".exchange-log"), null);
    } finally {
      vi.useRealTimers();
      vi.mocked(Motion.reduced_motion).mockReturnValue(false);
    }
  },
);

test.each(["scenario", "campaign"])(
  "%s New Game filters playable content and retains typed setup choices through start",
  async (mode) => {
    session = await Session.create_with_audio(async () => {}, audio, factory);
    const mounted = session;
    await act(async () =>
      root.render(React.createElement(App.make, { session: mounted, onMount: () => () => {} })),
    );
    await click("#settings-button");
    await click("#new-game");
    const catalog = Session.view(mounted).catalog;
    const choices = () =>
      [...host.querySelectorAll(".ng-choice > strong")].map((node) =>
        node.textContent.toLowerCase(),
      );
    assert.deepEqual(
      choices(),
      Catalog.campaigns(catalog)
        .filter((campaign) => campaign.human)
        .map((campaign) => campaign.name.toLowerCase()),
    );
    await click(".ng-modes button:nth-child(2)");
    const scenarios = Catalog.scenario_maps(catalog);
    assert.equal(choices().length, scenarios.filter((map) => map.human).length);
    for (const map of scenarios.filter((map) => !map.human))
      assert(!choices().includes(map.name.toLowerCase()));
    if (mode === "campaign") await click(".ng-modes button:first-child");
    await click(".ng-choice");
    assert.equal(
      host.querySelector('[role="group"][aria-label="First turn"]') !== null,
      mode === "scenario",
    );
    const choose = async (group, value) => {
      const container = element(`[role="group"][aria-label="${group}"]`);
      const target = required(
        [...container.querySelectorAll("button")].find((button) => button.textContent === value),
      );
      await act(async () => target.click());
    };
    await click(".new-game-dialog .advanced > summary");
    await choose("Opponent planning", "Full");
    await click(".ng-recruitment > summary");
    const aggressive = required(
      [...host.querySelectorAll(".ng-recruit-option")].find((row) =>
        row.textContent.includes("Aggressive Predictor"),
      ),
    );
    await act(async () => aggressive.querySelector("input").click());
    assert.equal(element('.ng-difficulty button[aria-pressed="true"]').textContent, "Custom");
    await click(".new-game-dialog .ng-start-footer .primary");
    const saved = JSON.parse(await Save.flashText());
    assert.deepEqual(saved.setup, {
      player: null,
      first: "player",
      enemy: {
        policy: "predictor",
        diplomacy: "diplomatic",
        blocks: { planner: "full", recruiter: "aggressive-predictor", campaign: "mission" },
      },
    });
    if (mode === "scenario")
      assert.deepEqual(saved.checkpoint.state.armies, ["human", "skeletons"]);
    else
      assert.equal(
        saved.checkpoint.state.campaign.id,
        Catalog.campaigns(catalog).find((campaign) => campaign.human).id,
      );
  },
);

test("Sound-only dialogue waits for phase persistence and advances when that same game settles", async () => {
  const checkpoint = structuredClone(
    Native.save(createBattle({ kind: "campaign", id: "lantern-road" })),
  );
  checkpoint.state.campaign.dialogue = [
    { portrait: "swordsman", speaker: "", text: "", sound: "bell" },
  ];
  checkpoint.state.campaign.dialogueIndex = 0;
  await Save.write(Save.create(), checkpoint, false);
  session = await Session.create_with_audio(async () => {}, audio, factory);
  const { Dialogue } = await import("@frontend/Dialogs");
  const advance = vi.fn(),
    actions = { ...Session.commands(session), dialogue: advance };
  const view = Session.view(session),
    render = (settling) =>
      root.render(
        React.createElement(Dialogue.make, {
          view: { ...view, settling },
          actions,
          narrate: () => {},
        }),
      );
  await act(async () => render(true));
  assert.equal(advance.mock.calls.length, 0);
  await act(async () => render(false));
  assert.deepEqual(advance.mock.calls, [[false]]);
});

test("Live dialogue reveals text without delaying Next or Back and cancels its voice on dismissal", async () => {
  const checkpoint = structuredClone(
    Native.save(createBattle({ kind: "campaign", id: "lantern-road" })),
  );
  checkpoint.state.campaign.dialogue = [
    { portrait: "swordsman", character: "ivo", speaker: "Ivo", text: "Wait here." },
    { portrait: "swordsman", character: "nera", speaker: "Nera", text: "The ferry is ready." },
  ];
  checkpoint.state.campaign.dialogueIndex = 0;
  await Save.write(Save.create(), checkpoint, false);
  session = await Session.create_with_audio(async () => {}, audio, factory);
  const { VoicePlayer } = await import("../src/presentation/voice/player.mjs");
  const { Dialogue } = await import("@frontend/Dialogs");
  const worker = { postMessage: vi.fn(), terminate: vi.fn() };
  const voice = new VoicePlayer(() => {}, { workerFactory: () => worker });
  vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  vi.spyOn(performance, "now").mockReturnValue(0);
  const frames = new Map();
  let sequence = 0;
  vi.stubGlobal("requestAnimationFrame", (fn) => {
    frames.set(++sequence, fn);
    return sequence;
  });
  vi.stubGlobal("cancelAnimationFrame", (id) => frames.delete(id));
  const render = () =>
    root.render(
      React.createElement(Dialogue.make, {
        view: Session.view(session),
        actions: Session.commands(session),
        narrate: () => {},
        voice,
        scope: "live-dialogue",
      }),
    );
  try {
    await act(async () => render());
    assert.equal(element(".voice-visible").textContent, "W");
    assert.equal(element(".voice-accessible").textContent, "Wait here.");
    assert.equal(element(".story-next").disabled, false);
    assert.equal(worker.postMessage.mock.calls.length, 1, "voice preparation starts on mount");
    await click(".story-next");
    await act(async () => render());
    assert.equal(element(".voice-visible").textContent, "T");
    assert.equal(frames.size, 1, "previous reveal was cancelled");
    await click(".story-back");
    await act(async () => render());
    assert.equal(element(".voice-visible").textContent, "W");
    await act(async () => {
      Session.commands(session).dialogue(true);
      render();
    });
    assert.equal(host.querySelector(".voice-visible"), null);
    assert.equal(frames.size, 0);
    assert.equal(voice.active, null);
  } finally {
    voice.close();
  }
});

test("Campaign opening reuses the loading screen until two actual dialogue tracks are ready", async () => {
  const checkpoint = structuredClone(
    Native.save(createBattle({ kind: "campaign", id: "lantern-road" })),
  );
  await Save.write(Save.create(), checkpoint, false);
  session = await Session.create_with_audio(async () => {}, audio, factory);
  vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  const jobs = [];
  vi.stubGlobal(
    "Worker",
    class {
      postMessage(job) {
        jobs.push({ worker: this, job });
      }
      terminate() {}
    },
  );
  await act(async () =>
    root.render(React.createElement(App.make, { session, onMount: () => () => {} })),
  );
  const canvas = element("#map canvas");
  await act(async () => new Promise((resolve) => setTimeout(resolve, 0)));
  assert.match(element(".loading-action").textContent, /Trying to pick words/);
  assert.match(element(".loading-detail").textContent, /0 \/ 2 voices ready/);
  assert.equal(host.querySelector(".story-dialog"), null);
  assert.equal(element("#end-turn").disabled, true);
  const complete = async () =>
    act(async () => {
      const { worker, job } = jobs.shift();
      worker.onmessage({ data: { ...job, samples: new Float32Array(8), ms: 1 } });
    });
  await complete();
  assert.match(element(".loading-detail").textContent, /1 \/ 2 voices ready/);
  assert.equal(host.querySelector(".story-dialog"), null);
  await complete();
  assert.equal(
    element("#map canvas"),
    canvas,
    "the renderer is retained through voice preparation",
  );
  assert.equal(element(".voice-visible").textContent.length, 1);
  assert(element(".map-loading").classList.contains("is-complete"));
  await click(".story-next");
  assert.equal(element(".story-count").textContent.trim().startsWith("2 /"), true);
  assert.equal(
    host.querySelector(".map-loading:not(.is-complete)"),
    null,
    "later dialogue does not re-lock",
  );
});

test("One determinate loading bar retains progress when the shell becomes the battlefield", async () => {
  LoadingProgress.begin_battlefield();
  await act(async () => root.render(React.createElement(Shell.make, { failure: undefined })));
  assert.equal(host.querySelectorAll("progress").length, 1);
  assert.equal(Number(element("progress").getAttribute("value")), 2);
  await act(async () => LoadingProgress.advance(0.5, "6 / 12 resources ready", 4));
  const value = element("progress").getAttribute("value");
  assert.equal(element("progress").max, 5);
  assert.match(element(".loading-card").textContent, /Step 4 \/ 5.*6 \/ 12/);
  await act(async () => root.render(React.createElement(BattlefieldWindow.make, {})));
  assert.equal(host.querySelectorAll("progress").length, 1);
  assert.equal(element("progress").getAttribute("value"), value);
  await act(async () => LoadingProgress.advance(undefined, undefined, 5));
  assert.match(element(".loading-card").textContent, /Drawing battlefield/);
  assert.equal(Number(element("progress").getAttribute("value")), 4);
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
  await act(async () => {
    LoadingProgress.finish();
    root.render(React.createElement(BattlefieldWindow.make, { phase: 2 }));
  });
  assert.equal(Number(element("progress").getAttribute("value")), 5);
  assert.match(element(".loading-action").textContent, /Battlefield ready/);
  await act(async () => vi.advanceTimersByTime(250));
  assert.equal(host.querySelector("progress"), null);
});

test("The debug-report action includes retained startup diagnostics in its downloaded JSON", async () => {
  const previous = window.BOOT_PROCESS;
  delete window.BOOT_PROCESS;
  try {
    BootProcess.snapshot();
    BootProcess.progress(4, "Loading resources", 3.5, "5 / 10 ready");
    BootProcess.response("/audio.wav", 200, "phaser-xhr", (name) =>
      name === "cache-control" ? "max-age=0" : null,
    );
    BootProcess.progress(5, "Drawing battlefield", 5, "");
    const expected = BootProcess.snapshot();
    const downloaded = new Promise((resolve) => {
      vi.spyOn(BrowserDownload, "json").mockImplementation((_revoke, _filename, text) => {
        resolve(JSON.parse(text));
      });
    });
    session = await Session.create_with_audio(async () => {}, audio, factory);
    Session.commands(session).download_debug_report();
    const report = await downloaded;
    const { browserTiming: _expectedTiming, ...expectedBoot } = expected;
    const { browserTiming, ...exportedBoot } = report.bootProcess;
    assert.deepEqual(exportedBoot, expectedBoot);
    assert(browserTiming);
    assert.equal(browserTiming.startupCutoffAtMs, expected.startedAtMs + expected.elapsedMs);
    assert.equal(report.format, "hex-command-debug");
    assert.equal(report.bootProcess.responseHeaders[0].headers["cache-control"], "max-age=0");
    assert(report.checkpoint);
    assert(report.browserSaves);
  } finally {
    window.BOOT_PROCESS = previous;
  }
});
