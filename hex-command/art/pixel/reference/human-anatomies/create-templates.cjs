const fs=require('node:fs');
const {PNG}=require('pngjs');
const {execFileSync}=require('node:child_process');
const dir='art/pixel/reference/human-anatomies';
fs.mkdirSync('tmp/human-anatomies',{recursive:true});
fs.mkdirSync(`${dir}/templates`,{recursive:true});
const source=JSON.parse(fs.readFileSync('art/pixel/units/soldier/palette.json'));
const palette=Object.fromEntries(['.','O','B','b','T','t','C','S','A','a','k','G','g'].map(s=>[s,source[s]]));
palette.c=[179,166,139,255];palette.s=[211,154,103,255];palette.H=[145,91,49,255];palette.h=[87,55,31,255];
fs.writeFileSync(`${dir}/palette.json`,JSON.stringify(palette,null,2)+'\n');
const anchors={
 'crossbow':['units/archer','units/marksman','units/slinger'],
 'staff-infantry':['units/swordsman','units/spearman','units/champion'],
 'mounted-lance':['units/swordsman','units/spearman','units/archer'],
 'mounted-bow':['units/archer','units/marksman','units/swordsman']
};
for(const [id,paths] of Object.entries(anchors)){
 const out=new PNG({width:1024,height:1024});
 for(let i=0;i<out.data.length;i+=4)out.data.set([255,0,255,255],i);
 paths.forEach((path,n)=>{
  const p=PNG.sync.read(fs.readFileSync(`art/pixel/${path}/player.png`));
  const bx=(n%2)*512+64,by=Math.floor(n/2)*512+64;
  for(let y=0;y<384;y++)for(let x=0;x<384;x++){
   const si=(Math.floor(y/8)*48+Math.floor(x/8))*4;
   if(p.data[si+3])out.data.set(p.data.subarray(si,si+4),((by+y)*1024+bx+x)*4);
  }
 });
 Object.values(palette).filter(v=>v[3]).forEach((color,n)=>{
  const bx=10+n*29,by=482;
  for(let y=0;y<18;y++)for(let x=0;x<24;x++)out.data.set(color,((by+y)*1024+bx+x)*4);
 });
 fs.writeFileSync(`tmp/human-anatomies/${id}-base.png`,PNG.sync.write(out));
 const name=id.toUpperCase().replaceAll('-',' ');
 execFileSync('magick',[`tmp/human-anatomies/${id}-base.png`,'-draw',"fill '#ffffff'\nfont 'DejaVu-Sans-Bold'\nfont-size 21\ntext 16,34 'EXISTING STYLE ANCHOR'\ntext 528,34 'EXISTING STYLE ANCHOR'\ntext 16,546 'EXISTING STYLE ANCHOR'\n"+`text 528,546 '${name} — FILL HERE'\n`+"fill '#111111'\nstroke '#111111'\nstroke-width 2\nline 512,0 512,1023\nline 0,512 1023,512",`${dir}/templates/${id}.png`]);
}
