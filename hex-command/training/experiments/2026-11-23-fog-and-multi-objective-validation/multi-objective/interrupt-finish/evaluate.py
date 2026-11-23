"""Run matched complete games using the frozen native Python extension."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from build import BASE, MODES, OUT, ROOT, RUN, VARIANTS, read, write


def digest(path):
    return hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()


def config(variant, mode, seed, planner):
    return {'objective': 'territory', 'seed': seed, 'landscape': f'rl-marlow-cut-{variant}-{mode}',
            'armies': {'player': 'human', 'enemy': 'skeletons'}, 'first': 'player', 'recruitLimit': 0,
            'player': 'balanced', 'enemy': 'classic', 'maxRounds': 30,
            'playerBlocks': {'recruiter': 'situational', 'campaign': 'arena', 'planner': planner}}


def run(catalog, dest, variants, start, count, players):
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    models = {f"hour{entry['hour']}": Path(entry['model'])
              for entry in read(RUN / 'evaluation/hourly-summary.json')}
    panel = {player: [config(variant, mode, seed, 'queued' if player in models else player)
                      for variant in variants for mode in MODES for seed in range(start, start + count)]
             for player in players}
    scenario_files = sorted(path for variant in variants for mode in MODES
                            for path in (catalog / f'game/scenario/rl-marlow-cut-{variant}-{mode}').glob('*.json'))
    files = [catalog / 'manifest.json', catalog / 'simulation/arena.json', *scenario_files]
    source_commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    write(dest / 'protocol.json', dict(catalog=str(catalog), frozenCatalog=str(BASE), sourceCommit=source_commit,
                                      sha256={str(path.relative_to(catalog)): digest(path) for path in files},
                                      seeds=list(range(start, start + count)), cases=panel,
                                      native=dict(path=native.__file__, sha256=digest(Path(native.__file__))),
                                      models={p: dict(path=str(models[p]), sha256=digest(models[p]))
                                              for p in players if p in models}))
    results = []
    with (dest / 'games.jsonl').open('w') as file:
        for player, cases in panel.items():
            model = models[player].read_text() if player in models else None
            for offset in range(0, len(cases), count):
                selected = cases[offset:offset+count]
                games = [json.loads(row) for row in native.evaluate([json.dumps(case) for case in selected], model)]
                assert len(games) == len(selected)
                for case, game in zip(selected, games):
                    file.write(json.dumps(dict(player=player, config=case, result=game)) + '\n')
                file.flush()
                outcomes = Counter((game.get('winner'), game.get('reason')) for game in games)
                row = dict(player=player, landscape=selected[0]['landscape'], games=len(games),
                           outcomes=[dict(winner=key[0], reason=key[1], count=value) for key, value in outcomes.items()])
                results.append(row)
                print(json.dumps(row), flush=True)
    write(dest / 'summary.json', results)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('name')
    parser.add_argument('start', type=int)
    parser.add_argument('count', type=int)
    parser.add_argument('--variants', default=','.join(VARIANTS))
    parser.add_argument('--players', default='full,queued,classic,hour8,hour10')
    parser.add_argument('--catalog', type=Path, default=OUT / 'catalog')
    args = parser.parse_args()
    run(args.catalog.resolve(), OUT / args.name, args.variants.split(','), args.start, args.count, args.players.split(','))
