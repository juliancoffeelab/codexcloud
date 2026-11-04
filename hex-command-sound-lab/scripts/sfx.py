#!/usr/bin/env python3
"""Acquire pinned originals or verify this checkout, without Python dependencies."""
import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import urllib.error
import urllib.request
from urllib.parse import urlparse
import zipfile

PROJECT = Path(__file__).resolve().parents[1]
VENDOR = PROJECT / 'vendor/sfx'
AUDIO = {'.wav', '.ogg', '.mp3', '.flac', '.m4a', '.aiff', '.aif', '.opus'}


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def matches(path, entry):
    return path.is_file() and not path.is_symlink() and path.stat().st_size == entry['bytes'] and digest(path) == entry['sha256']


def checked_path(root, relative):
    p = root / relative
    if not p.resolve().is_relative_to(root.resolve()):
        raise ValueError(f'Path escapes destination: {relative}')
    return p


def run(args):
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(f'{args[0]} failed: {r.stderr or r.stdout}')
    return r.stdout


def acquire(root, inventory, sources, cache):
    """Only missing/mismatched assets need network; reuse identical bundled files first."""
    cache.mkdir(parents=True, exist_ok=True)
    by_artifact = defaultdict(list)
    for entry in inventory:
        by_artifact[entry['artifact']].append(entry)
    local = defaultdict(list)
    for p in (PROJECT / 'dist/audio').rglob('*'):
        if p.is_file():
            local[digest(p)].append(p)
    for artifact in sources['downloads']:
        entries = by_artifact[artifact['id']]
        needed = [e for e in entries if not matches(checked_path(root, e['path']), e)]
        for e in needed[:]:
            candidates = local.get(e['sha256'], [])
            if candidates:
                dest = checked_path(root, e['path'])
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(candidates[0], dest)
                needed.remove(e)
        if not needed:
            continue
        blob = cache / artifact['sha256']
        if not matches(blob, artifact):
            print('Downloading', artifact['url'], flush=True)
            part = blob.with_suffix('.partial')
            try:
                with urllib.request.urlopen(artifact['url'], timeout=120) as response, part.open('wb') as out:
                    shutil.copyfileobj(response, out)
            except (urllib.error.URLError, TimeoutError) as error:
                raise RuntimeError(f"Download failed: {artifact['url']} (host {urlparse(artifact['url']).netloc}): {error}") from error
            if not matches(part, artifact):
                part.unlink(missing_ok=True)
                raise RuntimeError(f"Download hash/size mismatch: {artifact['url']}")
            part.replace(blob)
        if artifact['format'] == 'file':
            extracted = None
        else:
            # CRC test before extraction; ZIP payload CRCs also checked independently.
            test_output = run([sevenzip(), 't', str(blob)])
            if 'WARNINGS:' in test_output and not artifact.get('warning'):
                raise RuntimeError(f"Unexpected archive warning: {artifact['id']}")
            if artifact.get('warning'):
                print('Upstream archive caveat:', artifact['id'], artifact['warning'])
            if artifact['format'] == 'zip':
                with zipfile.ZipFile(blob) as z:
                    if z.testzip() is not None:
                        raise RuntimeError('ZIP CRC failure')
            extracted = tempfile.TemporaryDirectory(prefix='hex-sfx-')
            # Extract only the locked member names; compare their hashes before installation.
            run([sevenzip(), 'x', '-y', str(blob), '-o' + extracted.name] + [e['member'] for e in entries])
        try:
            for e in needed:
                src = checked_path(Path(extracted.name), e['member']) if extracted else blob
                if not matches(src, e):
                    raise RuntimeError(f"Extracted payload mismatch: {e['path']}")
                dest = checked_path(root, e['path'])
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(src, dest)
        finally:
            if extracted:
                extracted.cleanup()


def sevenzip():
    value = shutil.which('7zz') or shutil.which('7z')
    if not value:
        raise RuntimeError('Install 7-Zip (7zz or 7z) to reacquire archives')
    return value


def verify(root, inventory, sources, decode):
    for record in sources['provenance_files']:
        if not matches(checked_path(VENDOR, record['path']), record):
            raise RuntimeError('Missing or changed provenance: ' + record['path'])
    def check(e):
        path = checked_path(root, e['path'])
        if e['bytes'] <= 0 or not matches(path, e):
            raise RuntimeError(f"Missing, empty or changed file: {e['path']}")
        if e['bytes'] >= 100 * 1024 * 1024:
            raise RuntimeError(f"Exceeds GitHub ordinary Git file limit: {e['path']}")
        info = dict(e)
        if path.suffix.lower() in AUDIO and decode:
            probe = json.loads(run(['ffprobe', '-v', 'error', '-show_streams', '-show_format', '-of', 'json', str(path)]))
            stream = next(s for s in probe['streams'] if s['codec_type'] == 'audio')
            duration = float(probe['format'].get('duration', stream.get('duration', 0)))
            if duration <= 0:
                raise RuntimeError(f'Empty audio stream: {path}')
            run(['ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-i', str(path), '-map', '0:a', '-f', 'null', '-'])
            info.update(duration_seconds=duration, codec=stream['codec_name'], sample_rate=int(stream['sample_rate']), channels=stream['channels'])
        return info
    with ThreadPoolExecutor(max_workers=6) as executor:
        rows = list(executor.map(check, inventory))
    expected = {e['path'] for e in inventory if Path(e['path']).suffix.lower() in AUDIO}
    actual = {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p.suffix.lower() in AUDIO}
    if expected != actual:
        raise RuntimeError(f'Audio inventory mismatch: extra={actual-expected}, missing={expected-actual}')
    hashes = defaultdict(list)
    summary = []
    for e in rows:
        hashes[e['sha256']].append(e['path'])
    for p in sources['packs']:
        entries = [e for e in rows if e['pack'] == p['id'] and Path(e['path']).suffix.lower() in AUDIO]
        if not entries:
            raise RuntimeError(f"No audio for {p['id']}")
        summary.append({'pack':p['id'], 'audio_files':len(entries), 'bytes':sum(e['bytes'] for e in entries)})
    coverage = json.loads((VENDOR / 'coverage.json').read_text())
    for group in coverage['groups']:
        if not group['examples'] or any(p not in expected for p in group['examples']):
            raise RuntimeError('Invalid coverage references: ' + group['name'])
    if not {'movement','attack','impact','vocalization'} <= {c for g in coverage['groups'] for c in g['categories']}:
        raise RuntimeError('Missing coverage category')
    return {'schema_version':1, 'decoded':decode, 'audio_files':len(expected), 'audio_bytes':sum(s['bytes'] for s in summary), 'packs':summary, 'duplicate_hash_groups':[v for v in hashes.values() if len(v)>1], 'largest_file':max(rows, key=lambda e:e['bytes'])['path'], 'files':rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify','acquire'])
    parser.add_argument('--output', type=Path, default=VENDOR, help='Asset root; metadata always comes from this checkout')
    parser.add_argument('--cache', type=Path, default=PROJECT/'tmp/sfx-downloads')
    parser.add_argument('--hash-only', action='store_true', help='Skip decoding (Python alone suffices)')
    parser.add_argument('--report', type=Path, help='Write machine-readable validation results')
    args = parser.parse_args()
    inventory = json.loads((VENDOR/'inventory.json').read_text())
    sources = json.loads((VENDOR/'sources.json').read_text())
    if args.command == 'acquire':
        acquire(args.output, inventory, sources, args.cache)
    result = verify(args.output, inventory, sources, not args.hash_only)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k != 'files'}, indent=2))


if __name__ == '__main__':
    main()
