"""Fixed panels, restartable per-game results, independent confirmation/showcase."""
from concurrent.futures import ProcessPoolExecutor, as_completed
import fcntl
import hashlib
import json
import multiprocessing
import os
from pathlib import Path
import sys
import time
from hex_command_rl.mixed_evaluate import job, summarize
from hex_command_rl.recovery import atomic_json, read_jsonl

root=Path(__file__).parent.resolve()
os.environ['HEX_COMMAND_DATA_ROOT']=str(root/'catalog')
panels=json.loads((root/'workload.json').read_text())['evaluation_panels']

def key(c):return json.dumps(c,sort_keys=True,separators=(',',':'))

def evaluate(label,model,panel_name,cases,phase='monitor'):
    folder=root/'evaluation'/label/phase;folder.mkdir(parents=True,exist_ok=True)
    destination=folder/(panel_name+'.json')
    if destination.exists():return json.loads(destination.read_text())
    log=folder/(panel_name+'.jsonl')
    rows=read_jsonl(log,repair=True)
    done={key(r['config']) for r in rows}
    remaining=[c for c in cases if key(c) not in done]
    with ProcessPoolExecutor(max_workers=2,mp_context=multiprocessing.get_context('spawn')) as pool:
        futures=[pool.submit(job,(remaining[i:i+4],str(model))) for i in range(0,len(remaining),4)]
        with log.open('a') as stream:
            for future in as_completed(futures):
                batch=future.result()
                for row in batch:stream.write(json.dumps(row)+'\n')
                stream.flush();rows.extend(batch)
                atomic_json(root/'evaluation/status.json',dict(time=time.time(),label=label,
                            phase=phase,panel=panel_name,completed=len(rows),total=len(cases)))
    assert len(rows)==len(cases)
    report=dict(rows=rows,**summarize(rows))
    report.update(label=label,phase=phase,panel=panel_name,
                  model_sha256=hashlib.sha256(model.read_bytes()).hexdigest())
    atomic_json(destination,report)
    print(json.dumps(dict(label=label,phase=phase,panel=panel_name,score=report['score'],
                          wins=report['wins'],caps=report['caps'],runs=report['runs'])),flush=True)
    return report


def plots(records):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from collections import defaultdict
    folder=root/'plots';folder.mkdir(exist_ok=True)
    correction=json.loads((root/'allocation-correction.json').read_text()) if (root/'allocation-correction.json').exists() else None
    def mark(axes):
        if correction:
            for ax in axes.flat:
                ax.axvline(correction['elapsed']/3600,color='crimson',linestyle=':',alpha=.7)
            fig.text(.5,.005,'Dotted red line: arena allocation fix; earlier training used the faulty sampler',ha='center',fontsize=9)

    xs=[r['hour'] for r in records]
    fig,axes=plt.subplots(1,3,figsize=(14,4))
    for ax,name in zip(axes,panels):
        ys=[r['panels'][name]['score']*100 for r in records]
        ax.plot(xs,ys,'o-');ax.set(title=name.replace('_',' '),xlabel='Training hours',ylabel='Completion / win (%)',ylim=(0,100));ax.grid(alpha=.25)
    mark(axes);fig.tight_layout();fig.savefig(folder/'gameplay-over-time.png',dpi=150);plt.close(fig)
    stages=defaultdict(dict)
    for r in records:
        for name in ['training_diagnostic','lantern_showcase']:
            report=json.loads((root/'evaluation'/r['label']/'monitor'/(name+'.json')).read_text())
            for g in report['groups']:
                for stage,v in g.get('stages',{}).items():
                    k=(g['group'][1],g['group'][2],stage)
                    pair=stages[k].setdefault(r['hour'],[0,0]);pair[0]+=v['wins'];pair[1]+=v['entered']
    groups=sorted(set(k[:2] for k in stages))
    if groups:
        fig,axes=plt.subplots(len(groups),1,figsize=(10,max(4,len(groups)*3)),squeeze=False)
        for ax,g in zip(axes.flat,groups):
            for k,points in stages.items():
                if k[:2]!=g:continue
                ax.plot(xs,[100*points[h][0]/points[h][1] if h in points and points[h][1] else float('nan') for h in xs],'o-',label=k[2])
            ax.set(title=' · '.join(g),xlabel='Training hours',ylabel='Clears / entrants (%)',ylim=(0,100));ax.grid(alpha=.25);ax.legend(fontsize=7)
        mark(axes);fig.tight_layout();fig.savefig(folder/'campaign-chapters-over-time.png',dpi=120);plt.close(fig)
    rows=read_jsonl(root/'rl/metrics.jsonl',repair=False)
    fig,axes=plt.subplots(2,2,figsize=(12,7))
    for ax,field in zip(axes.flat,['policy_loss','value_loss','entropy','reference_kl']):
        pts=[(r['seconds']/3600,r[field]) for r in rows if field in r]
        if pts:ax.plot(*zip(*pts),linewidth=.5)
        ax.set(title=field,xlabel='Training hours');ax.grid(alpha=.25)
    mark(axes);fig.tight_layout();fig.savefig(folder/'training-curves.png',dpi=150);plt.close(fig)

def main():
    from hex_command_rl.mixed_evaluate import reliability
    from hex_command_rl.handoff import reliable_choice
    (root/'evaluation').mkdir(exist_ok=True)
    lock=(root/'evaluation/worker.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    records=[]
    for hour in range(10):
        label=f'hour-{hour:02d}'
        if hour==0:model=root/'warm-start/planner.json';actual=0
        else:
            while True:
                if (root/'stop').exists():return
                index=json.loads((root/'checkpoint-index.json').read_text()) if (root/'checkpoint-index.json').exists() else {}
                eligible=[r for r in index.values() if r['elapsed']>=hour*3600]
                if eligible:
                    chosen=min(eligible,key=lambda r:r['elapsed']);model=root/'rl'/chosen['path']/'planner.json';actual=chosen['elapsed'];break
                time.sleep(15)
        reports={name:evaluate(label,model,name,cases) for name,cases in panels.items()}
        key_,strata=reliability(reports['heldout'])
        record=dict(label=label,hour=hour,actual_seconds=actual,model=str(model),reliability_key=list(key_),strata=strata,
                    panels={k:{f:r[f] for f in ['score','wins','caps','runs']} for k,r in reports.items()})
        records.append(record);atomic_json(root/'evaluation/hourly-summary.json',records);plots(records)
    selected=reliable_choice(records)
    result=dict(selected=selected,records=records,criterion='Held-out only; worst-stratum tolerance .025 then overall score and fewer caps. Starting policy eligible. Training diagnostics and Lantern excluded.',time=time.time())
    atomic_json(root/'evaluation/selection.json',result)
    atomic_json(root/'evaluation/complete.json',result)

if __name__=='__main__':main()
