from pathlib import Path
import json,shutil,subprocess,sys
ROOT=Path.cwd();p=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
read=lambda p:json.loads(p.read_text())
campaign=read(p/'campaign.json');roster=read(p/'roster.json');mp=read(p/'map.json')
for name in ['map.json','roster.json']:shutil.copy2(p/name,out/name)
# Render-only art connection metadata; gameplay catalog remains byte-identical.
render_map=read(out/'map.json')
for tile in render_map['tiles']:
    if tile['terrain']=='signpost' and not tile.get('connections'):tile['connections']=['e','w']
(out/'map.json').write_text(json.dumps(render_map))
chars={c['id']:c for c in campaign.get('characters',[])}
for f in roster['factions']:
 for u in f['units']:
  if u.get('character') and u['character'] not in chars:chars[u['character']]=dict(id=u['character'],name=u['character'].replace('-',' ').title())
scenario=dict(map='map.json',roster='roster.json',fixedArmies=True,characters=list(chars.values()),tags=['simulation'])
(out/'scenario.json').write_text(json.dumps(scenario))
annotations=[]
def positions(x):
 found=[]
 if isinstance(x,dict):
  if 'hex' in x and isinstance(x['hex'],dict) and 'col' in x['hex']:found.append(x['hex'])
  else:
   for v in x.values():found+=positions(v)
 elif isinstance(x,list):
  for v in x:found+=positions(v)
 return found
for event in read(p/'scripts.json')['events']:
 objective=event.get('objective')
 if objective:
  for pos in positions(objective.get('intent',{})):
   annotations.append(dict(**pos,text=('Post' if 'capture' in objective.get('intent',{}) else 'Exit' if 'goTo' in objective.get('intent',{}) else objective['label'][:20]),color='#e7cb7a',dy=-40,ring=True))
overlay=dict(title=campaign['name'],notes=[campaign.get('description','')],annotations=annotations)
(out/'overlay.json').write_text(json.dumps(overlay))
cmd=['node','tmp/rl/current-arena-renders/renderer.mjs','--map',str(out/'map.json'),'--roster',str(out/'roster.json'),'--scenario',str(out/'scenario.json'),'--overlay',str(out/'overlay.json'),'--reveal-terrain','--output',str(out/'preview.png')]
subprocess.run(cmd,check=True)
print(out/'preview.png')
