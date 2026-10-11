import assert from "node:assert/strict";
import { test, vi } from "vitest";
import * as Prefetch from "@frontend/Terrain_prefetch";
import * as Terrain from "@frontend/Terrain";
import * as Art from "@frontend/Art";
import type { State } from "../rust/pkg/hex_command";
import { required } from "./helpers";

function idleQueue() {
  const tasks: { run: () => void; cancelled: boolean }[] = [];
  return {
    defer(run: () => void) {
      const task = { run, cancelled: false };
      tasks.push(task);
      return () => {
        task.cancelled = true;
      };
    },
    flush() {
      for (const task of tasks.splice(0)) if (!task.cancelled) task.run();
    },
    flushStale() {
      for (const task of tasks.splice(0)) task.run();
    },
  };
}
/** Synthetic native frame metadata drives the actual OCaml scene projection. */
function scene(urls: string[]): Terrain.Scene {
  const state: State = {
    mapId: "synthetic",
    mapRevision: 1,
    cols: urls.length,
    rows: 1,
    round: 1,
    turn: "player",
    gold: { player: 0, enemy: 0 },
    winner: null,
    nextId: 1,
    log: [],
    units: [],
    tiles: urls.map((_, col) => ({
      id: `${col},0`,
      col,
      row: 0,
      terrain: `synthetic-${col}`,
      owner: null,
    })),
  };
  const frames = Object.fromEntries(
    urls.map((url, col) => [
      Art.frame_key("terrain", `synthetic-${col}`),
      { url, width: 64, height: 64 },
    ]),
  );
  return Terrain.prepare(Terrain.create({}), state, { frames } as unknown as Art.Artwork);
}
function fixture() {
  const idle = idleQueue();
  const requests: { url: string; resolve: () => void; reject: () => void }[] = [];
  const owner = Prefetch.create(
    idle.defer,
    (url) =>
      new Promise<void>((resolve, reject) => {
        requests.push({ url, resolve, reject: () => reject(new Error("Unavailable")) });
      }),
  );
  return { idle, requests, owner, urls: () => requests.map((request) => request.url) };
}
const settled = () => new Promise<void>((resolve) => setImmediate(resolve));

test("Prefetch is deferred, deduplicated, bounded and reused across displayed maps", async () => {
  const { idle, requests, owner, urls } = fixture();
  Prefetch.start(owner, scene(["a", "b", "a", "c"]));
  assert.deepEqual(urls(), []);
  idle.flush();
  assert.deepEqual(urls(), ["a", "b"]);
  required(requests[0]).resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["a", "b", "c"]);
  required(requests[1]).resolve();
  required(requests[2]).resolve();
  await settled();
  Prefetch.start(owner, scene(["a", "c", "d", "d"]));
  idle.flush();
  assert.deepEqual(urls(), ["a", "b", "c", "d"]);
  required(requests[3]).resolve();
  await settled();
  Prefetch.close(owner);
});

test("Map changes drop queued tiles, preserve both occupied slots and ignore stale cleanup", async () => {
  const { idle, requests, owner, urls } = fixture();
  const stopOld = Prefetch.start(owner, scene(["old-a", "old-b", "old-queued"]));
  idle.flush();
  stopOld();
  const stopNew = Prefetch.start(owner, scene(["new-a", "new-b", "new-queued"]));
  stopOld();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b"]);
  required(requests[0]).resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b", "new-a"]);
  required(requests[1]).resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b", "new-a", "new-b"]);
  stopNew();
  required(requests[2]).resolve();
  required(requests[3]).resolve();
  await settled();
  idle.flush();
  assert.equal(requests.length, 4);
  const stopPending = Prefetch.start(owner, scene(["never-requested"]));
  stopPending();
  idle.flushStale();
  assert.equal(requests.length, 4);
  Prefetch.close(owner);
});

test("Stale scheduled callbacks cannot consume the current map's queue", () => {
  const { idle, owner, urls } = fixture();
  Prefetch.start(owner, scene(["old"]));
  Prefetch.start(owner, scene(["new"]));
  idle.flushStale();
  assert.deepEqual(urls(), ["new"]);
  Prefetch.close(owner);
});

test("Failed prefetches do not loop or poison success reuse, and later starts retry", async () => {
  const { idle, requests, owner } = fixture();
  Prefetch.start(owner, scene(["missing"]));
  idle.flush();
  required(requests[0]).reject();
  await settled();
  idle.flush();
  assert.equal(requests.length, 1);
  Prefetch.start(owner, scene(["missing"]));
  idle.flush();
  assert.equal(requests.length, 2);
  required(requests[1]).resolve();
  await settled();
  Prefetch.close(owner);
});

test("Synchronous decoder failures stay silent and release request capacity", async () => {
  const idle = idleQueue();
  const urls: string[] = [];
  const owner = Prefetch.create(idle.defer, (url) => {
    urls.push(url);
    if (url === "broken") throw new Error("Bad decoder");
    return Promise.resolve();
  });
  Prefetch.start(owner, scene(["broken", "valid", "next"]));
  idle.flush();
  assert.deepEqual(urls, ["broken", "valid"]);
  await settled();
  idle.flush();
  assert.deepEqual(urls, ["broken", "valid", "next"]);
  Prefetch.close(owner);
});

test("Background images use low network priority and asynchronous decoding", async () => {
  const images: BackgroundImage[] = [];
  class BackgroundImage {
    fetchPriority = "";
    decoding = "";
    src = "";
    constructor() {
      images.push(this);
    }
    decode() {
      return Promise.resolve();
    }
  }
  vi.stubGlobal("Image", BackgroundImage);
  const idle = idleQueue();
  const owner = Prefetch.create(idle.defer);
  try {
    Prefetch.start(owner, scene(["tile.png"]));
    idle.flush();
    await settled();
    const image = required(images[0]);
    assert.equal(image.src, "tile.png");
    assert.equal(image.fetchPriority, "low");
    assert.equal(image.decoding, "async");
  } finally {
    Prefetch.close(owner);
    vi.unstubAllGlobals();
  }
});

test("Default scheduling waits for a frame and then idle time, and cancels either stage", () => {
  let frame: FrameRequestCallback | undefined;
  let idle: (() => void) | undefined;
  const requests: string[] = [];
  const cancelled: number[] = [];
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    frame = callback;
    return 1;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => cancelled.push(id));
  vi.stubGlobal("requestIdleCallback", (callback: () => void) => {
    idle = callback;
    return 2;
  });
  vi.stubGlobal("cancelIdleCallback", (id: number) => cancelled.push(id));
  const owner = Prefetch.create(undefined, (url) => {
    requests.push(url);
    return Promise.resolve();
  });
  try {
    const stopBeforeFrame = Prefetch.start(owner, scene(["not-started"]));
    stopBeforeFrame();
    required(frame)(0);
    assert.equal(Boolean(idle), false);
    assert.deepEqual(requests, []);
    const stopBeforeIdle = Prefetch.start(owner, scene(["also-not-started"]));
    required(frame)(0);
    assert.deepEqual(requests, []);
    stopBeforeIdle();
    required(idle)();
    assert.deepEqual(requests, []);
    assert(cancelled.includes(2));
    Prefetch.start(owner, scene(["visible"]));
    required(frame)(0);
    assert.deepEqual(requests, []);
    required(idle)();
    assert.deepEqual(requests, ["visible"]);
  } finally {
    Prefetch.close(owner);
    vi.unstubAllGlobals();
  }
});

test("Timer fallback remains cancellable after the frame when idle callbacks are unavailable", async () => {
  vi.useFakeTimers();
  let frame: FrameRequestCallback | undefined;
  const requests: string[] = [];
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    frame = callback;
    return 1;
  });
  vi.stubGlobal("cancelAnimationFrame", () => {});
  vi.stubGlobal("requestIdleCallback", undefined);
  vi.stubGlobal("cancelIdleCallback", undefined);
  const owner = Prefetch.create(undefined, (url) => {
    requests.push(url);
    return Promise.resolve();
  });
  try {
    const stop = Prefetch.start(owner, scene(["cancelled"]));
    required(frame)(0);
    assert.deepEqual(requests, []);
    assert.equal(vi.getTimerCount(), 1);
    stop();
    assert.equal(vi.getTimerCount(), 0);
    Prefetch.start(owner, scene(["visible"]));
    required(frame)(0);
    await vi.runAllTimersAsync();
    assert.deepEqual(requests, ["visible"]);
  } finally {
    Prefetch.close(owner);
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});

test("Prefetch follows each displayed floor, road cap, connection and deterministic layout variant", async () => {
  const state: State = {
    mapId: "river",
    mapRevision: 1,
    cols: 5,
    rows: 1,
    round: 1,
    turn: "player",
    gold: { player: 0, enemy: 0 },
    winner: null,
    nextId: 1,
    log: [],
    units: [],
    tiles: [
      {
        id: "0,0",
        col: 0,
        row: 0,
        terrain: "road",
        floor: "snow-forest",
        connections: ["e", "w"],
        owner: null,
      },
      {
        id: "1,0",
        col: 1,
        row: 0,
        terrain: "road",
        floor: "green-forest",
        connections: ["e", "w"],
        owner: null,
      },
      { id: "2,0", col: 2, row: 0, terrain: "bridge", connections: ["nw", "se"], owner: null },
      { id: "3,0", col: 3, row: 0, terrain: "forest", owner: null },
      {
        id: "4,0",
        col: 4,
        row: 0,
        terrain: "road",
        connections: ["se", "ne"],
        deadEnds: ["w"],
        owner: null,
      },
    ],
  };
  const displayed = Terrain.prepare(Terrain.create({}), state, Art.current());
  const expected = [...new Set(Terrain.tiles(displayed).map((tile) => required(tile.frame).url))];
  assert.equal(expected.length, 5, "Distinct variants must retain their actual native images");
  const idle = idleQueue();
  const urls: string[] = [];
  const owner = Prefetch.create(idle.defer, (url) => {
    urls.push(url);
    return Promise.resolve();
  });
  try {
    Prefetch.start(owner, displayed);
    for (let batch = 0; batch < 3; batch++) {
      idle.flush();
      await settled();
    }
    assert.deepEqual(urls, expected);
  } finally {
    Prefetch.close(owner);
  }
});
