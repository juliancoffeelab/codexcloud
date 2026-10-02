import type { UnitType, TerrainType } from "../game/types";

export interface InteractionSound {
  attackType?: UnitType;
  unitType?: UnitType;
  terrain?: TerrainType;
}

interface Voice {
  source: AudioBufferSourceNode;
  gain: GainNode;
}
type AudioGlobals = typeof globalThis & {
  webkitAudioContext?: typeof AudioContext;
};

// Short material-inspired WAVs. Audio is created only after a real gesture.
const STORAGE_KEY = "hex-command-sound-enabled";
const VOLUME = 0.29;
const AUDIO_REVISION = "6ad634f16189";
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
  "select",
  "recruit",
  "turn",
  "victory",
  "defeat",
]);
export function createSound() {
  let enabled = true;
  try {
    enabled = localStorage.getItem(STORAGE_KEY) !== "false";
  } catch {}
  let context: AudioContext | null = null;
  let master: GainNode | null = null;
  let unlocking: Promise<void> | null = null;
  let primed = false,
    request = 0;
  const buffers = new Map<string, Promise<AudioBuffer>>();
  const bags = new Map<string, number[]>();
  const lastVariation = new Map<string, number>();
  const voices = new Set<Voice>();
  function soundEnabled() {
    return enabled;
  }

  // A tile interaction has exactly one family: attack > occupant > ground.
  function soundForInteraction({ attackType, unitType, terrain }: InteractionSound = {}) {
    if (attackType) return attackType === "archer" ? "attack-bow" : "attack-melee";
    if (unitType === "scout" || unitType === "archer") return "terrain-plains";
    // Give armored soldiers the heavier footstep formerly used by archers.
    if (unitType === "soldier") return "unit-archer";
    if (families.has(`unit-${unitType}`)) return `unit-${unitType}`;
    if (terrain === "ford") terrain = "water";
    return families.has(`terrain-${terrain}`) ? `terrain-${terrain}` : "select";
  }
  function playInteraction(details: InteractionSound) {
    return playSound(soundForInteraction(details));
  }

  function stopVoices() {
    for (const voice of voices) {
      try {
        const now = context!.currentTime;
        voice.gain.gain.cancelScheduledValues(now);
        voice.gain.gain.setValueAtTime(voice.gain.gain.value, now);
        voice.gain.gain.linearRampToValueAtTime(0, now + 0.008);
        voice.source.stop(now + 0.009);
      } catch {}
    }
    voices.clear();
  }
  function setSoundEnabled(value: boolean) {
    enabled = Boolean(value);
    request++; // Also discard sounds whose samples are still loading.
    try {
      localStorage.setItem(STORAGE_KEY, String(enabled));
    } catch {}
    if (!enabled) stopVoices();
  }
  async function unlockSound(event?: Pick<Event, "isTrusted">) {
    if (!enabled || event?.isTrusted === false) return false;
    try {
      if (!context || context.state === "closed") {
        const AudioContextClass =
          globalThis.AudioContext || (globalThis as AudioGlobals).webkitAudioContext;
        if (!AudioContextClass) return;
        context = new AudioContextClass({ latencyHint: "interactive" });
        primed = false;
        master = context.createGain();
        master.gain.value = VOLUME;
        master.connect(context.destination);
        // Warm every variation once after the first gesture, without playing it.
        for (const family of families) {
          for (const variant of [1, 2, 3]) void sample(family, variant).catch(() => {});
        }
      }
      if (context.state === "running") unlocking = null;
      else {
        // A touch-start resume may remain pending until a completed tap. Retry
        // inside each trusted gesture instead of letting that promise lock us out.
        const attempt = context.resume().catch(() => {});
        unlocking = attempt;
        void attempt.finally(() => {
          if (unlocking === attempt) unlocking = null;
        });
      }
      if (!primed) {
        // Starting a silent source in the gesture also primes mobile audio output.
        const source = context.createBufferSource();
        source.buffer = context.createBuffer(1, 1, context.sampleRate);
        source.connect(master!);
        source.onended = () => source.disconnect();
        source.start();
        primed = true;
      }
      if (context.state !== "running" && unlocking) await unlocking;
      return context.state === "running";
    } catch {
      /* Unavailable audio never interrupts a battle. */
    }
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
  async function sample(family: string, variant: number): Promise<AudioBuffer> {
    const name = `${family}-${variant}`;
    if (!buffers.has(name)) {
      const loading = (async () => {
        const response = await fetch(
          `${import.meta.env?.BASE_URL ?? "/"}assets/audio/${name}.wav?v=${AUDIO_REVISION}`,
        );
        if (!response.ok) throw Error("Sound unavailable");
        return await context!.decodeAudioData(await response.arrayBuffer());
      })().catch((error) => {
        buffers.delete(name);
        throw error;
      });
      buffers.set(name, loading);
    }
    return buffers.get(name)!;
  }
  async function playSound(family: string) {
    // Playback cannot construct or resume audio outside a user gesture.
    if (!enabled || !context || !master || !families.has(family)) return false;
    const currentRequest = ++request;
    stopVoices(); // Rapid clicks replace feedback instead of stacking it.
    try {
      if (context.state !== "running" && unlocking) await unlocking;
      if (context.state !== "running") return false;
      const buffer = await sample(family, variation(family));
      if (!enabled || currentRequest !== request || context.state !== "running") return false;
      const source = context.createBufferSource(),
        gain = context.createGain();
      source.buffer = buffer;
      gain.gain.value = family === "select" ? 0.25 : 1;
      source.connect(gain);
      gain.connect(master);
      const voice = { source, gain };
      voices.add(voice);
      source.onended = () => {
        voices.delete(voice);
        try {
          source.disconnect();
          gain.disconnect();
        } catch {}
      };
      source.start();
      return true;
    } catch {
      return false;
    }
  }

  return {
    soundEnabled,
    soundForInteraction,
    playInteraction,
    setSoundEnabled,
    unlockSound,
    playSound,
  };
}

export const {
  soundEnabled,
  soundForInteraction,
  playInteraction,
  setSoundEnabled,
  unlockSound,
  playSound,
} = createSound();
