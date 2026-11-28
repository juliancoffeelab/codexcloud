"""Evaluate matched frozen native games and preserve terminal outcomes."""
import argparse
from collections import Counter
import hashlib
import json
import os
import sys

from build import OUT, RUN, read, write


def sha256(path):
    with open(path, 'rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def run(catalog, name, players, variants, start, count):
    panel = OUT / name
    panel.mkdir(exist_ok=False)
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    models = {f"hour{entry['hour']}": entry['model']
              for entry in read(RUN / 'evaluation/hourly-summary.json')}
    seeds = list(range(start, start + count))
    metadata = dict(catalog=str(catalog), seeds=seeds, players=players, variants=variants,
                    models={p: dict(path=models[p], sha256=sha256(models[p])) for p in players if p in models},
                    native=dict(path=native.__file__, sha256=sha256(native.__file__)), workers=1,
                    maxRounds=100, recruitLimit=0, diplomacy='unrestricted', enemy='classic',
                    objective='authored', playerBlocks=dict(recruiter='situational', campaign='mission'),
                    reward='Standard authored objective reward in frozen engine')
    write(panel / 'protocol.json', metadata)
    summaries = []
    with (panel / 'results.jsonl').open('w') as raw:
        for player in players:
            model = open(models[player]).read() if player in models else None
            for variant in variants:
                player_blocks = dict(metadata['playerBlocks'], planner='queued' if model else player)
                cases = [dict(campaign=f'rl-hollowbank-extraction-{variant}', seed=seed,
                              player='balanced', playerBlocks=player_blocks, enemy='classic',
                              diplomacy='unrestricted', objective='authored', maxRounds=100, recruitLimit=0)
                         for seed in seeds]
                games = [json.loads(row) for row in native.evaluate([json.dumps(config) for config in cases], model)]
                assert len(games) == len(cases)
                for config, game in zip(cases, games):
                    raw.write(json.dumps(dict(player=player, variant=variant, config=config, result=game)) + '\n')
                raw.flush()
                result = dict(player=player, variant=variant, games=count, wins=sum(g['success'] for g in games),
                              caps=sum(g['truncated'] for g in games), outcomes=Counter(g['outcome'] for g in games),
                              errors=sum(not g['missions'] for g in games))
                summaries.append(result)
                print(json.dumps(result), flush=True)
    write(panel / 'summary.json', summaries)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('catalog')
    parser.add_argument('name')
    parser.add_argument('start', type=int)
    parser.add_argument('count', type=int)
    parser.add_argument('--players', default='full,hour10,queued,classic,hour8')
    parser.add_argument('--variants', required=True)
    args = parser.parse_args()
    run(OUT / args.catalog, args.name, args.players.split(','), args.variants.split(','), args.start, args.count)
