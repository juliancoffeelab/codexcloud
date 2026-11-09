"""Post-tune → six-hour PPO → held-out report, with restartable phase control."""
import argparse
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from .mixed_evaluate import evaluate,plot
from .recovery import atomic_json,resolve


def live_pid(module,root):
    for entry in Path('/proc').iterdir():
        if not entry.name.isdigit():continue
        try:parts=(entry/'cmdline').read_bytes().decode().split('\0')
        except (OSError,UnicodeError):continue
        if module in parts and '--root' in parts:
            try:
                if Path(parts[parts.index('--root')+1]).resolve()==root.resolve():return int(entry.name)
            except (ValueError,IndexError):pass
    return None


def resources(root):
    rows=[]
    for phase,module in [('posttune','hex_command_rl.mixed_imitation'),('rl','hex_command_rl.mixed_ppo')]:
        pid=live_pid(module,root)
        if pid:
            try:status=Path(f'/proc/{pid}/status').read_text()
            except FileNotFoundError:continue
            rss=next((line for line in status.splitlines() if line.startswith('VmRSS:')),'')
            rows.append(dict(phase=phase,pid=pid,rss=rss))
    disk=os.statvfs(root)
    return dict(time=time.time(),processes=rows,disk_free_bytes=disk.f_bavail*disk.f_frsize)


def final_report(root):
    from .mixed_evaluate import evaluate
    from .mixed_workload import group
    import hashlib
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    spec=json.loads((root/'workload.json').read_text());os.environ['HEX_COMMAND_DATA_ROOT']=spec['catalog']
    models={'original':'training/checkpoints/campaign-cnn-imitation-v1/planner.json',
            'posttune':str(resolve(root/'posttune','handoff-best.json')/'planner.json'),
            'rl_best':str(resolve(root/'rl','best.json')/'planner.json'),
            'rl_last':str(resolve(root/'rl')/'planner.json')}
    reports={};cache={}
    for label,path in models.items():
        digest=hashlib.sha256(Path(path).read_bytes()).hexdigest()
        if digest not in cache:
            report=evaluate(spec['test'],path,workers=3);atomic_json(root/(label+'-test.json'),report);cache[digest]=report
        reports[label]=cache[digest]
        print(json.dumps(dict(event='final_evaluation',model=label,score=reports[label]['score'],wins=reports[label]['wins'],caps=reports[label]['caps'],runs=reports[label]['runs'])),flush=True)
    compact={label:{k:v for k,v in r.items() if k!='rows'} for label,r in reports.items()}
    atomic_json(root/'summary.json',dict(models=models,results=compact,test_seeds_used_for_selection=False))
    fig,axes=plt.subplots(1,2,figsize=(12,5));labels=list(reports)
    for ax,family in zip(axes,['arena','campaign']):
        values=[]
        for label in labels:
            rows=[r for r in reports[label]['rows'] if ('campaign' in r['config'])==(family=='campaign')]
            from .mixed_evaluate import summarize
            values.append(100*summarize(rows)['score'])
        ax.bar(labels,values);ax.set(title=family+' · balanced final held-out score',ylabel='Win / completion score (%)',ylim=(0,100));ax.tick_params(axis='x',rotation=15);ax.grid(axis='y',alpha=.2)
    fig.tight_layout();fig.savefig(root/'final-scores.png',dpi=150);plt.close(fig)
    for phase in ['posttune','rl']:plot(root/phase)
    for phase in ['posttune','rl']:
        with (root/phase/'metrics.jsonl').open('rb') as source,gzip.open(root/(phase+'-metrics.jsonl.gz'),'wb') as out:
            import shutil
            shutil.copyfileobj(source,out)


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--rl-seconds',type=int,default=32400);a=p.parse_args();root=a.root
    spec=json.loads((root/'workload.json').read_text());os.environ['HEX_COMMAND_DATA_ROOT']=spec['catalog'];handles=[];children=[];last_resource=last_report=0.;retries=0
    def launch(module,phase,extra):
        f=(root/(phase+'.log')).open('a');handles.append(f)
        child=subprocess.Popen([sys.executable,'-m',module,'--root',str(root),*extra],stdout=f,stderr=subprocess.STDOUT,start_new_session=True);children.append(child);return child
    def resource_tick():
        nonlocal last_resource,last_report
        now=time.time()
        if now-last_resource>=60:
            row=resources(root)
            with (root/'resources.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
            last_resource=now
        if now-last_report>=1800:
            state=json.loads((root/'control.json').read_text());print(json.dumps(dict(event='monitor',phase=state['phase'],utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),resources=resources(root))),flush=True);last_report=now
    def ensure_post_evaluator():
        if not (root/'posttune/handoff-complete.json').exists() and not live_pid('hex_command_rl.handoff',root):
            launch('hex_command_rl.handoff','handoff',[])
        if not (root/'posttune/evaluation-complete.json').exists() and not live_pid('hex_command_rl.mixed_evaluate',root):
            (root/'posttune/evaluation-active.json').unlink(missing_ok=True)
            launch('hex_command_rl.mixed_evaluate','posttune-evaluation',['--phase','posttune','--interval','600'])
    while not (root/'posttune/complete.json').exists():
        resource_tick();ensure_post_evaluator()
        if not live_pid('hex_command_rl.mixed_imitation',root):
            if retries>=2:raise RuntimeError('post-tune failed repeatedly; inspect posttune.log')
            launch('hex_command_rl.mixed_imitation','posttune',['--seconds','5400','--resume',str(root/'posttune')]);retries+=1
            time.sleep(10)
        time.sleep(5)
    while not (root/'posttune/evaluation-complete.json').exists() or not (root/'posttune/handoff-complete.json').exists() or not (root/'sampler-ready.json').exists():
        resource_tick();ensure_post_evaluator();time.sleep(5)
    warm=resolve(root/'posttune','handoff-best.json')/'resume.pt';atomic_json(root/'control.json',dict(phase='rl',start=time.time(),warm_start=str(warm)))
    command=['--seconds',str(a.rl_seconds),'--warm-start',str(warm)]
    if (root/'rl/latest.json').exists():command=['--seconds',str(a.rl_seconds),'--resume',str(root/'rl')]
    actor=launch('hex_command_rl.mixed_ppo','rl',command)
    evaluator=launch('hex_command_rl.mixed_evaluate','rl-evaluation',['--phase','rl','--interval','600']);retries=0
    print(json.dumps(dict(event='rl_started',pid=actor.pid,warm_start=str(warm),seconds=a.rl_seconds)),flush=True)
    while actor.poll() is None or not (root/'rl/complete.json').exists():
        resource_tick()
        if actor.poll() not in (None,0):
            if retries>=2:raise RuntimeError('PPO failed repeatedly; inspect rl.log')
            actor=launch('hex_command_rl.mixed_ppo','rl',['--seconds',str(a.rl_seconds),'--resume',str(root/'rl')]);retries+=1
        if evaluator.poll() not in (None,0):
            (root/'rl/evaluation-active.json').unlink(missing_ok=True)
            evaluator=launch('hex_command_rl.mixed_evaluate','rl-evaluation',['--phase','rl','--interval','600'])
        time.sleep(5)
    while evaluator.poll() is None:resource_tick();time.sleep(5)
    if evaluator.returncode:raise RuntimeError('final PPO validation failed')
    atomic_json(root/'control.json',dict(phase='rl',status='final_evaluation'));final_report(root)
    atomic_json(root/'finished.json',dict(time=time.time(),ok=True))
    from .remote_backup import backup
    backup(root)
    print(json.dumps(dict(event='complete',root=str(root))),flush=True)
    for f in handles:f.close()
if __name__=='__main__':main()
