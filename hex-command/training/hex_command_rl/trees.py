"""Shallow LambdaMART ranker with bounded training and native Rust export."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
import numpy as np
import hex_command_training as native
from .data import load_dataset


def export_model(booster, iteration=None):
    dump=booster.dump_model(num_iteration=iteration)
    if dump.get('average_output'):raise ValueError('averaged forests are not supported')
    trees=[]
    for tree in dump['tree_info']:
        nodes=[]
        def flatten(node):
            index=len(nodes);nodes.append(None)
            if 'leaf_value' in node:
                nodes[index]=dict(kind='leaf',value=node['leaf_value'])
            else:
                if node['decision_type']!='<=' or node.get('missing_type')=='Zero':
                    raise ValueError('only numerical non-zero-missing splits are supported')
                left=flatten(node['left_child']);right=flatten(node['right_child'])
                nodes[index]=dict(kind='split',feature=node['split_feature'],threshold=float(node['threshold']),
                                  left=left,right=right,default_left=bool(node['default_left']))
            return index
        flatten(tree['tree_structure']);trees.append(dict(nodes=nodes))
    return dict(schema=native.SCHEMA,feature_names=native.FEATURE_NAMES,trees=trees)


def grouped_choices(scores, offsets):
    sizes=np.diff(offsets)
    maxima=np.maximum.reduceat(scores,offsets[:-1])
    return np.minimum.reduceat(np.where(scores==np.repeat(maxima,sizes),np.arange(len(scores)),len(scores)),offsets[:-1])-offsets[:-1]


def grouped_metrics(scores, offsets, labels):
    sizes=np.diff(offsets)
    maxima=np.maximum.reduceat(scores,offsets[:-1])
    sums=np.add.reduceat(np.exp(scores-np.repeat(maxima,sizes)),offsets[:-1])
    loss=np.log(sums)+maxima-scores[offsets[:-1]+labels]
    return dict(accuracy=float(np.mean(grouped_choices(scores,offsets)==labels)),cross_entropy=float(np.mean(loss)))


def save_json(path,data):
    temporary=path.with_suffix(path.suffix+'.tmp');temporary.write_text(json.dumps(data,indent=2));temporary.replace(path)


def train_worker(args):
    import lightgbm as lgb
    started=time.monotonic()
    train=load_dataset(args.dataset_dir/'train');valid=load_dataset(args.dataset_dir/'validation')
    train_manifest=json.loads((args.dataset_dir/'train'/'manifest.json').read_text())
    valid_manifest=json.loads((args.dataset_dir/'validation'/'manifest.json').read_text())
    source=json.loads(args.source_report.read_text())
    fingerprints=json.loads(native.fingerprints())
    for split,manifest in [('train',train_manifest),('validation',valid_manifest)]:
        if manifest['fingerprints']['content']!=fingerprints['content']:raise ValueError('dataset content fingerprint mismatch')
        if manifest['configs']!=source[f'{split}_configs'] or manifest['collection_side']!=source['collect_side']:
            raise ValueError(f'source {split} dataset configuration differs')
    train_labels=np.zeros(len(train[0]),dtype=np.int32);train_labels[train[1][:-1]+train[2]]=1
    val_labels=np.zeros(len(valid[0]),dtype=np.int32);val_labels[valid[1][:-1]+valid[2]]=1
    params=dict(objective='lambdarank',metric='None',max_depth=args.depth,num_leaves=min(31,2**args.depth),
                learning_rate=0.08,min_data_in_leaf=100,max_bin=63,num_threads=3,
                deterministic=True,force_col_wise=True,seed=42,verbosity=-1,lambdarank_truncation_level=5)
    training=lgb.Dataset(train[0],label=train_labels,group=np.diff(train[1]),feature_name=[f'f{i:03d}_{name}' for i,name in enumerate(native.FEATURE_NAMES)],params=params,free_raw_data=True)
    validation=lgb.Dataset(valid[0],label=val_labels,group=np.diff(valid[1]),reference=training,free_raw_data=True)
    history=[];best=-1.;best_iteration=0
    log=(args.out/'metrics.jsonl').open('w')
    def evaluate(predictions,dataset):
        m=grouped_metrics(predictions,valid[1],valid[2])
        return [('choice_accuracy',m['accuracy'],True),('choice_cross_entropy',m['cross_entropy'],False)]
    def checkpoint(env):
        nonlocal best,best_iteration
        metrics={row[1]:float(row[2]) for row in env.evaluation_result_list}
        row=dict(iteration=env.iteration+1,seconds=time.monotonic()-started,**metrics)
        history.append(row);log.write(json.dumps(row)+'\n');log.flush();print(json.dumps(row),flush=True)
        if metrics['choice_accuracy']>best:
            best=metrics['choice_accuracy'];best_iteration=env.iteration+1
            model=export_model(env.model,best_iteration)
            model_file=f'planner-{best_iteration:04d}.json';booster_file=f'booster-{best_iteration:04d}.txt'
            save_json(args.out/model_file,model)
            env.model.save_model(str(args.out/booster_file),num_iteration=best_iteration)
            save_json(args.out/'best-training.json',dict(run_id=args.run_id,best_iteration=best_iteration,
                model_file=model_file,booster_file=booster_file,validation=metrics,parameters=params,tree_limit=args.trees,depth_limit=args.depth,
                early_stopping_rounds=args.early_stopping_rounds,
                dataset_source=str(args.dataset_dir.resolve()),fingerprints=fingerprints,
                data_fingerprints=[train_manifest['fingerprints'],valid_manifest['fingerprints']],
                lightgbm_version=lgb.__version__))
    checkpoint.order=20
    callbacks=[checkpoint,lgb.early_stopping(args.early_stopping_rounds,first_metric_only=True,verbose=False)]
    lgb.train(params,training,num_boost_round=args.trees,valid_sets=[validation],valid_names=['validation'],feval=evaluate,callbacks=callbacks)
    log.close()
    save_json(args.out/'training-complete.json',dict(run_id=args.run_id,seconds=time.monotonic()-started,
              iterations=len(history),best_iteration=best_iteration,history=history))


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--dataset-dir',type=Path,default=Path('tmp/rl/deathmatch-classic-20m/data'))
    p.add_argument('--source-report',type=Path,default=Path('tmp/rl/deathmatch-classic-20m/report.json'))
    p.add_argument('--out',type=Path,default=Path('tmp/rl/deathmatch-classic-trees'))
    p.add_argument('--seconds',type=float,default=1200)
    p.add_argument('--trees',type=int,default=64)
    p.add_argument('--depth',type=int,default=6)
    p.add_argument('--early-stopping-rounds',type=int,default=12)
    p.add_argument('--worker',action='store_true',help=argparse.SUPPRESS)
    p.add_argument('--run-id',default='',help=argparse.SUPPRESS)
    args=p.parse_args()
    if not np.isfinite(args.seconds) or not 0<args.seconds<=1200 or not 1<=args.trees<=1024 or not 1<=args.depth<=6 or not 1<=args.early_stopping_rounds<=1024:
        p.error('use <=1200 seconds, 1..1024 trees, depth 1..6 and early stopping rounds 1..1024')
    args.out.mkdir(parents=True,exist_ok=True)
    if args.worker:train_worker(args);return
    run_id=str(uuid.uuid4());started=time.monotonic();cutoff=False
    child=subprocess.Popen([sys.executable,'-m','hex_command_rl.trees','--worker','--run-id',run_id,*sys.argv[1:]])
    try:
        code=child.wait(timeout=args.seconds)
        if code:raise RuntimeError(f'tree trainer exited with {code}')
    except subprocess.TimeoutExpired:
        cutoff=True;child.terminate()
        try:child.wait(timeout=5)
        except subprocess.TimeoutExpired:child.kill();child.wait()
    except BaseException:
        child.kill();child.wait();raise
    training_seconds=time.monotonic()-started
    if not (args.out/'best-training.json').exists():raise RuntimeError('training ended before its first completed checkpoint')
    best=json.loads((args.out/'best-training.json').read_text())
    if best['run_id']!=run_id:raise RuntimeError('no checkpoint from this training run')
    import lightgbm as lgb
    from .train import evaluate,results_by_recruiter,results_summary
    booster=lgb.Booster(model_file=str(args.out/best['booster_file']))
    valid=load_dataset(args.dataset_dir/'validation');source=json.loads(args.source_report.read_text())
    manifest=json.loads((args.dataset_dir/'validation'/'manifest.json').read_text())
    if manifest['configs']!=source['validation_configs']:raise ValueError('source validation configs differ')
    model=(args.out/best['model_file']).read_text()
    save_json(args.out/'planner.json',json.loads(model))
    (args.out/'booster.txt').write_text((args.out/best['booster_file']).read_text())
    scores=booster.predict(valid[0],num_threads=3)
    actual=native.score(model,np.asarray(valid[0]));error=float(np.max(np.abs(scores-actual)))
    np.testing.assert_allclose(actual,scores,atol=1e-12,rtol=1e-12)
    np.testing.assert_array_equal(grouped_choices(actual,valid[1]),grouped_choices(scores,valid[1]))
    validation=grouped_metrics(scores,valid[1],valid[2])
    configs=source['evaluation_configs'];evaluation_started=time.monotonic()
    baseline=evaluate(configs);outcomes=evaluate(configs,model)
    if baseline!=source['teacher_results']:raise ValueError('native baseline outcomes changed')
    report=dict(architecture='gbdt_ranker',schema=native.SCHEMA,feature_names=native.FEATURE_NAMES,
                training_seconds=training_seconds,training_cutoff=cutoff,training_limit_seconds=args.seconds,
                best=best,tree_count=len(json.loads(model)['trees']),validation=validation,
                inference_max_absolute_error=error,inference_parity_rows=len(actual),
                inference_parity_decisions=len(valid[2]),inference_parity_choices_equal=True,evaluation_configs=configs,
                teacher_results=baseline,learned_results=outcomes,results_by_recruiter=results_by_recruiter(configs,baseline,outcomes),
                final_evaluation_seconds=time.monotonic()-evaluation_started,fingerprints=json.loads(native.fingerprints()))
    save_json(args.out/'report.json',report)
    print(json.dumps(dict(event='finished',trees=report['tree_count'],training_seconds=training_seconds,cutoff=cutoff,
                          validation=validation,learned=results_summary(outcomes),baseline=results_summary(baseline),
                          by_recruiter=report['results_by_recruiter'],parity_error=error)),flush=True)


if __name__=='__main__':main()
