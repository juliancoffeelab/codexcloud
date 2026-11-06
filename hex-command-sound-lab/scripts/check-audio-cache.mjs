import assert from "node:assert/strict";
import { AudioCache } from "../dist/audio-cache.js";
const originalFetch = globalThis.fetch;
let calls = [],
  fail = false;
const context = {
  decodeAudioData: async (bytes) => ({
    length: new Uint8Array(bytes)[0],
    numberOfChannels: 1,
  }),
};
globalThis.fetch = async (path) => {
  calls.push(path);
  if (fail) return { ok: false, status: 503 };
  return {
    ok: true,
    arrayBuffer: async () => Uint8Array.of(Number(path)).buffer,
  };
};
const item = (id, n = 10) => ({ id, path: String(n), duration: n / 44100 });
try {
  const cache = new AudioCache(() => context, { budget: 100 });
  const [a, b] = await Promise.all([
    cache.load(item("a")),
    cache.load(item("a")),
  ]);
  assert.equal(a, b);
  assert.equal(calls.length, 1);
  await cache.load(item("b"));
  cache.peek("a");
  await cache.load(item("c"));
  assert(cache.ready.has("a"));
  assert(!cache.ready.has("b"));
  assert.equal(cache.bytes, 80);
  fail = true;
  await assert.rejects(cache.load(item("retry")), /503/);
  assert(!cache.pending.has("retry"));
  fail = false;
  await cache.load(item("retry"));
  assert(cache.ready.has("retry"));
  // A new foreground choice must warm even when the previous scene filled the cache.
  await cache.warm([item("new", 20)], () => {}, 1);
  assert(cache.ready.has("new"));
  assert(cache.bytes <= 100);
  // Obsolete prefetch stops before starting the remaining queued recordings.
  const releases = [];
  let started = 0;
  globalThis.fetch = async () => {
    started++;
    await new Promise((r) => releases.push(r));
    return { ok: true, arrayBuffer: async () => Uint8Array.of(10).buffer };
  };
  const stale = new AudioCache(() => context);
  const warming = stale.warm([item("one"), item("two"), item("three")]);
  await Promise.resolve();
  assert.equal(started, 2);
  stale.cancelWarm();
  releases.forEach((release) => release());
  await warming;
  assert.equal(started, 2);
  console.log(
    "Audio cache: dedupe, LRU budget, retry, stale prefetch cancellation and foreground preparation passed.",
  );
} finally {
  globalThis.fetch = originalFetch;
}
