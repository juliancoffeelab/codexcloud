"""Capture phase records and require them to reproduce the frozen evaluator outcomes."""
import argparse
import gzip
import json
import os
import subprocess
import sys

from build import OUT, ROOT, RUN, read, write

SIM = ROOT / 'rust/target/debug/hex-command-sim'
LEARNER = ROOT / 'rust/target/release/companion_trace'


def compact(row, match):
    mission, = match['missions']
    end = mission['final']
    turns = []
    for phase in mission['phases']:
        before = {unit['id']: [unit['tile']['col'], unit['tile']['row'], unit['hp']] for unit in phase['before']}
        positions = {unit['id']: [unit['tile']['col'], unit['tile']['row'], unit['hp']] for unit in phase['units']}
        turns.append(dict(turn=phase['round'], side=phase['side'], before=before, after=positions,
                          actions=phase['actions'], losses=phase['losses']))
    return dict(player=row['player'], config=row['config'], outcome=row['result']['outcome'],
                success=row['result']['success'], rounds=mission['round'],
                goals=mission['goals'], final=end.get('units'), phases=turns)


def run(version, players, sample):
    result_folder = OUT / version / 'confirmation'
    protocol = read(result_folder / 'protocol.json')
    all_rows = [json.loads(line) for line in (result_folder / 'results.jsonl').read_text().splitlines()]
    catalog = protocol['catalog']
    target = result_folder / 'traces'
    target.mkdir(exist_ok=True)
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=catalog, RAYON_NUM_THREADS='1')
    bundles = []
    for player in players:
        picked = [row for row in all_rows if row['player'] == player and row['config']['seed'] < protocol['seed_start'] + sample]
        cases = [row['config'] for row in picked]
        if player in protocol['models']:
            casefile = target / f'{player}-cases.json'
            write(casefile, cases)
            raw = subprocess.check_output([LEARNER, protocol['models'][player]['path'], catalog, casefile], env=env, text=True)
        else:
            raw = subprocess.check_output([SIM, '--data-dir', catalog, '--stdio'],
                                          input=''.join(json.dumps(case) + '\n' for case in cases), env=env, text=True)
        entries = [json.loads(line) for line in raw.splitlines()]
        assert len(entries) == len(cases), (player, len(entries), len(cases))
        path = target / f'{player}.jsonl.gz'
        with gzip.open(path, 'wt') as file:
            for row, entry in zip(picked, entries):
                battle = entry.get('result', entry)
                want, = row['result']['missions']
                final, = battle['missions']
                assert (want['winner'], want['round']) == (final['winner'], final['round']), (player, row['config'], want, final['winner'], final['round'])
                file.write(json.dumps(dict(player=player, config=row['config'], result=battle)) + '\n')
                bundles.append(compact(row, battle))
    write(target / 'trace-extract.json', bundles)
    print(f'{len(bundles)} matching traces in {target}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('version')
    parser.add_argument('sample', type=int)
    parser.add_argument('--players', default='hour8,hour10,full')
    args = parser.parse_args()
    run(args.version, args.players.split(','), args.sample)
