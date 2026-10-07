const fs=require('node:fs');
const path=require('node:path');
const sourceDirectory=require('../../../../scripts/pixel-source-path.cjs');
const root=__dirname;
const project=path.resolve(root,'../../../..');
const variants=[];
function create(id,label,base,level,edit){
 const source=path.join(root,'base-'+base);
 const original=fs.readFileSync(path.join(source,'player.txt'),'utf8').trimEnd().split('\n').map(r=>[...r]);
 const rows=original.map(r=>[...r]);
 const originalPalette=JSON.parse(fs.readFileSync(path.join(source,'palette.json')));
 const palette=structuredClone(originalPalette);
 palette.L=[150,103,59,255];palette.l=[92,58,33,255];
 const width=rows[0].length,height=rows.length;
 const set=(x,y,s)=>{if(x<0||x>=width||y<0||y>=height)throw new Error('Weapon outside canvas');rows[y][x]=s;};
 const region=(test,map)=>{for(let y=0;y<height;y++)for(let x=0;x<width;x++)if(test(x,y)&&map[rows[y][x]])set(x,y,map[rows[y][x]]);};
 const bright=()=>Object.assign(palette,{A:[239,245,244,255],a:[161,182,191,255],k:[72,89,103,255],G:[255,205,77,255],g:[180,119,27,255],T:[2,112,118,255],t:[1,55,63,255]});
 edit({rows,palette,set,region,bright});
 if(level===3&&base!=='mounted-bow'){
  // Leather and wood share some source tones; separate the weapon first.
  if(base==='staff-infantry')region((x,y)=>Math.abs(x-(45.3-0.833*y))<=(y<=15?4:2.5),{'B':'H','b':'h'});
  if(base==='crossbow')region((x,y)=>x>=21&&y>=19&&y<=28,{'B':'H','b':'h'});
  if(base==='mounted-lance'){
   palette.W=originalPalette.H;palette.w=originalPalette.h;
   region((x,y)=>x>=34&&y<=25&&Math.abs(x-(53-0.77*y))<=3,{'B':'W','b':'w','H':'W','h':'w'});
  }
  const leather=base==='mounted-lance'
   ?(x,y)=>x>=20&&x<=37&&y>=18&&y<40&&!(y<=19&&x>=24&&x<=30)
   :(x,y)=>y<41&&!(base==='crossbow'&&x>=15&&x<=24&&y>=14&&y<=19);
  region(leather,{'B':'G','b':'g'});
 }
 const dir=sourceDirectory('human-anatomies/'+id);fs.mkdirSync(dir,{recursive:true});
 fs.writeFileSync(path.join(dir,'player.txt'),rows.map(r=>r.join('')).join('\n')+'\n');
 fs.writeFileSync(path.join(dir,'palette.json'),JSON.stringify(palette,null,2)+'\n');
 const edits=[];
 for(let y=0;y<height;y++)for(let x=0;x<width;x++)if(rows[y][x]!==original[y][x])edits.push({x,y,from:original[y][x],to:rows[y][x]});
 fs.writeFileSync(path.join(dir,'changes.json'),JSON.stringify({base,level,coordinateSpace:[width,height],edits,paletteChanges:Object.fromEntries(Object.entries(palette).filter(([s,c])=>JSON.stringify(c)!==JSON.stringify(originalPalette[s])))},null,2)+'\n');
 variants.push({id,label,base,level,image:path.relative(project,path.join(dir,'player.png'))});
}
function silverArmor({region},base){
 const metal={'B':'A','b':'a','C':'A','c':'a'};
 if(base==='crossbow'){
  region((x,y)=>x>=14&&x<=26&&y>=25&&y<=30,metal);
  region((x,y)=>x>=9&&x<=17&&y>=23&&y<=27,metal);
  region((x,y)=>x>=26&&x<=29&&y>=25&&y<=29,metal);
  region((x,y)=>y>=36&&y<=41&&((x>=10&&x<=17)||(x>=25&&x<=30)),metal);
 }else if(base==='staff-infantry'){
  const shaft=(x,y)=>Math.abs(x-(45.3-0.833*y))<=3;
  region((x,y)=>!shaft(x,y)&&x>=20&&x<=29&&y>=18&&y<=27,metal);
  region((x,y)=>!shaft(x,y)&&x>=13&&x<=19&&y>=19&&y<=25,metal);
  region((x,y)=>!shaft(x,y)&&x>=29&&x<=36&&y>=20&&y<=25,metal);
  region((x,y)=>!shaft(x,y)&&y>=34&&y<=40&&((x>=12&&x<=19)||(x>=28&&x<=34)),metal);
 }else{
  region((x,y)=>x>=22&&x<=32&&y>=20&&y<=28,metal);
  region((x,y)=>x>=27&&x<=33&&y>=29&&y<=35,metal);
  region((x,y)=>x>=29&&x<=35&&y>=35&&y<=39,metal);
 }
}
function footWeapon({set},kind,level){
 // Weapon edits stay outside both grips. The shaft retains its upright pose.
 if(kind==='hammer'){
  for(let y=0;y<=6;y++)for(let x=36;x<48;x++)set(x,y,'.');
  // Short upper-left poll, broad lower-right striking face, across the shaft.
  for(let y=0;y<=11;y++)for(let x=36;x<48;x++){
   const along=(x+y-46)/Math.SQRT2,across=(x-y-36)/Math.SQRT2;
   const halfWidth=along<-1.4?1.6:along>1.4?3:2.3;
   if(Math.abs(along)>4.5||Math.abs(across)>halfWidth)continue;
   set(x,y,Math.abs(along)>3.7||Math.abs(across)>halfWidth-0.7?'k':across>0?'A':'a');
  }
 }else{
  for(let y=0;y<=6;y++)for(let x=36;x<48;x++)set(x,y,'.');
  const head=[
   [45,0,'k'],[44,1,'Aa'],[43,2,'AAk'],[42,3,'aAk'],[41,4,'aAk'],[40,5,'aGk'],
   [37,4,'kAak'],[38,5,'kAaG'],
   [41,6,'AAAk'],[41,7,'aAAAk'],[42,8,'aAAAk'],[43,9,'aAAAk'],
   [43,10,'kaAAk'],[44,11,'kaAk'],[45,12,'kak']
  ];
  for(const [x,y,line]of head)for(let n=0;n<line.length;n++)set(x+n,y,line[n]);
 }
 if(level===3){if(kind==='hammer'){set(40,8,'G');}else{set(40,6,'G');set(41,6,'g');}}
}
function infantryArmor(api,level,kind){
 const {region,set,bright}=api;
 if(level===3){
  bright();
  silverArmor(api,'staff-infantry');
  region((x,y)=>y>=15&&y<=40,{'C':'A','c':'a'});
  region((x,y)=>y>=5&&y<=11&&x>=19&&x<=30,{'B':'A','b':'k'});
  for(let y=15;y<=18;y++)set(19,y,'G');
  if(kind==='hammer'){
   for(let y=12;y<=14;y++)for(let x=23;x<=26;x++)if(api.rows[y][x]==='S')set(x,y,y===12?'O':'a');
  }else{
   set(24,3,'T');set(25,3,'t');set(23,4,'T');set(24,4,'T');set(25,4,'t');
  }
 }
}
// Crossbow branch: steel helmet already belongs to the cleaned level-two pose.
create('crossbowman-l2','Crossbowman — L2','crossbow',2,()=>{});
create('arbalester-l3','Arbalester — L3','crossbow',3,api=>{
 const {region,set,bright,palette}=api;
 bright();palette.H=[169,112,60,255];palette.h=[92,58,30,255];
 silverArmor(api,'crossbow');
 region((x,y)=>y===21&&x<=13,{'C':'A','c':'a'});
 set(22,12,'G');set(23,12,'G');set(24,12,'g');
 set(12,22,'A');set(12,23,'a');
});
for(const [kind,ids,labels]of [
 ['hammer',['hammerman-l2','warhammer-l3'],['Hammerman — L2','Warhammer — L3']],
 ['halberd',['halberdier-l2','halberdmaster-l3'],['Halberdier — L2','Halberd master — L3']]
])for(let n=0;n<2;n++)create(ids[n],labels[n],'staff-infantry',n+2,api=>{
 infantryArmor(api,n+2,kind);footWeapon(api,kind,n+2);
});
function lanceHead({set},level){
 const head=level===1?[[51,0,'k'],[50,1,'Ak'],[49,2,'AAk'],[48,3,'aAAk'],[47,4,'kAak'],[47,5,'gk']]:
 [[51,0,'k'],[50,1,'Ak'],[49,2,'AAk'],[48,3,'aAAk'],[47,4,'aAAak'],[46,5,'kaAAk'],[46,6,'kaaGk'],[46,7,'gG']];
 for(const [x,y,line]of head)for(let n=0;n<line.length;n++)set(x+n,y,line[n]);
 if(level===3){set(45,8,'G');set(44,9,'G');set(45,9,'T');set(46,9,'T');set(47,9,'t');set(44,10,'t');set(45,10,'T');set(46,10,'t');}
}
for(let level=1;level<=3;level++)create(`lancer-l${level}`,['','Rider — L1','Lancer — L2','Knight — L3'][level],'mounted-lance',level,api=>{
 const {region,palette,bright,set}=api;
 region((x,y)=>x>=27&&x<=37&&y>=30&&y<=43,{'H':'B','h':'b'});
 if(level===1){
  region((x,y)=>x>=20&&x<=34&&y>=20&&y<=37,{'C':'L','c':'l'});
  region((x,y)=>y<30,{'G':'B','g':'b'});
 }else if(level===3){
  bright();palette.H=[211,214,205,255];palette.h=[134,149,148,255];
  silverArmor(api,'mounted-lance');
  region((x,y)=>x>=21&&x<=35&&y>=21&&y<=28,{'C':'A','c':'a'});
  region((x,y)=>x>=25&&x<=33&&y>=10&&y<=14,{'B':'A','b':'k','h':'k'});
  set(25,21,'G');set(25,22,'G');set(25,23,'g');
  // Rounded pauldrons sit over the upper arms, below the neck and above the grip.
  for(const [x,y,line]of [
   [23,18,'gAAag'],[22,19,'gAAAAag'],[22,20,'gAAAaag'],[23,21,'gaaaag'],
   [31,20,'gAag'],[31,21,'gAAag'],[32,22,'gaag']
  ])for(let n=0;n<line.length;n++)set(x+n,y,line[n]);
 }
 lanceHead(api,level);
});
for(let level=1;level<=3;level++)create(`horse-archer-l${level}`,['','Horse archer — L1','Horse bowman — L2','Horse marksman — L3'][level],'mounted-bow',level,api=>{
 const {region,palette,bright,set}=api;
 region((x,y)=>x>=27&&x<=38&&y>=29&&y<=42,{'H':'B','h':'b'});
 if(level===1){
  region((x,y)=>x>=27&&x<=41&&y>=17&&y<=28,{'C':'L','c':'l'});
  region((x,y)=>y<28,{'G':'B','g':'b'});
 }else if(level===3){
  bright();palette.P=[244,231,191,255];palette.p=[178,165,132,255];
  palette.H=[193,151,100,255];palette.h=[120,88,52,255];
  region((x,y)=>x>=24&&x<=35&&y>=7&&y<=14,{'T':'P','t':'p'});
  set(32,23,'G');set(33,23,'G');
 }
});
fs.writeFileSync(path.join(root,'lineup.json'),JSON.stringify(variants,null,2)+'\n');
console.log(`Created ${variants.length} tier studies`);
