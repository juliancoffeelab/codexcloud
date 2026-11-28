from collections import Counter, defaultdict
import hashlib
import json
from statistics import median

from build import BASE, CATALOG, HERE, read, write


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(rows):
    stats = {}
    for policy, games in rows.items():
        wins = [g for g in games if g['result']['success']]
        failures = Counter(g['result']['failure'] for g in games if not g['result']['success'])
        endings = Counter(g['result']['outcome'] for g in games)
        stats[policy] = dict(wins=len(wins), losses=len(games)-len(wins)-sum(g['result']['truncated'] for g in games),
                             caps=sum(g['result']['truncated'] for g in games),
                             failure_classification=failures, terminal_outcomes=endings,
                             median_win_round=median(g['result']['missions'][0]['round'] for g in wins) if wins else None)
    return stats


def groups(file, name):
    by_route = defaultdict(lambda: defaultdict(list))
    for line in (HERE / file).read_text().splitlines():
        row = json.loads(line)
        if row['config']['campaign'] != f'{BASE}-{name}' or row['opponent'] != 'classic':
            continue
        by_route[row['config']['objective']][row['policy']].append(row)
    return by_route


def main():
    name = 'screen-close-fog'
    metadata = read(HERE / 'confirmation-metadata.json')
    discovery = groups('tune-4.jsonl', name)
    confirmation = groups('confirmation.jsonl', name)
    exact_catalog = CATALOG / 'game/campaigns' / f'{BASE}-{name}'
    rules = CATALOG / 'simulation/campaign.json'
    seeds = list(range(metadata['seed_start'], metadata['seed_start'] + metadata['seeds']))
    conditions = []
    for route, visibility in [('player-fog', 'Player-only radius 2'),
                              ('both-fog', 'Both sides radius 2'),
                              ('authored', 'Full vision')]:
        cases = confirmation[route]
        for games in cases.values():
            assert sorted(g['config']['seed'] for g in games) == seeds
        gap = summarize(cases)['queued']['wins'] - summarize(cases)['hour10']['wins']
        queued_by_seed = {g['config']['seed']: g['result']['success'] for g in cases['queued']}
        hour10_by_seed = {g['config']['seed']: g['result']['success'] for g in cases['hour10']}
        conditions.append(dict(route=route, visibility=visibility, confirmation=summarize(cases),
                               tuning=summarize(discovery[route]), queued_lead_wins=gap,
                               queued_only_wins=sum(queued_by_seed[s] and not hour10_by_seed[s] for s in seeds),
                               hour10_only_wins=sum(hour10_by_seed[s] and not queued_by_seed[s] for s in seeds),
                               qualification='rejected'))
    rendered = HERE / 'render' / 'preview.png'
    trace = HERE / 'paired-mechanism-traces.jsonl.gz'
    record = dict(accepted=[], rejected=[{
        'id': f'{BASE}-{name}', 'name': 'Kiln Crossing',
        'reason': 'The discovery separation did not independently reproduce. Queued led by 2/20 in player-only fog and trailed hour10 by 1/20 in both-sides fog. All hour10 losses were commander deaths.',
        'conditions': conditions, 'layout': read(HERE / 'variants.json')[name],
        'files': {str(p.relative_to(HERE)): sha(p) for p in sorted(exact_catalog.iterdir())},
    }], opponent='Classic', confirmation_seed_list=seeds, tuning_seeds=list(range(591030001,591030009)),
        native_binding=metadata['native'], native_sha256=metadata['native_sha256'], models=metadata['models'],
        references={'queued': 'Ordinary rule-based Queued planner', 'full': 'Ordinary rule-based Full planner'},
        episode_budget='100 harness rounds with authored defeat at round 11',
        catalog_sha256={str(p.relative_to(HERE)): sha(p) for p in [CATALOG / 'manifest.json', rules]},
        raw={'discovery': [f'tune-{i}.jsonl' for i in range(1,6)], 'confirmation': 'confirmation.jsonl'},
        render='render/preview.png' if rendered.is_file() else None, trace=trace.name if trace.is_file() else None)
    write(HERE / 'final-summary.json', record)


if __name__ == '__main__':
    main()
