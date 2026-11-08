"""Native CPU comparison on identical frozen planner-entry positions."""
import argparse
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parents[2]


def player(profile):
    return sum(r['seconds'] for r in profile['rows'] if r['side']=='player'), sum(r['calls'] for r in profile['rows'] if r['side']=='player')


def worker(args):
    import hex_command_training as native
    fixture=json.loads(args.fixtures.read_text())
    cases=[]
    for position in fixture['positions']:
        request=position['request'].copy()
        assert not request.get('beginTurn') and not request.get('advanceTurn')
        request.update(policy='balanced',blocks=dict(recruiter=args.recruiter,campaign='mission',planner=args.planner))
        cases.append(json.dumps(request))
    model=args.model.read_text() if args.planner=='learned' else None
    if native.profile_snapshot() is None:raise RuntimeError('set HEX_COMMAND_AI_PROFILE=1 before worker startup')
    native.replay_phases(cases,model)  # warm the actual selected planner
    samples=[]
    for _ in range(args.repeat):
        before=player(json.loads(native.profile_snapshot()));started=time.perf_counter()
        output=native.replay_phases(cases,model)
        elapsed=time.perf_counter()-started;after=player(json.loads(native.profile_snapshot()))
        commands=sum(len(json.loads(r)['records']['actions']) for r in output)
        samples.append(dict(cpu_seconds=after[0]-before[0],planning_chunks=after[1]-before[1],phases=len(cases),commands=commands,wall_seconds=elapsed))
    per_phase=[s['cpu_seconds']/s['phases']*1000 for s in samples]
    return dict(planner=args.planner,recruiter=args.recruiter,positions=len(cases),repeat=args.repeat,
                median_cpu_ms_per_phase=statistics.median(per_phase),min_cpu_ms_per_phase=min(per_phase),max_cpu_ms_per_phase=max(per_phase),
                mean_commands_per_phase=sum(s['commands'] for s in samples)/sum(s['phases'] for s in samples),
                samples=samples,fingerprints=json.loads(native.fingerprints()))


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--model',type=Path,required=True)
    p.add_argument('--data-dir',type=Path,default=Path(os.environ.get('HEX_COMMAND_DATA_ROOT',ROOT/'rust/data')))
    p.add_argument('--source-report',type=Path)
    p.add_argument('--fixtures',type=Path)
    p.add_argument('--binary',type=Path,default=ROOT/'rust/target/simulation/hex-command-sim')
    p.add_argument('--repeat',type=int,default=10)
    p.add_argument('--recruiters',nargs='+',choices=['balanced','situational','predictor'],default=['balanced','situational','predictor'])
    p.add_argument('--planners',nargs='+',choices=['queued','full','learned'],default=['queued','full','learned'])
    p.add_argument('--out',type=Path,default=ROOT/'tmp/rl/native-timing')
    p.add_argument('--worker',action='store_true',help=argparse.SUPPRESS)
    p.add_argument('--planner',choices=['queued','full','learned'])
    p.add_argument('--recruiter',choices=['balanced','situational','predictor'])
    args=p.parse_args()
    if args.repeat<1:p.error('repeat must be positive')
    if args.worker:
        print(json.dumps(worker(args)));return
    args.out.mkdir(parents=True,exist_ok=True)
    if not args.fixtures:
        if not args.source_report:p.error('supply --source-report or --fixtures')
        spec=importlib.util.spec_from_file_location('phase_tools',ROOT/'scripts/benchmark-policy-phases.py')
        helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
        source=json.loads(args.source_report.read_text());configs=[]
        from .workloads import FACTIONS
        capture_seed=min(c['seed'] for c in source['evaluation_configs'])
        capture_recruiter=source['evaluation_configs'][0]['playerBlocks']['recruiter']
        for c in source['evaluation_configs']:
            if c['seed']!=capture_seed or c['playerBlocks']['recruiter']!=capture_recruiter:continue
            if c['armies']['enemy']!=FACTIONS[(FACTIONS.index(c['armies']['player'])+1)%len(FACTIONS)]:continue
            configs.append(c)
        captured=helper.capture(args.binary.resolve(),args.data_dir.resolve(),configs,[1,4,10,20],'player')
        fixture=dict(positions=helper.positions(captured),capture=captured)
        args.fixtures=args.out/'positions.json';args.fixtures.write_text(json.dumps(fixture))
    rows=[]
    for recruiter in args.recruiters:
        for planner in args.planners:
            command=[sys.executable,'-m','hex_command_rl.benchmark','--worker','--fixtures',str(args.fixtures.resolve()),'--model',str(args.model.resolve()),'--recruiter',recruiter,'--planner',planner,'--repeat',str(args.repeat)]
            row=json.loads(subprocess.check_output(command,cwd=ROOT,env={**os.environ,'HEX_COMMAND_AI_PROFILE':'1','HEX_COMMAND_DATA_ROOT':str(args.data_dir.resolve())},text=True))
            rows.append(row);print(json.dumps({k:v for k,v in row.items() if k not in ('samples','fingerprints')}),flush=True)
            report=dict(data_dir=str(args.data_dir.resolve()),model_path=str(args.model.resolve()),model_sha256=hashlib.sha256(args.model.read_bytes()).hexdigest(),fixture_sha256=hashlib.sha256(args.fixtures.read_bytes()).hexdigest(), clock='thread_cpu', statistic='Median of repeated-panel mean CPU per own phase', scope='Native thread CPU inside tactical run_turn (recruiter/campaigner/planner); excludes state restore, encoding, and command execution outside planning forecasts',
                        fixtures=str(args.fixtures.resolve()),rows=rows)
            (args.out/'timings.json').write_text(json.dumps(report,indent=2))


if __name__=='__main__':main()
