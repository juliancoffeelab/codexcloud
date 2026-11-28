from collections import defaultdict
import gzip
import json
import os
from pathlib import Path
import subprocess

from run import CATALOG, CLEAR, OUT, ROOT, RUN, SCOPES, VARIANTS, campaign_dir, digest, read, write


REFERENCES = {
    ('dread-long', 'clear'): 'full',
    ('dread-wounded-long', 'clear'): 'classic',
    ('dread-wounded-healer', 'clear'): 'classic',
    ('dread-wounded-healer', 'both-fog'): 'full',
    **{(variant, 'player-fog'): 'queued' for variant in VARIANTS},
}


def coord(tile):
    return tile['col'], tile['row']


def subset(rows, outcome_tables):
    selected = []
    for (variant, scope), candidates in rows.items():
        scores = outcome_tables[variant, scope]
        reference = REFERENCES.get((variant, scope)) or max(('full', 'classic', 'queued'), key=lambda p: scores[p]['successes'])
        h10 = {r['config']['seed']: r for r in candidates['hour10']}
        ref = {r['config']['seed']: r for r in candidates[reference]}
        seeds = sorted(ref)
        reference_wins = [s for s in seeds if ref[s]['result']['success'] and not h10[s]['result']['success']]
        learned_wins = [s for s in seeds if h10[s]['result']['success'] and not ref[s]['result']['success']]
        selected_seeds = {seeds[0], *reference_wins[:2], *learned_wins[:1]}
        for s in sorted(selected_seeds):
            selected.extend([h10[s], ref[s]])
    return selected


def verify_view(row, mission, observations):
    scope = row['scope']
    should_hide = {'player': scope != 'clear', 'enemy': scope == 'both-fog'}
    expected_fog = [] if scope == 'clear' else [dict(kind='fog', revealWithin=3, requires=dict(terrains=None, illumination=None))]
    records = {}
    for moment in ('initial', 'final'):
        state = mission[moment]
        assert state['cols'] * state['rows'] == 204
        assert all([overlay for overlay in tile['perception']['concealment'] if overlay['kind'] == 'fog'] == expected_fog
                   for tile in state['tiles'])
        groups = {g['id']: g['sight'] for g in state['visionGroups']}
        views = {obs['faction']: obs for obs in observations[moment]}
        records[moment] = {}
        unit_ids = {unit['id'] for unit in state['units']}
        for faction in state['factions']:
            id = faction['id']
            g = groups[faction['visionGroup']]
            assert faction['visionGroup'] == id and g['unlimited'] and g['penetrates'] == ([] if should_hide[id] else ['fog'])
            fog = {coord(tile) for tile in views[id]['fog']}
            visible_units = set(views[id]['visibleUnits'])
            if moment == 'initial':
                assert bool(fog) == should_hide[id]
                assert (visible_units != unit_ids) == should_hide[id]
            records[moment][id] = dict(visionGroup=faction['visionGroup'], penetrates=g['penetrates'],
                                       unlimitedLegacySight=g['unlimited'], hiddenTiles=len(fog),
                                       visibleUnits=sorted(visible_units), hiddenUnits=sorted(unit_ids - visible_units))
            if moment == 'final':
                first_fog = {coord(tile) for tile in observations['initial'][0 if id == 'player' else 1]['fog']}
                faction_alive = any(unit['faction'] == id for unit in state['units'])
                records[moment][id]['tilesFoggedAgain'] = len(fog - first_fog) if should_hide[id] and faction_alive else 0
    return dict(overlay=expected_fog, authoredTiles=204 if scope != 'clear' else 0, **records)


def select_first_actions(mission):
    profiles = []
    for phase in mission['phases']:
        units = {u['id']: dict(tile=coord(u['tile']), hp=u['hp'], faction=u['faction'], type=u['type']) for u in phase['units']}
        post = next(own['owner'] for own in phase['ownership'] if coord(own['tile']) == (13, 6))
        profiles.append(dict(round=phase['round'], side=phase['side'], units=units, postOwner=post,
                             actions=phase['actions'], combats=phase['combats']))
    return profiles


def main():
    confirmation = read(OUT / 'confirmation-summary.json')
    outcome_tables = defaultdict(dict)
    for item in confirmation['grouped']:
        outcome_tables[item['variant'], item['scope']][item['policy']] = item
    rows = defaultdict(lambda: defaultdict(list))
    for line in (OUT / 'confirmation.jsonl').open():
        row = json.loads(line)
        rows[row['variant'], row['scope']][row['policy']].append(row)
    selected = subset(rows, outcome_tables)
    binary = ROOT / 'rust/target/simulation/commander-fog-probe'
    model = confirmation['models']['hour10']['path']
    evidence = []
    for label in ('hour10', 'rules'):
        traced = [row for row in selected if (row['policy'] == 'hour10') == (label == 'hour10')]
        job_path = OUT / f'trace-{label}-jobs.json'
        raw_path = OUT / f'trace-{label}.jsonl'
        write(job_path, traced)
        cmd = [binary, CATALOG, job_path, raw_path]
        if label == 'hour10':
            cmd += [model]
        subprocess.run(cmd, env=dict(os.environ, RAYON_NUM_THREADS='1'), check=True)
        with raw_path.open() as raw, gzip.open(str(raw_path) + '.gz', 'wt') as keep:
            for line, row in zip(raw, traced, strict=True):
                keep.write(line)
                parsed = json.loads(line)
                assert parsed['job'] == row
                result = parsed['result']
                mission, = result['missions']
                assert result['completed'] == row['result']['success']
                assert mission['round'] == row['result']['missions'][0]['round']
                assert mission['winner'] == row['result']['missions'][0]['winner']
                profiles = select_first_actions(mission)
                if result['completed']:
                    terminal = profiles[-1]
                    assert terminal['postOwner'] == 'player' and terminal['units']['u1']['tile'] == (15, 6)
                evidence.append(dict(row, vision=verify_view(row, mission, parsed['observations']), phases=profiles))
        raw_path.unlink()
    audit = {}
    for (variant, scope), scores in outcome_tables.items():
        group = [item for item in evidence if (item['variant'], item['scope']) == (variant, scope)]
        s = group[0]['vision']
        for id in ('player', 'enemy'):
            if s['initial'][id]['hiddenTiles']:
                assert any(item['vision']['final'][id]['tilesFoggedAgain'] > 0 for item in group)
        highest = max(('full', 'classic', 'queued'), key=lambda p: scores[p]['successes'])
        reference = REFERENCES.get((variant, scope), highest)
        gap = scores[reference]['successes'] - scores['hour10']['successes']
        paths = {path.name: dict(path=str(path), sha256=digest(path)) for path in campaign_dir(variant, scope).glob('*.json')}
        audit[f'{variant}/{scope}'] = dict(
            variant=variant, scope=scope, enemy='classic', reference=reference, referenceType='ordinary native rule-based planner',
            referenceBasis='discovery' if (variant, scope) in REFERENCES else 'best confirmation score; descriptive only',
            wins={policy: row['successes'] for policy, row in scores.items()},
            qualifies=(variant, scope) in REFERENCES and scores[reference]['successes'] >= 10 and gap >= 8,
            calibrationGap=gap, comparisons=scores, files=paths,
            visionAudit=dict(overlay=s['overlay'], authoredTiles=s['authoredTiles'], initial=s['initial'],
                             representativeSeed=group[0]['config']['seed'],
                             maximumTilesFoggedAgain={id: max(item['vision']['final'][id]['tilesFoggedAgain'] for item in group)
                                                      for id in ('player', 'enemy')},
                             matchingTerminalReplays=len(group)),
        )
    write(OUT / 'action-evidence.json', evidence)
    write(OUT / 'final-summary.json', dict(
        enemy='classic', exactFixtureCatalog=str(CATALOG), seeds=confirmation['seeds'],
        nativeBinding=dict(path=str(RUN / 'native/hex_command_training.so'), sha256=digest(RUN / 'native/hex_command_training.so')),
        models=confirmation['models'], qualification='At least 10 reference successes and an eight-win advantage over Hour 10 on 20 paired seeds.',
        scope=dict(authoring='static authored fog-reveal-3 on every tile; objective route authored has no route actions',
                   clear='both factions penetrate fog; no overlay',
                   playerFog='only enemy penetrates fog', bothFog='neither faction penetrates fog'),
        cells=audit, traceReplays=len(evidence), previousClear=str(CLEAR / 'final-summary.json'),
        discovery=str(OUT / 'discovery-summary.json'), confirmation=str(OUT / 'confirmation-summary.json'),
        actionEvidence=str(OUT / 'action-evidence.json'), rawTraces=[str(path) for path in sorted(OUT.glob('trace-*.jsonl.gz'))],
        traceSource=str(OUT / 'probe/src/main.rs'), builderAndRunner=str(OUT / 'run.py'),
    ))


if __name__ == '__main__':
    main()
