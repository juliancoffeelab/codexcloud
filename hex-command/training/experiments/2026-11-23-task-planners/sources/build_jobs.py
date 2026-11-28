from pathlib import Path
import json,gzip
ROOT=Path.cwd();OUT=ROOT/'tmp/rl/task-planners-20261123'
COORDS={
 'rl-deadline-wait7':dict(task='crown',anchor=(16,4),home=(2,7),staging=(4,7)),
 'rl-briar-saddle':dict(task='briar',anchor=(6,3),home=(1,2),staging=(2,4),healer_home=(2,3)),
 'rl-vault-rally':dict(task='rally',anchor=(14,10),home=(6,24),staging=(9,25)),
 'rl-defend-tidegate-stand-110':dict(task='tide',anchor=(6,5),home=(2,5),staging=(5,4),supply=(9,1),guard1=(7,4),guard2=(7,5),guard3=(6,6)),
 'rl-defend-lower-gallery-0':dict(task='gallery',anchor=(15,25),home=(9,5),staging=(13,23),guard1=(16,26),guard2=(16,25),guard3=(15,26)),
}
with gzip.open(ROOT/'training/experiments/2026-11-23-targeted-curriculum-audit/fresh.jsonl.gz','rt') as f:rows=[json.loads(l) for l in f]
jobs=[]
for cid,coords in COORDS.items():
 field=json.loads((ROOT/'rust/data/game/campaigns'/cid/'map.json').read_text());tiles=field['tiles'];minr=min(t['row'] for t in tiles);minq=min(t['col']-t['row']//2 for t in tiles);w=max(t['col']-t['row']//2 for t in tiles)-minq+1
 p={k:((v[1]-minr)*w+v[0]-v[1]//2-minq if isinstance(v,tuple) else v) for k,v in coords.items()}
 for r in rows:
  if r['config']['campaign']==cid:
   for variant in [0,1]:jobs.append(dict(config=r['config'],plan=dict(p,variant=variant),trace=r['config']['seed']==min(x['config']['seed'] for x in rows if x['config']['campaign']==cid)))
(OUT/'discovery-jobs.json').write_text(json.dumps(jobs,indent=2)+'\n')
print('Role-planner screen:',len(jobs),'matches, eight existing conditions')
