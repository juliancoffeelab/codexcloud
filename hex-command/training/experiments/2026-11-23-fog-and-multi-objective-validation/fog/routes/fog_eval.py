"""Run matched Scree Relay visibility panels against the frozen native evaluator."""
import argparse
import copy
import hashlib
import json
import os
import shutil
import sys
from collections import Counter
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
CLEAR = ROOT / 'tmp/rl/new-env-validation/routes/v8/catalog'
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
SCOPES = ('clear', 'player-fog', 'both-fog')
IDS = ('rl-scree-relay-v8-switch', 'rl-scree-relay-v8-stay')


def read(path):
    return json.loads(path.read_text())


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def catalog_for(scope):
    return CLEAR if scope == 'clear' else OUT / scope / 'catalog'


def build():
    old = read(CLEAR / 'simulation/campaign.json')
    event = old['campaigns']['rl-underground-escape']['routes']['fog']['stages']['rl-underground-escape-1']['scriptEvents'][0]
    assert event['when'] == {'chapterStarted': True}
    assert event['actions'][0]['addConcealment']['revealWithin'] == 3
    for scope in SCOPES[1:]:
        catalog = catalog_for(scope)
        shutil.copytree(CLEAR, catalog, dirs_exist_ok=True)
        simulation = copy.deepcopy(old)
        for cid in IDS:
            rel = Path('game/campaigns') / cid
            roster = read(CLEAR / rel / 'roster.json')
            assert [(f['id'], f['visionGroup']) for f in roster['factions']] == [('player', 'player'), ('enemy', 'enemy')]
            for group in roster['visionGroups']:
                assert group['sight']['unlimited'] and group['sight']['penetrates'] == ['fog']
                if group['id'] == 'player' or scope == 'both-fog':
                    group['sight']['penetrates'] = []
            write(catalog / rel / 'roster.json', roster)
            simulation['campaigns'][cid]['routes'][scope] = {'stages': {cid: {
                'scriptConditions': {}, 'scriptEvents': [event], 'ignoreOptionalWhen': None}}}
            for name in ('map', 'scripts', 'stage', 'story', 'campaign'):
                assert digest(CLEAR / rel / f'{name}.json') == digest(catalog / rel / f'{name}.json')
        write(catalog / 'simulation/campaign.json', simulation)


def capture_configs(scope, players, first, count):
    return {player: [dict(
        campaign=cid, seed=seed, diplomacy='unrestricted',
        objective='authored' if scope == 'clear' else scope,
        player='balanced', enemy='classic', maxRounds=100, recruitLimit=0,
        playerBlocks=dict(recruiter='situational', campaign='mission',
                          planner='queued' if player.startswith('hour') else player),
    ) for cid in IDS for seed in range(first, first+count)] for player in players}


def evaluate(scope, cohort, first, count, players):
    catalog = catalog_for(scope)
    choices = capture_configs(scope, players, first, count)
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    models = {f"hour{r['hour']}": Path(r['model']) for r in read(RUN / 'evaluation/hourly-summary.json')}
    target = OUT / scope / cohort
    files = ['manifest.json', 'simulation/campaign.json'] + [
        f'game/campaigns/{cid}/{kind}.json'
        for cid in IDS for kind in ('map','roster','scripts','stage','story','campaign')]
    write(target / 'protocol.json', dict(
        catalog=str(catalog), visibility=scope, seeds=[first, first+count-1],
        fingerprints=json.loads(native.fingerprints()), files={p:digest(catalog / p) for p in files},
        models={p:{'path':str(models[p]), 'sha256':digest(models[p])} for p in players if p in models},
        cases=choices,
    ))
    rows = []
    with (target / 'results.jsonl').open('w') as raw:
        for policy, cases in choices.items():
            model = models[policy].read_text() if policy in models else None
            for cid in IDS:
                selected = [c for c in cases if c['campaign']==cid]
                outcomes = [json.loads(r) for r in native.evaluate([json.dumps(c) for c in selected], model)]
                assert len(outcomes) == count
                for config, result in zip(selected, outcomes):
                    raw.write(json.dumps(dict(player=policy, config=config, result=result))+'\n')
                raw.flush()
                wins = sum(r['success'] for r in outcomes)
                caps = sum(r['truncated'] for r in outcomes)
                result = dict(player=policy,campaign=cid,successes=wins,defeats=count-wins-caps,caps=caps,
                              outcomes=Counter(r['outcome'] for r in outcomes))
                rows.append(result)
                print(scope, cohort, json.dumps(result), flush=True)
    write(target / 'summary.json', rows)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('scope', choices=(*SCOPES,'build'))
    parser.add_argument('cohort', nargs='?')
    parser.add_argument('first', nargs='?', type=int)
    parser.add_argument('count', nargs='?', type=int)
    parser.add_argument('--players', default='hour8,hour10,full,queued,classic')
    a = parser.parse_args()
    if a.scope == 'build':
        build()
    else:
        evaluate(a.scope, a.cohort, a.first, a.count, a.players.split(','))
