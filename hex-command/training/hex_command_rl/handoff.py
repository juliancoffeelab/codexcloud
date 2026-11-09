"""Choose a robust RL starting policy on a larger, separate validation panel."""
import argparse
import json
import os
from pathlib import Path
import time
from .mixed_evaluate import evaluate,reliability
from .recovery import atomic_json


def candidates(run):
    result=set()
    if (run/'evaluations.jsonl').exists():
        for line in (run/'evaluations.jsonl').read_text().splitlines():
            try:result.add(json.loads(line)['checkpoint'])
            except json.JSONDecodeError:pass
    # Completed epochs are less sensitive to which shard was trained most recently.
    for p in (run/'checkpoints').glob('epoch-*'):result.add(str(p.relative_to(run)))
    if (run/'complete.json').exists():result.add(json.loads((run/'latest.json').read_text())['path'])
    return sorted(result,key=lambda p:int(p.rsplit('-',1)[-1]))


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);a=p.parse_args();root=a.root;run=root/'posttune'
    spec=json.loads((root/'workload.json').read_text());os.environ['HEX_COMMAND_DATA_ROOT']=spec['catalog'];os.environ['HEX_COMMAND_EVALUATION_LOCK']=str(root/'gameplay.lock')
    cache=run/'handoff-validation';cache.mkdir(exist_ok=True);best=None;best_key=(-1.,-1.,-1.)
    if (run/'handoff-results.jsonl').exists():
        for line in (run/'handoff-results.jsonl').read_text().splitlines():
            r=json.loads(line);key=tuple(r['reliability_key'])
            if key>best_key:best_key=key;best=r
    while True:
        pending=[]
        for relative in candidates(run):
            folder=run/relative;output=cache/(folder.name+'.json')
            if not output.exists():pending.append((relative,folder,output))
        for relative,folder,output in pending:
            r=evaluate(spec['validation'],str(folder/'planner.json'));key,strata=reliability(r)
            r.update(checkpoint=relative,reliability_key=list(key),strata=strata);atomic_json(output,r)
            compact={k:v for k,v in r.items() if k!='rows'}
            with (run/'handoff-results.jsonl').open('a') as f:f.write(json.dumps(compact)+'\n')
            if key>best_key:
                best_key=key;best=compact;m=json.loads((folder/'manifest.json').read_text())
                atomic_json(run/'handoff-best.json',dict(path=relative,label=m['label'],files=m['files']))
            print(json.dumps(dict(checkpoint=relative,reliability=list(key),strata=strata,runs=r['runs'],best=best['checkpoint'])),flush=True)
        if (run/'complete.json').exists() and (run/'evaluation-complete.json').exists() and not any(not (cache/((run/r).name+'.json')).exists() for r in candidates(run)):
            atomic_json(run/'handoff-complete.json',dict(best=best['checkpoint'],reliability=list(best_key),strata=best['strata'],criterion='Maximize minimum balanced result across arena/campaign × Classic/Predictor; then overall score and fewer caps',validation_runs=len(spec['validation']),test_seeds_used=False));break
        time.sleep(5)
if __name__=='__main__':main()
