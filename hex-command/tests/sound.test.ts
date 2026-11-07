import { test, beforeEach, afterEach, vi } from "vitest";
import assert from "node:assert/strict";
import { TestStorage } from "./helpers.ts";
import { createSound, audioSamples } from "../src/presentation/sound.ts";
import palette from "../src/presentation/audio-palette.json";
import type Phaser from "phaser";
const stored = new Map<string, string>();
beforeEach(() => {
  vi.stubGlobal("localStorage", new TestStorage(stored));
  stored.clear();
});
afterEach(() => vi.unstubAllGlobals());
// Priority resolves a single family even when all three contexts exist.
test("Attack sound takes priority over unit and terrain", async () => {
  const sound = createSound();
  assert.equal(
    sound.soundForInteraction({
      attackType: "archer",
      attackId: "bow",
      unitType: "soldier",
      terrain: "village",
    }),
    palette.units.archer.attacks.bow,
  );
  assert.equal(
    sound.soundForInteraction({
      attackType: "scout",
      attackId: "knife",
      unitType: "archer",
      terrain: "forest",
    }),
    palette.units.scout.attacks.knife,
  );
});
for (const unitType of ["soldier", "archer", "scout", "commander"] as const) {
  test("Unit sound priority follows its material — " + JSON.stringify([unitType]), async () => {
    const sound = createSound();
    assert.equal(
      sound.soundForInteraction({ unitType, terrain: "hill" }),
      palette.units[unitType].selection,
    );
  });
}
test("Terrain sound resolves its own family", () => {
  const sound = createSound();
  assert.equal(sound.soundForInteraction({ terrain: "plains" }), palette.terrain.plains);
});
test("Fords use water sounds and empty interactions use selection", async () => {
  const sound = createSound();
  assert.equal(sound.soundForInteraction({ terrain: "ford" }), palette.terrain.ford);
  assert.equal(sound.soundForInteraction({}), "select");
});
// Existing variation/muting contract now exercises the native manager boundary.
test("Shuffle bags avoid repeats, reuse preload and stop voices when muted", async () => {
  const sound = createSound();
  const played: { key: string; rate: number }[] = [];
  let stopped = 0;
  const game = {
    cache: {
      audio: { exists: (key: string) => audioSamples.some((sample) => sample.key === key) },
    },
    sound: {
      context: { state: "running", currentTime: 0 },
      add(key: string, config: { rate: number }) {
        return {
          duration: 0.1,
          startTime: 0,
          once() {},
          destroy() {
            stopped++;
          },
          play() {
            played.push({ key, rate: config.rate });
            return true;
          },
        };
      },
    },
  } as unknown as Phaser.Game;
  vi.stubGlobal("document", { addEventListener() {}, removeEventListener() {} });
  const detach = sound.attachSound(game, async () => true);
  assert.equal(await sound.playSound("unit-soldier"), false);
  sound.soundReady(game);
  for (let i = 0; i < 9; i++) assert.equal(await sound.playSound("unit-soldier"), true);
  assert.equal(new Set(played.slice(0, 7).map((value) => value.rate)).size, 7);
  assert.equal(new Set(played.map((value) => value.key)).size, 1);
  assert.ok(played.every((value, i) => i === 0 || value.rate !== played[i - 1]!.rate));
  sound.setSoundEnabled(false);
  assert.equal(stopped, 9);
  assert.equal(await sound.playSound("attack-bow"), false);
  assert.equal(stored.get("hex-command-sound-enabled"), "false");
  detach();
});
