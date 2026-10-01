// Small, gesture-unlocked effects. No audio is started by loading a battle.
const STORAGE_KEY = 'hex-command-sound-enabled';
let enabled = true;
try { enabled = localStorage.getItem(STORAGE_KEY) !== 'false'; } catch {}

let context = null;
let master = null;
let unlocked = false;
let lastSelection = -Infinity;
const voices = new Set();

export function soundEnabled() { return enabled; }

function removeVoice(voice) {
  voices.delete(voice);
  try { voice.oscillator.disconnect(); } catch {}
  try { voice.gain.disconnect(); } catch {}
}

export function setSoundEnabled(value) {
  enabled = Boolean(value);
  try { localStorage.setItem(STORAGE_KEY, String(enabled)); } catch {}
  if (!context || !master) return;
  try {
    const now = context.currentTime;
    master.gain.cancelScheduledValues(now);
    master.gain.setValueAtTime(master.gain.value, now);
    master.gain.linearRampToValueAtTime(enabled ? 0.6 : 0, now + 0.012);
    if (!enabled) {
      for (const voice of voices) {
        try { voice.oscillator.stop(now + 0.015); } catch {}
      }
    }
  } catch { /* Audio failures must never interrupt a battle. */ }
}

// Call only from a click/key gesture. Effects never try to unlock audio themselves.
export async function unlockSound() {
  if (!enabled) return;
  try {
    if (!context) {
      const AudioContextClass = globalThis.AudioContext || globalThis.webkitAudioContext;
      if (!AudioContextClass) return;
      context = new AudioContextClass();
      master = context.createGain();
      master.gain.value = 0.6;
      master.connect(context.destination);
    }
    if (context.state === 'suspended') await context.resume();
    unlocked = context.state === 'running';
  } catch { unlocked = false; }
}

// [frequency, offset, duration, gain, ending frequency, oscillator shape]
const effects = {
  select: [[440, 0, 0.075, 0.025, 410, 'sine']],
  move: [[220, 0, 0.13, 0.04, 175, 'triangle']],
  attack: [[105, 0, 0.14, 0.055, 58, 'sine']],
  recruit: [[330, 0, 0.15, 0.035, 330, 'triangle'], [440, 0.08, 0.17, 0.03, 440, 'sine']],
  turn: [[294, 0, 0.16, 0.03, 294, 'sine'], [392, 0.095, 0.18, 0.03, 392, 'sine']],
  victory: [[330, 0, 0.2, 0.035, 330, 'triangle'], [440, 0.1, 0.22, 0.03, 440, 'sine'], [523, 0.2, 0.24, 0.03, 523, 'sine']],
  defeat: [[220, 0, 0.2, 0.03, 220, 'sine'], [165, 0.12, 0.24, 0.03, 150, 'sine']],
};

export function playSound(kind) {
  if (!enabled || !unlocked || !context || !master || context.state !== 'running') return;
  const notes = effects[kind];
  if (!notes) return;
  const now = context.currentTime;
  if (kind === 'select' && now - lastSelection < 0.06) return;
  if (kind === 'select') lastSelection = now;
  // Drop rapid excess feedback instead of accumulating a loud queue.
  if (voices.size + notes.length > 6) return;
  for (const [frequency, offset, duration, volume, ending, shape] of notes) {
    let voice;
    try {
      const oscillator = context.createOscillator();
      const gain = context.createGain();
      voice = { oscillator, gain };
      voices.add(voice);
      const start = now + offset;
      oscillator.type = shape;
      oscillator.frequency.setValueAtTime(frequency, start);
      oscillator.frequency.exponentialRampToValueAtTime(ending, start + duration);
      gain.gain.setValueAtTime(0, start);
      gain.gain.linearRampToValueAtTime(volume, start + 0.008);
      gain.gain.exponentialRampToValueAtTime(0.0001, start + duration);
      oscillator.connect(gain);
      gain.connect(master);
      oscillator.onended = () => removeVoice(voice);
      oscillator.start(start);
      oscillator.stop(start + duration + 0.01);
    } catch {
      if (voice) {
        try { voice.oscillator.stop(); } catch {}
        removeVoice(voice);
      }
    }
  }
}
