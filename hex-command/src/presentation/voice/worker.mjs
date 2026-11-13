import { synthesize, sampleRate } from "./synthesis.mjs";
import bankUrl from "/assets/audio/voice/animalese.wav?url";

let bank;
async function loadBank() {
  if (!bank) {
    bank = fetch(bankUrl).then(async (response) => {
      if (!response.ok) throw Error(`Voice bank: HTTP ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.length < 44 + 26 * Math.floor(0.15 * sampleRate))
        throw Error("Voice bank is incomplete");
      return bytes;
    });
  }
  return bank;
}

self.onmessage = async ({ data }) => {
  const { id, epoch, line } = data;
  try {
    const letters = await loadBank(),
      started = performance.now();
    const samples = synthesize(letters, line);
    self.postMessage({ id, epoch, samples, ms: performance.now() - started }, [samples.buffer]);
  } catch (error) {
    bank = null;
    self.postMessage({ id, epoch, error: error.stack || String(error) });
  }
};
