from collections import Counter, defaultdict
import json
from pathlib import Path

from audit import compare_queued_predictor
from run import OUT, OPPONENTS, PLAYERS, SCOPES, read, write


def main():
    screen = read(OUT / 'discovery-summary.json')
    check = read(OUT / 'confirmation-summary.json')
    plan = read(OUT / 'confirmation-cells.json')
    audit_path = OUT / 'audit-summary.json'
    audit = read(audit_path)
    catalog = read(OUT / 'catalog-manifest.json')
    assert screen['catalogSha256'] == check['catalogSha256'] == audit['catalogSha256'] == catalog['catalogSha256']
    assert screen['count'] == 480 and check['count'] == len(plan['selected']) * 60
    assert not set(screen['seeds']) & set(check['seeds'])
    discovery = defaultdict(dict)
    confirmation = defaultdict(dict)
    for groups, rows in ((discovery, screen['grouped']), (confirmation, check['grouped'])):
        for row in rows:
            groups[row['scope'], row['opponent']][row['policy']] = row
    paired = defaultdict(dict)
    confirmation_rows = [json.loads(line) for line in Path(check['raw']).read_text().splitlines()]
    audit['queuedVsPredictorConfirmation'] = compare_queued_predictor(confirmation_rows)
    write(audit_path, audit)
    for row in confirmation_rows:
        paired[row['scope'], row['opponent'], row['config']['seed']][row['policy']] = row['result']['success']
    selected = {(c['scope'], c['opponent']): c for c in plan['selected']}
    rejected = {(c['scope'], c['opponent']): c for c in plan['rejected']}
    cells = []
    for scope in SCOPES:
        for opponent in OPPONENTS:
            key = scope, opponent
            row = dict(scope=scope, opponent=opponent, discovery=discovery[key])
            if key not in selected:
                cells.append(dict(row, decision='screened out', reason=rejected[key]['reason']))
                continue
            ref = selected[key]['reference']
            outcomes = confirmation[key]
            assert set(outcomes) == {ref, 'hour8', 'hour10'}
            gaps = {p: outcomes[ref]['wins'] - outcomes[p]['wins'] for p in ('hour8', 'hour10')}
            counts = defaultdict(Counter)
            for seed in check['seeds']:
                match = paired[scope, opponent, seed]
                for p in gaps:
                    label = ('refWin' if match[ref] else 'refLoss') + ('LearnerWin' if match[p] else 'LearnerLoss')
                    counts[p][label] += 1
            enough_reference = outcomes[ref]['wins'] >= 15
            passes = enough_reference and gaps['hour10'] >= 6
            cells.append(dict(row, reference=ref, confirmation=outcomes, pairedGap=gaps, matchedOutcomes=counts,
                              clearedGapFor=[p for p in gaps if enough_reference and gaps[p] >= 6],
                              decision='confirmed' if passes else 'failed confirmation'))
    final = dict(
        name='Ashbank Relay', cells=cells, catalog=catalog,
        policy=dict(discovery=plan['discovery'], confirmation=plan['confirmation'],
                    referenceTieBreak=plan['referenceTieBreak']),
        native=screen['native'], models=screen['models'],
        discovery=dict(summary=str(OUT / 'discovery-summary.json'), seeds=screen['seeds'],
                       matches=screen['count'], raw=screen['raw'], sha256=screen['rawSha256'],
                       schedule=screen['schedule'], elapsedSeconds=screen['elapsedSeconds']),
        confirmation=dict(summary=str(OUT / 'confirmation-summary.json'), seeds=check['seeds'],
                          matches=check['count'], raw=check['raw'], sha256=check['rawSha256'],
                          schedule=check['schedule'], elapsedSeconds=check['elapsedSeconds']),
        nativeAudit=audit,
    )
    write(OUT / 'final-summary.json', final)
    with (OUT / 'matrices.md').open('w') as file:
        print('| Visibility | Enemy | H8 | H10 | Classic | Queued | Full |', file=file)
        print('| --- | --- | ---: | ---: | ---: | ---: | ---: |', file=file)
        for row in cells:
            vals = [row['discovery'][p]['wins'] for p in PLAYERS]
            print('| ' + ' | '.join([row['scope'], row['opponent'], *(str(v) for v in vals)]) + ' |', file=file)
        print('\n| Visibility | Enemy | Reference | Ref. wins | H8 | H10 | Result |', file=file)
        print('| --- | --- | --- | ---: | ---: | ---: | --- |', file=file)
        for row in cells:
            if 'confirmation' not in row:
                continue
            counts = row['confirmation']
            ref = row['reference']
            values = [counts[p]['wins'] for p in (ref, 'hour8', 'hour10')]
            print('| ' + ' | '.join([row['scope'], row['opponent'], ref, *(str(v) for v in values),
                                      row['decision']]) + ' |', file=file)
    for row in cells:
        if 'confirmation' in row:
            print(json.dumps(dict(scope=row['scope'], opponent=row['opponent'], reference=row['reference'],
                                  decision=row['decision'], gaps=row['pairedGap'])))


if __name__ == '__main__':
    main()
