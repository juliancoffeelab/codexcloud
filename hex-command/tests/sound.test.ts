import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { required, TestStorage } from "./helpers.ts";
import * as Audio from "@frontend/Audio";
const stored = new Map<string, string>();

let created = 0,
  resumed = 0;
let audio: FakeContext | undefined;
const fetched: string[] = [];

class Param {
  value = 0;
  cancelScheduledValues() {}
  setValueAtTime(value: number) {
    this.value = value;
  }
  linearRampToValueAtTime(value: number) {
    this.value = value;
  }
}
interface FakeBuffer {
  name?: string;
  silent?: boolean;
}
class FakeSource {
  buffer: FakeBuffer = {};
  started = false;
  stopped = false;
  disconnected = false;
  connect(): void {}
  disconnect(): void {
    this.disconnected = true;
  }
  start(): void {
    this.started = true;
  }
  stop(): void {
    this.stopped = true;
  }
}
class FakeGain {
  gain = new Param();
  disconnected = false;
  connect(): void {}
  disconnect(): void {
    this.disconnected = true;
  }
}
class FakeContext {
  state = "suspended";
  currentTime = 10;
  destination = {};
  sources: FakeSource[] = [];
  gains: FakeGain[] = [];
  closes = 0;
  constructor() {
    created++;
    audio = this;
  }
  async resume() {
    resumed++;
    this.state = "running";
  }
  async close() {
    this.closes++;
    this.state = "closed";
  }
  createBuffer() {
    return { silent: true };
  }
  createGain() {
    const gain = new FakeGain();
    this.gains.push(gain);
    return gain;
  }
  async decodeAudioData(bytes: ArrayBuffer): Promise<FakeBuffer> {
    return { name: new TextDecoder().decode(bytes) };
  }
  createBufferSource() {
    const source = new FakeSource();
    this.sources.push(source);
    return source;
  }
}

test("Audio only starts after an enabled trusted gesture", async () => {
  const sound = createSound();
  assert.equal(sound.soundEnabled(), true);
  await sound.playSound("turn");
  assert.equal(created, 0);
  assert.equal(fetched.length, 0);
  assert.equal(await sound.unlockSound({ isTrusted: false }), false);
  assert.equal(created, 0, "Scripted gestures cannot create an audio context");
  sound.setSoundEnabled(false);
  await sound.unlockSound();
  assert.equal(created, 0);
  sound.setSoundEnabled(true);
  await sound.unlockSound();
  assert.equal(created, 1);
  assert.equal(resumed, 1);
});
// Priority resolves a single family even when all three contexts exist.
test("Attack sound takes priority over unit and terrain", async () => {
  const sound = createSound();
  assert.equal(
    sound.soundForInteraction({
      attackType: "archer",
      unitType: "soldier",
      terrain: "village",
    }),
    "attack-bow",
  );
  assert.equal(
    sound.soundForInteraction({
      attackType: "scout",
      unitType: "archer",
      terrain: "forest",
    }),
    "attack-melee",
  );
});
for (const unitType of ["soldier", "archer", "scout", "commander"] as const) {
  test("Unit sound priority follows its material — " + JSON.stringify([unitType]), async () => {
    const sound = createSound();
    assert.equal(
      sound.soundForInteraction({ unitType, terrain: "hill" }),
      ["archer", "scout"].includes(unitType)
        ? "terrain-plains"
        : unitType === "soldier"
          ? "unit-archer"
          : `unit-${unitType}`,
    );
  });
}
for (const terrain of ["village", "water", "forest", "hill", "plains", "keep"] as const) {
  test("Terrain sound resolves its own family — " + JSON.stringify([terrain]), async () => {
    const sound = createSound();
    assert.equal(sound.soundForInteraction({ terrain }), `terrain-${terrain}`);
  });
}
test("Fords use water sounds and empty interactions use selection", async () => {
  const sound = createSound();
  assert.equal(sound.soundForInteraction({ terrain: "ford" }), "terrain-water");
  assert.equal(sound.soundForInteraction({}), "select");
});
// A shuffle bag plays all three variations, without an immediate repeat at refill.
test("Shuffle bags avoid repeats, reuse decoded samples and stop voices when muted", async () => {
  const sound = createSound();
  await sound.unlockSound();
  const variants: string[] = [];
  for (let i = 0; i < 9; i++) {
    assert.equal(await sound.playSound("unit-soldier"), true);
    const source = required(required(audio).sources.at(-1));
    const match = required(required(source.buffer.name).match(/-(\d)\.wav/));
    variants.push(required(match[1]));
  }
  for (let i = 0; i < 9; i += 3) assert.equal(new Set(variants.slice(i, i + 3)).size, 3);
  assert.ok(variants.every((value, i) => i === 0 || value !== variants[i - 1]));
  assert.equal(fetched.length, 3, "Only this family's three variations load and are reused");
  assert.ok(
    required(audio)
      .sources.filter((source) => !source.buffer.silent)
      .slice(0, -1)
      .every((source) => source.stopped),
    "new cues replace earlier cues",
  );
  sound.setSoundEnabled(false);
  assert.ok(
    required(audio)
      .sources.filter((source) => !source.buffer.silent)
      .every((source) => source.stopped),
  );
  const count = fetched.length;
  await sound.playSound("attack-bow");
  assert.equal(fetched.length, count);
  assert.equal(stored.get("hex-command-sound-enabled"), "false");
  sound.setSoundEnabled(true);
});
// Delayed preload responses cannot revive superseded or muted clicks.
async function delayedSound() {
  const pending: { name: string; completed: boolean; resolve: () => void }[] = [];
  const waiting: { family: string; resolve: () => void }[] = [];
  const requested = (family: string) =>
    pending.some((request) => !request.completed && request.name.includes(`/${family}-`))
      ? Promise.resolve()
      : new Promise<void>((resolve) => waiting.push({ family, resolve }));
  globalThis.fetch = (url) =>
    new Promise<Response>((resolve) => {
      const request: { name: string; completed: boolean; resolve: () => void } = {
        name: String(url),
        completed: false,
        resolve: () => {
          request.completed = true;
          resolve(new Response(String(url)));
        },
      };
      pending.push(request);
      for (let index = waiting.length - 1; index >= 0; index--) {
        const waiter = waiting[index]!;
        if (request.name.includes(`/${waiter.family}-`)) {
          waiting.splice(index, 1);
          waiter.resolve();
        }
      }
    });
  const cold = createSound();
  await cold.unlockSound();
  assert.equal(pending.length, 0, "Unlocking never starts background sample requests");
  return { pending, cold, requested };
}
test("Delayed fetches cannot revive superseded, muted or suspended cues", async () => {
  const { pending, cold, requested } = await delayedSound();
  const old = cold.playSound("terrain-forest");
  const latest = cold.playInteraction({
    attackType: "archer",
    unitType: "soldier",
    terrain: "forest",
  });
  await requested("terrain-forest");
  await requested("attack-bow");
  for (const sample of pending.filter((p) => p.name.includes("/terrain-forest-"))) sample.resolve();
  assert.equal(await old, false);
  for (const sample of pending.filter((p) => p.name.includes("/attack-bow-"))) sample.resolve();
  assert.equal(await latest, true);
  const muted = cold.playSound("terrain-water");
  await requested("terrain-water");
  cold.setSoundEnabled(false);
  for (const sample of pending.filter((p) => p.name.includes("/terrain-water-"))) sample.resolve();
  assert.equal(await muted, false);
  cold.setSoundEnabled(true);
  required(audio).state = "suspended";
  assert.equal(await cold.playSound("turn"), false);
  const resumeCount = resumed;
  await cold.unlockSound();
  assert.equal(resumed, resumeCount + 1);
});
// Reset/unmount cancellation stops active feedback without muting future cues.
test("Session cancellation stops active and still-decoding samples without muting future cues", async () => {
  const { pending, cold, requested } = await delayedSound();
  const forest = cold.playSound("terrain-forest");
  await requested("terrain-forest");
  for (const sample of pending.filter((p) => p.name.includes("/terrain-forest-"))) sample.resolve();
  assert.equal(await forest, true);
  const cancellationContext = required(audio);
  const activeSource = required(cancellationContext.sources.at(-1));
  const savedPreference = stored.get("hex-command-sound-enabled");
  cold.cancelSound();
  assert.equal(activeSource.stopped, true);
  assert.equal(cold.soundEnabled(), true);
  assert.equal(stored.get("hex-command-sound-enabled"), savedPreference);

  // A sample already decoding when its session ends must never start afterward.
  const originalDecode = cancellationContext.decodeAudioData.bind(cancellationContext);
  const finishDecoding: (() => void)[] = [];
  let announceDecoding = () => {};
  const decodingStarted = new Promise<void>((resolve) => {
    announceDecoding = resolve;
  });
  cancellationContext.decodeAudioData = (bytes) =>
    new Promise<FakeBuffer>((resolve) => {
      finishDecoding.push(() => {
        void originalDecode(bytes).then(resolve);
      });
      announceDecoding();
    });
  const cancelled = cold.playSound("terrain-hill");
  await requested("terrain-hill");
  for (const sample of pending.filter((p) => p.name.includes("/terrain-hill-"))) sample.resolve();
  await decodingStarted;
  const sourcesBeforeCancel = cancellationContext.sources.length;
  cold.cancelSound();
  for (const finish of finishDecoding) finish();
  assert.equal(await cancelled, false);
  assert.equal(cancellationContext.sources.length, sourcesBeforeCancel);
  cancellationContext.decodeAudioData = originalDecode;
  // Complete the rest of the shuffle bag, then revisit decoded variations.
  const subsequentRequests: string[] = [];
  globalThis.fetch = async (url) => {
    subsequentRequests.push(String(url));
    return new Response(String(url));
  };
  for (let index = 0; index < 3; index++) {
    assert.equal(
      await cold.playSound("terrain-hill"),
      true,
      "Later sessions can play decoded cues",
    );
  }
  assert.equal(
    subsequentRequests.length,
    2,
    "Only the two undecoded variations fetch; completed decoding remains cached",
  );
  cold.cancelSound();
});
// Mobile browsers can leave a touch-start resume pending. A completed tap must
// make a fresh attempt, and resume an interrupted context after app switching.
test("Completed mobile taps retry pending resume and recreate closed contexts", async () => {
  const NativeFake = globalThis.AudioContext;
  const gesture = (type: string, isTrusted = true) => ({ isTrusted, type });
  let attempts = 0;
  const TouchContext = class extends FakeContext {
    resume() {
      resumed++;
      attempts++;
      if (attempts === 1) return new Promise<void>(() => {});
      this.state = "running";
      return Promise.resolve();
    }
  };
  globalThis.AudioContext = TouchContext as unknown as typeof AudioContext;
  globalThis.fetch = async (url) => new Response(String(url));
  const touch = createSound();
  void touch.unlockSound(gesture("pointerdown", true));
  assert.equal(attempts, 1);
  assert.equal(required(audio).sources.length, 1, "the gesture primes mobile output with silence");
  vi.useFakeTimers();
  try {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    const deadline = new Promise<never>((_, reject) => {
      timeout = setTimeout(() => reject(Error("touch resume remained stuck")), 100);
    });
    const retried = Promise.race([touch.unlockSound(gesture("touchend", true)), deadline]).finally(
      () => {
        if (timeout !== undefined) clearTimeout(timeout);
      },
    );
    await vi.advanceTimersByTimeAsync(100);
    await retried;
  } finally {
    vi.useRealTimers();
  }
  assert.equal(attempts, 2);
  assert.equal(await touch.playSound("terrain-plains"), true);
  required(audio).state = "interrupted";
  assert.equal(await touch.playSound("terrain-plains"), false);
  await touch.unlockSound(gesture("pointerup", true));
  assert.equal(attempts, 3);
  assert.equal(await touch.playSound("terrain-plains"), true);
  required(audio).state = "closed";
  await touch.unlockSound(gesture("click", true));
  assert.equal(required(audio).state, "running", "closed mobile contexts can be recreated");
  const attemptsBefore = attempts;
  await touch.unlockSound(gesture("click", false));
  assert.equal(attempts, attemptsBefore, "scripted events cannot unlock output");
  globalThis.AudioContext = NativeFake;
});
// Failed sample loading remains harmless.
test("Failed audio samples never throw", async () => {
  globalThis.fetch = async () => new Response(null, { status: 500 });
  const failed = createSound();
  await failed.unlockSound();
  assert.equal(await failed.playSound("terrain-hill"), false, "failed audio never throws");
});
test("Missing audio APIs degrade to silent playback", async () => {
  Reflect.deleteProperty(globalThis, "AudioContext");
  const unsupported = createSound();
  await unsupported.unlockSound();
  assert.equal(await unsupported.playSound("turn"), false);
});

test("Terminal disposal releases voices and context once and prevents new work", async () => {
  const sound = createSound();
  await sound.unlockSound();
  assert.equal(await sound.playSound("turn"), true);
  const context = required(audio);
  const savedPreference = stored.get("hex-command-sound-enabled");
  sound.dispose();
  sound.dispose();
  assert.equal(context.closes, 1);
  assert(context.sources.every((source) => source.disconnected));
  assert(context.gains.every((gain) => gain.disconnected));
  assert(
    context.sources.filter((source) => !source.buffer.silent).every((source) => source.stopped),
  );
  const requests = fetched.length;
  sound.setSoundEnabled(false);
  assert.equal(await sound.unlockSound(), false);
  assert.equal(await sound.playSound("victory"), false);
  assert.equal(fetched.length, requests);
  assert.equal(created, 1);
  assert.equal(stored.get("hex-command-sound-enabled"), savedPreference);
  const replacement = createSound();
  assert.equal(await replacement.unlockSound(), true);
  assert.equal(created, 2, "A new owner is independent of the disposed session");
});

test("Disposal settles a pending resume and prevents queued foreground fetches", async () => {
  let finishResume = () => {};
  const PendingContext = class extends FakeContext {
    resume() {
      resumed++;
      return new Promise<void>((resolve) => {
        finishResume = () => {
          this.state = "running";
          resolve();
        };
      });
    }
  };
  vi.stubGlobal("AudioContext", PendingContext);
  const sound = createSound();
  const unlocking = sound.unlockSound();
  const playing = sound.playSound("turn");
  const context = required(audio);
  sound.dispose();
  assert.equal(await unlocking, false, "Close does not wait for the browser resume promise");
  assert.equal(await playing, false);
  assert.equal(context.closes, 1);
  finishResume();
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(fetched.length, 0, "Late resume completion cannot fetch for a closed owner");
  assert.equal(context.sources.length, 1, "Only the gesture's silent prime was created");
  assert.equal(context.sources[0]!.disconnected, true);
});

test("Disposal settles pending decoding without reviving playback or surfacing close errors", async () => {
  const sound = createSound();
  await sound.unlockSound();
  const context = required(audio);
  let finishDecode = () => {};
  let announceDecode = () => {};
  const decoding = new Promise<void>((resolve) => {
    announceDecode = resolve;
  });
  context.decodeAudioData = (bytes) =>
    new Promise<FakeBuffer>((resolve) => {
      finishDecode = () => resolve({ name: new TextDecoder().decode(bytes) });
      announceDecode();
    });
  context.close = () => {
    context.closes++;
    return Promise.reject(Error("Already closing"));
  };
  const playing = sound.playSound("turn");
  await decoding;
  const beforeClose = context.sources.length;
  sound.dispose();
  assert.equal(await playing, false, "Close does not wait for browser decoding");
  finishDecode();
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(context.sources.length, beforeClose);
  assert.equal(context.closes, 1);
  assert.equal(await sound.playSound("turn"), false);
});

const sessions: Audio.Audio[] = [];
function createSound() {
  const audio = Audio.create();
  sessions.push(audio);
  // These test-only conveniences call the actual emitted OCaml owner.
  return {
    soundEnabled: () => Audio.enabled(audio),
    setSoundEnabled: (enabled: boolean) => Audio.set_enabled(audio, enabled),
    cancelSound: () => Audio.cancel(audio),
    dispose: () => Audio.close(audio),
    unlockSound: (event?: { isTrusted: boolean }) =>
      Audio.unlock(audio, event?.isTrusted !== false),
    playSound: (family: string) => Audio.play(audio, family),
    soundForInteraction: (details: { attackType?: string; unitType?: string; terrain?: string }) =>
      Audio.interaction_family(details.attackType, details.unitType, details.terrain, undefined),
    playInteraction: (details: { attackType?: string; unitType?: string; terrain?: string }) =>
      Audio.play_interaction(
        audio,
        details.attackType,
        details.unitType,
        details.terrain,
        undefined,
      ),
  };
}
beforeEach(() => {
  stored.clear();
  created = 0;
  resumed = 0;
  audio = undefined;
  fetched.length = 0;
  vi.stubGlobal("localStorage", new TestStorage(stored));
  vi.stubGlobal("AudioContext", FakeContext);
  vi.stubGlobal("fetch", async (url: RequestInfo | URL) => {
    fetched.push(String(url));
    return new Response(String(url));
  });
});
afterEach(() => {
  for (const audio of sessions.splice(0)) Audio.close(audio);
});

afterEach(() => {
  vi.unstubAllGlobals();
});
