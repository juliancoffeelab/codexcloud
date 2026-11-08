"""Crash-safe checkpoint bundles and validated recovery pointers."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
import time
import torch


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY)
    try: os.fsync(fd)
    finally: os.close(fd)


def atomic_json(path, value):
    path = Path(path); path.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix='.' + path.name, dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as f:
            json.dump(value, f, indent=2); f.write('\n'); f.flush(); os.fsync(f.fileno())
        os.replace(name, path); sync_dir(path.parent)
    finally:
        if os.path.exists(name): os.unlink(name)


def save_bundle(root, state, policy, label, best=False):
    root = Path(root); folder = root/'checkpoints'; folder.mkdir(parents=True, exist_ok=True)
    name = f'{label}-{time.time_ns()}'
    temporary = Path(tempfile.mkdtemp(prefix='.pending-', dir=folder))
    try:
        with (temporary/'resume.pt').open('wb') as f:
            torch.save(state, f); f.flush(); os.fsync(f.fileno())
        atomic_json(temporary/'planner.json', json.loads(policy.export()))
        files = {p.name: dict(bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in temporary.iterdir()}
        atomic_json(temporary/'manifest.json', dict(label=label, files=files))
        sync_dir(temporary); os.replace(temporary, folder/name); sync_dir(folder)
        pointer = dict(path=str(Path('checkpoints')/name), label=label, files=files)
        atomic_json(root/'latest.json', pointer)
        if best: atomic_json(root/'best.json', pointer)
        return pointer
    finally:
        if temporary.exists(): shutil.rmtree(temporary)


def resolve(root, pointer='latest.json'):
    root=Path(root); info=json.loads((root/pointer).read_text()); folder=root/info['path']
    for name, expected in info['files'].items():
        p=folder/name
        if p.stat().st_size != expected['bytes'] or hashlib.sha256(p.read_bytes()).hexdigest() != expected['sha256']:
            raise ValueError(f'incomplete or corrupt checkpoint: {p}')
    return folder


def load_resume(path):
    path=Path(path)
    if path.is_dir(): path=resolve(path)/'resume.pt'
    return torch.load(path, map_location='cpu', weights_only=True)
