from pathlib import Path
import json,gzip,hashlib,shutil,time
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[2];OUT=REPO/'training/experiments/2026-11-24-targeted-imitation-5m'
def zipped(src,dst):dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(gzip.compress(src.read_bytes(),compresslevel=6,mtime=0))
def main():
 assert (ROOT/'posttune/complete.json').exists() and (ROOT/'evaluation/complete.json').exists()
 train=json.loads((ROOT/'posttune/complete.json').read_text());assert train['elapsed']>=300
 analysis=json.loads((ROOT/'analysis.json').read_text());assert {r['minute'] for r in analysis['records']}=={0,1,3,5}
 OUT.mkdir(parents=True,exist_ok=True)
 for name in ['protocol.json','analysis.json','baseline-parity.json','supervisor.py','eval_runner.py','prepare_panel.py','report.py','archive.py']:
  shutil.copyfile(ROOT/name,OUT/name)
 for name in ['workload.json','evaluation-panels.json','trainer.log','evaluator.log','supervisor.log','posttune/metrics.jsonl','posttune/complete.json','corpus/train/manifest.json','corpus/validation/manifest.json']:
  zipped(ROOT/name,OUT/(name+'.gz'))
 shutil.copytree(ROOT/'evaluation',OUT/'evaluation',dirs_exist_ok=True)
 shutil.copytree(ROOT/'plots',OUT/'plots',dirs_exist_ok=True)
 shutil.copyfile(ROOT/'catalog/simulation/rl.json',OUT/'authored-eligibility.json')
 checkpoints=[]
 for record in analysis['records']:
  minute=record['minute'];model=Path(record['model']);destination=OUT/'recovery'/f'minute-{minute:02d}';destination.mkdir(parents=True,exist_ok=True)
  zipped(model,destination/'planner.json.gz')
  if minute:
   folder=model.parent;manifest=json.loads((folder/'manifest.json').read_text())
   for name,meta in manifest['files'].items():assert hashlib.sha256((folder/name).read_bytes()).hexdigest()==meta['sha256']
   zipped(folder/'resume.pt',destination/'resume.pt.gz');shutil.copyfile(folder/'manifest.json',destination/'manifest.json')
  else:zipped(ROOT/'hour10-policy.pt',destination/'policy.pt.gz')
  checkpoints.append(dict(minute=minute,actual_active_seconds=record['active_seconds'],model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),original_path=str(model),archive_path=str(destination.relative_to(OUT))))
 (OUT/'checkpoint-index.json').write_text(json.dumps(checkpoints,indent=2)+'\n')
 finished=dict(time=time.time(),training=train,evaluation=json.loads((ROOT/'evaluation/complete.json').read_text()),archive=str(OUT),automatic_promotion=False)
 (ROOT/'finished.json').write_text(json.dumps(finished,indent=2)+'\n');shutil.copyfile(ROOT/'finished.json',Path('/workspace/shared')/ROOT.name/'finished.json')
 print(OUT)
if __name__=='__main__':main()
