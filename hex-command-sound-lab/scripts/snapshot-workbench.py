#!/usr/bin/env python3
"""Copy a self-contained faction/artwork snapshot from an explicitly supplied game checkout."""
import argparse,json,re,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--game-dir',type=Path,required=True);args=p.parse_args()
game=args.game_dir.resolve();root=Path(__file__).resolve().parents[1];out=root/'dist'
manifest=json.loads((game/'rust/data/manifest.json').read_text())
definitions={}
for f in manifest['units']:
 d=json.loads((game/'rust/data'/f).read_text());definitions[d['id']]=d
art=(game/'src/presentation/pixel-art.ts').read_text()
frames=dict(re.findall(r'"?([\w-]+)"?: "(assets/sprites/[^"]+)"',art))
models=dict(re.findall(r'"?([\w-]+)"?: "([^"]+)"',art[art.index('export const pixelUnitModels'):]))
def image(key,dest):
 if key not in frames:
  candidates=[k for k in frames if k.startswith(key+'_') and '__' not in k]
  key=next((k for k in [key+'_0',key+'_9'] if k in frames),sorted(candidates)[0] if candidates else key)
 if key not in frames:raise ValueError('Missing artwork '+key)
 target=out/dest;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy(game/'public'/frames[key],target);return dest

def inferred(member,role,base):
 text=' '.join([member['id'],member['name'],role]).lower()
 if 'arquebus' in text or 'pistol' in member['id']: names=['Arquebus' if 'arquebus' in text else 'Pistol','Knife']
 elif 'crossbow' in text:names=['Crossbow','Knife']
 elif 'runesmith' in text or 'stonespell' in text:names=['Rune stone','Hammer']
 elif 'thundersmith' in text:names=['Rune spark','Hammer']
 elif 'caster' in text or 'corruption' in text:names=['Corruption spell' if 'zombie' in text else 'Arcane spell','Staff']
 elif 'pickaxe' in text or 'axe-pick' in text:names=['Pickaxe','Axe']
 elif 'hammer' in text or 'forge-lord' in text:names=['Hammer']
 elif 'poleaxe' in text:names=['Poleaxe','Axe']
 elif 'axeguard' in text:names=['Axe','Shield bash']
 elif 'archer' in text or 'deer-archer' in text:names=['Bow','Knife']
 elif 'spearman' in text or 'banner-lord' in text or 'goat-rider' in text:names=['Spear']
 elif any(w in text for w in ['bladedancer','flanker','jumper','fighter','cleaver']):names=['Cleaver' if 'cleaver' in text else 'Blade']
 elif any(w in text for w in ['dragon','serpent','basilisk']):names=['Fire breath','Claws'] if 'dragon' in text else ['Bite']
 elif any(w in text for w in ['owl','raven','vulture','gryphon']):names=['Talons','Beak']
 elif any(w in text for w in ['crab','scorpion']):names=['Pincers']
 elif 'wasp' in text:names=['Sting']
 elif 'stag' in text or 'horse' in text:names=['Trample','Antlers' if 'stag' in text else 'Hooves']
 elif any(w in text for w in ['bear','lynx','werewolf']):names=['Claws','Bite']
 elif any(w in text for w in ['ray','octopus']):names=['Tail strike' if 'ray' in text else 'Tentacles']
 elif any(w in text for w in ['ghoul','walker','shambler','plague']):names=['Grasping hands','Bite']
 else:names=['Bite']
 return [dict(id=re.sub('[^a-z0-9]+','-',n.lower()),name=n,proposed=True) for n in names]
factions=[]
for file in sorted((game/'pixel/art/units/lineups').glob('*.json')):
 data=json.loads(file.read_text());members=[]
 def walk(obj,role='',base=''):
  if isinstance(obj,dict):
   role=obj.get('role',role);base=obj.get('base',{}).get('id',base)
   if 'id' in obj and 'level' in obj:
    member=obj;identifier=member['id'];gid=member.get('gameplayId',identifier);definition=definitions.get(gid)
    source=member.get('source',models.get(gid,gid));key='player_'+('bone_archer' if source=='bonearcher' else source)
    if key not in frames:key='player_'+models.get(gid,gid)
    entity=dict(id=identifier,key=f'{file.stem}/{identifier}',kind='unit',faction=file.stem,name=member['name'],level=member['level'],role=role or base,gameplayId=gid if definition else None,defined=bool(definition),artwork=image(key,f'workbench/units/{file.stem}/{identifier}.png'),attacks=definition.get('attacks',[]) if definition else inferred(member,role,base))
    members.append(entity)
   for value in obj.values():walk(value,role,base)
  elif isinstance(obj,list):
   for value in obj:walk(value,role,base)
 walk(data)
 unique={m['id']:m for m in members};members=list(unique.values())
 factions.append(dict(id=file.stem,name=file.stem.replace('-',' ').title(),units=members,definedCount=sum(m['defined'] for m in members)))
terrains=[]
for t in json.loads((game/'rust/data/game/terrain.json').read_text()):
 id=t['id'];key='terrain_'+('sacred_grove' if id=='grove' else id)
 terrains.append(dict(id=id,key='terrain/'+id,kind='terrain',name=t['name'],role=t.get('role',id),artwork=image(key,f'workbench/terrain/{id}.png'),description=t.get('description','')))
revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=game,text=True).strip()
(out/'workbench.json').write_text(json.dumps(dict(schemaVersion=1,sourceRevision=revision,factions=factions,terrains=terrains),indent=2)+'\n')
print(json.dumps(dict(factions=len(factions),units=sum(len(f['units']) for f in factions),terrain=len(terrains))))
