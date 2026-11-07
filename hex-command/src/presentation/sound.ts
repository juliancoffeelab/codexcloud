import { traceInstant } from "./performance-trace";
import paletteData from "./audio-palette.json";
import type { UnitType } from "../game/types";

export type SoundOwner = object | string;
interface CueOwnership {
  owner?: SoundOwner;
  key?: string;
  replace?: boolean;
}
export interface InteractionSound {
  owner?: SoundOwner;
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
  clips: Record<string, { file: string; impactMs: number }>;
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
const AUDIO_REVISION = "reviewed-ranges-5";
// Highest absolute-amplitude samples in decoded 44.1 kHz mono; no RMS window.
const legacyImpactMs: Record<string, number> = {
  "attack-bow": 121.632653,
  "attack-melee": 29.024943,
  bell: 61.814059,
  defeat: 258.61678,
  recruit: 88.798186,
  select: 8.344671,
  "terrain-forest": 28.049887,
  "terrain-hill": 53.446712,
  "terrain-keep": 24.126984,
  "terrain-plains": 52.60771,
  "terrain-village": 10.385488,
  "terrain-water": 11.972789,
  turn: 221.746032,
  "unit-archer": 31.655329,
  "unit-commander": 40.521542,
  "unit-scout": 19.84127,
  "unit-soldier": 9.727891,
  victory: 221.85941,
};
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
  type Voice = {
    sound: Phaser.Sound.BaseSound;
    channel: "feedback" | "combat";
    owner: SoundOwner;
    key?: string;
    family: string;
    rate: number;
    volume: number;
    seek: number;
  };
  const voices = new Set<Voice>();
  const paused = new Set<Omit<Voice, "sound">>();
  const requests = new Set<{ owner: SoundOwner; cancelled: boolean }>();
  const watchers = new Map<SoundOwner, Set<() => void>>();
  function notify(owner: SoundOwner) {
    for (const watcher of watchers.get(owner) ?? []) watcher();
  }
  function watchSoundOwner(owner: SoundOwner, watcher: () => void) {
    const set = watchers.get(owner) ?? new Set();
    set.add(watcher);
    watchers.set(owner, set);
    return () => {
      set.delete(watcher);
      if (!set.size) watchers.delete(owner);
    };
  }
  function soundPending(owner: SoundOwner) {
    return (
      [...voices, ...paused].some((voice) => voice.owner === owner) ||
      [...requests].some((request) => request.owner === owner && !request.cancelled)
    );
  }
  function removeVoice(voice: Voice) {
    voices.delete(voice);
    voice.sound.destroy();
    notify(voice.owner);
  }
  function stopSoundOwner(owner: SoundOwner, key?: string) {
    if (key === undefined)
      for (const request of requests) if (request.owner === owner) request.cancelled = true;
    for (const voice of voices)
      if (voice.owner === owner && (key === undefined || voice.key === key)) removeVoice(voice);
    for (const voice of paused)
      if (voice.owner === owner && (key === undefined || voice.key === key)) paused.delete(voice);
    notify(owner);
  }
  /** A renderer restart retains unfinished action audio, not an unrelated UI voice. */
  function pauseSoundOwner(owner: SoundOwner) {
    for (const voice of voices)
      if (voice.owner === owner) {
        const seek = (voice.sound as Phaser.Sound.WebAudioSound).seek;
        const { sound, ...record } = voice;
        paused.add({ ...record, seek: Number.isFinite(seek) ? seek : voice.seek });
        voices.delete(voice);
        voice.sound.destroy();
      }
    notify(owner);
  }
  function resumeSoundOwner(owner: SoundOwner) {
    if (!game || !ready) return;
    for (const voice of paused)
      if (voice.owner === owner) {
        const manager = game.sound as Phaser.Sound.WebAudioSoundManager;
        const key = paletteClip(voice.family) ? voice.family : `${voice.family}-1`;
        if (!game.cache.audio.exists(key))
          throw new Error(`Cannot resume prepared action cue: ${key}`);
        const sound = manager.add(key, { volume: voice.volume, rate: voice.rate });
        const resumed = { ...voice, sound };
        voices.add(resumed);
        paused.delete(voice);
        sound.once("complete", () => removeVoice(resumed));
        if (!sound.play({ seek: voice.seek })) {
          removeVoice(resumed);
          paused.add(voice);
          throw new Error(`Cannot resume action cue: ${key}`);
        }
      }
    notify(owner);
  }
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
      details.owner ?? {},
      true,
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
    if (!channel)
      for (const request of requests) {
        request.cancelled = true;
        notify(request.owner);
      }
    for (const voice of voices) {
      if (channel && voice.channel !== channel) continue;
      removeVoice(voice);
    }
  }
  function cancelSound() {
    hud?.stop();
    stopNative();
    for (const voice of paused) {
      paused.delete(voice);
      notify(voice.owner);
    }
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
    owner: SoundOwner = {},
    replace = false,
  ) {
    if (!enabled || !game || !ready) return false;
    const gameOwner = game;
    const manager = gameOwner.sound as Phaser.Sound.WebAudioSoundManager;
    if (replace) stopSoundOwner(owner);
    const pending = { owner, cancelled: false };
    requests.add(pending);
    notify(owner);
    try {
      const currentRequest = voiceRequest;
      if (manager.context && manager.context.state !== "running") {
        if (!unlocking || !(await unlocking)) return false;
      }
      if (!enabled || gameOwner !== game || currentRequest !== voiceRequest || pending.cancelled)
        return false;
      let at = performance.now(),
        started = false;
      for (const [i, family] of cues.entries()) {
        const spread =
          pitchSpread ?? (family === "select" ? 0.35 : channel === "combat" && i > 0 ? 1.2 : 0.8);
        const timing = scheduleCue(family, at, channel, false, spread, { owner });
        started ||= timing.played;
        at = Math.min(timing.endAt, at + 180);
      }
      return started;
    } finally {
      requests.delete(pending);
      notify(owner);
    }
  }
  function playSound(family: string, owner: SoundOwner = {}) {
    return playSequence([family], "feedback", undefined, owner);
  }
  /** Schedule the recording ahead of its audible anchor, on Phaser's own voice.
   * Output timestamps map the audio device clock to performance time when exposed.
   * No file decoding or catalogue work happens on this path. */
  function scheduleCue(
    family: string,
    impactAt: number,
    channel: "feedback" | "combat",
    useImpact = true,
    pitchSpread = 0.8,
    ownership: CueOwnership = {},
  ): { impactAt: number; endAt: number; played: boolean } {
    if (!enabled || !game || !ready) return { impactAt, endAt: impactAt, played: false };
    const manager = game.sound as Phaser.Sound.WebAudioSoundManager;
    const context = manager.context;
    if (!context || context.state !== "running")
      return { impactAt, endAt: impactAt, played: false };
    const clip = paletteClip(family);
    if (!clip && !families.has(family)) throw new Error(`Unknown audio family: ${family}`);
    const key = clip ? family : `${family}-1`;
    if (!game.cache.audio.exists(key)) {
      audioReadiness.set({
        ready: false,
        error: errorDetails(new Error(`Prepared audio missing: ${key}`)),
      });
      return { impactAt, endAt: impactAt, played: false };
    }
    const owner = ownership.owner ?? {};
    if (ownership.replace || ownership.key !== undefined) stopSoundOwner(owner, ownership.key);
    const rate = 2 ** ((variation(family) * pitchSpread) / 12);
    const anchor = useImpact ? (clip?.impactMs ?? legacyImpactMs[family]!) / rate : 0;
    const now = performance.now();
    const output = context.getOutputTimestamp?.();
    const mapped = output && (output.contextTime ?? 0) > 0 && (output.performanceTime ?? 0) > 0;
    const clockAt = mapped ? output.contextTime! : context.currentTime;
    const performanceAt = mapped ? output.performanceTime! : now;
    const requestedStart = clockAt + (impactAt - performanceAt - anchor) / 1000;
    const start = Math.max(context.currentTime, requestedStart);
    const volume = clip ? 1 : VOLUME * (family === "select" ? 0.25 : 1);
    const sound = manager.add(key, { volume, rate });
    const voice: Voice = {
      sound,
      channel,
      owner,
      key: ownership.key,
      family,
      rate,
      volume,
      seek: 0,
    };
    voices.add(voice);
    notify(owner);
    sound.once("complete", () => removeVoice(voice));
    if (!sound.play({ delay: Math.max(0, start - context.currentTime) })) {
      removeVoice(voice);
      return { impactAt, endAt: impactAt, played: false };
    }
    const native = sound as Phaser.Sound.WebAudioSound;
    const duration = sound.duration / rate;
    const gain = native.volumeNode?.gain;
    gain?.setValueAtTime(0, native.startTime);
    gain?.linearRampToValueAtTime(volume, native.startTime + Math.min(0.005, duration / 4));
    gain?.setValueAtTime(volume, native.startTime + duration - Math.min(0.008, duration / 4));
    gain?.linearRampToValueAtTime(0, native.startTime + duration);
    const audibleStart = performanceAt + (native.startTime - clockAt) * 1000;
    const actualImpact = audibleStart + anchor;
    traceInstant("Audio cue scheduled", {
      family,
      requestedImpact: impactAt,
      impactAt: actualImpact,
      sourceStart: native.startTime,
      rate,
      anchor,
      outputTimestamp: !!mapped,
    });
    return { impactAt: actualImpact, endAt: audibleStart + duration * 1000, played: true };
  }
  function scheduleMovement(landingAt: number, owner: SoundOwner = {}) {
    return scheduleCue(palette.movement, landingAt, "feedback", true, 0.35, {
      owner,
      key: "movement",
    });
  }
  function scheduleDeath(type: UnitType, at: number, owner: SoundOwner = {}, key = String(type)) {
    const cue = palette.units[type]?.death;
    return cue
      ? scheduleCue(cue, at, "combat", false, 1.2, { owner, key: `death:${key}` })
      : { impactAt: at, endAt: at };
  }
  function cancelCombatSound(owner: SoundOwner) {
    stopSoundOwner(owner);
  }
  return {
    watchSoundOwner,
    soundPending,
    pauseSoundOwner,
    resumeSoundOwner,
    stopSoundOwner,
    prepareHudSound,
    playHudSound,
    disposeHudSound,
    soundEnabled,
    prepareSounds,
    soundForInteraction,
    playInteraction,
    scheduleCue,
    scheduleMovement,
    scheduleDeath,
    cancelCombatSound,
    cancelSound,
    setSoundEnabled,
    unlockSound,
    playSound,
    attachSound,
    soundReady,
  };
}
export const {
  watchSoundOwner,
  soundPending,
  pauseSoundOwner,
  resumeSoundOwner,
  stopSoundOwner,
  prepareHudSound,
  playHudSound,
  disposeHudSound,
  soundEnabled,
  prepareSounds,
  soundForInteraction,
  playInteraction,
  scheduleCue,
  scheduleMovement,
  scheduleDeath,
  cancelCombatSound,
  cancelSound,
  setSoundEnabled,
  unlockSound,
  playSound,
  attachSound,
  soundReady,
} = createSound();
