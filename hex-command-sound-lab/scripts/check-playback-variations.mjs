import assert from 'node:assert/strict';
import {PlaybackVariations} from '../dist/playback-variations.js';
let seed = 1;
const random = () => ((seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) / 2**32);
const v = new PlaybackVariations(random);
const byId = new Map([['blade01', {takes:[{},{}]}],['blade02', {takes:[{},{}]}],['blade03', {takes:[{},{}]}]]);
const action = {key:'attack-blade',suggested:['blade01','blade02'],alternatives:['blade03']};
const ref = {id:'blade01',take:0,levelClass:'attack'};
let last;
for (let i=0;i<700;i++) {
  const plan = v.plan(ref,action,byId);
  assert(['blade01','blade02'].includes(plan.id));
  assert.equal(plan.take,0);
  assert(plan.variationRate >= 2**(-.8/12) && plan.variationRate <= 2**(.8/12));
  if(last){assert.notEqual(plan.id,last.id);assert.notEqual(plan.variationRate,last.variationRate);}
  const state=JSON.stringify([...v.bags]);
  for(let j=0;j<4;j++)v.candidates(ref,action,byId);
  assert.equal(JSON.stringify([...v.bags]),state,'Prefetch must not consume variation state');
  last=plan;
}
for(let i=0;i<20;i++)assert.equal(v.plan(ref,action,byId,true).id,'blade01','Saved source must stay locked');
const death={key:'death',suggested:['blade01','blade02']};
for(let i=0;i<20;i++)assert.equal(v.plan(ref,death,byId).id,'blade01','Death comparisons must never enter rotation');
const pinned={...action,preferredTakes:{blade01:1,blade02:1}};
for(let i=0;i<20;i++)assert.equal(v.plan({...ref,take:1},pinned,byId).take,1);
assert.equal(v.plan({...ref,take:null},action,byId).variationRate,undefined,'Full recording stays exact');
assert.deepEqual(ref,{id:'blade01',take:0,levelClass:'attack'});
console.log('Variation properties passed: bounded pitch, no immediate repeat across bags, approved sources only, pinned takes/saved picks, exact full sources and prefetch state isolation.');
