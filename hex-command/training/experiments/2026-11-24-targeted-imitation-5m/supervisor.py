import fcntl,json,os,shutil,subprocess,sys,time
from pathlib import Path
from hex_command_rl.recovery import atomic_json,resolve
ROOT=Path(__file__).resolve().parent;SHARED=Path('/workspace/shared')/ROOT.name

def copy(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);tmp=dst.with_name('.'+dst.name+'.tmp');shutil.copy2(src,tmp);os.replace(tmp,dst)
def snapshot():
 for name in ['protocol.json','workload.json','supervisor.py','health.json','finished.json','failed.json']:
  if (ROOT/name).exists():copy(ROOT/name,SHARED/name)
 for folder in ['posttune','corpus']:
  if not (ROOT/folder).exists():continue
  for src in (ROOT/folder).rglob('*'):
   if not src.is_file() or src.suffix=='.tmp' or any(p.startswith('.') for p in src.relative_to(ROOT/folder).parts):continue
   dst=SHARED/src.relative_to(ROOT)
   try:
    if dst.exists() and dst.stat().st_size==src.stat().st_size and dst.stat().st_mtime_ns==src.stat().st_mtime_ns:continue
    copy(src,dst)
   except FileNotFoundError:continue
 if (SHARED/'posttune/latest.json').exists():resolve(SHARED/'posttune')
def alive(pid):
 try:return Path(f'/proc/{pid}/stat').read_text().split(') ',1)[1][0]!='Z' and str(ROOT) in Path(f'/proc/{pid}/cmdline').read_text()
 except (OSError,IndexError):return False
def main():
 lock=(ROOT/'supervisor.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 (ROOT/'supervisor.pid').write_text(str(os.getpid()));child=None;launches=0;last_snapshot=0
 pid=int((ROOT/"trainer.pid").read_text()) if (ROOT/"trainer.pid").exists() else 0
 while True:
  if (ROOT/'stop').exists():
   if alive(pid):os.kill(pid,15)
   atomic_json(ROOT/'stopped.json',dict(time=time.time()));snapshot();return
  if (ROOT/'posttune/complete.json').exists():
   resolve(ROOT/'posttune');atomic_json(ROOT/'finished.json',dict(time=time.time(),training=json.loads((ROOT/'posttune/complete.json').read_text())));snapshot();return
  if not alive(pid):
   if launches>=4:atomic_json(ROOT/'failed.json',dict(time=time.time(),launches=launches,reason='four child exits'));snapshot();return
   args=[sys.executable,'-m','hex_command_rl.mixed_imitation','--root',str(ROOT),'--seconds','300','--warm-start',str(ROOT/'hour10-policy.pt'),'--threads','2','--sequence-batch-size','1','--sequence-length','8','--checkpoint-seconds','60','--collection-workers','2','--games-per-shard','1']
   if (ROOT/'posttune/latest.json').exists():resolve(ROOT/'posttune');args+=['--resume',str(ROOT/'posttune')]
   with (ROOT/'trainer.log').open('a') as log:child=subprocess.Popen(args,stdout=log,stderr=log,stdin=subprocess.DEVNULL,start_new_session=True)
   launches+=1;pid=child.pid;(ROOT/'trainer.pid').write_text(str(pid))
  status=json.loads((ROOT/'posttune/status.json').read_text()) if (ROOT/'posttune/status.json').exists() else None
  atomic_json(ROOT/'health.json',dict(time=time.time(),supervisor=os.getpid(),trainer=pid,launches=launches,status=status))
  if time.time()-last_snapshot>=15:snapshot();last_snapshot=time.time()
  time.sleep(5)
if __name__=='__main__':main()
