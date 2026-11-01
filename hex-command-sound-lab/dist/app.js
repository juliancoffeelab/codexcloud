const $ = (s) => document.querySelector(s);
const catalog = await fetch('catalog.json').then(r => { if (!r.ok) throw Error('Catalog unavailable'); return r.json(); }).catch(e => { $('#playing').textContent = `${e.message}. Reload to retry.`; return []; });
const byId = new Map(catalog.map(x => [x.id, x]));
const impact = (prefix) => catalog.filter(x => x.pack === 'impact-sounds' && x.name.startsWith(prefix)).map(x => x.id);
const rpg = (...names) => names.map(n => `rpg-audio/${n}`);
const current = (family) => [1,2,3].map(n => `current/${family}-${n}`);
const profiles = [
 {id:'soldier',name:'Soldier',role:'Armored infantry',desc:'A grounded footfall, a short blade movement, and a solid metal impact.',move:impact('footstep_concrete'),attack:rpg('knifeSlice','knifeSlice2'),hit:impact('impactMetal_medium'),gear:rpg('drawKnife1','drawKnife2','drawKnife3'),old:'unit-archer',detail:'Knife slices · blade swing candidates'},
 {id:'archer',name:'Archer',role:'Ranged infantry',desc:'Light steps and soft equipment. The current bow remains a baseline until the medieval recordings are available.',move:impact('footstep_grass'),attack:current('attack-bow'),hit:impact('impactWood_light'),gear:rpg('cloth1','cloth2','cloth3','cloth4'),old:'terrain-plains',detail:'Current synthesized bow · no CC0 bow fetched'},
 {id:'scout',name:'Scout',role:'Light infantry',desc:'Quiet movement through grass, with leather handling and a light blade.',move:impact('footstep_grass'),attack:rpg('knifeSlice','knifeSlice2'),hit:impact('impactSoft_medium'),gear:rpg('handleSmallLeather','handleSmallLeather2','beltHandle1','beltHandle2'),old:'terrain-plains',detail:'Knife slices · light blade candidates'},
 {id:'commander',name:'Commander',role:'Battlefield leader',desc:'Firm steps and equipment detail, with a heavier metal impact for a commanding presence.',move:impact('footstep_concrete'),attack:rpg('knifeSlice','knifeSlice2'),hit:impact('impactMetal_heavy'),gear:rpg('drawKnife1','drawKnife2','drawKnife3'),old:'unit-commander',detail:'Knife slices · blade swing candidates'},
 {id:'spearman',name:'Spearman',role:'Polearm infantry',desc:'Wood and metal textures to explore the feel of a shaft, a thrust, and a shield contact.',move:impact('footstep_grass'),attack:impact('impactWood_light'),hit:impact('impactMetal_light'),gear:rpg('cloth1','cloth2','cloth3'),old:'terrain-plains',detail:'Wood contact · polearm texture candidate, not a recorded thrust'},
 {id:'warden',name:'Warden',role:'Heavy infantry',desc:'Deliberate footsteps, weighty plate contacts and short equipment movements.',move:impact('footstep_concrete'),attack:rpg('knifeSlice','knifeSlice2'),hit:impact('impactPlate_heavy'),gear:rpg('metalClick','metalLatch'),old:'unit-archer',detail:'Knife slices · weapon swing candidates'},
 {id:'adept',name:'Adept',role:'Arcane infantry',desc:'Cloth, pages, and a light step. There is no suitable CC0 magic attack in the fetched packs.',move:impact('footstep_carpet'),attack:current('attack-melee'),hit:impact('impactGeneric_light'),gear:rpg('bookFlip1','bookFlip2','bookFlip3'),old:'terrain-plains',detail:'Current melee fallback · no magic candidate available'},
 {id:'necromancer',name:'Necromancer',role:'Undead commander',desc:'Soft fabric and worn pages. Listen to material details while magic remains an open sound-design task.',move:impact('footstep_carpet'),attack:current('attack-melee'),hit:impact('impactGeneric_light'),gear:rpg('bookFlip1','bookFlip2','bookFlip3'),old:'unit-commander',detail:'Current melee fallback · no magic candidate available'},
 {id:'skeleton',name:'Skeleton',role:'Undead infantry',desc:'Hard foot contacts and dry wooden knocks as a bone-like texture. These are material suggestions, not bone recordings.',move:impact('footstep_wood'),attack:rpg('knifeSlice','knifeSlice2'),hit:impact('impactWood_medium'),gear:impact('impactWood_light'),old:'unit-commander',detail:'Knife slices · blade swing candidates'},
 {id:'wolf',name:'Wolf',role:'Forest creature',desc:'Soft contacts on grass and padded impacts. No animal vocalizations are included in the fetched packs.',move:impact('footstep_grass'),attack:impact('impactSoft_medium'),hit:impact('impactSoft_heavy'),gear:[],old:'terrain-plains',detail:'Soft impact · bite contact candidate, not a recorded animal'}
];
let selected = profiles[0], mode = 'candidate', context, master, source, serial = 0;
const buffers = new Map();
let favorites;
try { favorites = new Set(JSON.parse(localStorage.getItem('hex-sound-favorites') || '[]')); } catch { favorites = new Set(); }
const labels = {'impact-sounds':'Kenney · Impact Sounds','rpg-audio':'Kenney · RPG Audio',current:'Current Hexcommand'};
const pretty = (name) => name.replace(/_/g,' ').replace(/([a-z])([A-Z])/g,'$1 $2').replace(/-/g,' ');
function stop() { serial++; if (source) { try { source.stop(); } catch {} source = null; } $('#playing').textContent = 'Ready when you are.'; paintWave(); }
async function unlock() { if (!context) { context = new (window.AudioContext || window.webkitAudioContext)(); master = context.createGain(); master.gain.value = Number($('#volume').value)/100; master.connect(context.destination); } if (context.state !== 'running') await context.resume(); }
async function bufferFor(item) { if (!buffers.has(item.id)) buffers.set(item.id,fetch(item.path).then(r => { if (!r.ok) throw Error('Recording unavailable'); return r.arrayBuffer(); }).then(b => context.decodeAudioData(b)).catch(e => { buffers.delete(item.id); throw e; })); return buffers.get(item.id); }
function paintWave(buffer) { const canvas=$('#wave'),g=canvas.getContext('2d'); g.clearRect(0,0,canvas.width,canvas.height); g.fillStyle='#a8c380'; const data=buffer?.getChannelData(0); const count=84; for(let i=0;i<count;i++){let peak=0;if(data){const start=Math.floor(i*data.length/count),end=Math.floor((i+1)*data.length/count);for(let j=start;j<end;j++)peak=Math.max(peak,Math.abs(data[j]));}const h=Math.max(2,peak*40);g.globalAlpha=data?.length ? .8:.25;g.fillRect(i*5,22-h/2,2,h);} }
async function playOne(id, ticket) { const item=byId.get(id); if (!item) throw Error('Recording missing from catalog'); $('#playing').textContent = `Loading ${pretty(item.name)}…`; const b=await bufferFor(item); if(ticket!==serial) return; const node=context.createBufferSource(); node.buffer=b; node.connect(master); source=node; $('#playing').textContent=`${pretty(item.name)} · ${labels[item.pack]}`;paintWave(b);await new Promise(resolve=>{node.onended=()=>{node.disconnect();if(source===node) source=null;resolve();};node.start();}); }
async function play(ids) { stop(); const ticket=serial; try { await unlock(); for(const id of ids){if(ticket!==serial)break;await playOne(id,ticket);} if(ticket===serial)$('#playing').textContent='Finished · click any variation to listen again.'; } catch(e){if(ticket===serial)$('#playing').textContent=`${e.message}. Try again or reload.`;} }
function random(ids){return ids[Math.floor(Math.random()*ids.length)];}
function actions(){const isCurrent=mode==='current';return [
 {key:'move',icon:'↗',title:'Movement',caption:isCurrent?'Existing selection / movement cue':'Footstep recordings · compare the surface and weight',ids:isCurrent?current(selected.old):selected.move},
 {key:'attack',icon:'⚔',title:'Attack',caption:isCurrent?'Existing attack family':selected.detail,ids:isCurrent?current(selected.id==='archer'?'attack-bow':'attack-melee'):selected.attack},
 {key:'hit',icon:'◇',title:'Impact',caption:isCurrent?'The current game shares its attack cue with impact':'Material impact candidates · audition independently',ids:isCurrent?[]:selected.hit},
 {key:'gear',icon:'⋮',title:'Equipment & detail',caption:isCurrent?'No separate equipment cue in the current game':selected.id==='wolf'?'No animal recording in these packs':'Close material detail · optional layer',ids:isCurrent?[]:selected.gear}
 ];}
function el(tag,cls,text){const e=document.createElement(tag);if(cls)e.className=cls;if(text!==undefined)e.textContent=text;return e;}
function renderUnit(){
 $('#unit-role').textContent=selected.role;$('#unit-name').textContent=selected.name;$('#unit-description').textContent=selected.desc;$('#unit-image').src=`units/${selected.id}.png`;$('#unit-image').alt=`Hexcommand ${selected.name}`;
 document.querySelectorAll('[data-unit]').forEach(b=>{const active=b.dataset.unit===selected.id;b.classList.toggle('selected',active);b.setAttribute('aria-pressed',String(active));});
 const root=$('#action-cards');root.replaceChildren();for(const action of actions()){
 const card=el('article','sound-card'),icon=el('span','action-icon',action.icon),info=el('div'),title=el('h3','sound-title',action.title),caption=el('p','sound-caption',action.caption);info.append(title,caption);card.append(icon,info);
 if(action.ids.length){const external=action.ids.every(id=>byId.get(id)?.license==='CC0-1.0');const imported=action.ids.some(id=>id.startsWith('imported/'));info.append(el('span',external?'tag':'tag neutral',`${external?'CC0 RECORDINGS':imported?'LOCAL IMPORT · CHECK LICENSE':'CURRENT SYNTHESIZED AUDIO'} · ${action.ids.length} VARIATIONS`));const controls=el('div','sound-controls');const p=el('button','play','▶');p.setAttribute('aria-label',`Play random ${selected.name} ${action.title}`);p.title='Play a random variation';p.onclick=()=>play([random(action.ids)]);controls.append(p);action.ids.forEach((id,i)=>{const b=el('button','variant',String(i+1));b.setAttribute('aria-label',`${selected.name} ${action.title}, variation ${i+1}`);b.title=byId.get(id)?.name;b.onclick=()=>play([id]);controls.append(b);});const all=el('button','all','All →');all.setAttribute('aria-label',`Play all ${action.title} variations`);all.onclick=()=>play(action.ids);controls.append(all);card.append(controls);}else{info.append(el('span','tag neutral','NO SEPARATE RECORDING'));}root.append(card);}
}
for(const u of profiles){const b=el('button');b.dataset.unit=u.id;b.setAttribute('aria-pressed','false');const img=el('img');img.src=`units/${u.id}.png`;img.alt='';img.width=64;img.height=64;b.append(img,el('span','',u.name));b.onclick=()=>{stop();selected=u;renderUnit();};$('#roster').append(b);}
$('#candidate-mode').onclick=()=>setMode('candidate');$('#current-mode').onclick=()=>setMode('current');
function setMode(value){stop();mode=value;for(const [id,m] of [['candidate-mode','candidate'],['current-mode','current']]){const active=m===mode;$(`#${id}`).classList.toggle('selected',active);$(`#${id}`).setAttribute('aria-pressed',String(active));}renderUnit();}
$('#sequence').onclick=()=>play(actions().filter(a=>['move','attack','hit'].includes(a.key)&&a.ids.length).map(a=>random(a.ids)));
$('#stop').onclick=stop;document.addEventListener('keydown',e=>{if(e.key==='Escape')stop();});
$('#volume').oninput=e=>{const v=Number(e.target.value);$('#volume-value').textContent=`${v}%`;if(master)master.gain.setTargetAtTime(v/100,context.currentTime,.015);};
function saveFavorites(){try{localStorage.setItem('hex-sound-favorites',JSON.stringify([...favorites]));}catch{$('#playing').textContent='Favorites saved for this session only; browser storage is unavailable.';}}
function renderLibrary(){const query=$('#search').value.trim().toLowerCase(),pack=$('#pack').value,only=$('#favorites-only').checked;const items=catalog.filter(x=>(pack==='all'||x.pack===pack)&&(!only||favorites.has(x.id))&&`${x.name} ${pretty(x.name)} ${labels[x.pack]}`.toLowerCase().includes(query));$('#results').textContent=`${items.length} RECORDINGS${only?' · FAVORITES':''}`;const root=$('#library-list');root.replaceChildren();if(!items.length)root.append(el('p','empty','No sounds match. Try another search or source.'));for(const item of items){const row=el('div','library-row'),p=el('button','play','▶');p.setAttribute('aria-label',`Play ${pretty(item.name)}`);p.onclick=()=>play([item.id]);const info=el('div');info.append(el('strong','',pretty(item.name)),el('small','',`${labels[item.pack]} · ${item.pack==='current'?'Synthesized baseline':item.license}`));const dl=el('a','','↓');dl.href=item.path;dl.download=item.path.split('/').at(-1);dl.setAttribute('aria-label',`Download ${item.name}`);const fav=el('button',`favorite${favorites.has(item.id)?' saved':''}`,favorites.has(item.id)?'★':'☆');fav.setAttribute('aria-label',`Favorite ${item.name}`);fav.setAttribute('aria-pressed',String(favorites.has(item.id)));fav.onclick=()=>{if(favorites.has(item.id))favorites.delete(item.id);else favorites.add(item.id);saveFavorites();renderLibrary();};row.append(p,info,dl,fav);root.append(row);}}
$('#search').oninput=renderLibrary;$('#pack').onchange=renderLibrary;$('#favorites-only').onchange=renderLibrary;
for(const b of document.querySelectorAll('[data-view]'))b.onclick=()=>{for(const tab of document.querySelectorAll('[data-view]')){const active=tab===b;tab.classList.toggle('active',active);tab.setAttribute('aria-selected',String(active));}for(const view of ['units','library','sources'])$(`#${view}-view`).hidden=view!==b.dataset.view;if(b.dataset.view==='library')renderLibrary();};
$('#export').onclick=()=>{const content={unit:selected.id,exportedAt:new Date().toISOString(),favorites:catalog.filter(x=>favorites.has(x.id)).map(({path,...item})=>({...item,path:path.startsWith('blob:')?'Local browser import; see originalPath':path}))};const url=URL.createObjectURL(new Blob([JSON.stringify(content,null,2)],{type:'application/json'}));const a=el('a');a.href=url;a.download='hexcommand-sound-favorites.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
for(let i=0;i<49;i++){const b=el('b');b.style.height=`${8+Math.abs(Math.sin(i*2.1)*Math.sin(i*.17))*65}px`;$('.signal').append(b);}
$('#total').textContent=String(catalog.length);$('#library-count').textContent=String(catalog.length);paintWave();renderUnit();

// Browser-only ingestion: no server proxy and no environment network changes.
const remotePacks = [
 ['Weapon textures · part 1','medieval_sfx_textures_1_of_2.7z','89.9 MB'],
 ['Weapon textures · part 2','medieval_sfx_textures_2_of_2.7z','86.1 MB'],
 ['Weapon impacts · part 1','medieval_sfx_weapon_on_weapon_1_of_2.7z','82.1 MB'],
 ['Weapon impacts · part 2','medieval_sfx_weapon_on_weapon_2_of_2.7z','73.8 MB']
].map(([name,file,size])=>({name,file,size,url:`https://opengameart.org/sites/default/files/${file}`}));
let importing=false, importController, importWorker, importRun=0;
const importedNames = new Set();
const importMessage = text => { $('#import-status').textContent=text; };
function importState(busy){importing=busy;$('#import-files').disabled=busy;$('#cancel-import').hidden=!busy;document.querySelectorAll('.remote-pack button').forEach(b=>b.disabled=busy);}
function cancelImport(){importRun++;importController?.abort();importWorker?.terminate();importWorker=null;importState(false);importMessage('Import cancelled. Recordings already added remain available.');}
$('#cancel-import').onclick=cancelImport;
async function addRecording(file,path,origin){
 if(!/\.(wav|ogg|mp3|flac|m4a)$/i.test(file.name))return false;
 const key=`${origin}/${path}`;if(importedNames.has(key))return false;
 importedNames.add(key);const id=`imported/${key}`;const item={id,name:path.replace(/\.[^.]+$/,''),path:URL.createObjectURL(file),pack:'imported',license:remotePacks.some(p=>p.file===origin)?'CC0-1.0':'User import · verify source license',source:remotePacks.find(p=>p.file===origin)?.url||'Local file · not uploaded',originalPath:path};
 catalog.push(item);byId.set(id,item);return true;
}
function applyImportedCandidates(){
 labels.imported='Browser imports';
 const find=(pattern)=>catalog.filter(x=>x.pack==='imported'&&pattern.test(x.name.replace(/[_\-]/g,' '))).slice(0,5).map(x=>x.id);
 for(const unit of profiles){
 const pattern={soldier:/sabre.*swing|katana.*swing/i,commander:/sabre.*swing|katana.*swing/i,scout:/dagger.*swing|seax.*swing/i,spearman:/spear.*swing/i,warden:/spear.*swing|sabre.*swing/i,archer:/english.*longbow.*shoot|recurve.*shoot/i,skeleton:/sabre.*swing|seax.*swing/i}[unit.id];
 const attack=pattern?find(pattern):[];if(attack.length){unit.attack=attack;unit.detail='Imported weapon recordings · up to five matching takes';}
 if(['soldier','commander','warden','skeleton','spearman'].includes(unit.id)){const hits=find(unit.id==='spearman'?/spear.*(blade|haft)/i:/norse.*sword.*blade|sabre.*blade|katana.*blade/i);if(hits.length)unit.hit=hits;}
 const gear=unit.id==='archer'?find(/longbow.*draw|longbow.*nock/i):['soldier','commander','scout','skeleton'].includes(unit.id)?find(/(sabre|dagger|katana).*draw/i):[];if(gear.length)unit.gear=gear;
 }
 $('#total').textContent=String(catalog.length);$('#library-count').textContent=String(catalog.length);renderUnit();renderLibrary();
}
async function processFile(file,ticket){
 if(file.size>125*1024*1024)throw Error('This file is over 125 MB. Choose one source archive at a time.');
 if(/\.(7z|zip)$/i.test(file.name)){
  importMessage(`Opening ${file.name}… Large archives may take a minute.`);
  const {Archive}=await import('./vendor/libarchive/libarchive.js');
  Archive.init({getWorker:()=>{importWorker=new Worker(new URL('./vendor/libarchive/worker-bundle.js',import.meta.url),{type:'module'});return importWorker;}});
  let archive;try{
   archive=await Archive.open(file);if(ticket!==importRun)return 0;
   const listing=await archive.getFilesArray();const size=listing.reduce((total,e)=>total+(e.file.size||0),0);if(size>750*1024*1024)throw Error('Expanded archive is over 750 MB. Extract it on your computer and import selected audio files.');
   importMessage(`Extracting ${file.name} · ${listing.length} entries…`);
   const extracted=await archive.extractFiles();let count=0;
   async function walk(obj,prefix=''){for(const [name,value] of Object.entries(obj)){if(ticket!==importRun)return;if(value instanceof File){if(await addRecording(value,prefix+name,file.name))count++;}else if(value&&typeof value==='object')await walk(value,`${prefix}${name}/`);}}
   await walk(extracted);return count;
  }finally{await archive?.close();importWorker?.terminate();importWorker=null;}
 }
 return await addRecording(file,file.name,'Local audio')?1:0;
}
async function importFiles(files){if(importing)return;const ticket=++importRun;importState(true);let count=0;try{for(const file of files){if(ticket!==importRun)return;count+=await processFile(file,ticket);}if(ticket!==importRun)return;applyImportedCandidates();importMessage(`Added ${count} recordings. Matching weapon sounds are now assigned to units; all imports appear in Full library. Imports last for this tab only.`);}catch(e){if(ticket===importRun){applyImportedCandidates();importMessage(`Import failed: ${e.message}. You can also extract the archive on your computer and import its audio files.`);}}finally{if(ticket===importRun)importState(false);}}
$('#import-files').onchange=e=>{const files=[...e.target.files];e.target.value='';void importFiles(files);};
for(const pack of remotePacks){const row=el('div','remote-pack');row.append(el('span','',`${pack.name} · ${pack.size}`));const controls=el('div'),fetchButton=el('button','','Fetch in browser'),link=el('a','','Download archive ↗');link.href=pack.url;link.target='_blank';link.rel='noopener';fetchButton.onclick=async()=>{
 if(importing)return;const ticket=++importRun;importState(true);importController=new AbortController();let timeout=setTimeout(()=>importController.abort(),180000);
 try{importMessage(`Fetching ${pack.name} directly from OpenGameArt…`);const response=await fetch(pack.url,{mode:'cors',credentials:'omit',signal:importController.signal});if(!response.ok)throw Error(`Source returned HTTP ${response.status}`);const blob=await response.blob();clearTimeout(timeout);if(ticket!==importRun)return;const count=await processFile(new File([blob],pack.file),ticket);if(ticket!==importRun)return;applyImportedCandidates();importMessage(`Added ${count} recordings from ${pack.name}. Matching units now use the imported candidates. These files stay in this tab only.`);}catch(e){if(ticket===importRun)importMessage(`Browser fetch could not complete (${e.name==='AbortError'?'timed out':e.message}). The host may block CORS. Use “Download archive”, then “Import downloaded archives / audio”.`);}finally{clearTimeout(timeout);if(ticket===importRun)importState(false);}
 };controls.append(fetchButton,link);row.append(controls);$('#remote-packs').append(row);}
