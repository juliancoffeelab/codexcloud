import assert from 'node:assert/strict';
const stored = new Map();
globalThis.localStorage = { getItem: key => stored.get(key) ?? null, setItem: (key,value) => stored.set(key,value) };
let created = 0, resumed = 0, audio;
const fetched = [];
globalThis.fetch = async url => {
  fetched.push(String(url));
  return { ok:true, arrayBuffer:async () => String(url) };
};
class Param {
  value = 0;
  cancelScheduledValues() {}
  setValueAtTime(value) { this.value = value; }
  linearRampToValueAtTime(value) { this.value = value; }
}
class FakeContext {
  state = 'suspended'; currentTime = 10; destination = {}; sources = [];
  constructor() { created++; audio = this; }
  async resume() { resumed++; this.state = 'running'; }
  createGain() { return { gain:new Param(), connect(){}, disconnect(){} }; }
  async decodeAudioData(name) { return {name}; }
  createBufferSource() {
    const source = {connect(){},disconnect(){this.disconnected=true;},start(){this.started=true;},stop(){this.stopped=true;}};
    this.sources.push(source); return source;
  }
}
globalThis.AudioContext = FakeContext;
const sound = await import('../dist/sound.mjs');
assert.equal(sound.soundEnabled(),true);
await sound.playSound('turn');
assert.equal(created,0); assert.equal(fetched.length,0);
sound.setSoundEnabled(false);
await sound.unlockSound(); assert.equal(created,0);
sound.setSoundEnabled(true);
await sound.unlockSound(); assert.equal(created,1); assert.equal(resumed,1);
// Priority resolves a single family even when all three contexts exist.
assert.equal(sound.soundForInteraction({attackType:'archer',unitType:'soldier',terrain:'village'}),'attack-bow');
assert.equal(sound.soundForInteraction({attackType:'scout',unitType:'archer',terrain:'forest'}),'attack-melee');
for (const unitType of ['soldier','archer','scout','commander']) {
  assert.equal(sound.soundForInteraction({unitType,terrain:'hill'}),`unit-${unitType}`);
}
for (const terrain of ['village','water','forest','hill','plains','keep']) {
  assert.equal(sound.soundForInteraction({terrain}),`terrain-${terrain}`);
}
assert.equal(sound.soundForInteraction({terrain:'ford'}),'terrain-water');
assert.equal(sound.soundForInteraction({}),'select');
// A shuffle bag plays all three variations, without an immediate repeat at refill.
const variants = [];
for (let i=0;i<9;i++) {
  assert.equal(await sound.playSound('unit-soldier'),true);
  variants.push(audio.sources.at(-1).buffer.name.match(/-(\d)\.wav/)[1]);
}
for (let i=0;i<9;i+=3) assert.equal(new Set(variants.slice(i,i+3)).size,3);
assert.ok(variants.every((value,i)=>i===0 || value!==variants[i-1]));
assert.equal(fetched.length,3,'decoded samples are reused');
assert.ok(audio.sources.slice(0,-1).every(source=>source.stopped),'new cues replace earlier cues');
sound.setSoundEnabled(false);
assert.ok(audio.sources.every(source=>source.stopped));
const count = fetched.length;
await sound.playSound('attack-bow'); assert.equal(fetched.length,count);
assert.equal(stored.get('hex-command-sound-enabled'),'false');
sound.setSoundEnabled(true);
// Delayed lower-priority/sample responses cannot revive old audio.
const pending = [];
globalThis.fetch = url => new Promise(resolve => pending.push(() => resolve({ok:true,arrayBuffer:async()=>String(url)})));
const old = sound.playSound('terrain-forest');
const latest = sound.playInteraction({attackType:'archer',unitType:'soldier',terrain:'forest'});
assert.equal(pending.length,2);
pending[0](); assert.equal(await old,false);
pending[1](); assert.equal(await latest,true);
const muted = sound.playSound('terrain-water');
sound.setSoundEnabled(false); pending[2](); assert.equal(await muted,false);
sound.setSoundEnabled(true);
audio.state='suspended';
assert.equal(await sound.playSound('turn'),false);
assert.equal(resumed,1,'playback cannot unlock audio itself');
await sound.unlockSound(); assert.equal(resumed,2);
globalThis.fetch = async () => ({ok:false});
assert.equal(await sound.playSound('terrain-hill'),false,'failed audio never throws');
delete globalThis.AudioContext;
const unsupported = await import('../dist/sound.mjs?unsupported');
await unsupported.unlockSound(); assert.equal(await unsupported.playSound('turn'),false);
console.log('PASS: distinct cue priority, three variations, caching, latest-only playback, gestures, mute and unavailable audio');
