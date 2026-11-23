import gzip
import json
import os
import subprocess

from build import CATALOG, OUT, ROOT, RUN


def hex_pair(tile):
    return [tile['col'], tile['row']]


def profile(unit):
    return dict(id=unit['id'], type=unit['type'], faction=unit['faction'], hp=unit['hp'],
                tile=hex_pair(unit['tile']))


def phase_profile(phase):
    before = [profile(unit) for unit in phase['before']]
    after = [profile(unit) for unit in phase['units']]
    owners = [owner['owner'] for owner in phase['ownership'] if hex_pair(owner['tile']) == [13, 6]]
    actions = [action for action in phase['actions'] if action.get('unit') == 'u1' or action.get('attacker') == 'u1'
               or action.get('defender') == 'u1' or action.get('target') == 'u1']
    return dict(round=phase['round'], side=phase['side'], before=before, after=after,
                neraActions=actions, actionCount=len(phase['actions']), postOwner=owners[0] if owners else None)


def main():
    rows = [json.loads(line) for line in (OUT / 'confirmation.jsonl').read_text().splitlines()]
    selected = []
    for variant, teacher in [('dread-long', 'full'), ('dread-wounded-long', 'classic'),
                             ('dread-wounded-healer', 'classic')]:
        group = [row for row in rows if row['variant'] == variant]
        failures = {row['config']['seed'] for row in group if row['policy'] == 'hour10' and not row['result']['success']}
        seeds = [row['config']['seed'] for row in group if row['policy'] == teacher and row['result']['success']
                 and row['config']['seed'] in failures][:4]
        selected.extend(r for r in group if r['config']['seed'] in seeds and r['policy'] in (teacher, 'hour10', 'hour8'))
    model_paths = {f'hour{r["hour"]}': r['model'] for r in json.loads((RUN / 'evaluation/hourly-summary.json').read_text())}
    binary = ROOT / 'rust/target/simulation/commander-trace-probe'
    traced = []
    for label in ('hour8', 'hour10', 'rules'):
        matched = [row for row in selected if row['policy'] == label or (label == 'rules' and not row['policy'].startswith('hour'))]
        if not matched:
            continue
        job_path = OUT / f'trace-{label}-jobs.json'
        jobs = [dict(player='learner' if row['policy'].startswith('hour') else row['policy'],
                     imitator=None if row['policy'].startswith('hour') else row['policy'], config=row['config'])
                for row in matched]
        job_path.write_text(json.dumps(jobs, indent=2) + '\n')
        raw = OUT / f'trace-{label}.jsonl'
        env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(CATALOG), RAYON_NUM_THREADS='1')
        subprocess.run([str(binary), str(job_path), str(raw), model_paths.get(label, model_paths['hour10'])],
                       env=env, check=True)
        with raw.open() as reader, gzip.open(str(raw) + '.gz', 'wt') as zipped:
            for line, row in zip(reader, matched, strict=True):
                zipped.write(line)
                data = json.loads(line)
                result = data['result']
                mission = result['missions'][0]
                assert result['completed'] == row['result']['success']
                assert mission['round'] == row['result']['missions'][0]['round']
                assert mission['winner'] == row['result']['missions'][0]['winner']
                traced.append(dict(row, phases=[phase_profile(p) for p in mission['phases']]))
        raw.unlink()
    (OUT / 'action-evidence.json').write_text(json.dumps(traced, indent=2) + '\n')


if __name__ == '__main__':
    main()
