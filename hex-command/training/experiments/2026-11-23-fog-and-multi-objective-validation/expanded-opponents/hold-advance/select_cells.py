from collections import defaultdict

from run import OUT, OPPONENTS, REFERENCES, SCOPES, read, write


def main():
    discovery = read(OUT / 'discovery-summary.json')
    by_cell = defaultdict(dict)
    for row in discovery['grouped']:
        by_cell[row['scope'], row['opponent']][row['policy']] = row['wins']
    selected, rejected = [], []
    for scope in SCOPES:
        for opponent in OPPONENTS:
            wins = by_cell[scope, opponent]
            reference = max(REFERENCES, key=wins.__getitem__)
            gap8 = wins[reference] - wins['hour8']
            gap10 = wins[reference] - wins['hour10']
            row = dict(scope=scope, opponent=opponent, reference=reference, discoveryWins=wins,
                       discoveryGapHour8=gap8, discoveryGapHour10=gap10)
            if wins[reference] < 6:
                rejected.append(dict(row, reason='All references won fewer than six discovery matches.'))
            elif gap10 < 3:
                rejected.append(dict(row, reason='Best reference led Hour 10 by fewer than three discovery wins.'))
            else:
                selected.append(row)
    write(OUT / 'confirmation-cells.json', dict(
        discovery='Reference at least 6/8. Advantage of at least 3 wins over Hour 10.',
        confirmation='Selected reference at least 15/20. Advantage of at least 6 wins over Hour 10.',
        referenceTieBreak=list(REFERENCES), selected=selected, rejected=rejected,
    ))
    for row in selected:
        print(row)


if __name__ == '__main__':
    main()
