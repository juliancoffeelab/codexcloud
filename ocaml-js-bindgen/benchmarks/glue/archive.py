"""Archive measured generated artifacts and reports; verify every included byte."""
import argparse
import hashlib
import json
import tarfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--artifacts', type=Path, required=True)
parser.add_argument('--report', type=Path, action='append', required=True)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--evidence', type=Path, action='append', default=[],
                    help='Additional diagnostic logs or probe reports, preserved byte-for-byte')
parser.add_argument('--profiles', type=Path)
parser.add_argument('--profile-commit')
parser.add_argument('--profile-method', default='100 microsecond CPU sampling; release Rust with retained names, numeric-eight/16, 100000 warmup and 2000000 profiled calls per path. Profiling timings are not benchmark results.')
args = parser.parse_args()
sha = lambda b: hashlib.sha256(b).hexdigest()
metadata = json.loads((args.artifacts / 'build.json').read_text())
files = {'artifacts/build.json': args.artifacts / 'build.json'}
for variant, recorded in metadata['variants'].items():
    folder = args.artifacts / variant
    for key in ['wasm', 'adapter']:
        path = folder / Path(recorded[key]).name
        assert sha(path.read_bytes()) == recorded[key + 'Sha256'], path
    for path in folder.iterdir():
        if path.is_file() and path.suffix in {'.js', '.mjs', '.ml', '.mli', '.json', '.wasm'}:
            files['artifacts/' + variant + '/' + path.name] = path
for report in args.report:
    data = json.loads(report.read_text())
    assert data['metadata'] == metadata, report
    files['reports/' + report.name] = report
for path in args.evidence:
    name = 'evidence/' + path.name
    assert name not in files, path
    files[name] = path
if args.profiles:
    assert args.profile_commit and len(args.profile_commit) == 40
    for path in args.profiles.glob('*.cpuprofile'):
        files['profiles/' + path.name] = path
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
manifest = {'sourceCommit': metadata['sdkCommit'], 'archiveSha256': sha(args.out.read_bytes()), 'files': entries}
if args.profiles:
    manifest['profileEvidence'] = {
        'sourceCommit': args.profile_commit,
        'method': args.profile_method,
        'files': [name for name in sorted(files) if name.startswith('profiles/')],
    }
Path(str(args.out).replace('.tar.gz', '.manifest.json')).write_text(json.dumps(manifest, indent=2) + '\n')
print('Verified', len(entries), 'archived files and all measured artifact hashes')
