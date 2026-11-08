"""Prepare isolated, event-free combat variants of the requested authored maps."""
import argparse,copy,json,shutil
from pathlib import Path
from .workloads import FACTIONS

MAPS=('open','river-valley','villages','sluice')
AUTHORED={'villages':('lantern-road','lantern-villages'),'sluice':('borrowed-tide','tide-sluice')}


def prepare_catalog(source,destination):
    source=Path(source);destination=Path(destination)
    shutil.copytree(source,destination,dirs_exist_ok=True)
    manifest=json.loads((destination/'manifest.json').read_text())
    arena=json.loads((destination/'simulation/arena.json').read_text())
    human=next(r['pool'] for r in json.loads((source/'simulation/rosters.json').read_text()) if r['id']=='human')
    for name,(campaign,stage) in AUTHORED.items():
        original=source/'game/campaigns'/campaign/'stages'/stage
        folder=destination/'training'/name;folder.mkdir(parents=True,exist_ok=True)
        board=json.loads((original/'map.json').read_text());board['mapRevision']=board.get('mapRevision',1)+1000
        for tile in board['tiles']:
            for key in ('fog','light','glow'):tile.pop(key,None)
        roster=json.loads((original/'roster.json').read_text())
        roster['factions']=[f for f in roster['factions'] if f['controller']['kind']!='passive']
        ids={f['id'] for f in roster['factions']}
        for faction in roster['factions']:
            faction['relations']={key:value for key,value in faction.get('relations',{}).items() if key in ids}
            for unit in faction['units']:unit.pop('aiGroup',None)
            if name=='villages' and faction['id']=='player':faction['recruits']=[dict(unit=k) for k in human]
        (folder/'map.json').write_text(json.dumps(board,indent=2))
        (folder/'roster.json').write_text(json.dumps(roster,indent=2))
        (folder/'scenario.json').write_text(json.dumps(dict(map='map.json',roster='roster.json')))
        manifest.setdefault('arenaScenarios',[]).append(f'training/{name}/scenario.json')
        territory=next(t for t in board['tiles'] if t['terrain'] in ('village','fort','keep'))
        arena['landscapes'].append(dict(id=name,holdTurns=4,map=board['mapId'],authored=True,territory=[dict(col=territory['col'],row=territory['row'])]))
    river=json.loads((source/'game/scenario/river-valley-v2/map.json').read_text())
    point=next(t for t in river['tiles'] if t['terrain']=='village')
    arena['landscapes'].append(dict(id='river-valley',holdTurns=4,map='river-valley',territory=[dict(col=point['col'],row=point['row'])]))
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2))
    (destination/'simulation/arena.json').write_text(json.dumps(arena,indent=2))
    return dict(maps=MAPS,authored_sources=AUTHORED,villages_player_recruits=human)


def workload(start,seeds,all_pairs=False,rounds=30,opponent='classic'):
    configs=[]
    for name in MAPS:
        multiplier=(12 if all_pairs else 4) if name in AUTHORED else 1
        for seed in range(start,start+seeds*multiplier):
            for recruiter in ('situational','predictor'):
                pairs=[('human','skeletons')] if name=='villages' else [('coastal','human')] if name=='sluice' else [(f,e) for i,f in enumerate(FACTIONS) for e in (FACTIONS if all_pairs else (FACTIONS[(i+1)%4],)) if e!=f]
                for player,enemy in pairs:
                    for first in ('player','enemy'):
                        configs.append(dict(seed=seed,armies=dict(player=player,enemy=enemy),landscape=name,player='balanced',enemy=opponent,
                            playerBlocks=dict(recruiter=recruiter,campaign='mission',planner='queued'),first=first,maxRounds=rounds,recruitLimit=None,objective='deathmatch'))
    return configs


def main():
    p=argparse.ArgumentParser();p.add_argument('--out',type=Path,default=Path('tmp/rl/representative'))
    p.add_argument('--collect',action='store_true');p.add_argument('--catalog',type=Path,default=Path('rust/data'));p.add_argument('--train-seeds',type=int,default=16);p.add_argument('--eval-seeds',type=int,default=3)
    args=p.parse_args()
    if min(args.train_seeds,args.eval_seeds)<1:p.error('seed counts must be positive')
    args.out.mkdir(parents=True,exist_ok=True)
    meta=prepare_catalog(args.catalog,args.out/'catalog')
    train=workload(1,args.train_seeds);valid=workload(10001,1);evaluation=workload(20001,args.eval_seeds,all_pairs=True,rounds=100)
    partitions=[{c['seed'] for c in cases} for cases in (train,valid,evaluation)]
    if any(partitions[i]&partitions[j] for i in range(3) for j in range(i)):raise ValueError('seed partitions overlap')
    import os
    os.environ['HEX_COMMAND_DATA_ROOT']=str((args.out/'catalog').resolve())
    import hex_command_training as native
    teacher=[json.loads(r) for r in native.evaluate([json.dumps(c) for c in evaluation],None)]
    source=dict(**meta,collect_side='player',balance_maps=True,teacher='queued',opponent='classic',fingerprints=json.loads(native.fingerprints()),
        train_configs=train,validation_configs=valid,evaluation_configs=evaluation,teacher_results=teacher)
    (args.out/'source.json').write_text(json.dumps(source,indent=2))
    if args.collect:
        from .spatial_data import collect_spatial_dataset
        for split,cases in [('train',train),('validation',valid)]:
            collect_spatial_dataset(cases,args.out/'corpus'/split,progress=lambda row:print(json.dumps(row),flush=True))
    print(json.dumps(dict(train_games=len(train),validation_games=len(valid),evaluation_games=len(evaluation),catalog=str((args.out/'catalog').resolve()))),flush=True)


if __name__=='__main__':main()
