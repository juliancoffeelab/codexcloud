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
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def atomic_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix="." + path.name, dir=path.parent)
    try:
        with os.fdopen(fd, "w") as f:
            json.dump(value, f, indent=2)
            f.write("\n")
            f.flush()
            os.fsync(f.fileno())
        os.replace(name, path)
        sync_dir(path.parent)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def finite_tensors(value):
    if isinstance(value, torch.Tensor):
        if value.is_floating_point() and not torch.isfinite(value).all():
            raise ValueError("checkpoint contains a nonfinite tensor")
    elif isinstance(value, dict):
        for child in value.values():
            finite_tensors(child)
    elif isinstance(value, (list, tuple)):
        for child in value:
            finite_tensors(child)


def save_bundle(root, state, policy, label, best=False):
    finite_tensors(state)
    finite_tensors(policy.state_dict())
    root = Path(root)
    folder = root / "checkpoints"
    folder.mkdir(parents=True, exist_ok=True)
    name = f"{label}-{time.time_ns()}"
    temporary = Path(tempfile.mkdtemp(prefix=".pending-", dir=folder))
    try:
        with (temporary / "resume.pt").open("wb") as f:
            torch.save(state, f)
            f.flush()
            os.fsync(f.fileno())
        atomic_json(temporary / "planner.json", json.loads(policy.export()))
        files = {
            p.name: dict(
                bytes=p.stat().st_size,
                sha256=hashlib.sha256(p.read_bytes()).hexdigest(),
            )
            for p in temporary.iterdir()
        }
        atomic_json(temporary / "manifest.json", dict(label=label, files=files))
        sync_dir(temporary)
        os.replace(temporary, folder / name)
        sync_dir(folder)
        pointer = dict(path=str(Path("checkpoints") / name), label=label, files=files)
        atomic_json(root / "latest.json", pointer)
        if best:
            atomic_json(root / "best.json", pointer)
        return pointer
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def resolve_pointer(root, info):
    """Validate the exact pointer already observed by a reader."""
    folder = Path(root) / info["path"]
    for name, expected in info["files"].items():
        p = folder / name
        if (
            p.stat().st_size != expected["bytes"]
            or hashlib.sha256(p.read_bytes()).hexdigest() != expected["sha256"]
        ):
            raise ValueError(f"incomplete or corrupt checkpoint: {p}")
    return folder


def resolve(root, pointer="latest.json"):
    return resolve_pointer(root, json.loads((Path(root) / pointer).read_text()))


def read_jsonl(path, repair=False):
    """Read completed records; an interrupted trailing write is recoverable."""
    path = Path(path)
    if not path.exists():
        return []
    data = path.read_bytes()
    lines = data.splitlines(keepends=True)
    rows = []
    valid = 0
    for i, line in enumerate(lines):
        try:
            row = json.loads(line)
        except (json.JSONDecodeError, UnicodeDecodeError):
            if i + 1 < len(lines):
                raise
            break
        rows.append(row)
        valid += len(line)
    if repair and (valid < len(data) or data and not data[:valid].endswith(b"\n")):
        with path.open("r+b") as f:
            f.truncate(valid)
            f.seek(valid)
            if valid and not data[:valid].endswith(b"\n"):
                f.write(b"\n")
            f.flush()
            os.fsync(f.fileno())
    return rows


def load_resume(path):
    path = Path(path)
    if path.is_dir():
        path = resolve(path) / "resume.pt"
    return torch.load(path, map_location="cpu", weights_only=True)


def atomic_torch_save(value, path):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix="." + path.name, dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as f:
            torch.save(value, f)
            f.flush()
            os.fsync(f.fileno())
        os.replace(name, path)
        sync_dir(path.parent)
    finally:
        if os.path.exists(name):
            os.unlink(name)
