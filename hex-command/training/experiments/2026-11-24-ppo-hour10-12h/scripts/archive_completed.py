"""Archive the completed frozen experiment; no training or evaluation calls."""
from pathlib import Path
import json,gzip,hashlib,shutil,tarfile
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[2]
DEST=REPO/'training/experiments/2026-11-24-ppo-hour10-12h'
finished=json.loads((ROOT/'finished.json').read_text())
records=json.loads((ROOT/'evaluation/hourly-summary.json').read_text())
assert len(records)==13 and [x['hour'] for x in records]==list(range(13))
assert finished['training']['elapsed']>=43200

def copy(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
def zipped(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True)
 with src.open('rb') as f,dst.open('wb') as raw,gzip.GzipFile(filename='',mode='wb',fileobj=raw,mtime=0) as out:shutil.copyfileobj(f,out)
for name in ['finished.json','protocol.json','runtime-versions.json','checkpoint-index.json','launch-verification.json','supervisor-recovery-test.json','snapshot-error.json','health.json']:
 if (ROOT/name).exists():copy(ROOT/name,DEST/name)
for name in ['supervisor.py','eval_runner.py','extra_plots.py','monitor_check.py','lantern_curves.py','archive_completed.py','summarize_completed.py']:
 copy(ROOT/name,DEST/'scripts'/name)
for name in ['trainer.log','evaluator.log','supervisor.log','allocation-history.jsonl','workload.json']:
 zipped(ROOT/name,DEST/(name+'.gz'))
for name in ['metrics.jsonl','sampler-status.json']:
 zipped(ROOT/'rl'/name,DEST/'rl'/(name+'.gz'))
for name in ['parameters.json','status.json','complete.json','latest.json','best.json']:
 if (ROOT/'rl'/name).exists():copy(ROOT/'rl'/name,DEST/'rl'/name)
for p in (ROOT/'monitor-checks').glob('*.json'):copy(p,DEST/'monitor'/p.name)
for p in (ROOT/'evaluation').rglob('*'):
 if p.is_file() and p.suffix in ['.json','.jsonl']:
  dst=DEST/p.relative_to(ROOT)
  if p.suffix=='.jsonl' or '/monitor/' in str(p):zipped(p,dst.with_name(dst.name+'.gz'))
  else:copy(p,dst)
for p in (ROOT/'plots').glob('*.png'):copy(p,DEST/'plots'/p.name)
checkpoints=[]
for record in records:
 hour=record['hour'];model=Path(record['model']);folder=model.parent
 if hour==0:
  files=['planner.json','resume.pt']
 else:
  meta=json.loads((folder/'manifest.json').read_text());files=list(meta['files'])+['manifest.json']
  for name,e in meta['files'].items():
   b=(folder/name).read_bytes();assert len(b)==e['bytes'] and hashlib.sha256(b).hexdigest()==e['sha256']
  shared=Path('/workspace/shared')/ROOT.name/'rl/checkpoints'/folder.name
  for name,e in meta['files'].items():assert hashlib.sha256((shared/name).read_bytes()).hexdigest()==e['sha256']
 for name in files:
  p=folder/name
  if p.exists():zipped(p,DEST/f'recovery/hour-{hour:02d}'/(name+'.gz'))
 for panel,n in [('heldout',312),('training_diagnostic',214),('lantern_showcase',24)]:
  report=json.loads((ROOT/f'evaluation/hour-{hour:02d}/monitor/{panel}.json').read_text());assert len(report['rows'])==report['runs']==n
  assert report['model_sha256']==hashlib.sha256(model.read_bytes()).hexdigest()
 checkpoints.append(dict(hour=hour,actual_seconds=record['actual_seconds'],model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),archive=f'recovery/hour-{hour:02d}/planner.json.gz',source_folder=str(folder.relative_to(ROOT))))
(DEST/'hourly-checkpoints.json').write_text(json.dumps(checkpoints,indent=2)+'\n')
with tarfile.open(DEST/'recovery/frozen-runtime-inputs.tar.gz','w:gz') as tar:
 for name in ['python','native','catalog']:
  for p in sorted((ROOT/name).rglob('*')):
   if p.is_file() and '__pycache__' not in p.parts and p.suffix!='.pyc':tar.add(p,arcname=str(p.relative_to(ROOT)))
print('Archived',len(records),'checkpoints;',sum(1 for p in DEST.rglob('*') if p.is_file()),'files')
