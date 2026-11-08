import { test, vi } from "vitest";
import assert from "node:assert/strict";
import * as Sound from "@frontend/Phaser_sound";
function nativeSound() {
  vi.stubGlobal("document", new EventTarget());
  const voices = [];
  const game = {
    cache: { audio: { exists: () => true } },
    sound: {
      context: { state: "running", currentTime: 10 },
      add(key, config) {
        const voice = {
          key,
          config,
          duration: 0.3,
          play(options) {
            this.options = options;
            this.startTime = 10 + options.delay;
            return true;
          },
          once(name, callback) {
            assert.equal(name, "complete");
            this.complete = callback;
            return this;
          },
          destroy: vi.fn(),
        };
        voices.push(voice);
        return voice;
      },
    },
  };
  const audio = Sound.create();
  const detach = Sound.attach(audio, game, async () => true);
  Sound.sound_ready(audio, game);
  return { audio, game, voices, detach };
}
test("Silent and unprepared audio does not create native voices or pending owners", async () => {
  const sound = Sound.create();
  Sound.set_enabled(sound, false);
  const owner = {};
  assert.equal(await Sound.play(owner, sound, "turn"), false);
  assert.equal(Sound.pending(sound, owner), false);
  Sound.close(sound);
});
test("Native sound completion, rather than a guessed duration, settles owned voices", async () => {
  const { audio, voices } = nativeSound(),
    owner = {};
  let changes = 0;
  const unwatch = Sound.watch(audio, owner, () => changes++);
  assert.equal(await Sound.play(owner, audio, "turn"), true);
  assert.equal(voices.length, 1);
  assert.equal(Sound.pending(audio, owner), true);
  voices[0].complete();
  assert.equal(Sound.pending(audio, owner), false);
  assert.equal(voices[0].destroy.mock.calls.length, 1);
  assert(changes >= 2);
  unwatch();
  Sound.close(audio);
});
test("Owner cancellation cannot stop another action, and disable cancels remaining audio", async () => {
  const { audio, voices } = nativeSound(),
    a = {},
    b = {};
  await Sound.play(a, audio, "select");
  await Sound.play(b, audio, "turn");
  Sound.stop_owner(undefined, audio, a);
  assert.equal(Sound.pending(audio, a), false);
  assert.equal(Sound.pending(audio, b), true);
  assert.equal(voices[0].destroy.mock.calls.length, 1);
  assert.equal(voices[1].destroy.mock.calls.length, 0);
  Sound.set_enabled(audio, false);
  assert.equal(Sound.pending(audio, b), false);
  assert.equal(voices[1].destroy.mock.calls.length, 1);
  Sound.close(audio);
});
test("A replacement game detaches only its own native sound owner", async () => {
  const { audio, game, detach } = nativeSound();
  const replacement = { ...game, sound: { ...game.sound } };
  Sound.attach(audio, replacement, async () => true);
  Sound.sound_ready(audio, replacement);
  detach();
  assert.equal(await Sound.play({}, audio, "turn"), true);
  Sound.close(audio);
  Sound.close(audio);
});
test("Pitch bags cover seven distinct variations and avoid repeats across bag boundaries", () => {
  const { audio, voices } = nativeSound();
  const sample = () => {
    Sound.schedule_cue(
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      audio,
      "turn",
      10,
      /* Feedback */ 0,
    );
    const voice = voices.at(-1);
    const rate = voice.config.rate;
    voice.complete();
    return rate;
  };
  const first = Array.from({ length: 7 }, sample);
  assert.equal(new Set(first).size, 7);
  assert.notEqual(sample(), first.at(-1));
  Sound.close(audio);
});
