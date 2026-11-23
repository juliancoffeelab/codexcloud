from collections import Counter, defaultdict
import gzip
import json
import os
from pathlib import Path
import subprocess

from run import CATALOG, OUT, ROOT, OPPONENTS, SCOPES, digest, read, write


PROBE = ROOT / 'rust/target/simulation/commander-fog-probe'
PROBE_SOURCE = ROOT / 'tmp/rl/new-env-fog-validation/commander/probe/src/main.rs'
RESERVED = {'village': (4, 3), 'fort': (14, 7)}
FOG = dict(kind='fog', revealWithin=3, requires=dict(terrains=None, illumination=None))


def load_rows(cohort):
    path = OUT / f'{cohort}.jsonl'
    if not path.exists():
        return []
    with path.open() as lines:
        return [json.loads(line) for line in lines]


def index_rows(rows):
    groups = defaultdict(dict)
    for row in rows:
        groups[row['scope'], row['opponent'], row['config']['seed']][row['policy']] = row
    return groups


def select_jobs(discovery, confirmation):
    registry = read(OUT / 'confirmation-cells.json')
    selected = []
    discovery_index = index_rows(discovery)
    for cell in registry['selected'] + registry['rejected']:
        scope, opponent = cell['scope'], cell['opponent']
        representative = next(by_policy[cell['reference']] for (s, o, _), by_policy in discovery_index.items()
                              if (s, o) == (scope, opponent))
        selected.append(dict(representative, cohort='discovery'))
    confirmation_index = index_rows(confirmation)
    for cell in registry['selected']:
        scope, opponent, ref = cell['scope'], cell['opponent'], cell['reference']
        matches = [rows for (s, o, _), rows in confirmation_index.items() if (s, o) == (scope, opponent)]
        matched = [m for m in matches if m[ref]['result']['success'] and not m['hour10']['result']['success']]
        samples = matched[:2] or matches[:1]
        for sample in samples:
            selected.extend(dict(sample[p], cohort='confirmation') for p in (ref, 'hour8', 'hour10'))
    return selected


def coord(tile):
    return tile['col'], tile['row']


def fog_audit(scope, mission, observations):
    affected = dict(player=scope != 'clear', enemy=scope == 'both-fog')
    expected = [FOG] if scope != 'clear' else []
    settings = {}
    for moment in ('initial', 'final'):
        state = mission[moment]
        assert state['cols'] * state['rows'] == 187
        assert all([fog for fog in tile['perception']['concealment'] if fog['kind'] == 'fog'] == expected
                   for tile in state['tiles'])
        groups = {group['id']: group['sight'] for group in state['visionGroups']}
        shown = {view['faction']: view for view in observations[moment]}
        units = {u['id'] for u in state['units']}
        settings[moment] = {}
        for faction in state['factions']:
            side = faction['id']
            group = groups[faction['visionGroup']]
            assert faction['visionGroup'] == side and group['unlimited']
            if scope != 'clear':
                assert group['penetrates'] == ([] if affected[side] else ['fog'])
            fog_tiles = {coord(tile) for tile in shown[side]['fog']}
            visible_units = set(shown[side]['visibleUnits'])
            assert visible_units <= units
            assert {u['id'] for u in state['units'] if u['faction'] == side} <= visible_units
            if moment == 'initial':
                assert bool(fog_tiles) == affected[side]
                assert (visible_units != units) == affected[side]
            settings[moment][side] = dict(
                visionGroup=faction['visionGroup'], penetrates=group['penetrates'],
                factionAlive=any(u['faction'] == side for u in state['units']),
                hiddenTiles=len(fog_tiles), visibleUnits=sorted(visible_units), hiddenUnits=sorted(units - visible_units),
                fogTiles=sorted(fog_tiles),
            )
    for side in affected:
        initial = set(settings['initial'][side]['fogTiles'])
        final = set(settings['final'][side]['fogTiles'])
        survived = settings['final'][side]['factionAlive']
        settings['final'][side]['tilesFoggedAgain'] = len(final - initial) if affected[side] and survived else 0
        settings['final'][side]['tilesRevealed'] = len(initial - final) if affected[side] and survived else 0
    return dict(scopedSides=[side for side in affected if affected[side]], overlay=expected, **settings)


def ownership(tiles):
    by_cell = {coord(t.get('tile', t)): t['owner'] for t in tiles}
    return {label: by_cell[point] for label, point in RESERVED.items()}


def objective_audit(mission):
    phases = []
    for phase in mission['phases']:
        sites = ownership(phase['ownership'])
        hold, capture = phase['goals']
        assert hold['total'] == 10 and capture['total'] == 1
        phases.append(dict(round=phase['round'], side=phase['side'], hold=hold['current'],
                           capture=capture['current'], **sites,
                           attackCount=sum(a['kind'] == 'attack' for a in phase['actions']),
                           moveCount=sum(a['kind'] == 'move' for a in phase['actions']),
                           lossIds=[u['id'] for u in phase['losses']]))
    by_site = ownership(mission['final']['tiles'])
    hold, capture = mission['goals']
    if mission['winner'] == 'playerVictory':
        assert by_site == {'village': 'player', 'fort': 'player'} and (hold['current'], capture['current']) == (10, 1)
        clocks = {c['event']: c['turns'] for c in mission['final']['campaign']['scripts']['holds']}
        assert clocks['hold-village'] == clocks['win'] == 10
    first_capture = next((p['round'] for p in phases if p['fort'] == 'player'), None)
    if mission['winner'] == 'playerVictory':
        assert first_capture is not None and first_capture <= 7
    enemy = [p for p in phases if p['side'] == 'ai']
    return dict(firstCaptureRound=first_capture,
                lostVillage=any(p['village'] == 'enemy' for p in phases) or by_site['village'] == 'enemy',
                terminalHold=hold['current'], terminalCapture=capture['current'], terminalOwnership=by_site,
                enemyMoves=sum(p['moveCount'] for p in enemy), enemyAttacks=sum(p['attackCount'] for p in enemy),
                phases=phases)


def compare_queued_predictor(discovery):
    counts = Counter()
    different = []
    indexed = index_rows(discovery)
    for (scope, opponent, seed), rows in indexed.items():
        if opponent != 'queued':
            continue
        predictor = indexed[scope, 'predictor', seed]
        for policy, row in rows.items():
            other = predictor[policy]
            equal = row['result'] == other['result']
            counts['compared'] += 1
            if not equal:
                different.append(dict(scope=scope, seed=seed, policy=policy,
                                      queued=row['result'], predictor=other['result']))
    return dict(compared=counts['compared'], differences=different)


def main():
    discovery, confirmation = load_rows('discovery'), load_rows('confirmation')
    jobs = select_jobs(discovery, confirmation)
    models = read(OUT / 'discovery-summary.json')['models']
    metadata = dict(
        probe=dict(path=str(PROBE), sha256=digest(PROBE), source=str(PROBE_SOURCE), sourceSha256=digest(PROBE_SOURCE)),
        raw={}, count=len(jobs), catalog=str(CATALOG), catalogSha256=read(OUT / 'catalog-manifest.json')['catalogSha256'],
    )
    compact = []
    for policy in ('references', 'hour8', 'hour10'):
        selected = [job for job in jobs if (job['policy'] if job['policy'].startswith('hour') else 'references') == policy]
        if not selected:
            continue
        path = OUT / f'trace-{policy}-jobs.json'
        write(path, selected)
        temporary = OUT / f'trace-{policy}.jsonl'
        command = [str(PROBE), str(CATALOG), str(path), str(temporary)]
        if policy in models:
            command.append(models[policy]['path'])
        subprocess.run(command, check=True, env=dict(os.environ, RAYON_NUM_THREADS='1'))
        raw = temporary.with_suffix('.jsonl.gz')
        with temporary.open() as source, gzip.open(raw, 'wt') as stream:
            for line, expected in zip(source, selected, strict=True):
                row = json.loads(line)
                assert expected == row['job']
                result = row['result']
                assert result['config']['enemyBlocks'] == expected['config']['enemyBlocks']
                assert result['config']['enemy'] == expected['config']['enemy']
                mission, = result['missions']
                comparison, = expected['result']['missions']
                assert result['completed'] == expected['result']['success']
                assert (mission['winner'], mission['round'], mission['id']) == (
                    comparison['winner'], comparison['round'], comparison['id'])
                assert mission['final']['campaign']['outcome'] == expected['result']['outcome']
                compact.append(dict(expected, vision=fog_audit(expected['scope'], mission, row['observations']),
                                    objectives=objective_audit(mission)))
                stream.write(line)
        temporary.unlink()
        metadata['raw'][policy] = dict(path=str(raw), sha256=digest(raw), jobs=str(path))
    metadata['visibility'] = {}
    for scope in SCOPES:
        representatives = [row for row in compact if row['scope'] == scope]
        metadata['visibility'][scope] = dict(
            observedGames=len(representatives),
            initialHiddenTiles={s: representatives[0]['vision']['initial'][s]['hiddenTiles'] for s in ('player', 'enemy')},
            initialHiddenEnemies={s: len(representatives[0]['vision']['initial'][s]['hiddenUnits']) for s in ('player', 'enemy')},
            maxRefogged={s: max(row['vision']['final'][s]['tilesFoggedAgain'] for row in representatives)
                         for s in ('player', 'enemy')},
            enemyActionsByOpponent={
                opponent: [{"seed": r['config']['seed'], "moves": r['objectives']['enemyMoves'],
                            "attacks": r['objectives']['enemyAttacks']} for r in representatives
                           if r['opponent'] == opponent and r['cohort'] == 'discovery']
                for opponent in OPPONENTS
            },
        )
        for side in representatives[0]['vision']['scopedSides']:
            assert metadata['visibility'][scope]['maxRefogged'][side] > 0
    metadata['returnedFogMeasurement'] = 'Initially visible tiles hidden at terminal state for a fog-limited faction with surviving units.'
    metadata['queuedVsPredictorDiscovery'] = compare_queued_predictor(discovery)
    path = OUT / 'replay-evidence.json'
    write(path, compact)
    metadata['replayEvidence'] = dict(path=str(path), sha256=digest(path))
    write(OUT / 'audit-summary.json', metadata)


if __name__ == '__main__':
    main()
