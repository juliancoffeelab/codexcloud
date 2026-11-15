import assert from "node:assert/strict";
import { afterEach, beforeEach, test, vi } from "vitest";
import * as Process from "@frontend/Boot_process";

let now;
beforeEach(() => {
  delete globalThis.BOOT_PROCESS;
  now = 10;
  vi.stubGlobal("performance", { now: () => now, timeOrigin: 1000 });
});
afterEach(() => delete globalThis.BOOT_PROCESS);
const progress = (step, value, detail = "") =>
  Process.progress(step, `Stage ${step}…`, value, detail);

test("Boot diagnostics retain stage timings and resource progress after the first ready frame", () => {
  Process.snapshot();
  now = 30;
  progress(2, 1);
  now = 50;
  progress(4, 3.5, "5 / 10 ready");
  now = 70;
  progress(5, 4);
  now = 80;
  progress(5, 5);
  const report = Process.snapshot();
  assert.equal(globalThis.BOOT_PROCESS.status, "ready");
  assert.equal(report.elapsedMs, 70);
  assert.equal(report.stages[0].durationMs, 20);
  assert.equal(report.stages.at(-1).durationMs, 10);
  assert.equal(report.stages.at(-1).finishedAtMs, 80);
  assert.equal(report.events[2].detail, "5 / 10 ready");
  assert.equal(report.label, "Stage 5…");
  progress(3, 2); // Later battlefields must not overwrite the startup being investigated.
  Process.fail("Later rendering failure");
  assert.deepEqual(Process.snapshot(), report);
  report.events.length = 0;
  assert(globalThis.BOOT_PROCESS.events.length > 0);
});

test("Exporting a stalled boot updates its duration, and errors retain their backtrace", () => {
  Process.snapshot();
  now = 60;
  progress(3, 2);
  now = 100;
  const stalled = Process.snapshot();
  assert.equal(stalled.status, "loading");
  assert.equal(stalled.elapsedMs, 90);
  assert.equal(stalled.stages.at(-1).durationMs, 40);
  now = 120;
  const error = "Engine failed\n    at initialize (engine.js:12)";
  Process.fail(error);
  assert.equal(Process.snapshot().error, error);
  assert.equal(Process.snapshot().status, "failed");
  assert.equal(Process.snapshot().events.at(-1).status, "failed");
  progress(5, 5);
  assert.equal(Process.snapshot().status, "failed");
});

test("Resource history is bounded while initial timing and stage durations survive", () => {
  Process.snapshot();
  now = 20;
  for (let i = 0; i < 600; i++) progress(4, 3 + i / 600, `${i} resources ready`);
  const report = Process.snapshot();
  assert.equal(report.events.length, 512);
  assert.equal(report.droppedEvents, 89);
  assert.equal(report.events[0].atMs, 10);
  assert.equal(report.stages[0].durationMs, 10);
  assert.equal(report.events.at(-1).detail, "599 resources ready");
});

test("Native timing entries are read only on export and retain raw cache/connection evidence", () => {
  const entry = (fields) => ({ ...fields, toJSON: () => ({ ...fields }) });
  const navigation = entry({
    type: "reload",
    startTime: 0,
    requestStart: 3,
    responseStart: 8,
    domainLookupStart: 1,
    domainLookupEnd: 2,
    connectStart: 2,
    connectEnd: 3,
  });
  const cached = entry({
    name: "https://example.test/image.png",
    startTime: 25,
    responseEnd: 40,
    transferSize: 0,
    encodedBodySize: 100,
    decodedBodySize: 200,
    deliveryType: "cache",
  });
  const network = entry({
    name: "https://example.test/sound.wav",
    startTime: 30,
    responseEnd: 70,
    transferSize: 1000,
    deliveryType: "cache",
    encodedBodySize: 800,
    decodedBodySize: 800,
  });
  const later = entry({ name: "https://example.test/later.png", startTime: 100 });
  const nativeEntries = vi.fn((kind) =>
    kind === "navigation" ? [navigation] : [cached, network, later],
  );
  performance.getEntriesByType = nativeEntries;
  progress(2, 1);
  Process.resource("image", "https://example.test/image.png", "image-loaded");
  assert.equal(nativeEntries.mock.calls.length, 0);
  now = 50;
  progress(5, 5);
  now = 500;
  Process.resource("later", "later.png", "image-loaded");
  const report = Process.snapshot();
  assert.equal(nativeEntries.mock.calls.length, 2);
  assert.equal(report.browserTiming.navigation.type, "reload");
  assert.equal(report.browserTiming.navigationPhases.requestToFirstByteMs, 5);
  assert.equal(report.browserTiming.navigationPhases.responseEndToRecorderMs, null);
  assert.equal(report.browserTiming.resourceEvidence.cacheLabelWithReportedTransfer, 1);
  assert.equal(report.browserTiming.resourceEvidence.zeroReportedTransferWithBody, 1);
  assert.equal(report.browserTiming.headerCoverage.extraRequests, 0);
  assert.equal(report.browserTiming.startupCutoffAtMs, 50);
  assert.equal(report.browserTiming.collectedAtMs, 500);
  assert.deepEqual(report.browserTiming.resources, [cached.toJSON(), network.toJSON()]);
  assert.equal(report.resourceEvents.length, 1);
  assert.equal(report.resourceEvents[0].phase, "image-loaded");
  assert.equal(report.resourceEvents[0].atMs, 10);
});

test("Per-resource lifecycle timestamps are bounded and preserve phase order", () => {
  progress(4, 3);
  for (const phase of [
    "image-request-start",
    "image-loaded",
    "image-admission-start",
    "image-admitted",
  ]) {
    now++;
    Process.resource("image", "image.png", phase);
  }
  const report = Process.snapshot();
  assert.deepEqual(
    report.resourceEvents.map((event) => event.phase),
    ["image-request-start", "image-loaded", "image-admission-start", "image-admitted"],
  );
  for (let i = 0; i < 600; i++) Process.resource("image", "image.png", "resource-ready");
  const bounded = Process.snapshot();
  assert.equal(bounded.resourceEvents.length, 512);
  assert.equal(bounded.droppedResourceEvents, 92);
});

test("Existing responses capture only cache/CDN headers and stop reading once boot completes", () => {
  progress(4, 3);
  const fields = {
    "cache-control": "private, max-age=0",
    etag: '"hash"',
    "cf-cache-status": "HIT",
    "set-cookie": "secret",
    authorization: "secret",
  };
  const read = vi.fn((name) => fields[name] ?? null);
  Process.response("/audio.wav", 200, "phaser-xhr", read);
  const report = Process.snapshot();
  assert.deepEqual(report.responseHeaders[0].headers, {
    "cache-control": fields["cache-control"],
    etag: fields.etag,
    "cf-cache-status": "HIT",
  });
  assert.equal(report.responseHeaders[0].readError, false);
  progress(5, 5);
  read.mockClear();
  Process.response("/later.wav", 200, "phaser-xhr", read);
  assert.equal(read.mock.calls.length, 0);
  assert.equal(Process.snapshot().responseHeaders.length, 1);
});

test("Unavailable headers cannot break loading, and response capture remains bounded", () => {
  progress(4, 3);
  Process.response("/audio.wav", 200, "hud-fetch", () => {
    throw new Error("Headers unavailable");
  });
  assert.equal(Process.snapshot().responseHeaders[0].readError, true);
  const read = vi.fn(() => null);
  for (let i = 0; i < 64; i++) Process.response("/audio.wav", 200, "phaser-xhr", read);
  const report = Process.snapshot();
  assert.equal(report.responseHeaders.length, 64);
  assert.equal(report.droppedResponseHeaders, 1);
  read.mockClear();
  Process.response("/later.wav", 200, "phaser-xhr", read);
  assert.equal(read.mock.calls.length, 0);
  progress(5, 5);
  assert.equal(Process.snapshot().status, "ready");
});
