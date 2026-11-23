import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import statistics

from build import CATALOG, OUT, RUN


def config(variant, policy, enemy, seed):
    c = dict(campaign=f'rl-commander-left-behind-{variant}', objective='authored',
             diplomacy='unrestricted', player='predictor', enemy=enemy,
             playerBlocks=dict(recruiter='situational', campaign='mission',
                               planner='queued' if policy.startswith('hour') else policy),
             maxRounds=100, recruitLimit=0, seed=seed)
    if enemy == 'full':
        c.update(enemy='predictor', enemyBlocks=dict(recruiter='predictor', campaign='mission', planner='full'))
    return c


def main():
    from hex_command_training import evaluate

    cli = argparse.ArgumentParser()
    cli.add_argument('name')
    cli.add_argument('--variants', nargs='+', required=True)
    cli.add_argument('--policies', nargs='+', default=['hour8', 'hour10', 'full', 'queued', 'classic'])
    cli.add_argument('--enemies', nargs='+', default=['classic'])
    cli.add_argument('--seed', type=int, default=491230001)
    cli.add_argument('--count', type=int, default=8)
    args = cli.parse_args()
    hours = json.loads((RUN / 'evaluation/hourly-summary.json').read_text())
    models = {f'hour{x["hour"]}': Path(x['model']) for x in hours}
    rows = []
    groups = defaultdict(list)
    with (OUT / f'{args.name}.jsonl').open('w') as out:
        for policy in args.policies:
            cases = [(variant, enemy, config(variant, policy, enemy, seed))
                     for variant in args.variants for enemy in args.enemies
                     for seed in range(args.seed, args.seed + args.count)]
            model = models[policy].read_text() if policy in models else None
            outputs = evaluate([json.dumps(c) for _, _, c in cases], model)
            for (variant, enemy, c), r in zip(cases, outputs, strict=True):
                result = json.loads(r)
                row = dict(policy=policy, variant=variant, enemy=enemy, config=c, result=result)
                rows.append(row)
                groups[variant, enemy, policy].append(result)
                out.write(json.dumps(row) + '\n')
            out.flush()
    summaries = []
    for (variant, enemy, policy), results in sorted(groups.items()):
        wins = [r['missions'][0]['round'] for r in results if r['success']]
        caps = sum(r['truncated'] for r in results)
        s = dict(variant=variant, enemy=enemy, policy=policy, successes=len(wins),
                 defeats=len(results) - len(wins) - caps, caps=caps,
                 medianWinRound=statistics.median(wins) if wins else None,
                 outcomes=dict((outcome, sum(r.get('outcome') == outcome for r in results))
                               for outcome in set(r.get('outcome') for r in results)))
        summaries.append(s)
        print(json.dumps(s))
    summary = dict(name=args.name, catalog=str(CATALOG), raw=str(OUT / f'{args.name}.jsonl'),
                   seeds=list(range(args.seed, args.seed + args.count)), comparisons=summaries,
                   models={p: dict(path=str(models[p]), sha256=hashlib.sha256(models[p].read_bytes()).hexdigest())
                           for p in args.policies if p in models})
    (OUT / f'{args.name}-summary.json').write_text(json.dumps(summary, indent=2) + '\n')


if __name__ == '__main__':
    main()
