"""Re-run a frozen native job list; no training and no authored-data mutations."""
from pathlib import Path
import argparse, gzip, json, os, shutil, subprocess
HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'rust/Cargo.toml').exists())
STAGES = {
    'discovery': ('discovery-jobs.json', 'roles-v1.rs'),
    'discovery-v2': ('discovery-v2-jobs.json', 'roles-v2.rs'),
    'discovery-v3': ('discovery-v3-jobs.json', 'roles-v3.rs'),
    'discovery-v4': ('discovery-v4-jobs.json', 'roles-v4.rs'),
    'tide-v6': ('tide-v6-jobs.json', 'roles.rs'),
    'tide-v4': ('tide-v4-jobs.json', 'roles-v4.rs'),
    'gallery-v5': ('gallery-v5-jobs.json', 'roles-v5.rs'),
    'confirmation': ('confirmation-jobs.json', 'roles-v5.rs'),
    'gallery-confirmation': ('gallery-confirmation-jobs.json', 'roles-v5.rs'),
    'production-parity': ('production-parity-jobs.json', 'roles.rs'),
}
p = argparse.ArgumentParser()
p.add_argument('--stage', choices=STAGES, default='production-parity')
p.add_argument('--limit', type=int, help='Optional explicit smoke-test job count')
a = p.parse_args()
if a.limit is not None and a.limit < 1:
    p.error('--limit must be positive')
job, source = STAGES[a.stage]
work = ROOT / 'tmp/rl/task-planner-reproduce' / a.stage
(work / 'src').mkdir(parents=True, exist_ok=True)
shutil.copyfile(HERE / 'sources/task_planners.rs', work / 'src/main.rs')
shutil.copyfile(HERE / 'sources' / source, work / 'src/roles.rs')
jobs = json.loads(gzip.decompress((HERE / (job + '.gz')).read_bytes()))
if a.limit is not None:
    jobs = jobs[:a.limit]
(work / 'jobs.json').write_text(json.dumps(jobs))
manifest = '\n'.join([
    '[package]', 'name="task-planner-reproduce"', 'version="0.0.0"', 'edition="2024"',
    '[workspace]', '[dependencies]',
    'hex-command-simulation={path=' + json.dumps(str(ROOT / 'rust')) + ',features=["simulation","training"]}',
    'hex-command-battle={path=' + json.dumps(str(ROOT / 'rust/battle')) + '}',
    'serde_json="1"', 'mimalloc="0.1"', '',
])
(work / 'Cargo.toml').write_text(manifest)
env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / 'rust/target'), RAYON_NUM_THREADS='1')
subprocess.run(['cargo', 'build', '--release', '--manifest-path', str(work / 'Cargo.toml')], env=env, check=True)
result = work / 'results.jsonl'
with result.open('w') as f:
    subprocess.run([str(ROOT / 'rust/target/release/task-planner-reproduce'), str(ROOT / 'rust/data'), str(work / 'jobs.json')], env=env, stdout=f, check=True)
print(result)
