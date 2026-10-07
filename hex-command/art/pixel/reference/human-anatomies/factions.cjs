const fs=require('node:fs');
const path=require('node:path');
const project=path.resolve(__dirname,'../../../..');
const directories=new Set([
 ...fs.readdirSync(__dirname).map(name=>path.join(__dirname,name)),
 ...JSON.parse(fs.readFileSync(path.join(__dirname,'lineup.json'))).map(entry=>path.dirname(path.resolve(project,entry.image)))
]);
for(const dir of directories){
 const file=path.join(dir,'player.txt');
 if(!fs.existsSync(file))continue;
 const matrix=fs.readFileSync(file,'utf8').replaceAll('T','R').replaceAll('t','r');
 const palette=JSON.parse(fs.readFileSync(path.join(dir,'palette.json')));
 palette.R=[185,24,37,255];palette.r=[112,14,24,255];
 fs.writeFileSync(path.join(dir,'enemy.txt'),matrix);
 fs.writeFileSync(path.join(dir,'palette.json'),JSON.stringify(palette,null,2)+'\n');
}
