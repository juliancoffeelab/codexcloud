"""Screen the full matched matrix and confirm qualified discovery cells."""
import argparse
from collections import Counter
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
import sys

from build import BASE, CATALOG, OUT, ROOT, RUN, STARTS, VISIBILITY, arena, location, read, write

DISCOVERY = range(661136001, 661136009)
CONFIRMATION = range(661136101, 661136121)
LEARNERS = ('hour8', 'hour10')
REFERENCES = ('queued', 'full', 'classic')
ENEMIES = {
    'classic': ('classic', 'classic', 'mission', 'classic'),
    'queued': ('balanced', 'balanced', 'mission', 'queued'),
    'full': ('predictor', 'predictor', 'mission', 'full'),
    'predictor': ('predictor', 'predictor', 'mission', 'queued'),
}


def digest(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()


def blocks(recruiter, campaign, planner):
    return dict(recruiter=recruiter, campaign=campaign, planner=planner)


def configuration(cell, player, seed):
    start, visibility, opponent = cell
    preset, *composition = ENEMIES[opponent]
    planner = 'queued' if player in LEARNERS else player
    return dict(objective='territory', landscape=arena(start, visibility),
                armies={'player': 'human', 'enemy': 'skeletons'}, first='player',
                recruitLimit=0, maxRounds=30, seed=seed, player='balanced', enemy=preset,
                playerBlocks=blocks('situational', 'arena', planner), enemyBlocks=blocks(*composition))


def selection(discovery):
    chosen = []
    screen = []
    for cell in itertools.product(STARTS, VISIBILITY, ENEMIES):
        slots = {row['player']: row for row in discovery if tuple(row['cell']) == cell}
        assert set(slots) == set(LEARNERS + REFERENCES) and all(row['games'] == 8 for row in slots.values())
        reference = max(REFERENCES, key=lambda name: slots[name]['wins'])
        qualifies = slots[reference]['wins'] >= 6 and slots[reference]['wins'] - slots['hour10']['wins'] >= 3
        record = dict(cell=cell, reference=reference, referenceWins=slots[reference]['wins'],
                      hour10Wins=slots['hour10']['wins'], qualifies=qualifies)
        if qualifies:
            chosen.append(record)
        screen.append(record)
    write(OUT / 'selection.json', dict(rule='Reference >=6/8 and at least 3 more wins than Hour 10. Ties use Queued, Full, Classic.',
                                      confirmation='Same reference >=15/20 and at least 6 more wins than Hour 10.',
                                      cells=screen))
    return chosen


def run(panel):
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    target = OUT / panel
    if panel == 'discovery':
        seeds = DISCOVERY
        jobs = [(cell, player) for cell in itertools.product(STARTS, VISIBILITY, ENEMIES)
                for player in LEARNERS + REFERENCES]
    else:
        seeds = CONFIRMATION
        selected = selection(read(OUT / 'discovery/summary.json'))
        jobs = [(tuple(entry['cell']), player) for entry in selected for player in LEARNERS + (entry['reference'],)]
    entries = [dict(cell=cell, player=player, cases=[configuration(cell, player, seed) for seed in seeds])
               for cell, player in jobs]
    names = {f"hour{model['hour']}": Path(model['model']) for model in read(RUN / 'evaluation/hourly-summary.json')}
    models = {player: dict(path=str(names[player]), sha256=digest(names[player])) for player in LEARNERS}
    assets = [CATALOG / 'manifest.json', CATALOG / 'simulation/arena.json', CATALOG / 'environment-effects/fog-reveal-3.json']
    for start, visibility in itertools.product(STARTS, VISIBILITY):
        assets.extend(sorted(location(CATALOG, arena(start, visibility)).glob('*.json')))
    write(target / 'protocol.json', dict(panel=panel, seeds=list(seeds), jobs=entries, count=len(entries) * len(seeds),
                                         catalog=str(CATALOG), parentCatalog=str(BASE), models=models,
                                         opponents={name: dict(preset=preset, blocks=blocks(*composition))
                                                    for name, (preset, *composition) in ENEMIES.items()},
                                         native=dict(path=native.__file__, sha256=digest(Path(native.__file__)),
                                                     fingerprints=json.loads(native.fingerprints())),
                                         sourceCommit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                                         sha256={str(path.relative_to(CATALOG)): digest(path) for path in assets}))
    model_contents = {who: Path(info['path']).read_text() for who, info in models.items()}
    results = []
    with (target / 'games.jsonl').open('w') as output:
        for entry in entries:
            player, cases = entry['player'], entry['cases']
            games = [json.loads(value) for value in native.evaluate([json.dumps(case) for case in cases],
                                                                  model_contents.get(player))]
            assert len(games) == len(seeds)
            for game, case in zip(games, cases, strict=True):
                output.write(json.dumps(dict(cell=entry['cell'], player=player, config=case, result=game)) + '\n')
            output.flush()
            reasons = Counter((game['winner'], game['reason']) for game in games)
            summary = dict(cell=entry['cell'], player=player, games=len(games),
                           wins=sum(g['success'] for g in games), caps=sum(g['truncated'] for g in games),
                           outcomes=[dict(winner=winner, reason=reason, count=count) for (winner, reason), count in reasons.items()])
            results.append(summary)
            print(json.dumps(summary), flush=True)
    write(target / 'summary.json', results)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('panel', choices=('discovery', 'confirmation'))
    run(parser.parse_args().panel)
