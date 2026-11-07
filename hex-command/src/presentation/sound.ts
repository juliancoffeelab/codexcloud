import type { UnitType } from "../game/types";

export interface InteractionSound {
  attackType?: UnitType;
  unitType?: UnitType;
  /** Gameplay role supplied by the Rust catalog, never an artwork ID. */
  terrain?: import("../../rust/pkg/hex_command").TerrainRole;
}

import type Phaser from "phaser";
import { writable } from "svelte/store";
import { createHudAudio } from "./hud-sound";
import { errorDetails } from "../error-details";
const STORAGE_KEY = "hex-command-sound-enabled";
const VOLUME = 0.29;
const AUDIO_REVISION = "6ad634f16189";
/** Actions available as soon as a battlefield is interactive, including a
 * first-action win/loss. Contextual narrative cues remain in their own plan. */
export const battlefieldActionFamilies = [
  "select",
  "recruit",
  "turn",
  "victory",
  "defeat",
] as const;
const warmFamilies = new Set([
  "attack-bow",
  "attack-melee",
  "unit-soldier",
  "unit-archer",
  "unit-scout",
  "unit-commander",
  "terrain-village",
  "terrain-water",
  "terrain-forest",
  "terrain-hill",
  "terrain-plains",
  "terrain-keep",
  ...battlefieldActionFamilies,
]);
// Narrative cues are valid sounds, but do not belong to global gesture warmup.
const families = new Set([...warmFamilies, "bell"]);
export const audioReadiness = writable<{ ready: boolean; error?: string }>({ ready: false });
export const audioSamples = [...families].flatMap((family) =>
  [1, 2, 3].map((variant) => ({
    family,
    key: `${family}-${variant}`,
    url: `${import.meta.env?.BASE_URL ?? "/"}assets/audio/${family}-${variant}.wav?v=${AUDIO_REVISION}`,
  })),
);

export function createSound() {
  let enabled = true;
  try {
    enabled = localStorage.getItem(STORAGE_KEY) !== "false";
  } catch {}
  let game: Phaser.Game | undefined;
  let hud: ReturnType<typeof createHudAudio> | undefined;
  let ready = false;
  let requestPermission: (() => Promise<boolean>) | undefined;
  let voice: Phaser.Sound.BaseSound | undefined;
  let voiceRequest = 0;
  let unlocking: Promise<boolean> | undefined;
  const bags = new Map<string, number[]>();
  const lastVariation = new Map<string, number>();
  function soundEnabled() {
    return enabled;
  }
  function attachSound(owner: Phaser.Game, permission: () => Promise<boolean>) {
    voice?.destroy();
    voice = undefined;
    game = owner;
    requestPermission = permission;
    ready = false;
    audioReadiness.set({ ready: false });
    const activate = (event: Event) => {
      if (!enabled || !event.isTrusted) return;
      unlocking = permission().catch((cause) => {
        if (game !== owner) return false;
        audioReadiness.set({
          ready,
          error: errorDetails(
            new Error(
              "Browser audio permission could not be resumed. Retry with another gesture.",
              { cause },
            ),
          ),
        });
        return false;
      });
    };
    const gestures = ["pointerdown", "pointerup", "touchend", "keydown", "click"];
    for (const type of gestures)
      document.addEventListener(type, activate, { capture: true, passive: true });
    return () => {
      for (const type of gestures) document.removeEventListener(type, activate, true);
      if (game === owner) {
        voice?.destroy();
        voice = undefined;
        game = undefined;
        requestPermission = undefined;
        unlocking = undefined;
        voiceRequest++;
        ready = false;
      }
    };
  }
  function soundReady(owner: Phaser.Game) {
    if (game !== owner) return;
    ready = true;
    audioReadiness.set({ ready: true });
  }
  function prepareSounds(requested: Iterable<string>) {
    for (const family of requested)
      if (!families.has(family)) throw new Error(`Unknown audio family: ${family}`);
  }
  // A tile interaction has exactly one family: attack > occupant > ground.
  function soundForInteraction({ attackType, unitType, terrain }: InteractionSound = {}) {
    if (attackType) return attackType === "archer" ? "attack-bow" : "attack-melee";
    if (unitType === "scout" || unitType === "archer") return "terrain-plains";
    // Give armored soldiers the heavier footstep formerly used by archers.
    if (
      unitType === "soldier" ||
      unitType === "warden" ||
      unitType === "boar" ||
      unitType === "merfolk-maceguard" ||
      unitType === "merfolk-maceguard-l2"
    )
      return "unit-archer";
    if (
      unitType === "wildkeeper" ||
      unitType === "necromancer" ||
      unitType === "skeleton" ||
      unitType === "skeleton-impact-caster" ||
      unitType === "skeleton-impact-caster-l2" ||
      unitType === "bonearcher"
    )
      return "unit-commander";
    if (unitType === "wolf" || unitType === "thorncaller" || unitType === "adept")
      return "terrain-plains";
    if (families.has(`unit-${unitType}`)) return `unit-${unitType}`;
    if (terrain === "ford" || terrain === "marsh") terrain = "water";
    if (terrain === "grove") terrain = "forest";
    if (terrain === "ruins" || terrain === "fort" || terrain === "fortification") terrain = "keep";
    return families.has(`terrain-${terrain}`) ? `terrain-${terrain}` : "select";
  }
  function playInteraction(details: InteractionSound) {
    return playSound(soundForInteraction(details));
  }

  function prepareHudSound() {
    hud ??= createHudAudio(() => enabled);
    return hud.prepare();
  }
  function playHudSound() {
    return hud?.play() ?? false;
  }
  function disposeHudSound() {
    hud?.dispose();
    hud = undefined;
  }
  function stopNative() {
    voiceRequest++;
    voice?.destroy();
    voice = undefined;
  }
  function cancelSound() {
    hud?.stop();
    stopNative();
  }
  function setSoundEnabled(value: boolean) {
    enabled = Boolean(value);
    if (!enabled) cancelSound();
    try {
      localStorage.setItem(STORAGE_KEY, String(enabled));
    } catch {}
  }
  async function unlockSound(event?: Pick<Event, "isTrusted">) {
    if (!enabled || event?.isTrusted === false) return false;
    const allowed = hud?.unlock() ?? Promise.resolve(false);
    const native = requestPermission?.() ?? Promise.resolve(false);
    return (await Promise.all([allowed, native])).some(Boolean);
  }
  function variation(family: string): number {
    let bag = bags.get(family);
    if (!bag?.length) {
      bag = [1, 2, 3];
      for (let i = 2; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [bag[i], bag[j]] = [bag[j]!, bag[i]!];
      }
      if (bag.at(-1) === lastVariation.get(family)) [bag[0], bag[2]] = [bag[2]!, bag[0]!];
      bags.set(family, bag);
    }
    const chosen = bag.pop()!;
    lastVariation.set(family, chosen);
    return chosen;
  }
  async function playSound(family: string) {
    if (!enabled || !game || !ready || !families.has(family)) return false;
    const owner = game;
    const manager = owner.sound as Phaser.Sound.WebAudioSoundManager;
    stopNative();
    const currentRequest = voiceRequest;
    if (manager.context && manager.context.state !== "running") {
      if (!unlocking || !(await unlocking)) return false;
    }
    if (!enabled || owner !== game || currentRequest !== voiceRequest) return false;
    const key = `${family}-${variation(family)}`;
    if (!owner.cache.audio.exists(key)) {
      audioReadiness.set({
        ready: false,
        error: errorDetails(new Error(`Prepared audio missing: ${key}`)),
      });
      return false;
    }
    voice = manager.add(key, { volume: VOLUME * (family === "select" ? 0.25 : 1) });
    const current = voice;
    voice.once("complete", () => {
      current.destroy();
      if (voice === current) voice = undefined;
    });
    return voice.play();
  }
  return {
    prepareHudSound,
    playHudSound,
    disposeHudSound,
    soundEnabled,
    prepareSounds,
    soundForInteraction,
    playInteraction,
    cancelSound,
    setSoundEnabled,
    unlockSound,
    playSound,
    attachSound,
    soundReady,
  };
}
export const {
  prepareHudSound,
  playHudSound,
  disposeHudSound,
  soundEnabled,
  prepareSounds,
  soundForInteraction,
  playInteraction,
  cancelSound,
  setSoundEnabled,
  unlockSound,
  playSound,
  attachSound,
  soundReady,
} = createSound();
