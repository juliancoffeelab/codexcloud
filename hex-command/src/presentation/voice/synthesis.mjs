// Animalese algorithm adapted from Josh Simmons (2014), MIT; see LICENSE.md.
// The recorded letter bank is CC BY 4.0. Character processing matches the audition.
import settings from "./profiles.json";

export const sampleRate = 44100;
const letterSamples = Math.floor(0.075 * sampleRate);
export const duration = (text) => (text.length * letterSamples) / sampleRate / 3;

function hash(text) {
  let value = 2166136261;
  for (let i = 0; i < text.length; i++) value = Math.imul(value ^ text.charCodeAt(i), 16777619);
  return (value >>> 0).toString(16);
}

export function profileFor(line) {
  const id = line.character === "villagers" ? "villager" : line.character;
  return (
    settings.profiles[id] ||
    settings.profiles[settings.overrides[hash(line.speaker + "\0" + line.text)]] ||
    settings.profiles.narrator
  );
}

// Two-pole Butterworth filters, as in ffmpeg's audition highpass/lowpass stages.
function filter(samples, frequency, highpass) {
  const w = (2 * Math.PI * frequency) / sampleRate,
    c = Math.cos(w);
  const alpha = Math.sin(w) / Math.SQRT2,
    a0 = 1 + alpha;
  const b0 = (highpass ? 1 + c : 1 - c) / 2 / a0;
  const b1 = (highpass ? -(1 + c) : 1 - c) / a0;
  const a1 = (-2 * c) / a0,
    a2 = (1 - alpha) / a0;
  let x1 = 0,
    x2 = 0,
    y1 = 0,
    y2 = 0;
  for (let i = 0; i < samples.length; i++) {
    const x = samples[i],
      y = b0 * x + b1 * x1 + b0 * x2 - a1 * y1 - a2 * y2;
    samples[i] = y;
    x2 = x1;
    x1 = x;
    y2 = y1;
    y1 = y;
  }
}

export function synthesize(bank, line) {
  const p = profileFor(line),
    text = line.text;
  // Only generate the part heard before the approved 3x text reveal finishes.
  const samples = new Float32Array(Math.ceil(duration(text) * sampleRate));
  const librarySamples = Math.floor(0.15 * sampleRate);
  for (let i = 0; i < samples.length; i++) {
    const c = text[Math.floor(i / letterSamples)].toUpperCase().charCodeAt(0);
    let value =
      c >= 65 && c <= 90
        ? ((bank[44 + (c - 65) * librarySamples + Math.floor((i % letterSamples) * p.pitch)] ??
            128) -
            128) /
          128
        : -1 / 128;
    value *= Math.min(1, i / (sampleRate * 0.006));
    if (p.ring) value *= 1 - p.wet + p.wet * Math.sin((2 * Math.PI * p.ring * i) / sampleRate);
    if (p.tremolo)
      value *=
        1 - p.depth / 2 + (p.depth / 2) * Math.sin((2 * Math.PI * p.tremolo * i) / sampleRate);
    samples[i] = value;
  }
  filter(samples, p.highpass || 60, true);
  if (p.lowpass) filter(samples, p.lowpass, false);
  for (let i = 0; i < samples.length; i++) {
    samples[i] *=
      p.gain * Math.max(0, Math.min(1, (samples.length - 1 - i) / (sampleRate * 0.025)));
  }
  return samples;
}
