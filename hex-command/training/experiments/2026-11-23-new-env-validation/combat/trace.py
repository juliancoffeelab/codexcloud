from collections import defaultdict
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess

from build import CATALOG, HERE, ROOT, read, write


def terminal(mission, success, expected):
    return (success == expected['success']
            and mission['round'] == expected['missions'][0]['round']
            and mission['winner'] == expected['missions'][0]['winner'])


def main():
    rows = defaultdict(dict)
    for line in (HERE / 'confirmation.jsonl').read_text().splitlines():
        row = json.loads(line)
        key = row['config']['objective'], row['config']['seed']
        rows[key][row['policy']] = row
    sampled = []
    for route in ('player-fog', 'both-fog', 'authored'):
        cases = [row for (mode, _), row in rows.items() if mode == route]
        pairs = [row for row in cases if row['queued']['result']['success']
                 and not row['hour10']['result']['success']][:2]
        if not pairs:
            pairs = [row for row in cases if not row['queued']['result']['success']
                     and row['hour10']['result']['success']][:1]
            pairs.extend([row for row in cases if not row['queued']['result']['success']
                          and not row['hour10']['result']['success']][:1])
        sampled.extend(pairs)
    trace_binary = ROOT / 'rust/target/simulation/commander-trace-probe'
    model = read(HERE / 'confirmation-metadata.json')['models']['hour10']['path']
    job_input = HERE / 'trace-jobs.json'
    labeled = [(policy, pair[policy]) for pair in sampled for policy in ('hour10', 'queued')]
    write(job_input, [dict(player='learner' if policy == 'hour10' else policy,
                           imitator=None if policy == 'hour10' else policy,
                           config=row['config']) for policy, row in labeled])
    trace_temp = HERE / 'trace-output-working.jsonl'
    subprocess.run([trace_binary, job_input, trace_temp, model], check=True,
                   env=dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1'))
    traces = trace_temp.read_text().splitlines()
    trace_temp.unlink()
    details = []
    verification = []
    with gzip.open(HERE / 'paired-mechanism-traces.jsonl.gz', 'wt') as output:
        for (policy, row), line in zip(labeled, traces, strict=True):
            outcome = json.loads(line)['result']
            mission = outcome['missions'][0]
            expected = row['result']
            success = outcome['completed']
            matched = terminal(mission, success, expected)
            verification.append(dict(policy=policy, seed=row['config']['seed'],
                                     route=row['config']['objective'], matches=matched))
            if not matched:
                raise ValueError(verification[-1])
            print(json.dumps(dict(source=row, trace=outcome)), file=output)
            details.append(dict(verification[-1], rounds=mission['round'],
                                phases=[dict(round=p['round'], side=p['side'],
                                             actions=p['actions'], commander=next((u for u in p['units'] if u['id'] == 'u1'),None),
                                             losses=[dict(id=u['id'], type=u['type']) for u in p['losses']])
                                        for p in mission['phases']]))
    write(HERE / 'paired-mechanism-digest.json', details)
    write(HERE / 'trace-metadata.json', dict(verification=verification, binaries={
        str(trace_binary): hashlib.sha256(trace_binary.read_bytes()).hexdigest()}))
    for game in details:
        print(game['route'], game['seed'], game['policy'], game['rounds'],
              [('r' + str(p['round']), p['side'], p['commander']['hp'] if p['commander'] else None,
                [a for a in p['actions'] if a['unit'] == 'u1']) for p in game['phases'][:6]])


if __name__ == '__main__':
    main()
