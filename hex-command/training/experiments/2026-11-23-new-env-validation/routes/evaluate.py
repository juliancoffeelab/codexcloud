"""Evaluate one seeded panel using the frozen native training evaluator."""
import argparse
import hashlib
import json
import os
import sys
from collections import Counter

from build import OUT, RUN, read, write


def configs(version, modes, start, count, planner):
    return [{
        'campaign': f'rl-scree-relay-{version}-{mode}', 'seed': seed,
        'diplomacy': 'unrestricted', 'objective': 'authored', 'player': 'balanced',
        'enemy': 'classic', 'playerBlocks': {'recruiter': 'situational', 'campaign': 'mission',
                                              'planner': planner},
        'maxRounds': 100, 'recruitLimit': 0,
    } for mode in modes for seed in range(start, start + count)]


def run(version, name, players, modes, start, count):
    out = OUT / version / name
    catalog = OUT / version / 'catalog'
    out.mkdir(parents=True, exist_ok=True)
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    models = {f"hour{entry['hour']}": entry['model'] for entry in read(RUN / 'evaluation/hourly-summary.json')}
    results = []
    panel = {player: configs(version, modes, start, count, player if player not in models else 'queued')
             for player in players}
    write(out / 'protocol.json', dict(catalog=str(catalog), seed_start=start, seeds=count,
                                     models={p: dict(path=models[p], sha256=hashlib.sha256(open(models[p], 'rb').read()).hexdigest())
                                             for p in players if p in models}, cases=panel))
    with (out / 'results.jsonl').open('w') as raw:
        for player, cfgs in panel.items():
            model = open(models[player]).read() if player in models else None
            for mode in modes:
                cases = [config for config in cfgs if config['campaign'].endswith(f'-{mode}')]
                games = [json.loads(row) for row in native.evaluate([json.dumps(config) for config in cases], model)]
                assert len(games) == len(cases)
                for config, game in zip(cases, games):
                    raw.write(json.dumps(dict(player=player, config=config, result=game)) + '\n')
                raw.flush()
                outcomes = Counter(game['outcome'] for game in games)
                metrics = dict(player=player, mode=mode, wins=sum(game['success'] for game in games),
                               games=len(games), caps=sum(game['truncated'] for game in games), outcomes=outcomes)
                results.append(metrics)
                print(json.dumps(metrics), flush=True)
    write(out / 'summary.json', results)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('version')
    parser.add_argument('name')
    parser.add_argument('start', type=int)
    parser.add_argument('count', type=int)
    parser.add_argument('--players', default='hour8,hour10,full,queued,classic')
    parser.add_argument('--modes', default='switch,stay')
    args = parser.parse_args()
    run(args.version, args.name, args.players.split(','), args.modes.split(','), args.start, args.count)
