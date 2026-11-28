from pathlib import Path
import json
root=Path(__file__).resolve().parent;spec=json.loads((root/'workload.json').read_text());panels={'targeted':[],'ordinary_retention':[]}
seen=set()
for c in spec['imitation_validation']:
 key=(c['campaign'],c['objective'],json.dumps(c.get('enemyBlocks',{}),sort_keys=True),c['enemy'])
 if key in seen:continue
 seen.add(key)
 for seed in range(496110001,496110009):panels['targeted'].append(dict(c,seed=seed))
ordinary={a['id'] for a in json.loads((root/'catalog/simulation/rl.json').read_text())['arenas'] if a['samplingGroup']=='arena'}
seen=set()
for c in spec['validation']:
 if 'campaign' in c or c['landscape'] not in ordinary or c['landscape'] in seen:continue
 seen.add(c['landscape'])
 for seed in range(496120001,496120003):panels['ordinary_retention'].append(dict(c,seed=seed))
old=json.loads((root.parent/'ppo-from-pretrain-5m-9h-20261123/workload.json').read_text());panels['lantern_showcase']=old['evaluation_panels']['lantern_showcase']
assert len(panels['targeted'])==128 and len(panels['ordinary_retention'])==28 and len(panels['lantern_showcase'])==24
(root/'evaluation-panels.json').write_text(json.dumps(panels,indent=2)+'\n')
p=root/'protocol.json';j=json.loads(p.read_text());j['gameplay_panel']={k:len(v) for k,v in panels.items()};j['gameplay_seeds']='Targeted496110001..008 (eight per eligible condition, one configured recruiter); ordinary496120001..002; Lantern unchanged24-case historical showcase';j['gameplay_limitations']='Focused pilot, not statistical confirmation or broad campaign selection. No automatic promotion.';p.write_text(json.dumps(j,indent=2)+'\n')
print(j['gameplay_panel'])
