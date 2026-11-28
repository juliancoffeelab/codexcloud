import argparse
from collections import Counter, defaultdict
import gzip
import json
import os
from pathlib import Path
import subprocess

from build import OUT, ROOT, read, write
from run import hash_file


TRACE = ROOT / 'rust/target/simulation/commander-trace-probe'


def coord(tile):
    return tile['col'], tile['row']


def distance(first, second):
    q = first[0] - first[1] // 2 - second[0] + second[1] // 2
    r = first[1] - second[1]
    return (abs(q) + abs(r) + abs(q + r)) // 2


def region_units(units, point, faction='player', radius=2):
    return [u['id'] for u in units
            if u['faction'] == faction and distance(coord(u['tile']), point) <= radius]


def held_owner(ownership, site):
    return next(t['owner'] for t in ownership if coord(t.get('tile', t)) == site)


def trace_profiles(case, mission):
    village, fort = tuple(case['sites']['hold']), tuple(case['sites']['capture'])
    phases = []
    def effective_damage(combat):
        return combat['mitigation']['attack']['effectiveDamage']

    for phase in mission['phases']:
        units = phase['units']
        attacks = [a for a in phase['actions'] if a['kind'] == 'attack']
        damage_on_village = sum(effective_damage(c) for c in phase['combats'] if coord(c['targetTile']) == village)
        damage_near_village = sum(effective_damage(c) for c in phase['combats']
                                  if distance(coord(c['targetTile']), village) <= 2)
        damage_near_fort = sum(effective_damage(c) for c in phase['combats']
                               if distance(coord(c['targetTile']), fort) <= 2)
        village_occupant = next((u for u in units if coord(u['tile']) == village), None)
        fort_occupant = next((u for u in units if coord(u['tile']) == fort), None)
        phases.append(dict(round=phase['round'], side=phase['side'], living=phase['living'],
                           hold=phase['goals'][0], capture=phase['goals'][1],
                           villageOwner=held_owner(phase['ownership'], village),
                           fortOwner=held_owner(phase['ownership'], fort),
                           villageOccupant=village_occupant, fortOccupant=fort_occupant,
                           playersNearVillage=region_units(units, village),
                           playersNearFort=region_units(units, fort),
                           enemiesNearVillage=region_units(units, village, 'enemy'),
                           enemiesNearFort=region_units(units, fort, 'enemy'),
                           attacks=len(attacks), damageOnVillage=damage_on_village,
                           damageNearVillage=damage_near_village, damageNearFort=damage_near_fort,
                           playerAttackActors=[a['unit'] for a in attacks] if phase['side'] == 'player' else [],
                           healing=phase['healing'], losses=phase['losses']))
    return phases


def classify(labeled, mission, phases, case):
    terminal = mission['final']['tiles']
    hold, capture = mission['goals']
    actual_clocks = {c['event']: c['turns'] for c in mission['final']['campaign']['scripts']['holds']}
    assert actual_clocks['hold-village'] == hold['current'], labeled
    village, fort = tuple(case['sites']['hold']), tuple(case['sites']['capture'])
    if labeled['result']['success']:
        assert hold['done'] and hold['current'] == hold['total'] == case['holdTurns'], labeled
        assert capture['done'] and capture['current'] == 1, labeled
        assert actual_clocks['win'] == case['holdTurns'], labeled
        assert held_owner(terminal, village) == held_owner(terminal, fort) == 'player', labeled
    changes = []
    previous = 'enemy'
    for phase in phases:
        if phase['fortOwner'] != previous:
            changes.append(dict(round=phase['round'], side=phase['side'], owner=phase['fortOwner'],
                                holdTurns=phase['hold']['current'],
                                fortOccupant=phase['fortOccupant']))
            previous = phase['fortOwner']
    first_fort = next((p for p in phases if p['fortOwner'] == 'player'), None)
    if labeled['result']['success'] and case['captureBy'] is not None:
        assert first_fort is not None and first_fort['round'] <= case['captureBy'], labeled
    own_phases = [p for p in phases if p['side'] == 'player']
    enemy_phases = [p for p in phases if p['side'] == 'ai']
    fort_progress = [p['fortOwner'] for p in own_phases]
    village_occupants = sorted({p['villageOccupant']['id'] for p in phases
                                if p['villageOccupant'] and p['villageOccupant']['faction'] == 'player'})
    return dict(firstFortCapture=changes[0] if changes else None, fortChanges=changes,
                terminalHold=hold['current'], terminalCapture=capture['current'],
                terminalVillageOwner=held_owner(terminal, village),
                terminalFortOwner=held_owner(terminal, fort),
                lostVillage=any(p['villageOwner'] == 'enemy' for p in phases)
                or held_owner(terminal, village) == 'enemy',
                enemyAttacks=sum(p['attacks'] for p in enemy_phases),
                enemyDamageOnVillage=sum(p['damageOnVillage'] for p in enemy_phases),
                enemyDamageNearVillage=sum(p['damageNearVillage'] for p in enemy_phases),
                enemyDamageNearFort=sum(p['damageNearFort'] for p in enemy_phases),
                simultaneousFronts=[p['round'] for p in own_phases
                                    if p['playersNearVillage'] and p['playersNearFort']],
                activeThreatBothFronts=[p['round'] for p in enemy_phases
                                      if p['damageNearVillage'] > 0 and p['damageNearFort'] > 0],
                playerVillageOccupants=village_occupants,
                captureProgress=fort_progress)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('label')
    parser.add_argument('--cases', type=Path, default=OUT / 'variants.json')
    parser.add_argument('--skip', nargs='*', default=[])
    args = parser.parse_args()
    summary = read(OUT / f'{args.label}-summary.json')
    cases = read(args.cases)
    rows = [json.loads(line) for line in Path(summary['raw']).read_text().splitlines()]
    groups = defaultdict(list)
    for row in rows:
        policy = row['policy']
        if policy not in args.skip:
            groups[policy if policy.startswith('hour') else 'references'].append(row)
    metadata = dict(binary=str(TRACE), binarySha256=hash_file(TRACE),
                    inputs={}, raw={}, exactTerminalReplayCount=0,
                    catalog=summary['catalog'], catalogSha256=summary['catalogSha256'])
    digest = []
    for label, selected in groups.items():
        job_path = OUT / f'{args.label}-trace-{label}-jobs.json'
        raw = OUT / f'{args.label}-trace-{label}.jsonl'
        jobs = [dict(player='learner' if label.startswith('hour') else row['policy'],
                     imitator=None if label.startswith('hour') else row['policy'], config=row['config'])
                for row in selected]
        write(job_path, jobs)
        model = summary['models'][label if label.startswith('hour') else 'hour10']['path']
        subprocess.run([str(TRACE), str(job_path), str(raw), model], check=True,
                       env=dict(os.environ, HEX_COMMAND_DATA_ROOT=summary['catalog'], RAYON_NUM_THREADS='1'))
        zipped_path = Path(str(raw) + '.gz')
        with raw.open() as input_stream, gzip.open(zipped_path, 'wt') as stream:
            for line, source in zip(input_stream, selected, strict=True):
                row = json.loads(line)
                attempt = row['result']
                mission, = attempt['missions']
                expected, = source['result']['missions']
                assert row['job']['config'] == source['config'], row['job']
                assert attempt['completed'] == source['result']['success'], source
                assert (mission['round'], mission['winner']) == (expected['round'], expected['winner']), source
                metadata['exactTerminalReplayCount'] += 1
                case = cases[source['variant']]
                phases = trace_profiles(case, mission)
                details = classify(source, mission, phases, case)
                digest.append(dict(source=source, observations=details, phases=phases))
                stream.write(json.dumps(dict(source=source, trace=attempt)) + '\n')
        raw.unlink()
        metadata['inputs'][label] = str(job_path)
        metadata['raw'][label] = dict(path=str(zipped_path), sha256=hash_file(zipped_path))
    digest_path = OUT / f'{args.label}-objective-trajectories.jsonl'
    with digest_path.open('w') as stream:
        for row in digest:
            stream.write(json.dumps(row, separators=(',', ':')) + '\n')
    metadata['objectiveTrajectories'] = str(digest_path)
    metadata['objectiveTrajectoriesSha256'] = hash_file(digest_path)
    aggregate = []
    for (tag, enemy, policy), results in sorted(groupby(digest).items()):
        wins = [r for r in results if r['source']['result']['success']]
        losses = [r for r in results if not r['source']['result']['success']]
        observations = [r['observations'] for r in results]
        aggregate.append(dict(
            variant=tag, enemy=enemy, policy=policy, replays=len(results),
            terminalGoals=Counter(f'{o["terminalHold"]}/10 + {o["terminalCapture"]}/1' for o in observations),
            villageLostMatches=sum(o['lostVillage'] for o in observations),
            matchesWithVillageDamage=sum(o['enemyDamageOnVillage'] > 0 for o in observations),
            matchesWithLiveAttacksAtBothFronts=sum(bool(o['activeThreatBothFronts']) for o in observations),
            winsWithVillageRotation=sum(len(r['observations']['playerVillageOccupants']) > 1 for r in wins),
            winsWithSplitAtCapture=sum(
                any(p['round'] == r['observations']['firstFortCapture']['round']
                    and p['side'] == r['observations']['firstFortCapture']['side']
                    and p['playersNearVillage'] and p['playersNearFort'] for p in r['phases']) for r in wins),
            successfulFirstCaptureRounds=Counter(r['observations']['firstFortCapture']['round'] for r in wins),
            lossesWithoutFort=sum(r['observations']['terminalCapture'] == 0 for r in losses),
        ))
    metadata['aggregate'] = aggregate
    write(OUT / f'{args.label}-trace-summary.json', metadata)


def groupby(digest):
    groups = defaultdict(list)
    for row in digest:
        source = row['source']
        groups[source['variant'], source['enemy'], source['policy']].append(row)
    return groups


if __name__ == '__main__':
    main()
