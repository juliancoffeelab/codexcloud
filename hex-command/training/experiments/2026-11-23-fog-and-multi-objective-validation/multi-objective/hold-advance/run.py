import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import sys

from build import BASE, CATALOG, OUT, RUN, catalog_hash, read, write


def config(tag, policy, enemy, seed):
    return dict(campaign=f'{BASE}-{tag}', objective='authored', seed=seed, diplomacy='unrestricted',
                player='balanced', enemy=enemy,
                playerBlocks=dict(recruiter='situational', campaign='mission',
                                  planner='queued' if policy.startswith('hour') else policy),
                maxRounds=100, recruitLimit=0)


def hash_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('label')
    parser.add_argument('--variants', nargs='+', required=True)
    parser.add_argument('--policies', nargs='+', default=['full', 'queued', 'classic', 'hour8', 'hour10'])
    parser.add_argument('--enemies', nargs='+', default=['classic'])
    parser.add_argument('--seed-start', type=int, default=571230001)
    parser.add_argument('--count', type=int, default=8)
    parser.add_argument('--catalog', type=Path, default=CATALOG)
    args = parser.parse_args()
    catalog = args.catalog.resolve()
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    model_paths = {f'hour{r["hour"]}': Path(r['model']) for r in read(RUN / 'evaluation/hourly-summary.json')}
    raw = OUT / f'{args.label}.jsonl'
    seeds = list(range(args.seed_start, args.seed_start + args.count))
    summary = dict(label=args.label, raw=str(raw), catalog=str(catalog),
                   catalogSha256=catalog_hash(catalog), seeds=seeds,
                   native=str(native.__file__), nativeSha256=hash_file(Path(native.__file__)),
                   models={p: dict(path=str(model_paths[p]), sha256=hash_file(model_paths[p]))
                           for p in args.policies if p in model_paths}, outcomes=[])
    with raw.open('w') as output:
        for policy in args.policies:
            model = model_paths[policy].read_text() if policy in model_paths else None
            for tag in args.variants:
                for enemy in args.enemies:
                    cases = [config(tag, policy, enemy, seed) for seed in seeds]
                    results = [json.loads(item) for item in native.evaluate([json.dumps(c) for c in cases], model)]
                    for case, result in zip(cases, results, strict=True):
                        assert len(result['missions']) == 1 and result['missions'][0]['id'] == case['campaign'], result
                        output.write(json.dumps(dict(variant=tag, policy=policy, enemy=enemy,
                                                     config=case, result=result)) + '\n')
                    output.flush()
                    wins = [r['missions'][0]['round'] for r in results if r['success']]
                    caps = sum(r['truncated'] for r in results)
                    outcomes = dict(variant=tag, policy=policy, enemy=enemy, wins=len(wins),
                                    losses=len(results) - len(wins) - caps, caps=caps,
                                    winRounds=wins, reasons=Counter(r['outcome'] for r in results))
                    summary['outcomes'].append(outcomes)
                    print(json.dumps(outcomes), flush=True)
                    write(OUT / f'{args.label}-summary.json', summary)


if __name__ == '__main__':
    main()
