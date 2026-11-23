"""Look for contacts rehidden from the affected player in additional real matches."""
from collections import defaultdict
import gzip
from itertools import product
import json
import os
import subprocess

from fog_audit import PROBE, SIM
from panel import CATALOG, DISCOVERY, OUT, VARIANTS, config, write


def run():
    path = OUT / 'visibility'
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1')
    requested = [dict(config=config(variant, mode, 'classic', 'classic', seed),
                      rounds=list(range(1, 10)), side=side)
                 for variant, mode, seed, side in product(VARIANTS, ('player-fog', 'both-fog'), DISCOVERY[1:5], ('player', 'ai'))]
    captured = subprocess.check_output([SIM, '--data-dir', CATALOG, '--capture-stdio'],
                                      input=''.join(json.dumps(req) + '\n' for req in requested), text=True, env=env)
    views = subprocess.check_output([PROBE, CATALOG], input=captured, text=True, env=env)
    for name, raw in [('additional-positions', captured), ('additional-views', views)]:
        with gzip.open(path / f'{name}.jsonl.gz', 'wt') as file:
            file.write(raw)
    states = defaultdict(list)
    for raw in views.splitlines():
        row = json.loads(raw)
        if row['faction'] == 'player' and row['living_observers'] and row['hostile']:
            match = row['config']
            states[match['campaign'], match['objective'], match['seed']].append(row)
    found = {}
    for (campaign, route, seed), snapshots in states.items():
        if (campaign, route) in found:
            continue
        snapshots.sort(key=lambda entry: (entry['round'], entry['side'] == 'ai'))
        for before, after in zip(snapshots, snapshots[1:]):
            present = {u['id'] for u in before['hostile']} & {u['id'] for u in after['hostile']}
            left = set(before['visible']) & present - set(after['visible'])
            if left:
                found[campaign, route] = dict(campaign=campaign, mode=route, seed=seed, unit=min(left),
                                               before=dict(round=before['round'], side=before['side']),
                                               after=dict(round=after['round'], side=after['side']))
                break
    write(path / 'additional-contact-summary.json', dict(sampled_games=len(states), witnesses=list(found.values())))
    print(list(found.values()), flush=True)


if __name__ == '__main__':
    run()
