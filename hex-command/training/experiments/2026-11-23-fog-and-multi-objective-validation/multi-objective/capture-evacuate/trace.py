"""Replay seeded panels with native phase logs and verify terminal parity."""
import argparse
import gzip
import json
from itertools import batched
import os
import subprocess

from build import OUT, ROOT, read, write
from evaluate import sha256

SIM = ROOT / 'rust/target/debug/hex-command-sim'
LEARNER = ROOT / 'rust/target/release/companion_trace'


def actor(units, name):
    return next(({key: unit[key] for key in ('id', 'faction', 'hp', 'tile')}
                 for unit in units if unit.get('character') == name), None)


def digest(row, mission):
    final = mission['final']
    first = {}
    phases = []
    for phase in mission['phases']:
        goals = {goal['label']: goal['done'] for goal in phase['goals']}
        for label, done in goals.items():
            if done and label not in first:
                first[label] = dict(round=phase['round'], phase=phase['side'])
        phases.append(dict(round=phase['round'], side=phase['side'], goals=goals,
                           mara=actor(phase['units'], 'mara'), tavin=actor(phase['units'], 'tavin'),
                           required=[target for target in (phase.get('planning') or {}).get('orders', {}).get('tactics', {}).get('targets', [])
                                     if target['id'].startswith('required:')],
                           orderProgress=(phase.get('planning') or {}).get('orders', {}).get('tactics', {}).get('progress', []),
                           revealed=phase['campaign']['scripts']['revealed'], actions=phase['actions'],
                           post_owner=next(ownership['owner'] for ownership in phase['ownership']
                                           if ownership['tile'] == {'col': 7, 'row': 4}),
                           hostiles=[unit['id'] for unit in phase['units'] if unit['faction'] == 'enemy'],
                           losses=phase['losses']))
    return dict(player=row['player'], variant=row['variant'], seed=row['config']['seed'],
                winner=mission['winner'], round=mission['round'], first=first,
                final={key: actor(final['units'], key) for key in ('mara', 'tavin')},
                goals=mission['goals'], flags=final['campaign']['flags'], phases=phases)


def replay(panel, names, variants, max_seeds):
    protocol = read(panel / 'protocol.json')
    rows = [json.loads(line) for line in (panel / 'results.jsonl').open()]
    rows = [row for row in rows if row['player'] in names and row['variant'] in variants
            and row['config']['seed'] in protocol['seeds'][:max_seeds]]
    target = panel / ('traces-' + '-'.join(variants))
    target.mkdir(exist_ok=False)
    catalog = protocol['catalog']
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=catalog, RAYON_NUM_THREADS='1')
    write(target / 'protocol.json', dict(catalog=catalog, cases=[row['config'] for row in rows],
                                        models=protocol['models'], workers=1,
                                        native_cli=dict(path=str(SIM), sha256=sha256(SIM)),
                                        learner_probe=dict(path=str(LEARNER), sha256=sha256(LEARNER),
                                                           source=str(ROOT / 'tmp/rl/v2-native-timing/current-probe/src/bin/companion_trace.rs'))))
    digests = []
    for name in names:
        selected = [row for row in rows if row['player'] == name]
        with gzip.open(target / f'{name}.jsonl.gz', 'wt') as file:
            for index, chunk in enumerate(batched(selected, 5)):
                cases = [row['config'] for row in chunk]
                if name in protocol['models']:
                    casefile = target / f'{name}-cases-{index}.json'
                    write(casefile, cases)
                    raw = subprocess.check_output([LEARNER, protocol['models'][name]['path'], catalog, casefile],
                                                  env=env, text=True)
                else:
                    raw = subprocess.check_output([SIM, '--data-dir', catalog, '--stdio'],
                                                  input=''.join(json.dumps(c) + '\n' for c in cases), env=env, text=True)
                entries = [json.loads(line) for line in raw.splitlines()]
                assert len(entries) == len(cases), (name, len(entries), len(cases))
                for row, entry in zip(chunk, entries):
                    result = entry.get('result', entry)
                    replayed, = result['missions']
                    expected, = row['result']['missions']
                    completed = result['completed' if 'completed' in result else 'success']
                    assert (replayed['winner'], replayed['round'], completed) == (
                        expected['winner'], expected['round'], row['result']['success']), (name, row['config'])
                    assert replayed['final']['campaign']['outcome'] == row['result']['outcome'], (name, row['config'])
                    file.write(json.dumps(dict(player=name, config=row['config'], result=result)) + '\n')
                    digests.append(digest(row, replayed))
        print(f'{name}: {len(selected)} terminal-matched traces', flush=True)
    with gzip.open(target / 'digest.jsonl.gz', 'wt') as output:
        for item in digests:
            output.write(json.dumps(item) + '\n')
    return digests


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('panel')
    parser.add_argument('--players', default='full,hour10,queued,classic,hour8')
    parser.add_argument('--variants', required=True)
    parser.add_argument('--seeds', type=int, default=20)
    args = parser.parse_args()
    replay(OUT / args.panel, args.players.split(','), args.variants.split(','), args.seeds)
