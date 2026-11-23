import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import sys

from build import BASE, CATALOG, HERE, RUN, read, write


def config(campaign, case, seed, opponent, planner):
    c = dict(campaign=campaign, objective=case, seed=seed, diplomacy='unrestricted',
             player='balanced', enemy='classic' if opponent == 'classic' else 'predictor',
             playerBlocks=dict(recruiter='situational', campaign='mission', planner=planner),
             maxRounds=100, recruitLimit=None)
    if opponent == 'full':
        c['enemyBlocks'] = dict(recruiter='predictor', campaign='mission', planner='full')
    return c


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('label')
    parser.add_argument('--variants', default='original,longer,wounded,pierce,north,cohesion')
    parser.add_argument('--routes', default='authored,both-fog,player-fog')
    parser.add_argument('--policies', default='hour10,full')
    parser.add_argument('--opponents', default='classic')
    parser.add_argument('--seed-start', type=int, default=591030001)
    parser.add_argument('--seeds', type=int, default=8)
    args = parser.parse_args()
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native
    variants = args.variants.split(',')
    routes = args.routes.split(',')
    opponents = args.opponents.split(',')
    seeds = range(args.seed_start, args.seed_start + args.seeds)
    model_paths = {f'hour{r["hour"]}': Path(r['model']) for r in read(RUN / 'evaluation/hourly-summary.json')}
    output = HERE / f'{args.label}.jsonl'
    metadata = vars(args) | {
        'models': {p: dict(path=str(model_paths[p]), sha256=hashlib.sha256(model_paths[p].read_bytes()).hexdigest())
                   for p in args.policies.split(',') if p in model_paths},
        'catalog': str(CATALOG),
        'native': str(native.__file__),
        'native_sha256': hashlib.sha256(Path(native.__file__).read_bytes()).hexdigest(),
    }
    write(HERE / f'{args.label}-metadata.json', metadata)
    with output.open('w') as stream:
        for policy in args.policies.split(','):
            model = model_paths[policy].read_text() if policy in model_paths else None
            planner = 'queued' if model is not None else policy
            for variant in variants:
                for route in routes:
                    for opponent in opponents:
                        cases = [config(f'{BASE}-{variant}', route, seed, opponent, planner) for seed in seeds]
                        outcomes = [json.loads(item) for item in native.evaluate([json.dumps(c) for c in cases], model)]
                        for c, r in zip(cases, outcomes, strict=True):
                            assert len(r['missions']) == 1 and r['missions'][0]['id'] == c['campaign'], r
                            print(json.dumps(dict(policy=policy, opponent=opponent, config=c, result=r)), file=stream)
                        stream.flush()
                        reasons = Counter('win' if r['success'] else (r['failure'], r['outcome']) for r in outcomes)
                        print(policy, variant, route, opponent, dict(reasons), flush=True)


if __name__ == '__main__':
    main()
