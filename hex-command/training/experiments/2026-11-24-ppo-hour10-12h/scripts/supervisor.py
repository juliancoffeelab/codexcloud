"""Supervise one trainer/evaluator; mirror only validated checkpoint bundles."""
import fcntl,json,os,shutil,subprocess,sys,time,hashlib,signal
from pathlib import Path
from hex_command_rl.recovery import atomic_json,resolve_pointer,load_resume
ROOT=Path(__file__).resolve().parent;SHARED=Path('/workspace/shared')/ROOT.name

def alive(pid):
 try:return Path(f'/proc/{pid}/stat').read_text().split(') ',1)[1][0]!='Z' and str(ROOT) in Path(f'/proc/{pid}/cmdline').read_text()
 except (OSError,IndexError):return False

def copy(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);tmp=dst.with_name('.'+dst.name+'.tmp');shutil.copy2(src,tmp);os.replace(tmp,dst)

def validate(folder):
 meta=json.loads((folder/'manifest.json').read_text())
 for name,expected in meta['files'].items():
  p=folder/name
  if p.stat().st_size!=expected['bytes'] or hashlib.sha256(p.read_bytes()).hexdigest()!=expected['sha256']:raise ValueError(f'Corrupt checkpoint: {p}')
 return meta

def snapshot():
 run=ROOT/'rl';target=SHARED/'rl';index=json.loads((ROOT/'checkpoint-index.json').read_text()) if (ROOT/'checkpoint-index.json').exists() else {}
 for folder in (run/'checkpoints').glob('*'):
  if folder.name.startswith('.') or not (folder/'manifest.json').exists():continue
  if folder.name not in index:
   meta=validate(folder);state=load_resume(folder/'resume.pt');index[folder.name]=dict(path=str(folder.relative_to(run)),elapsed=state['elapsed'],updates=state['update'],episodes=state['episodes'],model_sha256=meta['files']['planner.json']['sha256'])
  dst=target/'checkpoints'/folder.name
  if not (dst/'manifest.json').exists():
   staging=dst.with_name('.'+dst.name);staging.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(folder,staging,dirs_exist_ok=True);validate(staging)
   if dst.exists():raise ValueError(f'Incomplete final checkpoint destination: {dst}')
   staging.rename(dst)
 atomic_json(ROOT/'checkpoint-index.json',index);copy(ROOT/'checkpoint-index.json',SHARED/'checkpoint-index.json')
 for name in ['latest.json','best.json']:
  if (run/name).exists():
   pointer=json.loads((run/name).read_text());resolve_pointer(target,pointer);atomic_json(target/name,pointer)
 for name in ['status.json','complete.json','metrics.jsonl','parameters.json','sampler-status.json']:
  if (run/name).exists():copy(run/name,target/name)
 for name in ['supervisor.py','eval_runner.py','protocol.json','control.json','health.json','finished.json','failed.json','snapshot-error.json','launch-verification.json']:
  if (ROOT/name).exists():copy(ROOT/name,SHARED/name)
 for folder in ['evaluation','plots']:
  if not (ROOT/folder).exists():continue
  for src in (ROOT/folder).rglob('*'):
   if src.is_file() and not any(p.startswith('.') for p in src.relative_to(ROOT/folder).parts) and src.suffix!='.tmp':
    try:copy(src,SHARED/src.relative_to(ROOT))
    except FileNotFoundError:pass
 # A separate lightweight exposure history is a monitor input, not a sampling control.
 if (run/'sampler-status.json').exists():
  report=json.loads((run/'sampler-status.json').read_text());maps={};pools={}
  from hex_command_rl.mixed_workload import task,definition
  settings=definition()
  for row in report['cases']:
   key=row['bucket']+':'+task(row['config'],settings);maps[key]=maps.get(key,0)+row['decisions'];pools[row['bucket']]=pools.get(row['bucket'],0)+row['decisions']
  row=dict(time=time.time(),maps=maps,pools=pools)
  with (ROOT/'allocation-history.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
  copy(ROOT/'allocation-history.jsonl',SHARED/'allocation-history.jsonl')

def main():
 lock=(ROOT/'supervisor.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 (ROOT/'supervisor.pid').write_text(str(os.getpid()));atomic_json(ROOT/'control.json',dict(phase='ppo',budget_seconds=43200,warm_start=str(ROOT/'warm-start/resume.pt'),seed_start=510000001))
 launches={'trainer':0,'evaluator':0};failures={'trainer':[],'evaluator':[]};last_snapshot=0;children={}
 while True:
  if (ROOT/'stop').exists():
   for role in launches:
    p=ROOT/(role+'.pid');pid=int(p.read_text()) if p.exists() else 0
    if alive(pid):os.killpg(pid,signal.SIGTERM)
   atomic_json(ROOT/'stopped.json',dict(time=time.time()));snapshot();return
  for role in launches:
   p=ROOT/(role+'.pid');pid=int(p.read_text()) if p.exists() else 0
   done=(ROOT/'rl/complete.json').exists() if role=='trainer' else (ROOT/'evaluation/complete.json').exists()
   if done or alive(pid):continue
   failures[role]=[t for t in failures[role] if time.time()-t<600]
   if len(failures[role])>=5:atomic_json(ROOT/'failed.json',dict(time=time.time(),role=role,reason='Five exits within ten minutes'));snapshot();return
   if role=='trainer':
    args=[sys.executable,'-m','hex_command_rl.mixed_ppo','--root',str(ROOT),'--seconds','43200','--envs','4','--horizon','64','--temperature','.35','--campaign-cap','-2','--critic-warmup-episodes','24']
    if (ROOT/'rl/latest.json').exists():resolve_pointer(ROOT/'rl',json.loads((ROOT/'rl/latest.json').read_text()));args+=['--resume',str(ROOT/'rl')]
    else:args+=['--warm-start',str(ROOT/'warm-start/resume.pt'),'--seed-start','510000001']
   else:args=[sys.executable,str(ROOT/'eval_runner.py')]
   with (ROOT/(role+'.log')).open('a') as log:child=subprocess.Popen(args,stdout=log,stderr=log,stdin=subprocess.DEVNULL,start_new_session=True)
   children[role]=child;p.write_text(str(child.pid));launches[role]+=1;failures[role].append(time.time())
  status=json.loads((ROOT/'rl/status.json').read_text()) if (ROOT/'rl/status.json').exists() else None
  atomic_json(ROOT/'health.json',dict(time=time.time(),supervisor=os.getpid(),trainer=int((ROOT/'trainer.pid').read_text()),evaluator=int((ROOT/'evaluator.pid').read_text()),training=status,launches=launches))
  if time.time()-last_snapshot>=30:
   try:snapshot()
   except Exception as error:atomic_json(ROOT/'snapshot-error.json',dict(time=time.time(),error=repr(error)));print('Snapshot retry:',repr(error),flush=True)
   last_snapshot=time.time()
  if (ROOT/'rl/complete.json').exists() and (ROOT/'evaluation/complete.json').exists():
   snapshot();atomic_json(ROOT/'finished.json',dict(time=time.time(),training=json.loads((ROOT/'rl/complete.json').read_text()),evaluation=json.loads((ROOT/'evaluation/complete.json').read_text())));copy(ROOT/'finished.json',SHARED/'finished.json');return
  time.sleep(5)
if __name__=='__main__':main()
