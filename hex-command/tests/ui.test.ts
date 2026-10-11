// @vitest-environment jsdom
import assert from "node:assert/strict";
import { afterEach, beforeEach, test, vi } from "vitest";
import React, { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import * as App from "@frontend/App";
import * as Application from "@frontend/Application";
import * as Session from "@frontend/Session";
import * as Prefetch from "@frontend/Terrain_prefetch";
import * as Plan from "@frontend/Terrain_plan";
import { factory, Native, Catalog, createBattle } from "./native-fixture";
import { required } from "./helpers";

// Rendering tests use an unavailable canvas; the native terrain owner therefore
// keeps its individual image fallback. No network, decoding or timer is needed.
vi.mock("@frontend/Motion", () => ({ animate_tile: () => {}, animate_turn: () => {} }));
const { terrainManifest } = vi.hoisted(() => ({
  terrainManifest: {} as Record<string, import("@frontend/Terrain").Sprite>,
}));
vi.mock("virtual:terrain-prebuilt", () => ({ default: terrainManifest }));
let root: Root | undefined, host: HTMLDivElement, session: Session.t | undefined;
let applicationCleanup: (() => void) | undefined;

beforeEach(() => {
  localStorage.clear();
  for (const key of Object.keys(terrainManifest)) delete terrainManifest[key];
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
});

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  applicationCleanup?.();
  if (session) Session.close(session);
  host.remove();
  root = undefined;
  session = undefined;
  applicationCleanup = undefined;
});

function element<T extends Element = HTMLElement>(selector: string): T {
  return required(host.querySelector<T>(selector));
}

async function click(selector: string) {
  await act(async () => {
    element(selector).dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

test("The React application publishes coherent controls and retains unit and health nodes through movement and undo", async () => {
  session = Session.create(async () => {}, factory);
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
    opening = Native.state(game),
    unit = required(
      opening.units.find(
        (candidate) =>
          candidate.side === "player" &&
          Object.keys(Native.reachable(game, candidate.id)).some(
            (id) => id !== candidate.tile && !Native.at(game, id),
          ),
      ),
    ),
    destination = required(
      Object.keys(Native.reachable(game, unit.id)).find(
        (id) => id !== unit.tile && !Native.at(game, id),
      ),
    ),
    sprite = element<SVGImageElement>(`[data-unit="${unit.id}"] image`),
    health = element<SVGRectElement>(`[data-health-unit="${unit.id}"] .hp`);
  assert.equal(element("#gold").textContent?.trim(), String(opening.gold.player));
  assert.equal(element("#round").textContent?.trim(), `Turn ${opening.round}`);
  assert.equal(element("#status").textContent?.trim(), Session.view(mounted).status);
  const tile = element<SVGGElement>(`[data-tile="${unit.tile}"]`);
  assert.equal(tile.getAttribute("role"), "button");
  assert.match(required(tile.getAttribute("aria-label")), /your .+health/);
  await click(`[data-tile="${unit.tile}"]`);
  const accessibleDestination = element<SVGGElement>(`[data-tile="${destination}"]`);
  assert.match(required(accessibleDestination.getAttribute("aria-label")), /move here/);
  await click(`[data-tile="${destination}"]`);
  assert.equal(required(Native.unit(Session.view(mounted).game, unit.id)).tile, destination);
  assert.equal(element(`[data-unit="${unit.id}"] image`), sprite);
  assert.equal(element(`[data-health-unit="${unit.id}"] .hp`), health);
  assert.equal(element(`[data-unit="${unit.id}"]`).getAttribute("data-unit-tile"), destination);
  assert.equal(element<HTMLButtonElement>("#undo-move").disabled, false);
  await click("#undo-move");
  assert.equal(required(Native.unit(Session.view(mounted).game, unit.id)).tile, unit.tile);
  assert.equal(element(`[data-unit="${unit.id}"] image`), sprite);
  assert.equal(element(`[data-health-unit="${unit.id}"] .hp`), health);
});

test("The real embedding mounts a React component and reports foreground artwork failure visibly", async () => {
  await act(async () => {
    applicationCleanup = await Application.mount(required(root), host, () => true);
  });
  assert.equal(element("#map").getAttribute("role"), "group");
  const foreground = element<SVGImageElement>(".unit-art image");
  await act(async () => {
    foreground.dispatchEvent(new Event("error"));
  });
  assert.match(element("#status").textContent ?? "", /artwork|image|sprite/i);
  assert.match(element("#status").textContent ?? "", /failed|could not|unavailable/i);
  const cleanup = required(applicationCleanup);
  assert.doesNotThrow(cleanup);
  assert.doesNotThrow(cleanup);
});

test("Terrain inspection warming waits for the current map image and cancels when its scene changes", async () => {
  const stopped = vi.fn(),
    start = vi.spyOn(Prefetch, "start").mockReturnValue(stopped);
  session = Session.create(async () => {}, factory);
  const mounted = session,
    view = Session.view(mounted),
    opening = Native.state(view.game);
  const next = required(Catalog.battle_maps(view.catalog).find((map) => map.id !== opening.mapId));
  for (const game of [
    opening,
    Native.state(Native.view(createBattle({ kind: "scenario", map: next.id }))),
  ]) {
    const plan = Plan.prepare(game.mapId, game.cols, game.rows, game.tiles, view.artwork);
    const size = Plan.dimensions(plan);
    terrainManifest[plan.key] = {
      url: `${game.mapId}.png`,
      width: size.width,
      height: size.height,
      displayWidth: size.display_width,
      displayHeight: size.display_height,
    };
  }
  await act(async () => {
    required(root).render(
      React.createElement(App.make, {
        session: mounted,
        onMount: () => () => {},
      }),
    );
  });
  const original = element<SVGImageElement>(".map-terrain");
  assert.equal(start.mock.calls.length, 0);
  await act(async () => {
    original.dispatchEvent(new Event("load"));
  });
  assert.equal(start.mock.calls.length, 1);
  await act(async () => {
    Session.commands(mounted).reset(next.id);
  });
  assert.equal(stopped.mock.calls.length, 1);
  await act(async () => {
    original.dispatchEvent(new Event("load"));
  });
  assert.equal(start.mock.calls.length, 1, "Detached stale map loads cannot warm a new scene");
  await act(async () => {
    element(".map-terrain").dispatchEvent(new Event("load"));
  });
  assert.equal(start.mock.calls.length, 2);
  await act(async () => {
    required(root).unmount();
  });
  root = undefined;
  assert.equal(stopped.mock.calls.length, 2);
});
