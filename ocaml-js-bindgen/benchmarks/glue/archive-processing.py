"""Archive historical paired-cohort reports (pre-Benchmark.js schema only)."""
import argparse
import hashlib
import json
import math
import statistics
import tarfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--artifacts', type=Path, required=True)
parser.add_argument('--report', type=Path, action='append', required=True)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--summary', type=Path, required=True)
args = parser.parse_args()
assert len(args.report) >= 3, 'Supply at least three matched cohorts'
reports = [json.loads(path.read_text()) for path in args.report]
first = reports[0]
sha = lambda data: hashlib.sha256(data).hexdigest()
files = {}
for variant in first['variants']:
    name = variant['name']
    for suffix, key in [('mjs', 'adapterSha256'), ('wasm', 'wasmSha256')]:
        path = args.artifacts / f'{name}.{suffix}'
        assert sha(path.read_bytes()) == variant[key], path
        files[f'processing/{path.name}'] = path
corpus = args.artifacts / 'corpus.json'
assert sha(corpus.read_bytes()) == first['input']['sha256']
files['processing/corpus.json'] = corpus
for path, report in zip(args.report, reports):
    assert report['input']['sha256'] == first['input']['sha256']
    assert report['input']['count'] == first['input']['count']
    assert report['mode'] == first['mode']
    for actual, expected in zip(report['variants'], first['variants'], strict=True):
        for key in ['name', 'commit', 'sourceSha256', 'adapterSha256', 'wasmSha256']:
            assert actual[key] == expected[key], (path, key)
    expected_hashes = dict(first['outcomeHashes'])
    assert dict(report['outcomeHashes']) == expected_hashes
    assert len(report['samples']) == first['input']['count'] * report['measuredPasses'] * len(report['variants'])
    observed = set()
    for sample in report['samples']:
        key = (sample['variant'], sample['pass'], sample['index'])
        assert sample['variant'] in report['summary']
        assert 0 <= sample['pass'] < report['measuredPasses']
        assert key not in observed
        observed.add(key)
        assert sample['outcomeSha256'] == expected_hashes[sample['index']]
    for variant in report['variants']:
        name = variant['name']
        samples = [s for s in report['samples'] if s['variant'] == name]
        medians = [statistics.median(s['ms'] for s in samples if s['index'] == index)
                   for index in range(report['input']['count'])]
        assert math.isclose(sum(medians) / len(medians),
                            report['summary'][name]['meanOfInputMediansMs'], abs_tol=1e-9)
        assert math.isclose(statistics.median(s['restoreMs'] for s in samples),
                            report['summary'][name]['medianRestoreMs'], abs_tol=1e-9)
    files[f'processing/{path.name}'] = path
ratios = [r['summary']['port']['meanOfInputMediansMs'] / r['summary']['upstream']['meanOfInputMediansMs'] for r in reports]
summary = {
    'method': 'Actual compiled application processing, including final save. Median of matched cohort ratios of mean per-input median phase times. Restore separate; no forced GC. Every measured command/save outcome hash agrees.',
    'mode': first['mode'], 'sourceCommit': first['variants'][0]['commit'],
    'node': first['node'], 'v8': first['v8'], 'input': first['input'],
    'variants': first['variants'], 'cohorts': [r['summary'] for r in reports],
    'pairedRatios': ratios, 'medianPairedRatio': statistics.median(ratios),
    'matchedMeasuredOutcomes': sum(len(r['samples']) for r in reports),
}
args.summary.parent.mkdir(parents=True, exist_ok=True)
args.summary.write_text(json.dumps(summary, indent=2) + '\n')
files['processing/' + args.summary.name] = args.summary
args.out.parent.mkdir(parents=True, exist_ok=True)
entries = []
with tarfile.open(args.out, 'w:gz') as archive:
    for name, path in sorted(files.items()):
        data = path.read_bytes()
        entries.append({'path': name, 'bytes': len(data), 'sha256': sha(data)})
        archive.add(path, arcname=name, recursive=False)
with tarfile.open(args.out) as archive:
    for entry in entries:
        data = archive.extractfile(entry['path']).read()
        assert len(data) == entry['bytes'] and sha(data) == entry['sha256']
manifest = {'sourceCommit': summary['sourceCommit'], 'archiveSha256': sha(args.out.read_bytes()), 'files': entries}
Path(str(args.out).replace('.tar.gz', '.manifest.json')).write_text(json.dumps(manifest, indent=2) + '\n')
print('Verified', len(entries), 'files;', summary['matchedMeasuredOutcomes'], 'matched measured outcomes; median ratio', summary['medianPairedRatio'])
