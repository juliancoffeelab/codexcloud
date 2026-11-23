"""Audit actual faction-filtered native views at saved planner boundaries."""
from collections import defaultdict
import gzip
from itertools import product
import json
import os
import subprocess

from panel import CATALOG, DISCOVERY, ENEMIES, OUT, ROOT, VARIANTS, VISIBILITY, config, sha256, write

SIM = ROOT / 'rust/target/debug/hex-command-sim'
PROBE = ROOT / 'rust/target/release/hollowbank_fog_view'


def coords(hexes):
    return {(tile['col'], tile['row']) for tile in hexes}


def analyze(fixture_rows):
    groups = defaultdict(list)
    for row in fixture_rows:
        key = (row['config']['campaign'], row['config']['objective'], row['faction'])
        groups[key].append(row)
    checks = []
    for (campaign, route, faction), rows in groups.items():
        visibility = next(mode for mode, (entry, _) in VISIBILITY.items() if route == entry)
        masked = faction in VISIBILITY[visibility][1]
        rows = [row for row in rows if row['living_observers'] and row['hostile']]
        rows.sort(key=lambda r: (r['round'], r['side'] != 'player'))
        first = rows[0]
        hidden = lambda row: {u['id'] for u in row['hostile']} - set(row['visible'])
        if not masked:
            assert all(not row['fog'] and not hidden(row) for row in rows)
        else:
            assert first['fog'] and hidden(first)
        tile_witness = next((dict(tile=min(back), from_round=before['round'], from_side=before['side'],
                                  to_round=after['round'], to_side=after['side'])
                             for before, after in zip(rows, rows[1:])
                             if (back := coords(after['fog']) - coords(before['fog']))), None)
        contact_witness = next((dict(unit=min(lost), from_round=before['round'], from_side=before['side'],
                                     to_round=after['round'], to_side=after['side'])
                                for before, after in zip(rows, rows[1:])
                                if (lost := set(before['visible']) & hidden(after))), None)
        if masked:
            assert tile_witness is not None, (campaign, route, faction)
        checks.append(dict(campaign=campaign, mode=visibility, faction=faction, sampled_positions=len(rows),
                           initial_fog_tiles=len(first['fog']), initial_hidden_hostiles=sorted(hidden(first)),
                           open_tile_returned_to_fog=tile_witness, living_contact_left_native_view=contact_witness))
    return checks


def run():
    path = OUT / 'visibility'
    path.mkdir(exist_ok=True)
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1')
    first_seed = DISCOVERY[0]
    requests = [dict(config=config(variant, mode, 'classic', 'classic', first_seed),
                     rounds=list(range(1, 14)), side=side)
                for variant, mode, side in product(VARIANTS, VISIBILITY, ('player', 'ai'))]
    requests += [dict(config=config(VARIANTS[0], 'both-fog', enemy, 'classic', first_seed),
                      rounds=[1], side='ai') for enemy in ('queued', 'full', 'predictor')]
    storage_path = path / 'captured-positions.jsonl.gz'
    if storage_path.exists():
        with gzip.open(storage_path, 'rt') as storage:
            captures = storage.read()
    else:
        captures = subprocess.check_output([SIM, '--data-dir', CATALOG, '--capture-stdio'],
                                           input=''.join(json.dumps(request) + '\n' for request in requests),
                                           text=True, env=env)
        with gzip.open(storage_path, 'wt') as storage:
            storage.write(captures)
    entries = [json.loads(line) for line in captures.splitlines()]
    assert len(entries) == len(requests)
    for requested, observed in zip(requests, entries):
        config_readback = observed['config']
        assert config_readback == requested['config']
        block_key = 'enemyBlocks' if requested['side'] == 'ai' else 'playerBlocks'
        assert observed['positions']
        assert all(position['request']['blocks'] == config_readback[block_key]
                   for position in observed['positions'])
    # Sample returning visibility from complete trajectories; the extra one-phase
    # requests only audit explicit opponent block readback.
    full_trajectories = '\n'.join(json.dumps(e) for e in entries if e['config']['enemy'] == 'classic') + '\n'
    native_views = subprocess.check_output([PROBE, CATALOG], input=full_trajectories, text=True, env=env)
    with gzip.open(path / 'native-views.jsonl.gz', 'wt') as storage:
        storage.write(native_views)
    checks = analyze([json.loads(line) for line in native_views.splitlines()])
    write(path / 'summary.json', dict(requests=requests, results=checks, verified_enemy_blocks=ENEMIES,
                                      capture_fingerprints=entries[0]['fingerprints'],
                                      cli=dict(path=str(SIM), sha256=sha256(SIM)),
                                      view_probe=dict(path=str(PROBE), sha256=sha256(PROBE))))
    for check in checks:
        print(check, flush=True)


if __name__ == '__main__':
    run()
