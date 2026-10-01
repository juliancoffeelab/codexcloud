import assert from 'node:assert/strict';

const stored = new Map();
globalThis.localStorage = {
  getItem: key => stored.get(key) ?? null,
  setItem: (key, value) => stored.set(key, value),
};
let created = 0;
let resumed = 0;
let audio;
class Param {
  value = 0;
  events = [];
  cancelScheduledValues(time) { this.events.push(['cancel', time]); }
  setValueAtTime(value, time) { this.events.push(['set', value, time]); this.value = value; }
  linearRampToValueAtTime(value, time) { this.events.push(['linear', value, time]); this.value = value; }
  exponentialRampToValueAtTime(value, time) { this.events.push(['exponential', value, time]); this.value = value; }
}
class Node {
  disconnected = false;
  connect() {}
  disconnect() { this.disconnected = true; }
}
class FakeContext {
  state = 'suspended';
  currentTime = 10;
  destination = {};
  oscillators = [];
  gains = [];
  constructor() { created++; audio = this; }
  async resume() { resumed++; this.state = 'running'; }
  createGain() {
    const node = new Node(); node.gain = new Param(); this.gains.push(node); return node;
  }
  createOscillator() {
    const node = new Node(); node.frequency = new Param();
    node.start = time => { node.startTime = time; };
    node.stop = time => { node.stopTime = time; };
    this.oscillators.push(node); return node;
  }
}
globalThis.AudioContext = FakeContext;
const sound = await import('../dist/sound.mjs');
assert.equal(sound.soundEnabled(), true);
sound.playSound('turn');
assert.equal(created, 0, 'restores and programmatic feedback must not construct audio');
sound.setSoundEnabled(false);
await sound.unlockSound();
assert.equal(created, 0, 'muted gestures must not construct audio');
assert.equal(stored.get('hex-command-sound-enabled'), 'false');
sound.setSoundEnabled(true);
await sound.unlockSound();
assert.equal(created, 1);
assert.equal(resumed, 1);
sound.playSound('turn');
assert.equal(audio.oscillators.length, 2);
assert.ok(audio.oscillators.every(osc => osc.stopTime > osc.startTime));
sound.playSound('victory');
sound.playSound('recruit');
assert.equal(audio.oscillators.length, 5, 'polyphony is capped');
sound.setSoundEnabled(false);
assert.ok(audio.oscillators.every(osc => osc.stopTime <= audio.currentTime + 0.015));
assert.equal(audio.gains[0].gain.events.at(-1)[1], 0, 'muting fades the master immediately');
for (const osc of audio.oscillators) osc.onended();
assert.ok(audio.oscillators.every(osc => osc.disconnected));
assert.ok(audio.gains.slice(1).every(gain => gain.disconnected));
sound.setSoundEnabled(true);
sound.playSound('attack');
assert.equal(audio.oscillators.length, 6, 'ended voices release the polyphony budget');
audio.state = 'suspended';
sound.playSound('select');
assert.equal(resumed, 1, 'effects cannot resume audio without a gesture');
await sound.unlockSound();
assert.equal(resumed, 2);
sound.playSound('select');
sound.playSound('select');
assert.equal(audio.oscillators.length, 7, 'rapid selection feedback is throttled');

globalThis.localStorage = { getItem() { throw Error('blocked'); }, setItem() { throw Error('blocked'); } };
delete globalThis.AudioContext;
const unavailable = await import('../dist/sound.mjs?unsupported');
assert.equal(unavailable.soundEnabled(), true);
await unavailable.unlockSound();
unavailable.playSound('move');
unavailable.setSoundEnabled(false);
assert.equal(unavailable.soundEnabled(), false);
globalThis.AudioContext = class { constructor() { throw Error('audio unavailable'); } };
const failing = await import('../dist/sound.mjs?failing');
await failing.unlockSound();
failing.playSound('attack');
console.log('Sound lifecycle checks passed.');
