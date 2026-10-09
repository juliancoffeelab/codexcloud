const fs = require('node:fs');
const path = require('node:path');
const { PNG } = require('pngjs');

// Extract the new lower-right slot from a prefilled four-slot generation.
// Usage: node extract.cjs <generated.png> <output-directory> [size=48] [x=8] [y=8]
const [sourcePath, outputDir, sizeArg, xArg, yArg] = process.argv.slice(2);
const size=Number(sizeArg || 48), originX=Number(xArg || (size===64?0:8)), originY=Number(yArg || (size===64?0:8));
if(![48,64].includes(size) || originX<0 || originY<0 || originX+size>64 || originY+size>64) throw new Error("Invalid native crop");
if (!sourcePath || !outputDir) throw new Error('Expected source PNG and output directory');
const source = PNG.sync.read(fs.readFileSync(sourcePath));
const palette = JSON.parse(fs.readFileSync(path.join(__dirname, 'palette.json')));
const colors = Object.entries(palette).filter(([s]) => s !== '.');
const x0 = Math.floor(source.width / 2), y0 = Math.floor(source.height / 2);
const crop = new PNG({width:source.width-x0, height:source.height-y0});
for (let y=0;y<crop.height;y++) for(let x=0;x<crop.width;x++) {
 const i=((y0+y)*source.width+x0+x)*4;
 crop.data.set(source.data.subarray(i,i+4),(y*crop.width+x)*4);
}
function symbol(i) {
 const r=crop.data[i],g=crop.data[i+1],b=crop.data[i+2];
 if(crop.data[i+3]===0 || (r>110 && b>90 && r-g>65 && b-g>65)) return '.';
 let best='.',distance=(r-255)**2+g*g+(b-255)**2;
 for(const [s,c] of colors) {
  const d=(r-c[0])**2+(g-c[1])**2+(b-c[2])**2;
  if(d<distance) {best=s;distance=d;}
 }
 return best;
}
const rows=[];
for(let y=0;y<64;y++) {
 const row=[];
 for(let x=0;x<64;x++) {
  if(y===0 || x===0 || x===63 || y===63) {row.push('.');continue;}
  const left=Math.floor(x*crop.width/64),right=Math.floor((x+1)*crop.width/64);
  const top=Math.floor(y*crop.height/64),bottom=Math.floor((y+1)*crop.height/64);
  const counts=new Map();let foreground=0,total=0;
  for(let yy=top;yy<bottom;yy++) for(let xx=left;xx<right;xx++) {
   const s=symbol((yy*crop.width+xx)*4);total++;
   if(s!=='.') {foreground++;counts.set(s,(counts.get(s)||0)+1);}
  }
  let best='.',max=0;
  if(foreground/total>=0.35) for(const [s,n] of counts) if(n>max) {best=s;max=n;}
  row.push(best);
 }
 rows.push(row);
}
fs.mkdirSync(outputDir,{recursive:true});
fs.writeFileSync(path.join(outputDir,'crop.png'),PNG.sync.write(crop));
fs.writeFileSync(path.join(outputDir,'extracted-64.txt'),rows.map(r=>r.join('')).join('\n')+'\n');
const outside=[];
for(let y=0;y<64;y++) for(let x=0;x<64;x++) {
 if(rows[y][x]!=='.' && (x<originX || x>=originX+size || y<originY || y>=originY+size)) outside.push([x,y]);
}
if(outside.length) throw new Error(`${outside.length} visible pixels outside requested native frame: ${JSON.stringify(outside)}`);
const native=rows.slice(originY,originY+size).map(row=>row.slice(originX,originX+size).join(''));
fs.writeFileSync(path.join(outputDir,'player.txt'),native.join('\n')+'\n');
fs.writeFileSync(path.join(outputDir,'palette.json'),JSON.stringify(palette,null,2)+'\n');
const report={source:[source.width,source.height],crop:[x0,y0,crop.width,crop.height],native:[size,size],origin:[originX,originY],colors:[...new Set(native.join('').replaceAll('.',''))]};
fs.writeFileSync(path.join(outputDir,'extraction.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
