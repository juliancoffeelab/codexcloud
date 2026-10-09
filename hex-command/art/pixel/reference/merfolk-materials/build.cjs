// Transfer approved generated material regions; preserve original bodies and Macer armour.
const fs = require('node:fs');
const path = require('node:path');
const root = __dirname;
const units = path.resolve(root, '../../units');
const materialPalette = JSON.parse(fs.readFileSync(path.join(root, 'palette.json')));
for (const [id, source] of [['reefguard','reefguard'],['merfolk-maceguard','macer']]) {
 const generated=fs.readFileSync(path.join(root,'extracted',source,'player.txt'),'utf8').trimEnd().split('\n');
 const palette=JSON.parse(fs.readFileSync(path.join(root,'baseline',id,'palette.json')));
 // Isolate generated browns so original leather armour on Macer stays unchanged.
 const remap={B:'J',b:'j',L:'L',l:'l',W:'U',w:'u',G:'Q',g:'q',K:'K'};
 for(const [from,to] of Object.entries(remap))palette[to]=materialPalette[from];
 for(const side of ['player','enemy']) {
 const rows=fs.readFileSync(path.join(root,'baseline',id,`${side}.txt`),'utf8').trimEnd().split('\n').map(row=>[...row]);
 for(let y=16;y<=35;y++)for(let x=0;x<=18;x++) {
 const c=generated[y][x];
 // Approved shield surface only; retain the native silhouette and cloak.
 if('BbWwIiGgCc'.includes(rows[y][x]) && remap[c])rows[y][x]=remap[c];
 }
 if(id==='reefguard')for(let y=16;y<=29;y++)for(let x=19;x<=31;x++) {
 const c=generated[y][x];
 if('BbCcAaGg'.includes(rows[y][x]) && remap[c])rows[y][x]=remap[c];
 }
 if(id==='merfolk-maceguard') {
  palette.Z=[78,95,105,255];palette.z=[43,57,67,255];palette.Y=[148,168,170,255];
  for(let y=0;y<10;y++)for(let x=33;x<45;x++) {
   const c=rows[y][x];if('AVaivI'.includes(c))rows[y][x]='avi'.includes(c)?'z':c==='I'?'Y':'Z';
  }
 }
 require('../merfolk-tier-designs/head-cleanup.cjs').apply(id,rows);
 fs.writeFileSync(path.join(units,id,`${side}.txt`),rows.map(row=>row.join('')).join('\n')+'\n');
 }
 fs.writeFileSync(path.join(units,id,'palette.json'),JSON.stringify(palette,null,2)+'\n');
}
