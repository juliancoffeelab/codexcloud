const fs=require('../../../../scripts/pixel-fs.cjs');
const path=require('node:path');
const {execFileSync}=require('node:child_process');
const root=__dirname,project=path.resolve(root,'../../../..');
const run=(tool,args)=>execFileSync(tool,args,{cwd:project,stdio:'inherit'});
for(const [name,crop]of [
 ['crossbow',[]],['staff-infantry',['48','8','9']],
 ['mounted-lance',['64']],['mounted-bow',['64']]
])run('node',[path.join(root,'extract.cjs'),path.join(root,'sources',name+'.png'),path.join(root,'base-'+name),...crop]);
for(const script of ['clean.cjs','variants.cjs','factions.cjs'])run('node',[path.join(root,script)]);
const directories=new Set([
 ...fs.readdirSync(root).map(name=>path.join(root,name)),
 ...JSON.parse(fs.readFileSync(path.join(root,'lineup.json'))).map(entry=>path.dirname(path.resolve(project,entry.image)))
]);
const matrices=[...directories].flatMap(dir=>['player','enemy'].map(side=>path.join(dir,side+'.txt'))).filter(file=>fs.existsSync(file));
run('python3',['scripts/render-pixel.py',...matrices]);
for(const [name,cols,scale]of [['bases',2,8],['crossbow',2,8],['infantry',2,8],['cavalry',3,8],['all',3,6]]){
 run('node',[path.join(root,'preview.cjs'),path.join(root,name==='all'?'lineup.json':name+'-lineup.json'),path.join(root,name+'-preview.png'),String(cols),String(scale)]);
}
