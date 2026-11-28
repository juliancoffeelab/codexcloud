"""Verify effective compiled fog and record each faction's native observations."""
import argparse
import gzip
import json
import os
import subprocess
import sys
from collections import Counter

from fog_eval import OUT, ROOT, RUN, IDS, SCOPES, catalog_for, read, write

SIM = ROOT / 'rust/target/debug/hex-command-sim'
LEARNER = ROOT / 'rust/target/release/companion_trace'
FOG = dict(kind='fog', revealWithin=3, requires=dict(terrains=None, illumination=None))


def verify_state(state, scope):
    declared = {record['id']:record['sight'] for record in state['visionGroups']}
    by_faction = {row['id']:row['visionGroup'] for row in state['factions']}
    assert by_faction == {'player':'player', 'enemy':'enemy'}, by_faction
    expected = {'player':scope == 'clear', 'enemy':scope != 'both-fog'}
    sightings = {faction:declared[group] for faction, group in by_faction.items()}
    for faction, sight in sightings.items():
        assert sight['unlimited'] and sight['penetrates'] == (['fog'] if expected[faction] else []), sight
    applied = [effect for tile in state['tiles'] for effect in tile['perception']['concealment'] if effect['kind'] == 'fog']
    for tile in state['tiles']:
        tile_fog = [effect for effect in tile['perception']['concealment'] if effect['kind']=='fog']
        assert tile_fog == ([] if scope=='clear' else [FOG]), (scope,tile)
    assert len(state['tiles']) == 198
    return dict(factionVisionGroup=by_faction, sight=sightings, totalTiles=len(state['tiles']),
                fogTileCount=len(applied), overlay=applied[0] if applied else None,
                stateTurn=state['turn'], stateRound=state['round'])


def scope_route(scope, cid):
    catalog = catalog_for(scope)
    registered = read(catalog/'simulation/campaign.json')['campaigns'][cid]['routes']
    if scope == 'clear':
        assert not registered
        return None
    stage = registered[scope]['stages'][cid]
    assert not stage['scriptConditions'] and stage['ignoreOptionalWhen'] is None
    event, = stage['scriptEvents']
    assert event['when']=={'chapterStarted':True}
    assert event['actions']==[{'addConcealment':dict(area='map',**FOG)}]
    assert len(registered) == 1
    return event


def sample(scope, count):
    catalog = catalog_for(scope)
    basis = OUT/scope/'confirmation'
    protocol = read(basis/'protocol.json')
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(catalog), RAYON_NUM_THREADS='1')
    output = basis/'evidence'
    output.mkdir(exist_ok=True)
    assert json.loads(subprocess.check_output([SIM,'--data-dir',catalog,'--fingerprint'],env=env)) == protocol['fingerprints']
    selected = [json.loads(line) for line in (basis/'results.jsonl').read_text().splitlines()]
    selected = [entry for entry in selected if entry['config']['seed'] < protocol['seeds'][0]+count]
    by_fixture = {cid:dict(routeEvent=scope_route(scope,cid), phaseWitnesses=[]) for cid in IDS}
    phases = []
    for pilot in protocol['cases']:
        cohort = [entry for entry in selected if entry['player']==pilot]
        configs = [entry['config'] for entry in cohort]
        if pilot.startswith('hour'):
            cases = output/f'{pilot}-configs.json'
            write(cases, configs)
            args = [LEARNER,protocol['models'][pilot]['path'],catalog,cases]
            raw = subprocess.check_output(args,env=env,text=True)
        else:
            raw = subprocess.check_output([SIM,'--data-dir',catalog,'--stdio'],env=env,text=True,
                                          input=''.join(json.dumps(c)+'\n' for c in configs))
        runs = [json.loads(line) for line in raw.splitlines()]
        assert len(runs) == len(configs)
        with gzip.open(output/f'{pilot}-phases.jsonl.gz','wt') as handle:
            for entry,replay in zip(cohort,runs):
                config = entry['config']
                cid = config['campaign']
                run = replay.get('result',replay)
                mission, = run['missions']
                expected, = entry['result']['missions']
                assert all(expected[key] == mission[key] for key in ('id','round','winner'))
                initial, final = [verify_state(mission[key],scope) for key in ('initial','final')]
                start = mission['phases'][0]
                assert (start['round'],start['side']) == (1,'player')
                captain_moves = [command for phase in mission['phases'] if phase['side']=='player'
                                 for command in phase['actions'] if command['kind']=='move' and command.get('unit')=='u1']
                player_bypass = any(step['row']==7 for cmd in captain_moves for step in cmd['path'])
                by_fixture[cid]['phaseWitnesses'].append(dict(seed=config['seed'], pilot=pilot,
                    engineInitial=initial, engineFinal=final, firstAppliedPhase={'round':start['round'],'side':start['side']},
                    captainUsedBypass=player_bypass, winner=mission['winner'], terminalRound=mission['round']))
                if len(phases) < 2*count or pilot == 'full':
                    phases.append(dict(config=config,pilot=pilot,firstPhase=start['actions'],captainMoves=captain_moves))
                handle.write(json.dumps(dict(config=config,pilot=pilot,result=run))+'\n')
    write(output/'fog-scope-audit.json',by_fixture)
    write(output/'captain-trace-examples.json',phases)
    print(scope,'verified replay starts and terminal states',len(selected))


def views(scope):
    catalog = catalog_for(scope)
    basis = OUT/scope/'confirmation'
    protocol = read(basis/'protocol.json')
    os.environ['HEX_COMMAND_DATA_ROOT']=str(catalog)
    os.environ['RAYON_NUM_THREADS']='1'
    sys.path.insert(0,str(RUN/'native'))
    import hex_command_training as native

    assert json.loads(native.fingerprints()) == protocol['fingerprints']
    channels = {name:i for i,name in enumerate(native.SPATIAL_CHANNEL_NAMES)}
    reports = []
    for config in protocol['cases']['full']:
        if config['seed'] != protocol['seeds'][0]:
            continue
        grid = read(catalog/'game/campaigns'/config['campaign']/'map.json')
        minimum_q = min(t['col']-t['row']//2 for t in grid['tiles'])
        minimum_r = min(t['row'] for t in grid['tiles'])
        for side in ('player','enemy'):
            _, spatial, metadata = native.collect_spatial_sequences([json.dumps(config)],256,side)
            values, offsets, shapes, _, _, _ = spatial
            records = json.loads(metadata)
            seen = set()
            last = set()
            for i, record in enumerate(records):
                turn = record['context']['round']
                if turn in seen:
                    continue
                seen.add(turn)
                height,width = (int(x) for x in shapes[i])
                planes = values[offsets[i]:offsets[i+1]].reshape(-1,height,width)
                def sites(channel):
                    ys,xs = (planes[channels[channel]]>0.5).nonzero()
                    return sorted((int(x+minimum_q+(y+minimum_r)//2), int(y+minimum_r)) for y,x in zip(ys,xs)
                                  if planes[channels['valid'],y,x]>0.5)
                sight = set(sites('tile_in_sight'))
                reports.append(dict(config=config, observer=record['context']['observer'],turn=turn,
                                    friendly=sites('friendly'),visibleHostile=sites('hostile'),
                                    visibleTiles=len(sight), rememberedTiles=len(sites('tile_remembered')),
                                    newlyFoggedTiles=sorted(last-sight), newlySeenTiles=sorted(sight-last)))
                last=sight
    write(basis/'evidence/native-visible-information.json',reports)
    print(scope,'native reference-observer snapshots',len(reports))


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('scope',choices=SCOPES)
    parser.add_argument('count',type=int,default=3,nargs='?')
    args=parser.parse_args()
    sample(args.scope,args.count)
    views(args.scope)
