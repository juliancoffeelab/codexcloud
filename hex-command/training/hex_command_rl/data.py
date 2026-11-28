"""Collect small native shards into a disk-backed candidate matrix."""

import json
from pathlib import Path
import numpy as np
import hex_command_training as native
from .workloads import encode, recruiter_pair


def collect_dataset(
    configs, directory, max_samples=None, batch_games=8, progress=None, side=None
):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "manifest.json").unlink(missing_ok=True)
    offsets = [0]
    labels = []
    shards = []
    rows = 0
    with (directory / "features.bin").open("wb") as output:
        for start in range(0, len(configs), batch_games):
            batch = configs[start : start + batch_games]
            # Per-shard guard. Reaching it is an error, never silent truncation.
            guard = 1_000_000
            x, boundaries, y, missing = native.collect(encode(batch), guard, side=side)
            if missing or len(y) >= guard:
                raise RuntimeError(
                    f"incomplete teacher shard: missing={missing}, decisions={len(y)}"
                )
            if max_samples is not None and len(labels) + len(y) > max_samples:
                raise RuntimeError(
                    f"dataset exceeds --max-samples={max_samples}; raise it or omit it to collect all configurations"
                )
            x.tofile(output)
            offsets.extend((boundaries[1:] + rows).tolist())
            labels.extend(y.tolist())
            rows += len(x)
            shard = dict(
                config_start=start,
                config_end=start + len(batch),
                decision_end=len(labels),
                candidate_end=rows,
                recruiters=sorted({recruiter_pair(c) for c in batch}),
            )
            shards.append(shard)
            if progress:
                progress(
                    dict(
                        event="collection",
                        games=start + len(batch),
                        total_games=len(configs),
                        decisions=len(labels),
                        candidates=rows,
                    )
                )
    if not labels:
        raise RuntimeError("empty teacher dataset")
    np.save(directory / "offsets.npy", np.asarray(offsets, dtype=np.int64))
    np.save(directory / "labels.npy", np.asarray(labels, dtype=np.int64))
    manifest = dict(
        collection_side=side,
        schema=native.SCHEMA,
        feature_names=native.FEATURE_NAMES,
        fingerprints=json.loads(native.fingerprints()),
        configs=configs,
        shards=shards,
        decisions=len(labels),
        candidates=rows,
    )
    (directory / "manifest.json").write_text(json.dumps(manifest, indent=2))
    return load_dataset(directory)


def load_dataset(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "manifest.json").read_text())
    current = (
        manifest["schema"] == native.SCHEMA
        and manifest["feature_names"] == native.FEATURE_NAMES
    )
    legacy = (
        manifest["schema"] == 2
        and manifest["feature_names"] == native.LEGACY_FEATURE_NAMES
    )
    duties = (
        manifest["schema"] == 4
        and manifest["feature_names"] == native.DUTY_FEATURE_NAMES
    )
    diplomacy = (
        manifest["schema"] == 6
        and manifest["feature_names"] == native.DIPLOMACY_FEATURE_NAMES
    )
    campaign = (
        manifest["schema"] == 8
        and manifest["feature_names"] == native.CAMPAIGN_FEATURE_NAMES
    )
    visibility = (
        manifest["schema"] == 10
        and manifest["feature_names"] == native.VISIBILITY_FEATURE_NAMES
    )
    idle = (
        manifest["schema"] == 12
        and manifest["feature_names"] == native.IDLE_FEATURE_NAMES
    )
    if not any((current, legacy, duties, diplomacy, campaign, visibility, idle)):
        raise ValueError("dataset feature schema mismatch")
    if not (current or visibility or idle) and any(
        "campaign" in c for c in manifest["configs"]
    ):
        raise ValueError(
            "legacy campaign observations need recollection with current policy and annotations"
        )
    shape = (manifest["candidates"], len(manifest["feature_names"]))
    expected = int(np.prod(shape)) * np.dtype(np.float32).itemsize
    if (directory / "features.bin").stat().st_size != expected:
        raise ValueError("incomplete candidate buffer")
    # Copy-on-write permits torch.from_numpy without writable-buffer warnings;
    # training gathers rows and never modifies the underlying observations.
    features = np.memmap(
        directory / "features.bin", dtype=np.float32, mode="c", shape=shape
    )
    offsets = np.load(directory / "offsets.npy")
    labels = np.load(directory / "labels.npy")
    if (
        len(offsets) != len(labels) + 1
        or offsets[0] != 0
        or offsets[-1] != len(features)
        or np.any(np.diff(offsets) <= labels)
        or np.any(labels < 0)
    ):
        raise ValueError("invalid ragged dataset offsets/labels")
    return features, offsets, labels, 0
