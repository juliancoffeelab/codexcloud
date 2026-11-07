import paletteData from "./audio-palette.json";
import type { UnitType } from "../game/types";

export interface InteractionSound {
  attackType?: UnitType;
  attackId?: string;
  attackSound?: string;
  attackCategory?: import("../game/types").AttackCategory;
  attackDamageType?: import("../game/types").DamageType;
  unitType?: UnitType;
  /** Canonical Rust terrain identifier, including special sites. */
  terrain?: import("../game/types").TerrainType;
}

interface PaletteUnit {
  selection: string | null;
  death: string | null;
  attacks: Record<string, string | null>;
}
const palette: {
  clips: Record<string, { file: string }>;
  units: Record<string, PaletteUnit>;
  terrain: Record<string, string | null>;
  fallbackAttacks: Record<string, string>;
  movement: string;
} = paletteData;
function paletteClip(cue: string) {
  return cue.startsWith("palette:") ? palette.clips[cue.slice(8)] : undefined;
}
/** Presentation-only cue IDs; rules and available units still come from Rust. */
export function battleSounds(
  unitTypes: Iterable<string>,
  terrainTypes: Iterable<string>,
): string[] {
  const cues = new Set<string>([palette.movement]);
  for (const type of unitTypes) {
    const unit = palette.units[type];
    if (!unit) continue;
    for (const cue of [unit.selection, unit.death, ...Object.values(unit.attacks)])
      if (cue) cues.add(cue);
  }
  for (const type of terrainTypes) if (palette.terrain[type]) cues.add(palette.terrain[type]!);
  return [...cues];
}

import type Phaser from "phaser";
import { writable } from "svelte/store";
import { createHudAudio } from "./hud-sound";
import { errorDetails } from "../error-details";
const STORAGE_KEY = "hex-command-sound-enabled";
const VOLUME = 0.29;
const AUDIO_REVISION = "phaser-palette-4";
/** Actions available as soon as a battlefield is interactive, including a
 * first-action win/loss. Contextual narrative cues remain in their own plan. */
export const battlefieldActionFamilies = [
  "select",
  "recruit",
  "turn",
  "victory",
  "defeat",
] as const;
const families = new Set([
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
  "bell",
]);
export const audioReadiness = writable<{ ready: boolean; error?: string }>({ ready: false });
export const audioSamples = [
  ...[...families].map((family) => ({
    family,
    key: `${family}-1`,
    url: `${import.meta.env?.BASE_URL ?? "/"}assets/audio/${family}-1.wav?v=${AUDIO_REVISION}`,
  })),
  ...Object.entries(palette.clips).map(([id, clip]) => ({
    family: `palette:${id}`,
    key: `palette:${id}`,
    url: `${import.meta.env?.BASE_URL ?? "/"}assets/audio/${clip.file}?v=${AUDIO_REVISION}`,
  })),
];

export function createSound() {
  let enabled = true;
  try {
    enabled = localStorage.getItem(STORAGE_KEY) !== "false";
  } catch {}
  let game: Phaser.Game | undefined;
  let hud: ReturnType<typeof createHudAudio> | undefined;
  let ready = false;
  let requestPermission: (() => Promise<boolean>) | undefined;
  const voices = new Set<{ sound: Phaser.Sound.BaseSound; channel: "feedback" | "combat" }>();
  let voiceRequest = 0;
  let unlocking: Promise<boolean> | undefined;
  const bags = new Map<string, number[]>();
  const lastVariation = new Map<string, number>();
  function soundEnabled() {
    return enabled;
  }
  function attachSound(owner: Phaser.Game, permission: () => Promise<boolean>) {
    stopNative();
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
        stopNative();
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
      if (!families.has(family) && !paletteClip(family))
        throw new Error(`Unknown audio family: ${family}`);
  }
  // One chosen source/take: attack > unit > ground. Never rotate recordings.
  function soundForInteraction({
    attackType,
    attackId,
    attackSound,
    attackCategory,
    attackDamageType,
    unitType,
    terrain,
  }: InteractionSound = {}) {
    if (attackType && attackId) {
      const unit = palette.units[attackType];
      const cue =
        unit?.attacks[attackId] ||
        (attackCategory && attackDamageType
          ? palette.fallbackAttacks[`${attackCategory}:${attackDamageType}`]
          : undefined);
      return (
        cue ||
        (attackSound && families.has(attackSound)
          ? attackSound
          : attackType === "archer"
            ? "attack-bow"
            : "attack-melee")
      );
    }
    if (unitType && palette.units[unitType]?.selection) return palette.units[unitType]!.selection!;
    if (terrain && palette.terrain[terrain]) return palette.terrain[terrain]!;
    return "select";
  }
  function playInteraction(details: InteractionSound) {
    return playSequence(
      [soundForInteraction(details)],
      "feedback",
      details.attackType || details.unitType ? 0.8 : details.terrain ? 0.6 : 0.35,
    );
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
  function stopNative(channel?: "feedback" | "combat") {
    voiceRequest++;
    for (const voice of voices) {
      if (channel && voice.channel !== channel) continue;
      voice.sound.destroy();
      voices.delete(voice);
    }
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
      bag = [-1, -2 / 3, -1 / 3, 0, 1 / 3, 2 / 3, 1];
      for (let i = bag.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [bag[i], bag[j]] = [bag[j]!, bag[i]!];
      }
      if (bag.at(-1) === lastVariation.get(family))
        [bag[0], bag[bag.length - 1]] = [bag[bag.length - 1]!, bag[0]!];
      bags.set(family, bag);
    }
    const chosen = bag.pop()!;
    lastVariation.set(family, chosen);
    return chosen;
  }
  async function playSequence(
    cues: readonly string[],
    channel: "feedback" | "combat" = "feedback",
    pitchSpread?: number,
  ) {
    if (!enabled || !game || !ready) return false;
    const owner = game;
    const manager = owner.sound as Phaser.Sound.WebAudioSoundManager;
    stopNative("feedback");
    const currentRequest = voiceRequest;
    if (manager.context && manager.context.state !== "running") {
      if (!unlocking || !(await unlocking)) return false;
    }
    if (!enabled || owner !== game || currentRequest !== voiceRequest) return false;
    let offset = 0,
      started = false;
    for (let i = 0; i < cues.length; i++) {
      const family = cues[i]!;
      const clip = paletteClip(family);
      if (!clip && !families.has(family)) continue;
      const key = clip ? family : `${family}-1`;
      if (!owner.cache.audio.exists(key)) {
        audioReadiness.set({
          ready: false,
          error: errorDetails(new Error(`Prepared audio missing: ${key}`)),
        });
        continue;
      }
      const spread =
        pitchSpread ?? (family === "select" ? 0.35 : channel === "combat" && i > 0 ? 1.2 : 0.8);
      const rate = 2 ** ((variation(family) * spread) / 12);
      const sound = manager.add(key, {
        volume: clip ? 1 : VOLUME * (family === "select" ? 0.25 : 1),
        rate,
      });
      const voice = { sound, channel };
      if (voices.size >= 12) {
        const oldest = voices.values().next().value;
        if (oldest) {
          oldest.sound.destroy();
          voices.delete(oldest);
        }
      }
      voices.add(voice);
      sound.once("complete", () => {
        sound.destroy();
        voices.delete(voice);
      });
      if (sound.play({ delay: offset })) {
        started = true;
        // Shape Phaser's own voice gain; keep loading and sources owned by Phaser.
        const native = sound as Phaser.Sound.WebAudioSound;
        if (native.volumeNode && manager.context) {
          const gain = native.volumeNode.gain;
          const at = native.startTime;
          const duration = sound.duration / rate;
          const volume = clip ? 1 : VOLUME * (family === "select" ? 0.25 : 1);
          gain.setValueAtTime(0, at);
          gain.linearRampToValueAtTime(volume, at + Math.min(0.005, duration / 4));
          gain.setValueAtTime(volume, at + duration - Math.min(0.008, duration / 4));
          gain.linearRampToValueAtTime(0, at + duration);
        }
      } else {
        sound.destroy();
        voices.delete(voice);
      }
      offset += Math.min(sound.duration / rate, 0.18);
    }
    return started;
  }
  function playSound(family: string) {
    return playSequence([family]);
  }
  function playMovement() {
    return playSequence([palette.movement], "feedback", 0.35);
  }
  function playCombat(attack: InteractionSound, deaths: readonly UnitType[]) {
    return playSequence(
      [
        soundForInteraction(attack),
        ...deaths
          .map((type) => palette.units[type]?.death)
          .filter((cue): cue is string => Boolean(cue)),
      ],
      "combat",
    );
  }
  return {
    prepareHudSound,
    playHudSound,
    disposeHudSound,
    soundEnabled,
    prepareSounds,
    soundForInteraction,
    playInteraction,
    playCombat,
    playMovement,
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
  playCombat,
  playMovement,
  cancelSound,
  setSoundEnabled,
  unlockSound,
  playSound,
  attachSound,
  soundReady,
} = createSound();
