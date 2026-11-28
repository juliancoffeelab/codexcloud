from concurrent.futures import ProcessPoolExecutor,as_completed
import multiprocessing,json,hashlib,os,time,fcntl,shutil
from pathlib import Path
from hex_command_rl.mixed_evaluate import job,summarize
from hex_command_rl.recovery import atomic_json,read_jsonl
ROOT=Path(__file__).resolve().parent;SHARED=Path('/workspace/shared')/ROOT.name
os.environ['HEX_COMMAND_DATA_ROOT']=str(ROOT/'catalog')
def snapshot(src):
 dst=SHARED/src.relative_to(ROOT);dst.parent.mkdir(parents=True,exist_ok=True);tmp=dst.with_name('.'+dst.name+'.tmp');shutil.copy2(src,tmp);os.replace(tmp,dst)
def main():
 lock=(ROOT/'evaluation.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 (ROOT/'evaluator.pid').write_text(str(os.getpid()));panels=json.loads((ROOT/'evaluation-panels.json').read_text());out=ROOT/'evaluation';out.mkdir(exist_ok=True)
 snapshot(ROOT/'evaluation-panels.json');snapshot(ROOT/'eval_runner.py')
 records=[]
 for minute in [0,1,3,5]:
  model=ROOT/'hour10-planner.json' if minute==0 else None
  while model is None:
   if (ROOT/'stop').exists():return
   matches=sorted((ROOT/'posttune/checkpoints').glob(f'minute-{minute:04d}-*'))
   if minute==5 and not matches and (ROOT/'posttune/complete.json').exists():matches=sorted((ROOT/'posttune/checkpoints').glob('last-*'))
   if matches and (matches[0]/'manifest.json').exists():model=matches[0]/'planner.json'
   else:time.sleep(5)
  folder=out/f'minute-{minute:02d}';folder.mkdir(exist_ok=True);record=dict(minute=minute,model=str(model),model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),panels={})
  for name,cases in panels.items():
   path=folder/(name+'.jsonl');rows=read_jsonl(path,repair=True);done={json.dumps(r['config'],sort_keys=True) for r in rows};todo=[c for c in cases if json.dumps(c,sort_keys=True) not in done]
   started=time.monotonic()
   with ProcessPoolExecutor(max_workers=2,mp_context=multiprocessing.get_context('spawn')) as pool:
    with path.open('a') as stream:
     futures=[pool.submit(job,([c],str(model))) for c in todo]
     for future in as_completed(futures):
      batch=future.result();rows.extend(batch)
      for row in batch:stream.write(json.dumps(row)+'\n')
      stream.flush();atomic_json(out/'status.json',dict(time=time.time(),minute=minute,panel=name,completed=len(rows),total=len(cases),wall_seconds=time.monotonic()-started))
      snapshot(out/'status.json');snapshot(path)
   assert len(rows)==len(cases)
   report=summarize(rows);report.update(model_sha256=record['model_sha256'],minute=minute,panel=name);atomic_json(folder/(name+'.json'),report);snapshot(folder/(name+'.json'))
   record['panels'][name]=dict(wins=report['wins'],runs=report['runs'],caps=report['caps'],score=report['score']);print(json.dumps(dict(minute=minute,panel=name,**record['panels'][name])),flush=True)
  records.append(record);atomic_json(out/'summary.json',records);snapshot(out/'summary.json')
 atomic_json(out/'complete.json',dict(time=time.time(),records=records));snapshot(out/'complete.json')
if __name__=='__main__':main()
