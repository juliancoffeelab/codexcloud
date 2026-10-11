import assert from "node:assert/strict";
import { test, vi } from "vitest";
import { TerrainTilePrefetcher, terrainInspectionUrls } from "../src/presentation/terrain-prefetch";
import { currentArtwork, artFrame } from "../src/presentation/art";
import { canonicalTerrainName } from "../src/presentation/terrain-layout";
import type { TileView } from "../src/game/view";

function idleQueue() {
  const tasks: { run: () => void; cancelled: boolean }[] = [];
  return {
    defer(run: () => void) {
      const task = { run, cancelled: false };
      tasks.push(task);
      return () => (task.cancelled = true);
    },
    flush() {
      for (const task of tasks.splice(0)) if (!task.cancelled) task.run();
    },
  };
}

function fixture() {
  const idle = idleQueue();
  const requests: { url: string; resolve: () => void; reject: () => void }[] = [];
  const prefetch = new TerrainTilePrefetcher(
    idle.defer,
    (url) =>
      new Promise<void>((resolve, reject) => {
        requests.push({ url, resolve, reject: () => reject(new Error("Unavailable")) });
      }),
  );
  return { idle, requests, prefetch, urls: () => requests.map((request) => request.url) };
}

const settled = () => new Promise<void>((resolve) => setImmediate(resolve));

test("Inspection prefetch is bounded by terrain types, independent of tile count and appearance", () => {
  const tiles: TileView[] = Array.from({ length: 1024 }, (_, id) => ({
    id: `${id},0`,
    col: id,
    row: 0,
    owner: null,
    terrain: id % 2 ? "forest" : "ford",
    floor: id % 3 ? "snow-forest" : "autumn-forest",
    ...(id % 2 ? {} : { connections: ["nw", "se"] as const }),
  }));
  const artwork = currentArtwork();
  const urls = terrainInspectionUrls(artwork, tiles);
  assert.deepEqual(
    urls,
    ["ford", "forest"].map(
      (terrain) =>
        artFrame(artwork, "terrain", canonicalTerrainName(terrain as "ford" | "forest"))!.url,
    ),
  );
  assert.deepEqual(terrainInspectionUrls(artwork, tiles.slice(0, 2)), urls);
  assert.deepEqual(terrainInspectionUrls({ frames: {} }, tiles), []);
});

test("Prefetch is deferred, deduplicated, bounded and reused across maps", async () => {
  const { idle, requests, prefetch, urls } = fixture();
  prefetch.start(["a", "b", "a", "c"]);
  assert.deepEqual(urls(), []);
  idle.flush();
  assert.deepEqual(urls(), ["a", "b"]);
  requests[0]!.resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["a", "b", "c"]);
  requests[1]!.resolve();
  requests[2]!.resolve();
  await settled();
  prefetch.start(["a", "c", "d", "d"]);
  idle.flush();
  assert.deepEqual(urls(), ["a", "b", "c", "d"]);
  requests[3]!.resolve();
  await settled();
});

test("Map changes drop queued tiles, preserve the request bound and ignore stale cleanup", async () => {
  const { idle, requests, prefetch, urls } = fixture();
  const stopOld = prefetch.start(["old-a", "old-b", "old-queued"]);
  idle.flush();
  stopOld();
  const stopNew = prefetch.start(["new-a", "new-b", "new-queued"]);
  stopOld();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b"]);
  requests[0]!.resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b", "new-a"]);
  requests[1]!.resolve();
  await settled();
  idle.flush();
  assert.deepEqual(urls(), ["old-a", "old-b", "new-a", "new-b"]);
  stopNew();
  requests[2]!.resolve();
  requests[3]!.resolve();
  await settled();
  idle.flush();
  assert.equal(requests.length, 4);
  const stopPending = prefetch.start(["never-requested"]);
  stopPending();
  idle.flush();
  assert.equal(requests.length, 4);
});

test("Failed prefetches do not loop and remain retryable", async () => {
  const { idle, requests, prefetch } = fixture();
  prefetch.start(["missing"]);
  idle.flush();
  requests[0]!.reject();
  await settled();
  idle.flush();
  assert.equal(requests.length, 1);
  prefetch.start(["missing"]);
  idle.flush();
  assert.equal(requests.length, 2);
  requests[1]!.resolve();
  await settled();
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
  const prefetch = new TerrainTilePrefetcher(idle.defer);
  prefetch.start(["tile.png"]);
  idle.flush();
  await settled();
  assert.equal(images[0]!.src, "tile.png");
  assert.equal(images[0]!.fetchPriority, "low");
  assert.equal(images[0]!.decoding, "async");
});
