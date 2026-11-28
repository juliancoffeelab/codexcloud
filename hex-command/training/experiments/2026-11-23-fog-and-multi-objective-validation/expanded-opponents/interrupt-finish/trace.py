"""Replay matched witnesses with the existing Marlow Cut arena trace adapter."""
from collections import defaultdict
import gzip
import itertools
import json
import os
from pathlib import Path
import subprocess

from build import CATALOG, ORIGINAL, OUT, POST, read, write
from evaluate import digest

REPLAY = ORIGINAL / 'probe/marlow-trace'


def events(panel):
    return [json.loads(line) for line in (OUT / panel / 'games.jsonl').read_text().splitlines()]


def histories(battle):
    player_taken = enemy_taken = None
    control = 'enemy'
    timers = {'player': 0, 'enemy': 0}
    phases = []
    for phase in battle['phases']:
        owner = next(tile['owner'] for tile in phase['ownership'] if tile['tile'] == POST)
        actions = phase['actions']
        if owner == 'player' and control != 'player' and player_taken is None:
            player_taken = phase['round']
        if player_taken is not None and owner == 'enemy' and control == 'player' and enemy_taken is None:
            enemy_taken = phase['round']
        if phase is not battle['phases'][-1] or battle['reason'] != 'leader':
            other = 'enemy' if phase['side'] == 'player' else 'player'
            for side in timers:
                if owner != side:
                    timers[side] = 0
                elif side == other:
                    timers[side] += 1
        phases.append(dict(round=phase['round'], side=phase['side'], ownerBefore=control, ownerAfter=owner,
                           clockAfter=dict(timers), actions=actions,
                           losses=[unit['id'] for unit in phase['losses']]))
        control = owner
    assert all(timers[side] == battle['objectives'][side]['held'] for side in timers)
    return dict(playerFirstTake=player_taken, enemyFirstRetake=enemy_taken, final=battle['objectives'], phases=phases)


def choose(screen, confirmation):
    target = ['finish', 'clear', 'classic']
    focus = next(row['config']['seed'] for row in screen if row['cell'] == target and row['player'] == 'hour10'
                 and row['result']['winner'] == 'playerDefeat')
    witnesses = []
    for selection in read(OUT / 'selection.json')['cells']:
        cell = selection['cell']
        sample = [row for row in screen if row['cell'] == cell and row['config']['seed'] == focus
                  and row['player'] in ('hour10', selection['reference'])]
        assert len(sample) == 2
        witnesses.extend(dict(row, panel='discovery') for row in sample)
    for cell, group in itertools.groupby(confirmation, key=lambda row: row['cell']):
        rows = list(group)
        seeds = {next(row['config']['seed'] for row in rows if row['player'] == 'hour10'
                      and row['result']['success'] == victory) for victory in (False, True)}
        witnesses.extend(dict(row, panel='confirmation') for row in rows if row['config']['seed'] in seeds)
    return witnesses


def main():
    candidates = choose(events('discovery'), events('confirmation'))
    protocol = read(OUT / 'discovery/protocol.json')
    groups = defaultdict(list)
    for row in candidates:
        groups[row['player']].append(row)
    target = OUT / 'traces'
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1')
    compact = []
    with gzip.open(target / 'raw.jsonl.gz', 'wt') as output:
        for player, rows in groups.items():
            cases = target / f'{player}-cases.json'
            write(cases, [row['config'] for row in rows])
            model = protocol['models'].get(player, {}).get('path', '-')
            with subprocess.Popen([REPLAY, model, CATALOG, cases], text=True, env=env,
                                  stdout=subprocess.PIPE) as process:
                for want, line in zip(rows, process.stdout, strict=True):
                    details = json.loads(line)
                    assert tuple(details[key] for key in ('winner', 'reason', 'rounds')) == tuple(
                        want['result'][key] for key in ('winner', 'reason', 'rounds'))
                    output.write(json.dumps(dict(want, detail=details)) + '\n')
                    compact.append(dict(want, objectiveHistory=histories(details)))
                assert process.wait() == 0
    write(target / 'objective-progress.json', compact)
    write(target / 'protocol.json', dict(replayer=dict(path=str(REPLAY), sha256=digest(REPLAY)), games=len(compact),
                                        reference='terminal winner, reason, round, and native hold scores verified'))
    print(f'Validated {len(compact)} matched phase replays.')


if __name__ == '__main__':
    (OUT / 'traces').mkdir(exist_ok=True)
    main()
