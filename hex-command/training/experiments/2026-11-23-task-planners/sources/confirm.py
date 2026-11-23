from pathlib import Path
import sys,json,gzip,subprocess
ROOT=Path.cwd();OUT=ROOT/'tmp/rl/task-planners-20261123';sys.path.insert(0,str(ROOT/'training'))
from hex_command_rl.mixed_workload import pretrain_imitator
rows=json.loads((OUT/'discovery-v2-jobs.json').read_text());jobs=[];learners=[]
for task in ['crown','briar','rally']:
 base=next(j for j in rows if j['plan']['task']==task and j['plan']['variant']==0)
 for seed in range(493110001,493110021):
  c=dict(base['config'],seed=seed);learners.append(c)
  jobs.append(dict(config=c,plan=base['plan'],trace=seed==493110001))
  jobs.append(dict(config=c,teacher=pretrain_imitator(c)))
(OUT/'confirmation-jobs.json').write_text(json.dumps(jobs,indent=2)+'\n')
with (OUT/'confirmation-teachers.jsonl').open('w') as f:subprocess.run([str(ROOT/'rust/target/release/task_planners'),str(ROOT/'rust/data'),str(OUT/'confirmation-jobs.json')],stdout=f,check=True)
import hex_command_training as native
RUN=ROOT/'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
model=next(Path(r['model']).read_text() for r in json.loads((RUN/'evaluation/hourly-summary.json').read_text()) if r['hour']==10)
with (OUT/'confirmation-learner.jsonl').open('w') as f:
 for c in learners:
  r=json.loads(native.evaluate([json.dumps(c)],model)[0]);f.write(json.dumps(dict(config=c,result=r))+'\n');f.flush()
print('Finished60freshcases:120teacher+60learnergames',flush=True)
