"""Compare native presets, Full diagnostics and learned planners vs fixed Classic."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import hex_command_training as native
from .workloads import RECRUITERS, arena, encode
from .train import results_summary


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--model',type=Path)
    p.add_argument('--baselines',type=Path,help='Reuse a completed baseline panel with identical configs/fingerprints')
    p.add_argument('--seeds',type=int,default=3)
    p.add_argument('--rounds',type=int,default=100)
    p.add_argument('--out',type=Path,default=Path('tmp/rl/classic-comparison.json'))
    args=p.parse_args()
    if args.seeds<1 or not 1<=args.rounds<=1000:p.error('invalid seeds or round limit')
    configs=arena(range(20001,20001+args.seeds),rounds=args.rounds,all_pairs=True,recruiters=None)
    fingerprints=json.loads(native.fingerprints())
    panels={}
    if args.baselines:
        saved=json.loads(args.baselines.read_text())
        if saved['configs']!=configs or saved['fingerprints']!=fingerprints:
            p.error('baseline workload or engine/content fingerprints differ')
        panels=saved['panels'].copy()
    args.out.parent.mkdir(parents=True,exist_ok=True)
    with args.out.with_suffix('.jsonl').open('w') as log:
        def run_panel(label,cases,model=None):
            results=[]
            for start in range(0,len(cases),8):
                rows=[json.loads(r) for r in native.evaluate(encode(cases[start:start+8]),model)]
                results.extend(rows)
                log.write(json.dumps(dict(panel=label,start=start,results=rows))+'\n');log.flush()
            panels[label]=dict(configs=cases,results=results,summary=results_summary(results))
            print(json.dumps(dict(panel=label,**panels[label]['summary'])),flush=True)
        if not args.baselines:
            for policy in ('classic','rush','rotated','balanced','situational','predictor'):
                cases=deepcopy(configs)
                for c in cases:c['player']=policy
                run_panel(policy,cases)
            for recruiter in RECRUITERS:
                cases=deepcopy(configs)
                for c in cases:c['playerBlocks']=dict(recruiter=recruiter,campaign='mission',planner='full')
                run_panel(f'full/{recruiter}',cases)
        if args.model:
            model=args.model.read_text()
            for recruiter in RECRUITERS:
                cases=deepcopy(configs)
                for c in cases:c['playerBlocks']=dict(recruiter=recruiter,campaign='mission',planner='queued')
                run_panel(f'learned/{recruiter}',cases,model)
    report=dict(fingerprints=fingerprints,configs=configs,panels=panels,opponent='classic',rounds=args.rounds)
    args.out.write_text(json.dumps(report,indent=2))


if __name__=='__main__':main()
