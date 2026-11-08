"""Fixed-seed hierarchical gameplay evaluation and checkpoint selection."""
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor,as_completed
import argparse
import hashlib
import json
import multiprocessing
import os
from pathlib import Path
import time
from .mixed_workload import group,mass
from .recovery import atomic_json,resolve


def job(task):
    cases,model=task
    import hex_command_training as native
    results=native.evaluate([json.dumps(c) for c in cases],None if model is None else Path(model).read_text())
    return [dict(config=c,result=json.loads(r)) for c,r in zip(cases,results)]


def summarize(rows):
    groups=defaultdict(list)
    for r in rows:groups[group(r['config'])].append(r)
    detailed=[];score=caps=weight=0.
    for k,rs in groups.items():
        w=mass(rs[0]['config']);n=len(rs);wins=sum(r['result']['success'] for r in rs);cap=sum(r['result']['truncated'] for r in rs)
        score+=w*wins/n;caps+=w*cap/n;weight+=w
        stages=defaultdict(lambda:dict(entered=0,wins=0,defeats=0,caps=0,rounds=[]))
        for row in rs:
            for mission in row['result'].get('missions',[]):
                d=stages[mission['id']];d['entered']+=1;d['wins']+=mission['winner']=='player';d['defeats']+=mission['winner']=='enemy';d['caps']+=mission['winner'] is None;d['rounds'].append(mission['round'])
        detailed.append(dict(group=list(k),runs=n,wins=wins,defeats=n-wins-cap,caps=cap,stages=dict(stages)))
    return dict(score=score/weight,weighted_caps=caps/weight,wins=sum(r['result']['success'] for r in rows),
                caps=sum(r['result']['truncated'] for r in rows),runs=len(rows),groups=detailed)


def evaluate(cases,model):
    rows=[]
    with ProcessPoolExecutor(max_workers=2,mp_context=multiprocessing.get_context('spawn')) as pool:
        tasks=[(cases[i:i+4],model) for i in range(0,len(cases),4)]
        for f in as_completed([pool.submit(job,t) for t in tasks]):rows.extend(f.result())
    return dict(rows=rows,**summarize(rows))


def plot(root):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    root=Path(root);history=[json.loads(l) for l in (root/'evaluations.jsonl').read_text().splitlines()] if (root/'evaluations.jsonl').exists() else []
    metrics=[json.loads(l) for l in (root/'metrics.jsonl').read_text().splitlines()] if (root/'metrics.jsonl').exists() else []
    fig,axes=plt.subplots(2,3,figsize=(14,7));x=[r['seconds']/60 for r in history]
    for ax in axes.flat:ax.grid(alpha=.2);ax.set_xlabel('Minutes')
    axes[0,0].plot(x,[r['score']*100 for r in history],marker='o');axes[0,0].set_title('Balanced held-out completion / win score (%)')
    for family in ['arena','campaign']:
        vals=[]
        for r in history:
            gs=[g for g in r['groups'] if g['group'][0]==family];vals.append(100*sum(g['wins']/g['runs'] for g in gs)/max(1,len(gs)))
        axes[0,1].plot(x,vals,marker='o',label=family)
    axes[0,1].legend();axes[0,1].set_title('Held-out family results')
    axes[0,2].plot(x,[r['caps'] for r in history],marker='o');axes[0,2].set_title('Held-out caps')
    validation=[r for r in metrics if r['event']=='validation'];updates=[r for r in metrics if r['event']=='update'];episodes=[r for r in metrics if r['event']=='episode']
    if validation:
        axes[1,0].plot([r['seconds']/60 for r in validation],[r['loss'] for r in validation],label='validation CE');axes[1,0].legend()
        axes[1,1].plot([r['seconds']/60 for r in validation],[r['accuracy']*100 for r in validation]);axes[1,1].set_title('Balanced teacher agreement (%)')
    if updates:
        tx=[r['seconds']/60 for r in updates]
        for k in ['policy_loss','value_loss']:axes[1,0].plot(tx,[r[k] for r in updates],label=k)
        axes[1,0].legend()
        for k in ['entropy','reference_kl']:axes[1,1].plot(tx,[r[k] for r in updates],label=k)
        axes[1,1].legend();axes[1,1].set_title('Policy entropy / KL')
    if episodes:
        axes[1,2].plot([r['seconds']/60 for r in episodes],[sum(e['reward'] for e in episodes[max(0,i-49):i+1])/len(episodes[max(0,i-49):i+1]) for i in range(len(episodes))]);axes[1,2].set_title('Exploration reward (last 50 episodes)')
    else:
        progress=[r for r in metrics if r['event']=='progress'];axes[1,2].plot([r['seconds']/60 for r in progress],[r['decisions'] for r in progress]);axes[1,2].set_title('Training decisions processed')
    fig.suptitle(root.name+' · mixed workload · validation seeds excluded from training');fig.tight_layout();fig.savefig(root/'curves.png',dpi=140);plt.close(fig)


def watch(root,phase,interval=600):
    root=Path(root);run=root/phase;run.mkdir(exist_ok=True);spec=json.loads((root/'workload.json').read_text());os.environ['HEX_COMMAND_DATA_ROOT']=spec['catalog']
    seen=set();last=-float('inf');best=(-1.,-float('inf'));start=time.monotonic()
    if (run/'evaluations.jsonl').exists():
        old=[json.loads(l) for l in (run/'evaluations.jsonl').read_text().splitlines()];seen={r['checkpoint'] for r in old};last=max((r['seconds'] for r in old),default=-float('inf'));best=max(((r['score'],-r['weighted_caps']) for r in old),default=best)
    while True:
        complete=(run/'complete.json').exists()
        if (run/'latest.json').exists():
            pointer=json.loads((run/'latest.json').read_text());folder=resolve(run);name=pointer['path'];elapsed=time.monotonic()-start
            state_time=json.loads((run/'status.json').read_text()).get('seconds',elapsed) if (run/'status.json').exists() else elapsed
            if name not in seen and (not seen or state_time-last>=interval or complete):
                atomic_json(run/'evaluation-active.json',dict(start=time.time()))
                try:result=evaluate(spec['periodic'],str(folder/'planner.json'))
                finally:(run/'evaluation-active.json').unlink(missing_ok=True)
                result.update(checkpoint=name,seconds=state_time,model_sha256=hashlib.sha256((folder/'planner.json').read_bytes()).hexdigest())
                atomic_json(folder/'gameplay.json',result)
                with (run/'evaluations.jsonl').open('a') as f:f.write(json.dumps(result)+'\n')
                score=(result['score'],-result['weighted_caps'])
                if score>best:best=score;atomic_json(run/'best.json',pointer)
                seen.add(name);last=state_time;plot(run)
                print(json.dumps(dict(phase=phase,checkpoint=name,seconds=state_time,score=result['score'],wins=result['wins'],caps=result['caps'],runs=result['runs'])),flush=True)
        if complete:
            atomic_json(run/'evaluation-complete.json',dict(best=best,checkpoints=len(seen)));break
        time.sleep(5)


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--phase',required=True);p.add_argument('--interval',type=int,default=600);a=p.parse_args();watch(a.root,a.phase,a.interval)
if __name__=='__main__':main()
