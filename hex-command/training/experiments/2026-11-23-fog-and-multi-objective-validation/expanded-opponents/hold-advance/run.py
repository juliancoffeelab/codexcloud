"""Matched native evaluation of Ashbank Relay across vision and enemy tactics."""
import argparse
from collections import Counter, defaultdict
from itertools import product
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import time


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREVIOUS = ROOT / 'tmp/rl/multi-objective-validation/hold-advance'
SOURCE = PREVIOUS / 'pressure-catalog'
CATALOG = OUT / 'catalog'
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
CAMPAIGN = 'rl-ashbank-relay-fork-vigil'
SCOPES = ('clear', 'player-fog', 'both-fog')
OPPONENTS = ('classic', 'queued', 'full', 'predictor')
REFERENCES = ('classic', 'queued', 'full')
PLAYERS = ('hour8', 'hour10', *REFERENCES)
DISCOVERY_SEEDS = list(range(571240001, 571240009))
CONFIRMATION_SEEDS = list(range(571241001, 571241021))
ENEMY_PRESETS = {
    'classic': ('classic', 'classic', 'classic'),
    'queued': ('balanced', 'balanced', 'queued'),
    'full': ('predictor', 'predictor', 'full'),
    'predictor': ('predictor', 'predictor', 'queued'),
}


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def catalog_digest(catalog):
    h = hashlib.sha256()
    for path in sorted(p for p in catalog.rglob('*') if p.is_file()):
        h.update(path.relative_to(catalog).as_posix().encode() + b'\0')
        h.update(bytes.fromhex(digest(path)))
    return h.hexdigest()


def case_id(scope):
    return CAMPAIGN if scope == 'clear' else f'{CAMPAIGN}-{scope}'


def stage(scope):
    return CATALOG / 'game/campaigns' / case_id(scope)


def build():
    source_digest = catalog_digest(SOURCE)
    assert read(PREVIOUS / 'final-summary.json')['catalogSha256'] == source_digest
    shutil.copytree(SOURCE, CATALOG, dirs_exist_ok=True)
    manifest_path, sim_path = CATALOG / 'manifest.json', CATALOG / 'simulation/campaign.json'
    manifest, simulation = read(manifest_path), read(sim_path)
    clear = stage('clear')
    for path in clear.glob('*.json'):
        assert path.read_bytes() == (SOURCE / path.relative_to(CATALOG)).read_bytes(), path
    for scope in SCOPES[1:]:
        dest = stage(scope)
        cid = case_id(scope)
        shutil.copytree(clear, dest, dirs_exist_ok=True)
        board = read(dest / 'map.json')
        assert all(tile['environmentEffects'] == [] for tile in board['tiles'])
        for tile in board['tiles']:
            tile['environmentEffects'] = ['fog-reveal-3']
        board['mapId'] = cid
        write(dest / 'map.json', board)
        roster = read(dest / 'roster.json')
        assert {group['id'] for group in roster['visionGroups']} == {'player', 'enemy'}
        for group in roster['visionGroups']:
            assert group['sight']['unlimited'] and not group['explored']
            group['sight']['penetrates'] = ['fog'] if scope == 'player-fog' and group['id'] == 'enemy' else []
        write(dest / 'roster.json', roster)
        for name in ('stage', 'campaign'):
            doc = read(dest / f'{name}.json')
            doc['id'] = cid
            write(dest / f'{name}.json', doc)
        assert (dest / 'scripts.json').read_bytes() == (clear / 'scripts.json').read_bytes()
        assert (dest / 'story.json').read_bytes() == (clear / 'story.json').read_bytes()
        manifest['campaigns'].append(f'game/campaigns/{cid}/campaign.json')
        simulation['campaigns'][cid] = simulation['campaigns'][CAMPAIGN]
    write(manifest_path, manifest)
    write(sim_path, simulation)
    write(OUT / 'catalog-manifest.json', dict(
        catalog=str(CATALOG), catalogSha256=catalog_digest(CATALOG),
        baseCatalog=str(SOURCE), baseCatalogSha256=source_digest,
        effectPath=str(CATALOG / 'environment-effects/fog-reveal-3.json'),
        stages={scope: {path.name: dict(path=str(path), sha256=digest(path))
                        for path in sorted(stage(scope).glob('*.json'))} for scope in SCOPES},
    ))


def blocks(recruiter, planner):
    return dict(recruiter=recruiter, campaign='mission', planner=planner)


def config(scope, opponent, player, seed):
    preset, recruiter, planner = ENEMY_PRESETS[opponent]
    return dict(campaign=case_id(scope), objective='authored', seed=seed, diplomacy='unrestricted',
                player='balanced', playerBlocks=blocks('situational', 'queued' if player.startswith('hour') else player),
                enemy=preset, enemyBlocks=blocks(recruiter, planner), maxRounds=100, recruitLimit=0)


def discover_jobs():
    return [dict(scope=scope, opponent=opponent, policy=player, config=config(scope, opponent, player, seed))
            for player, scope, opponent, seed in product(PLAYERS, SCOPES, OPPONENTS, DISCOVERY_SEEDS)]


def confirm_jobs():
    cells = read(OUT / 'confirmation-cells.json')['selected']
    return [dict(scope=scope, opponent=opp, policy=player, config=config(scope, opp, player, seed))
            for cell in cells for scope, opp, ref in [(cell['scope'], cell['opponent'], cell['reference'])]
            for player, seed in product(('hour8', 'hour10', ref), CONFIRMATION_SEEDS)]


def schedule(mode):
    jobs = discover_jobs() if mode == 'discovery' else confirm_jobs()
    seeds = DISCOVERY_SEEDS if mode == 'discovery' else CONFIRMATION_SEEDS
    catalog = read(OUT / 'catalog-manifest.json')
    model_paths = {f'hour{entry["hour"]}': Path(entry['model'])
                   for entry in read(RUN / 'evaluation/hourly-summary.json')}
    native = RUN / 'native/hex_command_training.so'
    manifest = dict(
        mode=mode, count=len(jobs), seeds=seeds, jobs=jobs,
        workers=1, catalog=str(CATALOG), catalogSha256=catalog['catalogSha256'],
        native=dict(path=str(native), sha256=digest(native)),
        models={p: dict(path=str(model_paths[p]), sha256=digest(model_paths[p])) for p in ('hour8', 'hour10')},
        expectedRuntimeSeconds=[round(len(jobs) * .025), round(len(jobs) * .080)],
    )
    write(OUT / f'{mode}-schedule.json', manifest)
    print(f'{mode}: {len(jobs)} matches, one worker, estimated {manifest["expectedRuntimeSeconds"]} seconds.', flush=True)
    return manifest


def evaluate(mode):
    schedule_path = OUT / f'{mode}-schedule.json'
    manifest = read(schedule_path)
    assert len(manifest['jobs']) == manifest['count']
    if not manifest['jobs']:
        write(OUT / f'{mode}-summary.json', dict(mode=mode, seeds=manifest['seeds'], grouped=[], count=0))
        return
    assert catalog_digest(CATALOG) == manifest['catalogSha256']
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    assert manifest['native']['sha256'] == digest(Path(native.__file__))
    saved = {}
    for policy, model in manifest['models'].items():
        path = Path(model['path'])
        assert digest(path) == model['sha256']
        saved[policy] = path.read_text()
    groups = defaultdict(list)
    for job in manifest['jobs']:
        groups[job['scope'], job['opponent'], job['policy']].append(job)
    raw = OUT / f'{mode}.jsonl'
    began = time.monotonic()
    rows = []
    with raw.open('w') as output:
        for (scope, opponent, policy), group in groups.items():
            results = [json.loads(item) for item in native.evaluate(
                [json.dumps(job['config']) for job in group], saved.get(policy))]
            for job, result in zip(group, results, strict=True):
                mission, = result['missions']
                assert mission['id'] == job['config']['campaign'], result
                print(json.dumps(dict(job, result=result), separators=(',', ':')), file=output)
            output.flush()
            caps = sum(r['truncated'] for r in results)
            wins = sum(r['success'] for r in results)
            row = dict(scope=scope, opponent=opponent, policy=policy, wins=wins,
                       losses=len(results) - wins - caps, caps=caps, count=len(results),
                       terminalReasons=Counter(r['outcome'] for r in results),
                       winRounds=Counter(r['missions'][0]['round'] for r in results if r['success']))
            rows.append(row)
            print(json.dumps(row), flush=True)
    elapsed = time.monotonic() - began
    payload = {k: v for k, v in manifest.items() if k != 'jobs'}
    write(OUT / f'{mode}-summary.json', dict(payload, grouped=rows, elapsedSeconds=elapsed,
                                              raw=str(raw), rawSha256=digest(raw),
                                              schedule=str(schedule_path),
                                              nativeFingerprints=json.loads(native.fingerprints())))


def main():
    cli = argparse.ArgumentParser()
    cli.add_argument('mode', choices=['build', 'discovery', 'confirmation'])
    cli.add_argument('--schedule-only', action='store_true')
    args = cli.parse_args()
    if args.mode == 'build':
        build()
    elif args.schedule_only:
        schedule(args.mode)
    else:
        evaluate(args.mode)


if __name__ == '__main__':
    main()
