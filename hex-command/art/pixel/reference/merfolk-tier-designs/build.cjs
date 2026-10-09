const fs=require('node:fs');
const path=require('node:path');
const {execFileSync}=require('node:child_process');
const root=__dirname,project=path.resolve(root,'../../../..');
const units=path.join(project,'art/pixel/units');
const designs=require('./designs.json');
const catalogPath=path.join(units,'catalog.json');
const catalog=JSON.parse(fs.readFileSync(catalogPath));
const selected = new Set(process.argv.slice(2));
for(const design of designs.filter(design => !selected.size || selected.has(design.family))){
 const {id,family,level}=design;
 const revised=family==='support'||family==='crushing';
 const light=family==='hunter'||family==='netter';
 const extracted=light && level===2 ? path.join(root,'revisions',`${family}-promoted`) : path.join(root,'extracted',`${family}${light?'-fancier':revised?'-revised':''}-l${level}`);
 const rows=fs.readFileSync(path.join(extracted,'player.txt'),'utf8').trimEnd().split('\n').map(row=>[...row]);
 const palette=JSON.parse(fs.readFileSync(path.join(root,'palette.json')));
 const baseline=fs.readFileSync(path.join(root,'baseline',id,'player.txt'),'utf8').trimEnd().split('\n');
 const offset=(rows.length-48)/2;
 const skinSymbols=['macer','trident','netter'].includes(family)?'Hh':'SsH';
 const baselineOffset=(baseline.length-48)/2;
 for(let y=0;y<48;y++)for(let x=0;x<48;x++){
  const c=rows[y+offset][x+offset];
  if(!(light && level===2) && skinSymbols.includes(baseline[y+baselineOffset][x+baselineOffset]) && 'NVAacv'.includes(c))rows[y+offset][x+offset]='NAc'.includes(c)?'H':'s';
 }
 // Keep the reviewed trident geometry; new coral armour has its own symbols.
 if(family==='reefguard'||family==='trident')for(let y=0;y<10;y++)for(let x=30;x<44;x++)rows[y][x]=baseline[y][x];
 // L2 two-handed armour has one lower coral band; L3 retains extended articulated coverage.
 if(family==='trident' && level===2)for(let y=33;y<=36;y++)for(let x=20;x<=30;x++) {
  const c=rows[y][x];if('JjDdKCIic'.includes(c))rows[y][x]='jdic'.includes(c)?'b':'B';
 }
 // Promote the previous L2 body unchanged; only transfer the new smaller head.
 if(revised && level===3) {
  const previous=fs.readFileSync(path.join(root,'revisions',family,'player.txt'),'utf8').trimEnd().split('\n');
  for(let y=0;y<rows.length;y++)for(let x=0;x<rows[y].length;x++) {
   if(!(x>=38 && x<=56 && y<=20))rows[y][x]=previous[y][x];
  }
 }
 // Restore the clean head ramp; the promoted draft had shadow/leather specks on flesh.
 if(family==='netter' && level===2) {
  const head=fs.readFileSync(path.join(root,'revisions/netter-head/player.txt'),'utf8').trimEnd().split('\n');
  for(let y=10;y<=20;y++)for(let x=24;x<=38;x++)rows[y][x]=head[y][x]==='b'?'s':head[y][x];
 }
 // All commander tiers share the reviewed L1 face; crowns retain tier details.
 if(family==='commander' && level>1) {
  const head=fs.readFileSync(path.join(root,'revisions/commander-head/player.txt'),'utf8').trimEnd().split('\n');
  for(let y=14;y<=20;y++)for(let x=24;x<=39;x++)rows[y][x]=head[y][x];
  // Copy the entire L1 crown, then lengthen its three existing spikes.
  for(let y=0;y<=13;y++)for(let x=24;x<=39;x++)rows[y][x]=head[y][x];
  const taller=level===2?1:2;
  for(let y=0;y<=11;y++)for(let x=26;x<=35;x++)rows[y][x]=head[Math.min(y+taller,11)][x];
 }
 require('./head-cleanup.cjs').apply(id,rows);
 const out=path.join(units,id);fs.mkdirSync(out,{recursive:true});
 const player=rows.map(row=>row.join('')).join('\n')+'\n';
 fs.writeFileSync(path.join(out,'player.txt'),player);
 fs.writeFileSync(path.join(out,'enemy.txt'),player.replaceAll('T','R').replaceAll('t','r'));
 fs.writeFileSync(path.join(out,'palette.json'),JSON.stringify(palette,null,2)+'\n');
 fs.writeFileSync(path.join(out,'components.md'),`# ${design.name} — L${level}\n\n${family==='commander'?'Aquatic commander: restrained coral-and-shell circlet, fitted leather/coral cuirass, connected shoulder guards, shell/wood shield and bone trident.':family==='hunter'||family==='netter'?'Light aquatic hunter: leather vest and straps, exposed upper arms; L3 adds a small attached shell shoulder cap and forearm brace.':family==='support'?'Support mage: fitted shell collar, flowing faction cloak, connected shell-and-pearl staff.':family==='crushing'?'Crushing mage: coral shoulder mantle, leather underlayer, shell clasp and river-stone pressure staff.':'Aquatic infantry: coral cuirass, attached shoulder guards, articulated abdominal protection and leather underlayer.'}\n\n${level===1?'L1 uses leather and restrained shell trim.':level===2?'L2 uses restrained coverage and smaller shoulder/staff shapes.':'L3 adds broader shell-faced shoulders or collar and more complete lower torso protection.'} ${family==='macer'?'Mace head is faceted river stone, not steel.':''}\n\nSource: reference/merfolk-tier-designs/sources/${family}${family==='commander'?`-l${level}`:light?'-fancier':revised?'-revised':''}.png, lower-${level===2?'left':'right'} slot. Native ${rows.length}×${rows.length} matrix; extra canvas is padding for staff/stone head, not a scale change. Rebuild with reference/merfolk-tier-designs/build.cjs. Faction colour occurs on cloth only. Gameplay balance remains independent of this artwork lineup.\n`);
 catalog[id]={...catalog[id],name:design.name,level,study:`merfolk-tier-designs/${family}-l${level}`,canvas:[rows.length,rows.length]};
 execFileSync('python3',['scripts/render-pixel.py',path.join(out,'player.txt'),path.join(out,'enemy.txt')],{cwd:project,stdio:'inherit'});
}
fs.writeFileSync(catalogPath,JSON.stringify(catalog,null,2)+'\n');
