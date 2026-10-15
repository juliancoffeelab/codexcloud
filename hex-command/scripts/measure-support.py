"""Measure support coverage from saved JSONL arena journals, without running battles.

Pass the unit-data directory matching the recorded run explicitly. This audits
turn-start healing and turn-start aura coverage, not aura damage during combat.
"""
import argparse
from collections import Counter, defaultdict
import gzip
import hashlib
import json
from pathlib import Path


def adjacent(a, b):
    x, y = map(int, a.split(','))
    u, v = map(int, b.split(','))
    q, r = x - y // 2 - u + v // 2, y - v
    return (abs(q) + abs(r) + abs(q + r)) == 2


def observe(units, side, ability, catalog, tiles, owners):
    friends = [u for u in units if u['side'] == side]
    providers = [u for u in friends if catalog[u['type']].get(ability, 0) > 0]
    counts = Counter(providerTurns=len(providers))
    for ally in friends:
        touching = [p for p in providers if p['id'] != ally['id'] and adjacent(p['tile'], ally['tile'])]
        if not touching:
            continue
        counts['contacts'] += len(touching)
        counts['coveredRecipients'] += 1
        counts['overlappingRecipients'] += len(touching) > 1
        if ability != 'healing':
            continue
        rule = catalog[ally['type']]
        terrain_heal = 4 if tiles[ally['tile']]['terrain'] in ('village', 'grove') and owners.get(ally['tile']) == side else 0
        missing = max(0, rule['hp'] - ally['hp'] - terrain_heal)
        if missing:
            # Count each actual recipient once, regardless of equal/stronger
            # providers overlapping. This yields useful recipients per provider.
            counts['healedRecipients'] += 1
            counts['healedHP'] += min(missing, max(catalog[p['type']]['healing'] for p in touching))
    return counts


def measure(report_path, unit_path):
    report = json.loads(report_path.read_text())
    if report.get('status') != 'complete':
        raise ValueError('Support measurements require a complete report')
    definitions = sorted(unit_path.glob('*.json'))
    catalog = {r['id']: r for p in definitions for r in [json.loads(p.read_text())]}
    digest = hashlib.sha256(b''.join(p.read_bytes() for p in definitions)).hexdigest()
    groups = defaultdict(Counter)
    readers = {}
    matches = 0
    try:
        for saved in sorted(report['outcomes'], key=lambda o: (o['detail']['file'], o['detail']['offset'])):
            config = saved['config']
            if saved['kind'] != 'arena' or config['armies']['player'] == config['armies']['enemy']:
                continue
            sides = [s for s, other in [('player', 'enemy'), ('enemy', 'player')] if config[other] == 'balanced']
            if not sides:
                continue
            ref = saved['detail']
            name = ref['file']
            if Path(name).name != name or not name.endswith('.jsonl'):
                raise ValueError('Expected an adjacent JSONL gameplay journal')
            if name not in readers:
                path = report_path.parent / name
                readers[name] = path.open('rb') if path.exists() else gzip.open(str(path) + '.gz', 'rb')
            stream = readers[name]
            stream.seek(ref['offset'])
            raw = stream.read(ref['bytes'])
            if hashlib.sha256(raw).hexdigest() != ref['sha256']:
                raise ValueError('Gameplay journal checksum mismatch')
            result = json.loads(raw)
            if result['config'] != config:
                raise ValueError('Gameplay journal configuration mismatch')
            matches += 1
            tiles = {t['id']: t for t in result['initial']['tiles']}
            previous = None
            for phase in result['phases']:
                side = phase['side']
                # A preceding enemy phase gives the exact pre-healing roster.
                # First turns have no such boundary and are excluded.
                if previous is not None and side in sides and previous['side'] != side:
                    owners = {t['tile']: t['owner'] for t in previous['ownership']}
                    for ability in ['healing', 'thornAura']:
                        stats = observe(previous['units'], side, ability, catalog, tiles, owners)
                        for landscape in [config['landscape'], 'all']:
                            groups[(config['armies'][side], config[side], landscape, ability)].update(stats)
                previous = phase
    finally:
        for stream in readers.values():
            stream.close()
    rows = []
    for (race, policy, landscape, ability), stats in sorted(groups.items()):
        if not stats['providerTurns']:
            continue
        rows.append(dict(race=race, policy=policy, map=landscape, ability=ability, **stats,
                         alliesPerProviderTurn=stats['contacts'] / stats['providerTurns'],
                         usefulRecipientsPerProviderTurn=stats['healedRecipients'] / stats['providerTurns'] if ability == 'healing' else None,
                         overlapPercent=100 * stats['overlappingRecipients'] / max(1, stats['coveredRecipients']),
                         redundantContactPercent=100 * (1 - stats['coveredRecipients'] / max(1, stats['contacts']))))
    return dict(report=str(report_path), sourceCommit=report.get('sourceCommit'), catalog=str(unit_path),
                catalogSHA256=digest, matches=matches, rows=rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--units', type=Path, required=True, help='Unit definitions matching the saved run')
    args = parser.parse_args()
    print(json.dumps(measure(args.report, args.units), indent=2))


if __name__ == '__main__':
    main()
