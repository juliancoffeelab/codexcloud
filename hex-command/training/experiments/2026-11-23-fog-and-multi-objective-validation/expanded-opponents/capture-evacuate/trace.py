"""Capture representative paired battles and require terminal agreement."""
import gzip
import json
import os
import subprocess

from panel import CATALOG, OUT, ROOT, VARIANTS, read, sha256, write

SIM = ROOT / 'rust/target/debug/hex-command-sim'
LEARNER = ROOT / 'rust/target/release/companion_trace'


def rows(name):
    with (OUT / name / 'results.jsonl').open() as result:
        return [json.loads(line) for line in result]


def paired(panel, variant, visibility, opponent, reference, reference_wins=None):
    index = {(row['player'], row['config']['seed']): row for row in panel
             if (row['variant'], row['visibility'], row['opponent']) == (variant, visibility, opponent)}
    seeds = sorted(seed for (policy, seed) in index if policy == 'hour10')
    seed = next(seed for seed in seeds if not index['hour10', seed]['result']['success']
                and (reference_wins is None or index[reference, seed]['result']['success'] == reference_wins))
    return [index[policy, seed] for policy in ('hour10', reference)]


def replay():
    discovery, confirmation = rows('discovery'), rows('confirmation')
    east, west = VARIANTS
    cases = paired(confirmation, east, 'player-fog', 'classic', 'classic', False)
    fog_loss_seed = cases[0]['config']['seed']
    cases += [r for r in confirmation if r['variant'] == east and r['visibility'] == 'clear' and
              r['config']['seed'] == fog_loss_seed and r['player'] in ('hour10', 'classic')]
    cases += paired(confirmation, east, 'player-fog', 'classic', 'classic', True)
    cases += paired(discovery, east, 'both-fog', 'classic', 'classic', False)
    cases += paired(confirmation, west, 'clear', 'queued', 'queued', True)
    path = OUT / 'trace'
    path.mkdir(exist_ok=False)
    protocol = read(OUT / 'confirmation/protocol.json')
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1')
    write(path / 'protocol.json', dict(cases=cases, models=protocol['models'], workers=1,
                                       native_cli=dict(path=str(SIM), sha256=sha256(SIM)),
                                       learner_probe=dict(path=str(LEARNER), sha256=sha256(LEARNER))))
    progress = []
    with gzip.open(path / 'replays.jsonl.gz', 'wt') as file:
        for row in cases:
            player = row['player']
            config = row['config']
            if player in protocol['models']:
                casefile = path / 'learner-case.json'
                write(casefile, [config])
                raw = subprocess.check_output([LEARNER, protocol['models'][player]['path'], CATALOG, casefile],
                                              env=env, text=True)
            else:
                raw = subprocess.check_output([SIM, '--data-dir', CATALOG, '--stdio'],
                                              input=json.dumps(config) + '\n', env=env, text=True)
            decoded = json.loads(raw)
            result = decoded.get('result', decoded)
            actual, = result['missions']
            expected, = row['result']['missions']
            assert (actual['winner'], actual['round'], actual['final']['campaign']['outcome']) == (
                expected['winner'], expected['round'], row['result']['outcome'])
            file.write(json.dumps(dict(case=row, result=result)) + '\n')
            goals = {}
            progress_by_phase = []
            for phase in actual['phases']:
                for goal in phase['goals']:
                    if goal['done'] and goal['label'] not in goals:
                        goals[goal['label']] = dict(round=phase['round'], side=phase['side'])
                if phase['side'] == 'player':
                    orders = (phase.get('planning') or {}).get('orders', {}).get('tactics', {})
                    progress_by_phase.append(dict(round=phase['round'], revealed=phase['campaign']['scripts']['revealed'],
                                                  targets=[t for t in orders.get('targets', []) if t['id'].startswith('required:')],
                                                  actors={u['character']: dict(tile=u['tile'], hp=u['hp'], faction=u['faction'])
                                                          for u in phase['units'] if u.get('character') in ('mara', 'tavin')}))
            if row['result']['success']:
                units = {u.get('character'): u for u in actual['final']['units']}
                for identity, tile in [('mara', {'col': 2, 'row': 9}), ('tavin', {'col': 16, 'row': 9})]:
                    unit = units[identity]
                    assert unit['hp'] > 0 and unit['faction'] == 'player' and unit['tile'] == tile
            progress.append(dict(variant=row['variant'], player=player, visibility=row['visibility'], opponent=row['opponent'],
                                 seed=config['seed'], reached=goals, round=actual['round'], outcome=row['result']['outcome'],
                                 player_progress=progress_by_phase))
    write(path / 'summary.json', progress)
    print(f'{len(progress)} replay samples matched winner, round, outcome text', flush=True)


if __name__ == '__main__':
    replay()
