import assert from 'node:assert/strict';
import {PlaybackVariations} from '../dist/playback-variations.js';
let seed = 1;
const random = () => ((seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) / 2**32);
const v = new PlaybackVariations(random);
for (const [key,spread,levelClass] of [['attack-spell',.8,'attack'],['death',1.2,null],['move',.35,null],['tap',.6,'terrain']]) {
  const action = {key,suggested:['spell1','spell2'],alternatives:['spell3']};
  const ref = {id:'spell1',take:3,levelClass};
  let last;
  for(let i=0;i<700;i++) {
    const plan=v.plan(ref,action);
    assert.equal(plan.id,ref.id,'Never substitute a different numbered recording');
    assert.equal(plan.take,3,'Never substitute another take');
    assert.equal(plan.levelClass,levelClass);
    assert(plan.variationRate >= 2**(-spread/12) && plan.variationRate <= 2**(spread/12));
    if(last)assert.notEqual(plan.variationRate,last.variationRate,'No immediate pitch repeat across bag boundaries');
    last=plan;
  }
  assert.deepEqual(ref,{id:'spell1',take:3,levelClass},'Never mutate the chosen reference');
  assert.equal(v.plan({...ref,take:null},action).variationRate,undefined,'Full recording stays exact');
}
console.log('Alterations passed: exact source/take identity preserved for all actions, bounded pitch, no immediate repeat across bags, immutable choices and exact full recordings.');
