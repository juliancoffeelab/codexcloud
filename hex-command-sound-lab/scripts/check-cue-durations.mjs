// Audit the actual default take, rather than the length of its source session.
import fs from 'node:fs';
import {fileURLToPath} from 'node:url';
import {buildActions} from '../dist/matching.js';
const root=fileURLToPath(new URL('../',import.meta.url));
const catalog=JSON.parse(fs.readFileSync(root+'dist/catalog.json'));
const workbench=JSON.parse(fs.readFileSync(root+'dist/workbench.json'));
const byId=new Map(catalog.map(x=>[x.id,x]));
const groups=new Map();
for(const entity of [...workbench.factions.flatMap(f=>f.units),...workbench.terrains]){
 for(const action of buildActions(entity,catalog)){
  if(action.key==='hit'||(action.hypothetical&&action.key!=='death'))continue;
  const id=action.suggested[0],item=byId.get(id);if(!item)continue;
  const take=action.preferredTakes?.[id]??0;
  const range=item.takes?.[take]??{start:0,duration:item.duration};
  if(range.duration<=.5)continue;
  const key=JSON.stringify([id,take]);
  if(!groups.has(key))groups.set(key,{id,name:item.name,take:take+1,...range,uses:[]});
  groups.get(key).uses.push({entity:entity.key,action:action.key});
 }
}
const cues=[...groups.values()].sort((a,b)=>b.duration-a.duration);
const report={thresholdSeconds:.5,scope:'Default suggested takes; includes shared death auditions; excludes deferred hurt and other hypothetical actions. Saved user choices and alternative takes may differ.',cues};
fs.writeFileSync(root+'dist/cue-duration-review.json',JSON.stringify(report,null,2)+'\n');
console.log(cues.map(x=>`${x.duration.toFixed(3)}s ${x.name} (take ${x.take}; ${x.uses.length} uses)`).join('\n'));
console.log(`${cues.length} unique default cues exceed 0.5 seconds.`);
