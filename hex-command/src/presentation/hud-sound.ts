import { writable } from "svelte/store";
import { errorDetails } from "../error-details";

export const hudAudioReadiness = writable<{ ready: boolean; error?: string }>({ ready: false });

/** Authorized HUD-only owner: one short menu cue, decoded before HUD publication. */
export function createHudAudio(enabled: () => boolean) {
  let context: AudioContext | undefined;
  let buffer: AudioBuffer | undefined;
  let source: AudioBufferSourceNode | undefined;
  let loading: Promise<void> | undefined;
  let disposed = false;
  let request = 0;
  const abort = new AbortController();
  const stop = () => {
    request++;
    try {
      source?.stop();
    } catch {}
    source?.disconnect();
    source = undefined;
  };
  const unlock = async () => {
    if (!enabled() || disposed || !context) return false;
    if (context.state === "closed") context = new AudioContext({ latencyHint: "interactive" });
    if (context.state !== "running") await context.resume();
    return !disposed && context.state === "running";
  };
  const activate = (event: Event) => {
    if (!event.isTrusted) return;
    void unlock().catch((cause) => {
      if (!disposed)
        hudAudioReadiness.set({
          ready: !!buffer,
          error: errorDetails(new Error("Cannot unlock HUD audio", { cause })),
        });
    });
  };
  const gestures = ["pointerdown", "pointerup", "touchend", "keydown", "click"];
  for (const type of gestures)
    document.addEventListener(type, activate, { capture: true, passive: true });
  const prepare = () => {
    if (loading) return loading;
    const attempt = (async () => {
      hudAudioReadiness.set({ ready: false });
      try {
        context ??= new AudioContext({ latencyHint: "interactive" });
        const owner = context;
        const url = `${import.meta.env?.BASE_URL ?? "/"}assets/audio/select-1.wav?v=3a6cff557cea`;
        let prepared: AudioBuffer;
        try {
          const response = await fetch(url, {
            signal: AbortSignal.any([abort.signal, AbortSignal.timeout(30000)]),
          });
          if (!response.ok) throw new Error(`HTTP ${response.status} ${response.statusText}`);
          prepared = await owner.decodeAudioData(await response.arrayBuffer());
        } catch (cause) {
          throw new Error(`Cannot prepare HUD cue ${url}`, { cause });
        }
        if (disposed) return;
        buffer = prepared;
        hudAudioReadiness.set({ ready: true });
      } catch (cause) {
        if (!disposed) hudAudioReadiness.set({ ready: false, error: errorDetails(cause) });
      }
    })();
    loading = attempt;
    void attempt.finally(() => {
      if (loading === attempt) loading = undefined;
    });
    return attempt;
  };
  return {
    prepare,
    unlock,
    stop,
    async play() {
      if (!enabled() || disposed || !context || !buffer) return false;
      stop();
      const current = request;
      try {
        if (!(await unlock()) || disposed || current !== request || !enabled() || !context)
          return false;
      } catch (cause) {
        if (!disposed)
          hudAudioReadiness.set({
            ready: true,
            error: errorDetails(new Error("Cannot unlock HUD audio for playback", { cause })),
          });
        return false;
      }
      const voice = context.createBufferSource();
      const gain = context.createGain();
      gain.gain.value = 0.29 * 0.25;
      voice.buffer = buffer;
      voice.connect(gain);
      gain.connect(context.destination);
      voice.onended = () => {
        voice.disconnect();
        gain.disconnect();
        if (source === voice) source = undefined;
      };
      source = voice;
      voice.start();
      return true;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      abort.abort();
      stop();
      buffer = undefined;
      for (const type of gestures) document.removeEventListener(type, activate, true);
      void context?.close().catch(() => {});
      context = undefined;
    },
  };
}
