const fs = require('../../../../scripts/pixel-fs.cjs');
const path = require('node:path');
const {execFileSync} = require('node:child_process');
const sourceDirectory = require('../../../../scripts/pixel-source-path.cjs');
const root = __dirname, project = path.resolve(root, '../../../..');
const dir = sourceDirectory('human-level3/grand-marshal');
const source = path.join(project, 'art/pixel/units/marshal');
const original = fs.readFileSync(path.join(source, 'player.txt'), 'utf8').trimEnd().split('\n');
const rows = original.map(row => [...row]);
const palette = JSON.parse(fs.readFileSync(path.join(source, 'palette.json')));
Object.assign(palette, {
 G:[255,211,82,255],g:[181,124,30,255],
 A:[237,244,244,255],a:[155,179,190,255],k:[69,87,100,255],
 X:[235,255,255,255],J:[106,215,238,255],j:[33,128,165,255],K:[18,57,77,255],
});
const set = (x,y,s) => {rows[y][x]=s;};
for (let y=0;y<48;y++) for(let x=0;x<48;x++) {
 const s=original[y][x];
 // Gold fittings on the body, excluding hands, the wooden banner pole and boots.
 if(x>=23&&x<=38&&y>=19&&y<=40&&'Bb'.includes(s))set(x,y,s==='B'?'G':'g');
 // Crystal blade follows the existing sword; nearby armored boots stay silver.
 if(y>=34&&y<=44&&Math.abs(x-(39+0.6*(y-34)))<=2.5){
  const crystal={A:'X',a:'J',k:'j',c:'J',B:'K',b:'K'};
  if(crystal[s])set(x,y,crystal[s]);
 }
}
// Three-point gold coronet joins the existing helmet, below the banner's finial.
for(const [x,y,line]of [
 [30,9,'G'],[27,10,'G'],[33,10,'G'],
 [27,11,'GG.GG.GG'],[27,12,'gGGGGGGg'],[28,13,'gGGGGg'],
])for(let n=0;n<line.length;n++)if(line[n]!=='.')set(x+n,y,line[n]);
// Connected gold edging down the outer shoulder mantle.
for(let y=23;y<=33;y++)for(let x=22;x<=23;x++)if('Tt'.includes(original[y][x]))set(x,y,x===22?'g':'G');
for(const y of [14,35]){set(6,y,'G');set(7,y,'g');}
for(let y=1;y<=2;y++){set(7,y,'G');set(8,y,'g');}
fs.mkdirSync(dir,{recursive:true});
fs.writeFileSync(path.join(dir,'palette.json'),JSON.stringify(palette,null,2)+'\n');
const edits=[];
for(let y=0;y<48;y++)for(let x=0;x<48;x++)if(rows[y][x]!==original[y][x])edits.push({x,y,from:original[y][x],to:rows[y][x]});
fs.writeFileSync(path.join(dir,'changes.json'),JSON.stringify({base:'marshal',level:3,edits},null,2)+'\n');
for(const side of ['player','enemy']){
 const file=path.join(dir,side+'.txt');
 fs.writeFileSync(file,rows.map(row=>row.map(s=>side==='enemy'?({T:'R',t:'r'}[s]||s):s).join('')).join('\n')+'\n');
 execFileSync('python3',['scripts/render-pixel.py',file],{cwd:project,stdio:'inherit'});
}
const entries=[
 {label:'Commander — L1',image:'art/pixel/units/commander/player.png'},
 {label:'Marshal — L2',image:'art/pixel/units/marshal/player.png'},
 {label:'Grand Marshal — L3',image:'art/pixel/units/grand-marshal/player.png'},
];
const file=path.join(root,'commander-lineup.json');
fs.writeFileSync(file,JSON.stringify(entries,null,2)+'\n');
execFileSync('node',[path.join(project,'art/pixel/reference/human-anatomies/preview.cjs'),file,path.join(root,'commander-preview.png'),'3','8'],{cwd:project,stdio:'inherit'});
