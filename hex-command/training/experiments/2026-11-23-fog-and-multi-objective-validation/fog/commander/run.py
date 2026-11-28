import argparse
from collections import Counter, defaultdict
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
CLEAR = ROOT / 'tmp/rl/new-env-validation/commander'
CATALOG = OUT / 'catalog'
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
PREFIX = 'rl-commander-left-behind-'
VARIANTS = ('dread-long', 'dread-wounded-long', 'dread-wounded-healer')
POLICIES = ('hour8', 'hour10', 'full', 'classic', 'queued')
SCOPES = ('clear', 'player-fog', 'both-fog')


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def campaign_id(variant, scope):
    return PREFIX + variant + ('' if scope == 'clear' else f'-{scope}')


def campaign_dir(variant, scope):
    return CATALOG / 'game/campaigns' / campaign_id(variant, scope)


def build():
    if not CATALOG.exists():
        shutil.copytree(CLEAR / 'catalog', CATALOG)
    manifest_path = CATALOG / 'manifest.json'
    simulation_path = CATALOG / 'simulation/campaign.json'
    manifest, simulation = map(read, (manifest_path, simulation_path))
    for variant in VARIANTS:
        base = campaign_dir(variant, 'clear')
        for path in base.glob('*.json'):
            assert path.read_bytes() == (CLEAR / 'catalog/game/campaigns' / base.name / path.name).read_bytes()
        for scope in SCOPES[1:]:
            cid = campaign_id(variant, scope)
            dest = campaign_dir(variant, scope)
            shutil.copytree(base, dest, dirs_exist_ok=True)
            m = read(dest / 'map.json')
            m['mapId'] = cid
            for tile in m['tiles']:
                assert tile['environmentEffects'] == []
                tile['environmentEffects'] = ['fog-reveal-3']
            write(dest / 'map.json', m)
            roster = read(dest / 'roster.json')
            assert {g['id'] for g in roster['visionGroups']} == {'player', 'enemy'}
            for group in roster['visionGroups']:
                group['sight']['penetrates'] = ['fog'] if scope == 'player-fog' and group['id'] == 'enemy' else []
                group['explored'] = []
            write(dest / 'roster.json', roster)
            for name in ('stage.json', 'campaign.json'):
                path = dest / name
                doc = read(path)
                doc['id'] = cid
                write(path, doc)
            assert (dest / 'scripts.json').read_bytes() == (base / 'scripts.json').read_bytes()
            rel = f'game/campaigns/{cid}/campaign.json'
            if rel not in manifest['campaigns']:
                manifest['campaigns'].append(rel)
            simulation['campaigns'][cid] = simulation['campaigns'][base.name]
    write(manifest_path, manifest)
    write(simulation_path, simulation)


def config(variant, scope, policy, seed):
    return dict(campaign=campaign_id(variant, scope), objective='authored', diplomacy='unrestricted',
                player='predictor', enemy='classic', maxRounds=100, recruitLimit=0, seed=seed,
                playerBlocks=dict(recruiter='situational', campaign='mission',
                                  planner='queued' if policy.startswith('hour') else policy))


def run(name, cells, seed, count):
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    from hex_command_training import evaluate

    models = {f'hour{x["hour"]}': Path(x['model']) for x in read(RUN / 'evaluation/hourly-summary.json')}
    raw_path = OUT / f'{name}.jsonl'
    rows = defaultdict(list)
    with raw_path.open('w') as out:
        for policy in POLICIES:
            cases = [(variant, scope, config(variant, scope, policy, s))
                     for variant, scope in cells for s in range(seed, seed + count)]
            model = models[policy].read_text() if policy in models else None
            for (variant, scope, c), value in zip(cases, evaluate([json.dumps(c) for _, _, c in cases], model), strict=True):
                result = json.loads(value)
                rows[variant, scope, policy].append(result)
                out.write(json.dumps(dict(variant=variant, scope=scope, policy=policy, config=c, result=result)) + '\n')
            out.flush()
    grouped = []
    for (variant, scope, policy), results in sorted(rows.items()):
        wins = [r['missions'][0]['round'] for r in results if r['success']]
        caps = sum(r['truncated'] for r in results)
        entry = dict(variant=variant, scope=scope, policy=policy, successes=len(wins),
                     defeats=len(results) - len(wins) - caps, caps=caps,
                     medianWinRound=statistics.median(wins) if wins else None,
                     outcomes=dict(Counter(r.get('outcome') for r in results)))
        grouped.append(entry)
        print(variant, scope, policy, f'{len(wins)}/{len(results)}', flush=True)
    write(OUT / f'{name}-summary.json', dict(catalog=str(CATALOG), raw=str(raw_path), grouped=grouped,
                                              enemy='classic', seeds=list(range(seed, seed + count)),
                                              models={k: dict(path=str(v), sha256=digest(v)) for k, v in models.items()
                                                      if k in POLICIES}))


def main():
    p = argparse.ArgumentParser()
    p.add_argument('mode', choices=['discovery', 'confirmation'])
    args = p.parse_args()
    build()
    if args.mode == 'discovery':
        run('discovery', [(v, s) for v in VARIANTS for s in SCOPES], 491240001, 8)
    else:
        cells = read(OUT / 'confirmation-cells.json')
        run('confirmation', cells, 491241001, 20)


if __name__ == '__main__':
    main()
