"""Keep recoverable latest/best snapshots on the authorized feature branch."""
import argparse
import fcntl
import hashlib
import gzip
import json
from pathlib import Path
import shutil
import subprocess
import time
from .recovery import atomic_json, resolve


def backup(root):
    root=Path(root);control=json.loads((root/'control.json').read_text());run=root/control['phase']
    if not (run/'latest.json').exists():return
    latest=resolve(run);best=resolve(run,'handoff-best.json') if (run/'handoff-best.json').exists() else resolve(run,'best.json') if (run/'best.json').exists() else latest
    destination=Path('training/checkpoints/overnight-recovery');destination.mkdir(parents=True,exist_ok=True)
    with (root/'git.lock').open('w') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        for name,folder in [('latest',latest),('best',best)]:
            temporary=destination/(name+'.tmp');shutil.copyfile(folder/'resume.pt',temporary);temporary.replace(destination/(name+'.pt'))
        if (root/'posttune/handoff-best.json').exists():
            selected=resolve(root/'posttune','handoff-best.json')
            temporary=destination/'posttune-selected.tmp';shutil.copyfile(selected/'resume.pt',temporary);temporary.replace(destination/'posttune-selected.pt')
            shutil.copyfile(root/'posttune/handoff-complete.json',destination/'posttune-selection.json')
        shutil.copyfile(root/'workload.json',destination/'workload.json')
        for phase in ['posttune','rl']:
            metrics=root/phase/'metrics.jsonl'
            if metrics.exists():
                temporary=destination/(phase+'-metrics.tmp')
                temporary.write_bytes(gzip.compress(metrics.read_bytes(),mtime=0))
                temporary.replace(destination/(phase+'-metrics.jsonl.gz'))
            parameters=root/phase/'parameters.json'
            if parameters.exists():shutil.copyfile(parameters,destination/(phase+'-parameters.json'))
        for source,name in [(root/'summary.json','summary.json'),(root/'final-scores.png','final-scores.png'),(root/'posttune/curves.png','posttune-curves.png'),(root/'rl/curves.png','rl-curves.png')]:
            if source.exists():shutil.copyfile(source,destination/name)
        if (root/'finished.json').exists():
            for phase in ['posttune','rl']:
                source=root/(phase+'-metrics.jsonl.gz')
                if source.exists():shutil.copyfile(source,destination/source.name)
        atomic_json(destination/'manifest.json',dict(phase=control['phase'],latest=json.loads((latest/'manifest.json').read_text()),
                    best=json.loads((best/'manifest.json').read_text()),backup_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
                    files={n:hashlib.sha256((destination/n).read_bytes()).hexdigest() for n in ['latest.pt','best.pt']}))
        subprocess.run(['git','add',str(destination)],check=True)
        changed=subprocess.run(['git','diff','--cached','--quiet','--',str(destination)]).returncode
        if changed:subprocess.run(['git','commit','-m',f"Back up {control['phase']} training recovery state"],check=True)
        subprocess.run(['git','push','origin','HEAD'],check=True)
        atomic_json(root/'backup-status.json',dict(ok=True,phase=control['phase'],time=time.time(),latest=str(latest),best=str(best)))


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--interval',type=int,default=300);a=p.parse_args()
    while True:
        try:backup(a.root)
        except Exception as e:
            atomic_json(a.root/'backup-status.json',dict(ok=False,error=repr(e),time=time.time()));print(repr(e),flush=True)
        if (a.root/'finished.json').exists():break
        time.sleep(a.interval)
if __name__=='__main__':main()
