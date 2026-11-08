"""Fresh matched combat evaluation, with tree prefixes from one fitted ensemble."""
import argparse,hashlib,json,multiprocessing,os,time
from concurrent.futures import ProcessPoolExecutor,as_completed
from pathlib import Path

CACHE={}


def summarize(rows):
    return dict(games=len(rows),wins=sum(v['success'] for v in rows),losses=sum(not v['success'] and not v['truncated'] for v in rows),caps=sum(v['truncated'] for v in rows),mean_rounds=sum(v['rounds'] for v in rows)/max(len(rows),1))


def job(item):
    import hex_command_training as native
    key,path,start,configs=item
    if path not in CACHE:CACHE[path]=None if path is None else Path(path).read_text()
    rows=[json.loads(v) for v in native.evaluate([json.dumps(c) for c in configs],CACHE[path])]
    if len(rows)!=len(configs):raise RuntimeError('incomplete evaluation shard')
    return dict(panel=key,start=start,results=rows)


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,default=Path('tmp/rl/representative'))
    p.add_argument('--workers',type=int,default=3);p.add_argument('--include-old',action='store_true');p.add_argument('--tree-prefixes',nargs='*',type=int,default=[64,128,256,512])
    args=p.parse_args()
    if not 1<=args.workers<=4 or any(n<1 for n in args.tree_prefixes):p.error('workers 1..4; positive tree counts')
    root=args.root.resolve();out=root/'evaluation';out.mkdir(parents=True,exist_ok=True)
    os.environ['HEX_COMMAND_DATA_ROOT']=str(root/'catalog')
    import hex_command_training as native
    import lightgbm as lgb
    from .trees import export_model
    source=json.loads((root/'source.json').read_text());fingerprints=json.loads(native.fingerprints())
    if fingerprints!=source['fingerprints']:raise ValueError('source catalog/engine fingerprint differs')
    models={name:str(root/'runs'/folder/'planner.json') for name,folder in [('cnn','cnn'),('mlp','mlp'),('trees','trees')]}
    models['queued']=None;classic_only=set()
    for name in ('cnn','mlp','trees'):
        report=json.loads((root/'runs'/name/'report.json').read_text())
        if report['fingerprints']!=fingerprints or report['evaluation_configs']!=source['evaluation_configs']:raise ValueError('fit provenance differs')
    booster=lgb.Booster(model_file=str(root/'runs/trees/booster.txt'))
    prefix_dir=out/'prefixes';prefix_dir.mkdir(exist_ok=True)
    for n in sorted(set(args.tree_prefixes)):
        if n>=booster.num_trees():continue
        path=prefix_dir/f'trees-{n}.json';path.write_text(json.dumps(export_model(booster,n)))
        models[f'trees-{n}']=str(path);classic_only.add(f'trees-{n}')
    if args.include_old:
        for name,folder in [('old-cnn','deathmatch-classic-cnn'),('old-mlp','deathmatch-classic-20m')]:
            models[name]=str(Path('tmp/rl').resolve()/folder/'planner.json');classic_only.add(name)
    panels={}
    for label,path in models.items():
        for enemy in (['classic'] if label in classic_only else ['classic','situational','predictor']):
            configs=[dict(c,enemy=enemy) for c in source['evaluation_configs']]
            panels[f'{label}/vs-{enemy}']=dict(model=label,enemy=enemy,configs=configs,results=[None]*len(configs))
    manifest=dict(fingerprints=fingerprints,models={name:None if path is None else dict(path=path,sha256=hashlib.sha256(Path(path).read_bytes()).hexdigest()) for name,path in models.items()},expected_games=sum(len(p['configs']) for p in panels.values()),tree_count=booster.num_trees())
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    started=time.monotonic();last=started;completed=0
    pool=ProcessPoolExecutor(max_workers=args.workers,mp_context=multiprocessing.get_context('spawn'))
    try:
        futures=[pool.submit(job,(key,models[p['model']],start,p['configs'][start:start+8])) for start in range(0,len(source['evaluation_configs']),8) for key,p in panels.items()]
        with (out/'results.jsonl').open('w') as log:
            for future in as_completed(futures):
                row=future.result();log.write(json.dumps(row)+'\n');log.flush();panel=panels[row['panel']];a=row['start'];panel['results'][a:a+len(row['results'])]=row['results'];completed+=len(row['results'])
                if time.monotonic()-last>=20:
                    progress=dict(event='progress',games=completed,total=manifest['expected_games'],seconds=time.monotonic()-started)
                    (out/'progress.json').write_text(json.dumps(progress));print(json.dumps(progress),flush=True);last=time.monotonic()
    except BaseException:
        for process in multiprocessing.active_children():process.terminate()
        raise
    finally:pool.shutdown(wait=True,cancel_futures=True)
    for key,panel in panels.items():
        if any(v is None for v in panel['results']):raise RuntimeError('incomplete panel')
        panel['summary']=summarize(panel['results']);panel['by_map']={};panel['by_recruiter']={}
        for name in source['maps']:
            rows=[v for c,v in zip(panel['configs'],panel['results']) if c['landscape']==name]
            panel['by_map'][name]=summarize(rows)
            if len(rows)!=len(source['evaluation_configs'])//4:raise RuntimeError('unequal map evaluation coverage')
            for recruiter in ('situational','predictor'):
                selected=[v for c,v in zip(panel['configs'],panel['results']) if c['landscape']==name and c['playerBlocks']['recruiter']==recruiter]
                panel['by_recruiter'][f'{name}/{recruiter}']=summarize(selected)
        print(key,panel['summary'],flush=True)
    if panels['queued/vs-classic']['results']!=source['teacher_results']:raise RuntimeError('fresh Queued reference outcomes changed')
    report=dict(**manifest,panels=panels,seconds=time.monotonic()-started)
    (out/'report.json').write_text(json.dumps(report,indent=2));print('all matched panels complete',flush=True)


if __name__=='__main__':main()
