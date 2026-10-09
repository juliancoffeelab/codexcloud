// Reviewed isolated non-skin pixels in the regenerated heads. Cloth is untouched.
const fs=require('../../../../scripts/pixel-fs.cjs');
const path=require('node:path');
const corrections={
 'merfolk-reef-champion':[[18,6,'s']],
 'merfolk-maceguard':[[19,6,'h']],
 'merfolk-maceguard-l2':[[27,11,'s']],
 'merfolk-maceguard-l3':[[27,11,'s']],
 'tidesage':[[27,11,'s']],
};
function apply(id,rows){for(const [x,y,to] of corrections[id]||[])rows[y][x]=to;}
if(require.main===module){
 const units=path.resolve(__dirname,'../../units');
 for(const id of Object.keys(corrections))for(const side of ['player','enemy']){
  const file=path.join(units,id,`${side}.txt`);
  const rows=fs.readFileSync(file,'utf8').trimEnd().split('\n').map(row=>[...row]);
  apply(id,rows);fs.writeFileSync(file,rows.map(row=>row.join('')).join('\n')+'\n');
 }
}
module.exports={apply};
