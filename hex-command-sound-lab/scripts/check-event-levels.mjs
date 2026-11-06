import assert from 'node:assert/strict';
import fs from 'node:fs';
import {eventLevelClass,playbackGainForRef} from '../dist/event-levels.js';
const read=name=>JSON.parse(fs.readFileSync(new URL(`../dist/${name}`,import.meta.url)));
const levels=read('playback-levels.json'), report=read('palette-levels.json'), loudness=read('loudness.json');
const measured=new Map(loudness.entries.map(x=>[x.id,x]));
const reference=levels.reference;
assert.equal(eventLevelClass({kind:'terrain',id:'cave'},{key:'tap'},reference),null);
assert.equal(eventLevelClass({kind:'terrain',id:'castle-bell'},{key:'tap'},reference),null);
assert.equal(eventLevelClass({kind:'terrain',id:'mountain'},{key:'tap'},reference),'terrain');
assert.equal(eventLevelClass({kind:'unit'},{key:'attack-bite'},reference),'attack');
assert.equal(eventLevelClass({kind:'unit'},{key:'tap'},reference),null);
assert.equal(eventLevelClass({kind:'unit'},{key:'move'},reference),null);
const volumeGain=20*Math.log10(reference.boardReferenceVolumePercent/100);
assert.equal(report.outOfRange.length,0);
for(const row of report.rows){
 const context=levels.eventRanges[row.context]?row.context:null;
 const profile=levels.entries[row.id];
 const gain=playbackGainForRef(profile,{take:row.take-1,levelClass:context});
 const raw=measured.get(row.id).takes[row.take-1];
 assert(Math.abs(raw.strongest50msRmsDbFS+gain+volumeGain-row.playbackStrongest50msRmsDbFS)<.02,row.name);
 if(context){
  const band=levels.eventRanges[context];
  assert(row.playbackStrongest50msRmsDbFS>=band.minDbFS-.03,row.name);
  assert(row.playbackStrongest50msRmsDbFS<=band.maxDbFS+.03,row.name);
 } else assert.equal(gain,profile.takes[row.take-1]);
}
const stone=levels.entries['rubberduck/80-cc0-rpg-sfx/stones_01.ogg'];
assert(stone.contexts.terrain.takes[0]<stone.contexts.attack.takes[0]);
const octopus=levels.entries['edits/octopus-gurgle-soft-v1'];
assert.equal(octopus.takes[0],0);
assert(octopus.contexts.attack.takes[0]>0);
console.log(`Event-level checks passed: ${report.rows.length} contextual choices in range; shared-source roles, octopus, selection/move and quarry/bell exceptions verified.`);
