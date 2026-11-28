"""Read-only run checks plus retained monitor evidence; no training controls."""
from pathlib import Path
from collections import defaultdict
import json,time,math,hashlib,shutil,torch
import hex_command_training as native
from hex_command_rl.recovery import resolve_pointer
from hex_command_rl.mixed_workload import task,definition
ROOT=Path(__file__).resolve().parent;SHARED=Path('/workspace/shared')/ROOT.name

def jsonlines(path):
 rows=[]
 for line in path.read_text().splitlines():
  try:rows.append(json.loads(line))
  except json.JSONDecodeError:break
 return rows

def main():
 now=time.time();protocol=json.loads((ROOT/'protocol.json').read_text());spec=json.loads((ROOT/'workload.json').read_text());params=json.loads((ROOT/'rl/parameters.json').read_text());health=json.loads((ROOT/'health.json').read_text())
 assert now-health['time']<60
 assert not (ROOT/'stop').exists()
 assert params['seconds']==43200 and params['temperature']==.35 and params['envs']==4 and params['horizon']==64 and params['reference_kl_coefficient']==.02
 assert params['rewards']['arena_win']==1 and params['rewards']['campaign_chapter']==1 and params['rewards']['campaign_finale']==2
 computed=hashlib.sha256(json.dumps({k:v for k,v in spec.items() if k not in ['workload_sha256','catalog','fingerprints']},sort_keys=True).encode()).hexdigest()
 assert computed==spec['workload_sha256']==protocol['workload_sha256']
 assert json.loads(native.fingerprints())==spec['fingerprints']
 assert hashlib.sha256((ROOT/'native/hex_command_training.so').read_bytes()).hexdigest()==protocol['native_sha256']
 assert hashlib.sha256((ROOT/'warm-start/planner.json').read_bytes()).hexdigest()==protocol['start_model_sha256']
 processes={k:[] for k in ['trainer','evaluator','supervisor']}
 for path in Path('/proc').glob('[0-9]*/cmdline'):
  try:args=path.read_bytes().split(b'\0')
  except OSError:continue
  if b'hex_command_rl.mixed_ppo' in args and str(ROOT).encode() in args:processes['trainer'].append(int(path.parent.name))
  for role,name in [('evaluator','eval_runner.py'),('supervisor','supervisor.py')]:
   if str(ROOT/name).encode() in args:processes[role].append(int(path.parent.name))
 assert all(len(v)==1 for v in processes.values()),processes
 verified={}
 for label,base in [('local',ROOT),('shared',SHARED)]:
  info=json.loads((base/'rl/latest.json').read_text());folder=resolve_pointer(base/'rl',info);state=torch.load(folder/'resume.pt',map_location='cpu',weights_only=False)
  assert state['sampler']['schema']==3 and state['temperature']==.35 and state['workload_sha256']==spec['workload_sha256']
  verified[label]=dict(path=info['path'],elapsed=state['elapsed'],updates=state['update'],episodes=state['episodes'],modelSha256=hashlib.sha256((folder/'planner.json').read_bytes()).hexdigest())
 metrics=jsonlines(ROOT/'rl/metrics.jsonl');updates=[x for x in metrics if x['event']=='update']
 for update in updates[-30:]:
  for key in ['policy_loss','value_loss','entropy','reference_kl','critic_explained_variance','gradient_norm']:assert math.isfinite(update[key]),(key,update)
 report=json.loads((ROOT/'rl/sampler-status.json').read_text());settings=definition();maps=defaultdict(int);pools=defaultdict(int)
 for row in report['cases']:
  maps[row['bucket']+':'+task(row['config'],settings)]+=row['decisions'];pools[row['bucket']]+=row['decisions']
 assert len(maps)==45
 folder=ROOT/'monitor-checks';folder.mkdir(exist_ok=True);last=folder/'latest.json';previous=json.loads(last.read_text()) if last.exists() else None
 delta={k:v-(previous['maps'].get(k,0) if previous else 0) for k,v in maps.items()};zero=[k for k,v in delta.items() if v==0]
 assert not zero,zero
 if previous:assert updates[-1]['seconds']>previous['lastUpdate']['seconds']
 assert all(v>=0 for v in delta.values())
 total=sum(pools.values());shares={k:v/total for k,v in pools.items()};episodes=[x for x in metrics if x['event']=='episode'];training={}
 for pool in ['arena','targeted','campaign']:
  rows=[x for x in episodes if x['sampling_group']==pool][-50:];training[pool]=dict(lastEpisodes=len(rows),wins=sum(x['success'] for x in rows),caps=sum(x['truncated'] for x in rows),meanReturn=sum(x['episode_return'] for x in rows)/max(1,len(rows)))
 summary=json.loads((ROOT/'evaluation/hourly-summary.json').read_text());history=jsonlines(ROOT/'allocation-history.jsonl');index=json.loads((ROOT/'checkpoint-index.json').read_text())
 record=dict(time=now,healthAgeSeconds=now-health['time'],training=health['training'],processes=processes,validatedCheckpoints=verified,decisionShares=shares,pools=dict(pools),maps=dict(maps),deltaMaps=delta,zeroIncrementMaps=zero,previousCheck=previous['time'] if previous else None,lastUpdate=updates[-1],recentTraining=training,hourlyResults=[dict(hour=x['hour'],actualSeconds=x['actual_seconds'],panels=x['panels']) for x in summary],evaluationStatus=json.loads((ROOT/'evaluation/status.json').read_text()),nativeAndWorkloadUnchanged=True,fullWorkloadContentHashValidated=True,recoveryNeeded=False,diskFreeGiB=shutil.disk_usage(ROOT).free/2**30,allocationHistorySnapshots=len(history),allocationHistoryLastTime=history[-1]['time'] if history else None,checkpointCount=len(index))
 path=folder/f'check-{int(now)}.json';path.write_text(json.dumps(record,indent=2)+'\n');last.write_text(path.read_text());dest=SHARED/'monitor-checks';dest.mkdir(exist_ok=True);shutil.copyfile(path,dest/path.name);shutil.copyfile(last,dest/'latest.json')
 print(json.dumps({k:record[k] for k in ['time','training','processes','validatedCheckpoints','decisionShares','zeroIncrementMaps','lastUpdate','recentTraining','hourlyResults','evaluationStatus','diskFreeGiB']},indent=2))
if __name__=='__main__':main()
