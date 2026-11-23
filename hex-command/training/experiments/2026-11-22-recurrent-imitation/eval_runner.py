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
panels=json.loads((root/'evaluation-panels.json').read_text())

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

def panel_set(label,model,phase):
    reports={name:evaluate(label,model,name,cases,phase)
             for name,cases in panels[phase].items()}
    selection=['targeted','training_campaigns','heldout']
    result=dict(label=label,score=sum(reports[k]['score'] for k in selection)/3,
                weighted_caps=sum(reports[k]['weighted_caps'] for k in selection)/3,
                panels={k:{f:r[f] for f in ['score','wins','caps','runs']} for k,r in reports.items()})
    atomic_json(root/'evaluation'/label/(phase+'.json'),result)
    return result

def plot(labels):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from collections import defaultdict
    folder=root/'plots';folder.mkdir(exist_ok=True)
    xs=[0,15,30,45,60]
    fig,axes=plt.subplots(2,2,figsize=(12,8))
    for ax,name in zip(axes.flat,['targeted','training_campaigns','heldout','lantern_showcase']):
        vals=[json.loads((root/'evaluation'/label/'monitor.json').read_text())['panels'][name]['score']*100 for label in labels]
        ax.plot(xs,vals,'o-');ax.axhline(vals[0],color='grey',linestyle='--',alpha=.5)
        ax.set(title=name.replace('_',' ').title(),xlabel='Training minutes',ylabel='Balanced completion / win (%)',ylim=(0,100))
        ax.grid(alpha=.25)
    fig.suptitle('V4 recurrent imitation · fixed monitor seeds · Lantern excluded from selection')
    fig.tight_layout();fig.savefig(folder/'gameplay-over-time.png',dpi=150);plt.close(fig)
    campaigns=['rl-woodland-defense','rl-underground-escape','rl-splitwake-crossing']
    fig,axes=plt.subplots(3,2,figsize=(13,12))
    for i,campaign in enumerate(campaigns):
        for j,visibility in enumerate(['authored','fog']):
            series=defaultdict(list)
            for label in labels:
                report=json.loads((root/'evaluation'/label/'monitor/training_campaigns.json').read_text())
                stages=defaultdict(lambda:[0,0])
                for g in report['groups']:
                    if g['group'][1:3]!=[campaign,visibility]:continue
                    for stage,values in g['stages'].items():
                        stages[stage][0]+=values['wins'];stages[stage][1]+=values['entered']
                for k in range(1,4):
                    values=stages.get(campaign+'-'+str(k),[0,0])
                    series[k].append(100*values[0]/values[1] if values[1] else float('nan'))
            ax=axes[i,j]
            for chapter,ys in series.items():ax.plot(xs,ys,'o-',label=f'Chapter {chapter}')
            ax.set(title=campaign.removeprefix('rl-')+' · '+('player-only fog' if visibility=='fog' else 'clear'),
                   xlabel='Training minutes',ylabel='Clears / chapter entrants (%)',ylim=(0,100))
            ax.grid(alpha=.25);ax.legend()
    fig.tight_layout();fig.savefig(folder/'campaign-chapters-over-time.png',dpi=150);plt.close(fig)

def main():
    (root/'evaluation').mkdir(exist_ok=True)
    lock=(root/'evaluation/worker.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    model=root/'v4-planner.json'
    # Zero new input/head columns preserve V4 behavior before pretraining.
    # Confirm this on actual arena, targeted and broad-campaign cases.
    proof=root/'evaluation/initial-parity.json'
    if not proof.exists():
        import hex_command_training as native
        subset=[panels['monitor']['heldout'][0],panels['monitor']['targeted'][0],panels['monitor']['training_campaigns'][0]]
        encoded=[json.dumps(c) for c in subset]
        before=native.evaluate(encoded,model.read_text())
        expanded=native.evaluate(encoded,(root/'expanded-start-planner.json').read_text())
        assert before==expanded,'expanded initialization changed gameplay'
        atomic_json(proof,dict(identical=True,cases=subset,results=before,
                              expanded_alias='V4 baseline; appended head weights are zero'))
    labels=['v4_start'];models={'v4_start':model};reports={'v4_start':panel_set('v4_start',model,'monitor')}
    for minute in [15,30,45,60]:
        label=f'minute-{minute:04d}'
        while True:
            paths=sorted((root/'posttune/checkpoints').glob(label+'-*/planner.json'))
            if paths:break
            if minute==60 and (root/'posttune/complete.json').exists():
                latest=json.loads((root/'posttune/latest.json').read_text())
                paths=[root/'posttune'/latest['path']/'planner.json'];break
            if (root/'stop').exists():return
            time.sleep(15)
        models[label]=paths[-1];labels.append(label)
        reports[label]=panel_set(label,paths[-1],'monitor')
        atomic_json(root/'evaluation/monitor-summary.json',reports)
    plot(labels)
    best=max(labels[1:],key=lambda name:(reports[name]['score'],-reports[name]['weighted_caps']))
    comparison={name:panel_set(name,models[name],'confirmation') for name in ['v4_start',best]}
    selected=max(comparison,key=lambda name:(comparison[name]['score'],-comparison[name]['weighted_caps']))
    result=dict(selected=selected,selected_model=str(models[selected]),best_trained=best,
                criterion='Mean targeted / training-campaign / heldout completion; fewer caps break ties. Independent confirmation. Lantern excluded.',
                monitor=reports,confirmation=comparison,
                source_revision=(root/'source-commit.txt').read_text().strip(),time=time.time())
    atomic_json(root/'evaluation/selection.json',result)
    atomic_json(root/'evaluation/complete.json',result)

if __name__=='__main__':main()
