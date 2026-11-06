import fs from 'node:fs';
import {buildActions} from '../dist/matching.js';
import {beastDeathId} from '../dist/death-picks.js';
const read=name=>JSON.parse(fs.readFileSync(new URL(`../dist/${name}`,import.meta.url)));
const catalog=read('catalog.json'),workbench=read('workbench.json'),plans=new Map();
for(const unit of workbench.factions.flatMap(f=>f.units)){
 const rows=buildActions(unit,catalog),death=rows.find(a=>a.key==='death');
 if(death?.deathGroup!=='beast')continue;
 for(const ref of death.selectionDeaths){
  const id=beastDeathId(ref.id,ref.take);
  if(!plans.has(id))plans.set(id,{id,sourceId:ref.id,take:ref.take,units:[]});
  plans.get(id).units.push(unit.key);
 }
}
process.stdout.write(JSON.stringify([...plans.values()]));
