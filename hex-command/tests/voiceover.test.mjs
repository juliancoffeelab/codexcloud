import { test, vi } from "vitest";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { VoicePlayer } from "../src/presentation/voice/player.mjs";
import * as Voiceover from "@frontend/Voiceover";
import {
  duration,
  profileFor,
  sampleRate,
  synthesize,
} from "../src/presentation/voice/synthesis.mjs";

function harness() {
  const document = new EventTarget();
  document.hidden = false;
  vi.stubGlobal("document", document);
  vi.stubGlobal("localStorage", { getItem: () => null, setItem: vi.fn() });
  let now = 0,
    sequence = 0;
  const frames = new Map(),
    jobs = [],
    events = [],
    sources = [];
  vi.stubGlobal("performance", { now: () => now });
  vi.stubGlobal("requestAnimationFrame", (fn) => {
    frames.set(++sequence, fn);
    return sequence;
  });
  vi.stubGlobal("cancelAnimationFrame", (id) => frames.delete(id));
  const worker = { postMessage: (job) => jobs.push(job), terminate: vi.fn() };
  const context = {
    state: "running",
    currentTime: 0,
    destination: {},
    close: async () => {},
    createBuffer: () => ({ copyToChannel: vi.fn() }),
    createBufferSource: () => {
      const source = { connect() {}, disconnect() {}, start: vi.fn(), stop: vi.fn() };
      sources.push(source);
      return source;
    },
    createGain: () => ({
      connect() {},
      disconnect() {},
      gain: {
        value: 1,
        setValueAtTime() {},
        linearRampToValueAtTime() {},
        cancelScheduledValues() {},
      },
    }),
  };
  const player = new VoicePlayer((name, args) => events.push({ name, args }), {
    workerFactory: () => worker,
    contextFactory: () => context,
  });
  function complete() {
    const job = jobs.shift();
    assert(job);
    const samples = new Float32Array(Math.ceil(duration(job.line.text) * sampleRate));
    worker.onmessage({ data: { ...job, samples, ms: 2 } });
  }
  function frame(ms) {
    now += ms;
    const callbacks = [...frames.values()];
    frames.clear();
    callbacks.forEach((fn) => fn());
  }
  const lines = Array.from({ length: 80 }, (_, i) => ({
    character: "ivo",
    speaker: "Ivo",
    text: `Example ${i}, a line to be voiced.`,
  }));
  return { player, lines, document, frames, jobs, sources, events, worker, complete, frame };
}

test("Worker synthesis keeps the audition pitch and 3x reveal timing, with finite faded audio", () => {
  const bank = new Uint8Array(
    readFileSync(new URL("../public/assets/audio/voice/animalese.wav", import.meta.url)),
  );
  const line = { character: "ivo", speaker: "Ivo", text: "Hello, traveller." };
  assert.equal(profileFor(line).pitch, 1.35);
  const samples = synthesize(bank, line);
  assert.equal(samples.length, Math.ceil(duration(line.text) * sampleRate));
  assert(samples.some((value) => Math.abs(value) > 0.1));
  assert(samples.every((value) => Number.isFinite(value) && Math.abs(value) <= 1));
  assert.equal(Math.abs(samples[0]), 0);
  assert.equal(Math.abs(samples.at(-1)), 0);
  assert.equal(profileFor({ ...line, character: "oren" }).pitch, 1.9);
  assert.notEqual(profileFor({ ...line, character: "orin" }).pitch, 1.9);
});

test("A rolling queue retains backward audio, prioritizes live dialogue and discards stale mission work", () => {
  const h = harness(),
    { player, lines } = h;
  player.prepare("mission-a", lines);
  while (h.jobs.length) h.complete();
  assert.equal(player.prepared.size, 20);
  player.unlock();
  player.speak("mission-a", lines, 4, () => {});
  assert.equal(h.sources.length, 1);
  const oldJobs = h.jobs.length;
  player.speak("mission-a", lines, 3, () => {});
  assert.equal(h.sources.length, 2);
  assert.equal(h.sources[0].stop.mock.calls.length, 1);
  assert.equal(h.jobs.length, oldJobs, "only one generation job at a time");
  while (h.jobs.length) h.complete();
  assert(player.prepared.size <= 41);
  player.speak("mission-a", lines, 60, () => {});
  assert.equal(h.jobs[0].line.text, lines[60].text);
  player.prepare("mission-b", [lines[79]]);
  h.complete();
  assert.equal(player.prepared.size, 0, "old mission result ignored");
  assert.equal(h.jobs[0].line.text, lines[79].text);
  h.complete();
  assert.equal(player.prepared.size, 1);
  assert(player.bytes <= 32 * 1024 * 1024);
  player.close();
  assert.equal(h.frames.size, 0);
  assert.equal(h.worker.terminate.mock.calls.length, 1);
});

test("Text never waits for audio; navigating or backgrounding cannot start a late voice", () => {
  const h = harness(),
    { player, lines } = h;
  let visible = "";
  player.unlock();
  const cleanup = Voiceover.speak(player, "mission", lines, 0, (text) => (visible = text));
  h.frame(duration(lines[0].text) * 500);
  assert(visible.length > 1 && visible.length < lines[0].text.length);
  assert(h.events.some((event) => event.name === "Voiceover deadline miss"));
  cleanup();
  h.complete();
  assert.equal(h.sources.length, 0);
  player.speak("mission", lines, 1, (text) => (visible = text));
  h.document.hidden = true;
  h.document.dispatchEvent(new Event("visibilitychange"));
  assert.equal(visible, lines[1].text);
  assert.equal(h.frames.size, 0);
  h.complete();
  assert.equal(h.sources.length, 0);
  h.document.hidden = false;
  h.document.dispatchEvent(new Event("visibilitychange"));
  while (h.jobs.length) h.complete();
  player.speak("mission", lines, 1, (text) => (visible = text));
  h.frame(duration(lines[1].text) * 1000 + 1);
  assert.equal(visible, lines[1].text);
  player.close();
});

test("Disabling voices stops sound and generation without disabling character reveal", () => {
  const h = harness(),
    { player, lines } = h;
  let visible = "";
  player.prepare("mission", lines);
  while (h.jobs.length) h.complete();
  player.unlock();
  player.speak("mission", lines, 0, (text) => (visible = text));
  player.setEnabled(false);
  assert.equal(h.sources[0].stop.mock.calls.length, 1);
  player.speak("mission", lines, 50, (text) => (visible = text));
  assert.equal(h.jobs.length, 1, "only the previously pending job remains");
  h.complete();
  assert.equal(h.jobs.length, 0);
  h.frame(duration(lines[50].text) * 1000 + 1);
  assert.equal(visible, lines[50].text);
  player.setEnabled(true);
  assert(h.jobs.length > 0);
  player.close();
});

test("Opening gate waits for the actual first two tracks and stays released for later dialogue", () => {
  const h = harness(),
    { player, lines } = h,
    updates = [];
  player.watchOpening("mission", [], 0, (state) => updates.push(state));
  assert.equal(updates.at(-1).total, 0);
  player.prepare("mission", lines);
  const unsubscribe = Voiceover.watch_opening(player, "mission", lines, 50, (state) =>
    updates.push(state),
  );
  assert.equal(updates.at(-1).ready, false);
  h.complete(); // An older background job cannot release the opening gate.
  assert.equal(updates.at(-1).completed, 0);
  h.complete();
  assert.equal(updates.at(-1).completed, 1);
  assert.equal(updates.at(-1).ready, false);
  h.complete();
  assert.equal(updates.at(-1).completed, 2);
  assert.equal(updates.at(-1).ready, true);
  unsubscribe();
  player.watchOpening("mission", lines, 70, (state) => updates.push(state));
  assert.equal(updates.at(-1).ready, true, "later conversations never re-lock the campaign");
  player.close();
});

test("Opening failures can retry or bypass; background time cannot expire their deadline", () => {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
  const h = harness(),
    { player, lines } = h,
    updates = [];
  try {
    player.watchOpening("mission", lines.slice(0, 1), 0, (state) => updates.push(state));
    assert.equal(updates.at(-1).total, 1);
    h.document.hidden = true;
    h.document.dispatchEvent(new Event("visibilitychange"));
    vi.advanceTimersByTime(60000);
    assert.equal(player.failed, false);
    h.document.hidden = false;
    h.document.dispatchEvent(new Event("visibilitychange"));
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.advanceTimersByTime(30001);
    assert.equal(updates.at(-1).ready, false);
    assert.match(updates.at(-1).error, /30 seconds/);
    h.jobs.length = 0;
    player.retry();
    assert.equal(updates.at(-1).error, null);
    h.complete();
    assert.equal(updates.at(-1).ready, true);
    player.watchOpening("next-mission", lines, 0, (state) => updates.push(state));
    assert.equal(updates.at(-1).ready, false);
    player.setEnabled(false);
    assert.equal(updates.at(-1).ready, true);
  } finally {
    player.close();
    vi.useRealTimers();
  }
});
