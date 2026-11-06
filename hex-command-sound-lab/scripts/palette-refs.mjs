import fs from 'node:fs';
import {fileURLToPath} from 'node:url';
import {buildActions} from '../dist/matching.js';
const root=fileURLToPath(new URL('../dist/',import.meta.url));
const catalog=JSON.parse(fs.readFileSync(root+'catalog.json'));
const workbench=JSON.parse(fs.readFileSync(root+'workbench.json'));
const refs=new Map();
for(const entity of [...workbench.factions.flatMap(f=>f.units),...workbench.terrains])
 for(const action of buildActions(entity,catalog)){
  if(action.key==='hit'||action.hypothetical)continue;
  for(const id of action.suggested){
   const take=action.preferredTakes?.[id]??0;
   const key=JSON.stringify([id,take]);
   if(!refs.has(key))refs.set(key,{id,take,uses:[]});
   refs.get(key).uses.push({entity:entity.key,action:action.key});
  }
 }
const list=[...refs.values()];
fs.writeFileSync(root+'palette-refs.json',JSON.stringify({scope:'Suggested cues including suggested recording variations; excludes hidden hurt and hypothetical actions. Personal saved choices may differ.',refs:list},null,2)+'\n');
console.log(`${list.length} distinct suggested cue/take references.`);
