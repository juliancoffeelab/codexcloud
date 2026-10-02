import assert from "node:assert/strict";
import { required, TestStorage } from "./helpers.ts";
import { createSound } from "../src/presentation/sound.ts";
const stored = new Map<string, string>();
globalThis.localStorage = new TestStorage(stored);
let created = 0,
  resumed = 0;
let audio: FakeContext | undefined;
const fetched: string[] = [];
globalThis.fetch = async (url) => {
  fetched.push(String(url));
  return new Response(String(url));
};
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
class FakeContext {
  state = "suspended";
  currentTime = 10;
  destination = {};
  sources: FakeSource[] = [];
  constructor() {
    created++;
    audio = this;
  }
  async resume() {
    resumed++;
    this.state = "running";
  }
  createBuffer() {
    return { silent: true };
  }
  createGain() {
    return { gain: new Param(), connect() {}, disconnect() {} };
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
globalThis.AudioContext = FakeContext as unknown as typeof AudioContext;
const sound = createSound();
assert.equal(sound.soundEnabled(), true);
await sound.playSound("turn");
assert.equal(created, 0);
assert.equal(fetched.length, 0);
sound.setSoundEnabled(false);
await sound.unlockSound();
assert.equal(created, 0);
sound.setSoundEnabled(true);
await sound.unlockSound();
assert.equal(created, 1);
assert.equal(resumed, 1);
// Priority resolves a single family even when all three contexts exist.
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
for (const unitType of ["soldier", "archer", "scout", "commander"] as const) {
  assert.equal(
    sound.soundForInteraction({ unitType, terrain: "hill" }),
    ["archer", "scout"].includes(unitType)
      ? "terrain-plains"
      : unitType === "soldier"
        ? "unit-archer"
        : `unit-${unitType}`,
  );
}
for (const terrain of [
  "village",
  "water",
  "forest",
  "hill",
  "plains",
  "keep",
] as const) {
  assert.equal(sound.soundForInteraction({ terrain }), `terrain-${terrain}`);
}
assert.equal(sound.soundForInteraction({ terrain: "ford" }), "terrain-water");
assert.equal(sound.soundForInteraction({}), "select");
// A shuffle bag plays all three variations, without an immediate repeat at refill.
const variants: string[] = [];
for (let i = 0; i < 9; i++) {
  assert.equal(await sound.playSound("unit-soldier"), true);
  const source = required(required(audio).sources.at(-1));
  const match = required(required(source.buffer.name).match(/-(\d)\.wav/));
  variants.push(required(match[1]));
}
for (let i = 0; i < 9; i += 3)
  assert.equal(new Set(variants.slice(i, i + 3)).size, 3);
assert.ok(variants.every((value, i) => i === 0 || value !== variants[i - 1]));
assert.equal(
  fetched.length,
  51,
  "all variations warm after the first gesture and are reused",
);
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
// Delayed preload responses cannot revive superseded or muted clicks.
const pending: { name: string; resolve: () => void }[] = [];
globalThis.fetch = (url) =>
  new Promise<Response>((resolve) =>
    pending.push({
      name: String(url),
      resolve: () => resolve(new Response(String(url))),
    }),
  );
const cold = createSound();
await cold.unlockSound();
assert.equal(pending.length, 51);
const old = cold.playSound("terrain-forest");
const latest = cold.playInteraction({
  attackType: "archer",
  unitType: "soldier",
  terrain: "forest",
});
for (const sample of pending.filter((p) => p.name.includes("/terrain-forest-")))
  sample.resolve();
assert.equal(await old, false);
for (const sample of pending.filter((p) => p.name.includes("/attack-bow-")))
  sample.resolve();
assert.equal(await latest, true);
const muted = cold.playSound("terrain-water");
cold.setSoundEnabled(false);
for (const sample of pending.filter((p) => p.name.includes("/terrain-water-")))
  sample.resolve();
assert.equal(await muted, false);
cold.setSoundEnabled(true);
required(audio).state = "suspended";
assert.equal(await cold.playSound("turn"), false);
const resumeCount = resumed;
await cold.unlockSound();
assert.equal(resumed, resumeCount + 1);
// Mobile browsers can leave a touch-start resume pending. A completed tap must
// make a fresh attempt, and resume an interrupted context after app switching.
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
assert.equal(
  required(audio).sources.length,
  1,
  "the gesture primes mobile output with silence",
);
await Promise.race([
  touch.unlockSound(gesture("touchend", true)),
  new Promise((_, reject) =>
    setTimeout(() => reject(Error("touch resume remained stuck")), 100),
  ),
]);
assert.equal(attempts, 2);
assert.equal(await touch.playSound("terrain-plains"), true);
required(audio).state = "interrupted";
assert.equal(await touch.playSound("terrain-plains"), false);
await touch.unlockSound(gesture("pointerup", true));
assert.equal(attempts, 3);
assert.equal(await touch.playSound("terrain-plains"), true);
required(audio).state = "closed";
await touch.unlockSound(gesture("click", true));
assert.equal(
  required(audio).state,
  "running",
  "closed mobile contexts can be recreated",
);
const attemptsBefore = attempts;
await touch.unlockSound(gesture("click", false));
assert.equal(attempts, attemptsBefore, "scripted events cannot unlock output");
globalThis.AudioContext = NativeFake;
// Failed preload and playback remain harmless.
globalThis.fetch = async () => new Response(null, { status: 500 });
const failed = createSound();
await failed.unlockSound();
assert.equal(
  await failed.playSound("terrain-hill"),
  false,
  "failed audio never throws",
);
Reflect.deleteProperty(globalThis, "AudioContext");
const unsupported = createSound();
await unsupported.unlockSound();
assert.equal(await unsupported.playSound("turn"), false);
console.log(
  "PASS: distinct cue priority, three variations, caching, latest-only playback, gestures, mute and unavailable audio",
);
