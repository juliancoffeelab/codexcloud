import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from hex_command_rl.recovery import atomic_json, resolve_pointer

root=Path(__file__).parent.resolve()
shared=Path('/workspace/shared')/root.name

def alive(pid):
    try:
        return (Path(f'/proc/{pid}/stat').read_text().split(') ',1)[1][0]!='Z'
                and str(root) in Path(f'/proc/{pid}/cmdline').read_text())
    except (OSError,IndexError):return False

def copy(src,dst):
    dst.parent.mkdir(parents=True,exist_ok=True)
    temp=dst.with_name('.'+dst.name+'.tmp');shutil.copy2(src,temp);os.replace(temp,dst)

def snapshot():
    run=root/'rl';target=shared/'rl'
    index=json.loads((root/'checkpoint-index.json').read_text()) if (root/'checkpoint-index.json').exists() else {}
    from hex_command_rl.recovery import load_resume
    for folder in (run/'checkpoints').glob('*'):
        if folder.name.startswith('.') or not (folder/'manifest.json').exists() or folder.name in index:continue
        state=load_resume(folder/'resume.pt')
        index[folder.name]=dict(path=str(folder.relative_to(run)),elapsed=state['elapsed'],updates=state['update'],episodes=state['episodes'])
    atomic_json(root/'checkpoint-index.json',index)
    copy(root/'checkpoint-index.json',shared/'checkpoint-index.json')
    for folder in (run/'checkpoints').glob('*'):
        if folder.name.startswith('.') or not (folder/'manifest.json').exists():continue
        destination=target/'checkpoints'/folder.name
        if not destination.exists():
            destination.parent.mkdir(parents=True,exist_ok=True)
            staging=destination.with_name('.'+destination.name)
            shutil.rmtree(staging,ignore_errors=True);shutil.copytree(folder,staging);staging.rename(destination)
    for name in ['latest.json','best.json']:
        if (run/name).exists():
            pointer=json.loads((run/name).read_text());resolve_pointer(target,pointer)
            atomic_json(target/name,pointer)
    for name in ['status.json','complete.json','metrics.jsonl','parameters.json','sampler-status.json']:
        if (run/name).exists():copy(run/name,target/name)
    for name in ['supervisor.py','eval_runner.py','control.json','health.json','finished.json','failed.json']:
        if (root/name).exists():copy(root/name,shared/name)
    for folder in ['evaluation','plots']:
        if (root/folder).exists():
            for path in (root/folder).rglob('*'):
                if path.is_file():copy(path,shared/folder/path.relative_to(root/folder))

def main():
    lock=(root/'supervisor.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    (root/'supervisor.pid').write_text(str(os.getpid()))
    control=dict(budget_seconds=32400,phase='ppo',warm_start=str(root/'warm-start/resume.pt'))
    atomic_json(root/'control.json',control)
    children={};failures={'trainer':0,'evaluator':0};last_snapshot=0
    while True:
        if (root/'stop').exists():
            for child in children.values():
                if child.poll() is None:child.terminate()
            atomic_json(root/'stopped.json',dict(time=time.time()));snapshot();return
        for role in ['trainer','evaluator']:
            pidfile=root/(role+'.pid')
            pid=int(pidfile.read_text()) if pidfile.exists() else 0
            complete=(root/'rl/complete.json').exists() if role=='trainer' else (root/'evaluation/complete.json').exists()
            if role=='evaluator' and not (root/'eval_runner.py').exists():continue
            if not complete and not alive(pid):
                if failures[role]>=5:
                    atomic_json(root/'failed.json',dict(role=role,time=time.time(),reason='five child exits'));snapshot();return
                if role=='trainer':
                    args=[sys.executable,'-m','hex_command_rl.mixed_ppo','--root',str(root),
                          '--seconds','32400','--envs','4','--horizon','64','--temperature','.35','--campaign-cap','-2']
                    if (root/'rl/latest.json').exists():args+=['--resume',str(root/'rl')]
                    else:args+=['--warm-start',str(root/'warm-start/resume.pt'),'--seed-start','450000001']
                else:args=[sys.executable,str(root/'eval_runner.py')]
                with (root/(role+'.log')).open('a') as log:
                    child=subprocess.Popen(args,stdout=log,stderr=log,stdin=subprocess.DEVNULL,start_new_session=True)
                children[role]=child;failures[role]+=1;pidfile.write_text(str(child.pid))
        status=json.loads((root/'rl/status.json').read_text()) if (root/'rl/status.json').exists() else None
        atomic_json(root/'health.json',dict(time=time.time(),supervisor=os.getpid(),
                    trainer=int((root/'trainer.pid').read_text()),
                    evaluator=int((root/'evaluator.pid').read_text()) if (root/'evaluator.pid').exists() else None,
                    training=status,launches=failures))
        if time.time()-last_snapshot>=30:
            snapshot();last_snapshot=time.time()
        if (root/'rl/complete.json').exists() and (root/'evaluation/complete.json').exists():
            atomic_json(root/'finished.json',dict(time=time.time(),
                        training=json.loads((root/'rl/complete.json').read_text()),
                        evaluation=json.loads((root/'evaluation/complete.json').read_text())))
            snapshot();return
        time.sleep(15)

if __name__=='__main__':main()
