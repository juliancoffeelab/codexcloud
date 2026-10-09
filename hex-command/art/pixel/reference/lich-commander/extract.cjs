// Extract only the fourth box. Preserve the generated source and native geometry.
// Usage: node extract.cjs source.png output-directory
const fs = require('node:fs');
const path = require('node:path');
const { PNG } = require('pngjs');
const [sourcePath, output] = process.argv.slice(2);
if (!sourcePath || !output) throw new Error('Expected source PNG and output directory');
const source = PNG.sync.read(fs.readFileSync(sourcePath));
const factions = require('../../factions.json');
const material = {
  '.': [0,0,0,0], O:[22,16,12,255], I:[242,230,202,255], i:[192,174,139,255],
  W:[105,64,40,255], w:[64,39,25,255], G:[241,179,45,255], g:[189,121,28,255],
  E:[176,196,120,255], e:[102,132,74,255],
};
const palette = {...material, ...factions.player};
const colours = Object.entries(palette).filter(([s])=>s!=='.');
const x0=Math.floor(source.width/2),y0=Math.floor(source.height/2);
const width=source.width-x0,height=source.height-y0;
const crop=new PNG({width,height});
for(let y=0;y<height;y++)for(let x=0;x<width;x++) {
 const start=((y+y0)*source.width+x+x0)*4;
 crop.data.set(source.data.subarray(start,start+4),(y*width+x)*4);
}
function classify(index) {
 const [r,g,b,a]=crop.data.subarray(index,index+4);
 if(a===0 || (r>110 && b>90 && r-g>65 && b-g>65)) return '.';
 let best='.',distance=Infinity;
 for(const [symbol,rgba] of colours) {
  const d=(r-rgba[0])**2+(g-rgba[1])**2+(b-rgba[2])**2;
  if(d<distance){distance=d;best=symbol;}
 }
 return best;
}
const rows=[];
for(let y=0;y<64;y++) {
 let row='';
 for(let x=0;x<64;x++) {
  const counts=new Map();let total=0,foreground=0;
  for(let yy=Math.floor(y*height/64);yy<Math.floor((y+1)*height/64);yy++)
   for(let xx=Math.floor(x*width/64);xx<Math.floor((x+1)*width/64);xx++) {
    const s=classify((yy*width+xx)*4);total++;
    if(s!=='.'){foreground++;counts.set(s,(counts.get(s)||0)+1);}
   }
  let best='.',count=0;
  // Captions occupy the upper edge; retain the full remaining logical canvas.
  if(y>=8 && foreground/total>=0.35) for(const [s,n] of counts) if(n>count){best=s;count=n;}
  row+=best;
 }
 rows.push(row);
}
const outside=[];
for(let y=0;y<64;y++)for(let x=0;x<64;x++)
 if(rows[y][x]!=='.' && (x<8||x>=56||y<8||y>=56))outside.push([x,y]);
fs.mkdirSync(output,{recursive:true});
fs.writeFileSync(path.join(output,'crop.png'),PNG.sync.write(crop));
fs.writeFileSync(path.join(output,'extracted-64.txt'),rows.join('\n')+'\n');
// Larger native canvases preserve a generated staff/cape instead of clipping it.
const size=outside.length?64:48;
const native=size===48?rows.slice(8,56).map(row=>row.slice(8,56)):rows;
fs.writeFileSync(path.join(output,'sprite.txt'),native.join('\n')+'\n');
fs.writeFileSync(path.join(output,'palette.json'),JSON.stringify(material,null,2)+'\n');
fs.writeFileSync(path.join(output,'extraction.json'),JSON.stringify({source:[source.width,source.height],slot:'bottom-right',native:[size,size],origin:size===48?[8,8]:[0,0],outsideCentralFrame:outside,captionRowsExcluded:8},null,2)+'\n');
console.log(JSON.stringify({output,size,outside:outside.length}));
