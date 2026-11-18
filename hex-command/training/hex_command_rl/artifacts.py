"""Verify committed model/run manifests and restore archived recovery files."""

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(manifest):
    manifest = Path(manifest)
    info = json.loads(manifest.read_text())
    if info.get("kind") not in [
        "native-planner-checkpoint",
        "immutable-training-experiment",
    ]:
        raise ValueError("unsupported artifact manifest")
    root = manifest.parent.resolve()
    for name, expected in info["files"].items():
        path = (root / name).resolve()
        if not path.is_relative_to(root):
            raise ValueError("artifact path escapes its directory")
        data = path.read_bytes()
        if len(data) != expected["bytes"] or digest(data) != expected["sha256"]:
            raise ValueError(f"artifact checksum mismatch: {path}")
    if info["kind"] == "immutable-training-experiment":
        for name, expected in info["recovery_files"].items():
            data = recovery_bytes(root, expected)
            if (
                len(data) != expected["original_bytes"]
                or digest(data) != expected["original_sha256"]
            ):
                raise ValueError(f"archived recovery mismatch: {name}")
        models = {info["selected_model"]}
        if "trained_model" in info:
            models.add(info["trained_model"])
        for model in models:
            verify(root / model)
    return info


def recovery_bytes(root, record):
    path = (Path(root) / record["path"]).resolve()
    if not path.is_relative_to(Path(root).resolve()):
        raise ValueError("recovery path escapes its archive")
    data = path.read_bytes()
    return gzip.decompress(data) if record["compression"] == "gzip" else data


def restore(manifest, destination):
    manifest = Path(manifest)
    info = verify(manifest)
    if info["kind"] != "immutable-training-experiment":
        raise ValueError("restore requires an experiment archive")
    destination = Path(destination)
    if destination.exists():
        raise ValueError("restore destination must be new")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(
        tempfile.mkdtemp(prefix="." + destination.name + "-", dir=destination.parent)
    )
    try:
        for name, record in info["recovery_files"].items():
            if Path(name).name != name:
                raise ValueError("invalid recovery filename")
            (temporary / name).write_bytes(recovery_bytes(manifest.parent, record))
        os.replace(temporary, destination)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--extract-recovery", type=Path)
    args = parser.parse_args()
    if args.extract_recovery is not None:
        restore(args.manifest, args.extract_recovery)
    else:
        verify(args.manifest)
    print(json.dumps(dict(verified=True, manifest=str(args.manifest))))


if __name__ == "__main__":
    main()
