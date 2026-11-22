"""Lossless compressed, independently recoverable native teacher shards."""

import hashlib
import json
import os
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
import multiprocessing
from types import SimpleNamespace
import numpy as np
import hex_command_training as native
from .mixed_workload import group, pretrain_imitator
from .recovery import atomic_json


def collect_one(task):
    path, configs = task
    path = Path(path)
    teachers = [pretrain_imitator(c) for c in configs]
    dense, spatial = native.collect_spatial(
        [json.dumps(c) for c in configs], 1000000, side="player",
        imitators=[json.dumps(t) for t in teachers],
    )
    x, offsets, labels, missing = dense
    boards, board_offsets, shapes, indices, context_values, contexts = spatial
    if missing or len(labels) >= 1000000:
        raise RuntimeError("incomplete teacher shard")
    temporary = path.with_suffix(".tmp")
    with temporary.open("wb") as f:
        np.savez_compressed(
            f,
            x=x,
            offsets=offsets,
            labels=labels,
            boards=boards,
            board_offsets=board_offsets,
            shapes=shapes,
            indices=indices,
            context_values=context_values,
            contexts=contexts,
        )
        f.flush()
        os.fsync(f.fileno())
    os.replace(temporary, path)
    return dict(
        path=path.name,
        decisions=len(labels),
        candidates=len(x),
        bytes=path.stat().st_size,
        sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
        configs=configs,
        pretrain_imitators=teachers,
        group=list(group(configs[0])),
    )


def collect(configs, root, record):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    manifest_path = root / "manifest.json"
    manifest = (
        json.loads(manifest_path.read_text())
        if manifest_path.exists()
        else dict(
            configs=configs, pretrain_imitators=[pretrain_imitator(c) for c in configs],
            fingerprints=json.loads(native.fingerprints()), shards={}
        )
    )
    if (manifest["configs"] != configs
        or manifest.get("pretrain_imitators") != [pretrain_imitator(c) for c in configs]
        or manifest["fingerprints"] != json.loads(native.fingerprints())):
        raise ValueError("collection provenance differs")
    tasks = []
    from itertools import groupby
    batches = []
    for _, cases in groupby(configs, key=group):
        cases = list(cases)
        batches.extend(cases[i:i+4] for i in range(0, len(cases), 4))
    for start, batch in enumerate(batches):
        name = f"{start:06d}.npz"
        p = root / name
        old = manifest["shards"].get(name)
        if (
            old
            and p.exists()
            and hashlib.sha256(p.read_bytes()).hexdigest() == old["sha256"]
        ):
            continue
        tasks.append((str(p), batch))
    with ProcessPoolExecutor(
        max_workers=2, mp_context=multiprocessing.get_context("spawn")
    ) as pool:
        for future in as_completed([pool.submit(collect_one, t) for t in tasks]):
            row = future.result()
            manifest["shards"][row["path"]] = row
            atomic_json(manifest_path, manifest)
            record(
                "collection",
                split=root.name,
                shards=len(manifest["shards"]),
                total_shards=len(batches),
                decisions=sum(s["decisions"] for s in manifest["shards"].values()),
                bytes=sum(s["bytes"] for s in manifest["shards"].values()),
            )
    manifest["shards"] = {k: manifest["shards"][k] for k in sorted(manifest["shards"])}
    atomic_json(manifest_path, manifest)
    return manifest


def load(path):
    with np.load(path) as z:
        a = {k: z[k] for k in z.files}
    return SimpleNamespace(
        dense=(a["x"], a["offsets"], a["labels"], 0),
        boards=a["boards"],
        offsets=a["board_offsets"],
        shapes=a["shapes"],
        indices=a["indices"],
        context_values=a.get("context_values", np.empty(0, np.float32)),
        contexts=a.get("contexts", np.empty((0, 3), np.int64)),
        channels=len(native.SPATIAL_CHANNEL_NAMES),
    )
