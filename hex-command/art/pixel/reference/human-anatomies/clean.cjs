const fs=require('node:fs');
const path=require('node:path');
const root=__dirname;
const names=['crossbow','staff-infantry','mounted-lance','mounted-bow'];
for(const name of names){
 const dir=path.join(root,'base-'+name),file=path.join(dir,'player.txt');
 const original=fs.readFileSync(file,'utf8').trimEnd().split('\n').map(r=>[...r]);
 const rows=original.map(r=>[...r]),height=rows.length,width=rows[0].length;
 const reasons=new Map();
 function set(x,y,s,reason){if(rows[y]?.[x]===undefined || rows[y][x]===s)return;rows[y][x]=s;reasons.set(`${x},${y}`,reason);}
 function mapRegion(test,from,to,reason){for(let y=0;y<height;y++)for(let x=0;x<width;x++)if(test(x,y)&&from.includes(rows[y][x]))set(x,y,to[rows[y][x]]||to,reason);}
 if(name==='crossbow'){
  mapRegion((x,y)=>y<=13,'HhBbc',{'H':'k','h':'k','B':'k','b':'k','c':'a'},'Remove leather/ivory flecks from steel helmet');
  mapRegion((x,y)=>y>=8&&y<=11&&x>=21&&x<=24,'a','A','Connect helmet highlight plane');
  mapRegion((x,y)=>y===12||y===13,'akA','k','Make kettle-helmet brim a connected steel shadow');
  mapRegion((x,y)=>(y===14&&x>=20&&x<=22)||(y===15&&x>=19&&x<=22)||(y>=16&&y<=18&&x>=19&&x<=22),'BHSsh',{'B':'S','H':'S','S':'S','s':'S','h':'S'},'Remove wrong-color face flecks');
  mapRegion(()=>true,'Hh',{'H':'B','h':'B'},'Unify human leather and hair browns');
  mapRegion((x,y)=>x>=37&&y>=18&&y<=28,'Cc',{'C':'A','c':'a'},'Keep crossbow prod and string steel-colored');
  mapRegion((x,y)=>y===22&&x>=14&&x<=41||y===23&&x>=23&&x<=40,'BHhbsS', 'H','Flatten wooden crossbow stock; remove skin-colored wood artifacts');
  mapRegion((x,y)=>y===24&&x>=24&&x<=40,'BbhH','h','Connect underside shadow of wooden stock');
  mapRegion((x,y)=>y>=14,'s','S','Use a flat skin plane on exposed arms and hands');
  mapRegion((x,y)=>y>=30&&x<=13,'aAk','b','Remove gray flecks from leather bolt-case and leg');
  mapRegion((x,y)=>y===29&&x>=10&&x<=13,'ak','a','Keep a single connected bolt-case rim');
  mapRegion((x,y)=>y>=30&&y<=36&&x>=15&&x<=26,'c','C','Flatten tunic interior');
 } else if(name==='staff-infantry'){
  const shaft=(x,y)=>Math.abs(x-(44.5-0.83*y))<=2.5;
  mapRegion((x,y)=>!shaft(x,y),'Hh',{'H':'B','h':'B'},'Separate leather/hair from wooden staff palette');
  mapRegion((x,y)=>shaft(x,y)&&y>=32,'aAk','h','Remove gray flecks from wooden staff');
  mapRegion((x,y)=>y>=39&&!shaft(x,y),'aAk','b','Remove gray flecks from leather boots');
  mapRegion((x,y)=>y>=20&&y<=26&&x>=21&&x<=25||y>=23&&y<=28&&x>=13&&x<=18||y>=22&&y<=26&&x>=31&&x<=36||y>=31&&y<=40&&x>=29&&x<=33,'aAk','c','Keep sleeve/armor shadows in the ivory material');
  mapRegion((x,y)=>y>=27&&y<=30&&x>=20&&x<=24,'Hs','S','Remove wood-colored speckles from lower gripping hand');
  mapRegion((x,y)=>y>=12&&y<=14&&x>=23&&x<=26,'s','S','Flatten face skin');
  mapRegion((x,y)=>y>=16&&y<=19&&x>=31&&x<=34,'s','S','Flatten upper gripping hand skin');
 } else {
  const bow=name==='mounted-bow';
  const tail=(x,y)=>bow?x<=12&&y>=30&&y<=47:x<=12&&y>=31&&y<=48;
  const coat=(x,y)=>{
   if(tail(x,y))return false;
   if(bow){
    if(y>=22&&y<=34&&x>=45)return true;
    if(y>=29&&y<=43&&x>=14&&x<=53&&!(x>=27&&x<=38&&y<=42))return true;
    if(y>=43&&y<=52&&x>=10&&x<=53)return true;
   } else {
    if(y>=18&&y<=25&&x>=47)return true;
    if(y>=26&&y<=32&&x>=44)return true;
    if(y>=30&&y<=43&&x>=13&&x<=52&&!(x>=27&&x<=37&&y<=43))return true;
    if(y>=43&&y<=53&&x>=10&&x<=53)return true;
   }
   return false;
  };
  mapRegion((x,y)=>y>=28,'GgsS','H','Remove gold/skin speckles from horse and tack');
  mapRegion((x,y)=>y>=18&&y<=32&&x>=43,'Gg','H','Remove gold flecks from horse face and ears');
  mapRegion(tail,'HBh','b','Flatten dark horse tail');
  mapRegion(coat,'HBh','H','Flatten horse-coat material planes');
  const snapshot=rows.map(r=>[...r]);
  for(let y=0;y<height;y++)for(let x=0;x<width;x++)if(coat(x,y)&&snapshot[y][x]==='H'){
   if(['.','O','b'].includes(snapshot[y+1]?.[x]||'.'))set(x,y,'h','Connect underside horse-coat shadow');
  }
  mapRegion((x,y)=>y>=54,'HhB','k','Keep hooves dark and distinct from the coat');
  mapRegion((x,y)=>y<29&&!coat(x,y)&&!tail(x,y)&& !(bow&&x>=42&&y<=23) &&!(!bow&&Math.abs(x-(53-0.8*y))<=2),'Hh',{'H':'B','h':'b'},'Remove horse-palette speckles from rider leather');
  mapRegion((x,y)=>y>=10&&y<=18,'s','S','Flatten rider face and hands');
  if(bow){
   mapRegion((x,y)=>y>=14&&y<=16&&x>=24&&x<=27,'g','S','Remove gold fleck from drawing hand');
   mapRegion((x,y)=>y>=16&&y<=18&&x>=47&&x<=49,'s','S','Flatten bow-gripping hand');
   mapRegion((x,y)=>y>=13&&y<=16&&x>=52,'gHh','k','Keep arrowhead steel gray');
  }else{
   mapRegion((x,y)=>y>=24&&y<=26&&x>=30&&x<=32,'s','S','Flatten lance-gripping hand');
  }
 }
 // Remove isolated wrong-tone brown dots inside a flat leather/coat plane.
 const snapshot=rows.map(r=>[...r]);
 for(let y=1;y<height-1;y++)for(let x=1;x<width-1;x++){
  if(snapshot[y][x]==='b'&&[[0,-1],[0,1],[-1,0],[1,0]].every(([dx,dy])=>snapshot[y+dy][x+dx]==='B'))set(x,y,'B','Remove enclosed leather speckle');
 }
 // Broad ivory runs get one right-edge shadow, with small details preserved.
 for(let y=0;y<height;y++){
  let x=0;
  while(x<width){
   if(!'Cc'.includes(rows[y][x])){x++;continue;}
   const start=x;while(x<width&&'Cc'.includes(rows[y][x]))x++;
   if(x-start>=3){for(let xx=start;xx<x;xx++)set(xx,y,xx===x-1?'c':'C','Flatten ivory panel with a connected contour shadow');}
  }
 }
 const edits=[];
 for(let y=0;y<height;y++)for(let x=0;x<width;x++)if(rows[y][x]!==original[y][x])edits.push({x,y,from:original[y][x],to:rows[y][x],reason:reasons.get(`${x},${y}`)});
 if(rows.some((row,y)=>row.some((s,x)=>(s==='.')!==(original[y][x]==='.'))))throw new Error('Cleanup changed silhouette');
 fs.writeFileSync(file,rows.map(r=>r.join('')).join('\n')+'\n');
 fs.writeFileSync(path.join(dir,'cleanup.json'),JSON.stringify({coordinateSpace:[width,height],edits},null,2)+'\n');
 console.log(`${name}: ${edits.length} material-color corrections; silhouette preserved`);
}
