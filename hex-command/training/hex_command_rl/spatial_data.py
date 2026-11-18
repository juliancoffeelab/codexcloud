"""Disk-backed boards stored once per decision, with ragged candidate rows."""

import json
from pathlib import Path
import numpy as np
import torch
import hex_command_training as native
from .data import load_dataset
from .workloads import encode


class SpatialData:
    def __init__(self, directory):
        directory = Path(directory)
        self.manifest = json.loads((directory / "manifest.json").read_text())
        current = (
            self.manifest.get("spatial_schema") == native.SPATIAL_SCHEMA
            and self.manifest.get("channel_names") == native.SPATIAL_CHANNEL_NAMES
        )
        legacy = (
            self.manifest.get("spatial_schema") == 3
            and self.manifest.get("channel_names")
            == native.LEGACY_SPATIAL_CHANNEL_NAMES
        )
        duties = (
            self.manifest.get("spatial_schema") == 5
            and self.manifest.get("channel_names") == native.DUTY_SPATIAL_CHANNEL_NAMES
        )
        diplomacy = (
            self.manifest.get("spatial_schema") == 7
            and self.manifest.get("channel_names")
            == native.CAMPAIGN_SPATIAL_CHANNEL_NAMES
        )
        campaign = (
            self.manifest.get("spatial_schema") == 9
            and self.manifest.get("channel_names")
            == native.CAMPAIGN_SPATIAL_CHANNEL_NAMES
        )
        if not current and not legacy and not duties and not diplomacy and not campaign:
            raise ValueError("spatial dataset schema mismatch")
        if (
            current
            and self.manifest.get("actor_channels") != native.ACTOR_SPATIAL_CHANNELS
        ):
            raise ValueError(
                "spatial actor-context schema mismatch; regenerate this dataset"
            )
        self.dense = load_dataset(directory)
        self.offsets = np.load(directory / "board_offsets.npy")
        self.shapes = np.load(directory / "shapes.npy")
        self.indices = np.load(directory / "indices.npy")
        n = len(self.dense[2])
        channels = len(self.manifest["channel_names"])
        self.channels = channels
        self.context_values = (
            np.load(directory / "context_values.npy", mmap_mode="r")
            if current
            else np.zeros(0, np.float32)
        )
        self.contexts = (
            np.load(directory / "contexts.npy")
            if current
            else np.empty((0, 3), np.int64)
        )
        if self.contexts.ndim != 2 or self.contexts.shape[1] != 3:
            raise ValueError("invalid actor context metadata")
        expected = 0
        actors = set()
        for owner, site, offset in self.contexts:
            if (
                not 0 <= owner < len(self.shapes)
                or not 0 <= site < np.prod(self.shapes[owner])
                or offset != expected
                or (owner, site) in actors
            ):
                raise ValueError("invalid actor context index or offset")
            actors.add((owner, site))
            expected += len(native.ACTOR_SPATIAL_CHANNELS) * int(
                np.prod(self.shapes[owner])
            )
        if (
            expected != len(self.context_values)
            or not np.isfinite(self.context_values).all()
        ):
            raise ValueError("invalid actor context values")
        expected_width = (
            len(native.LEGACY_FEATURE_NAMES)
            if legacy
            else len(native.DUTY_FEATURE_NAMES)
            if duties
            else len(native.DIPLOMACY_FEATURE_NAMES)
            if diplomacy
            else len(native.CAMPAIGN_FEATURE_NAMES)
            if campaign
            else len(native.FEATURE_NAMES)
        )
        if self.dense[0].shape[1] != expected_width:
            raise ValueError("mixed dense/spatial dataset schemas")
        if (
            self.shapes.shape != (n, 2)
            or self.offsets.shape != (n + 1,)
            or self.indices.shape != (len(self.dense[0]), 3)
        ):
            raise ValueError("spatial dataset dimensions mismatch")
        if (
            self.offsets[0] != 0
            or np.any(self.shapes <= 0)
            or np.any(self.shapes > 256)
            or np.any(self.shapes.prod(1) > 16384)
            or np.any(np.diff(self.offsets) != self.shapes.prod(1) * channels)
        ):
            raise ValueError("invalid spatial board offsets/shapes")
        if (directory / "boards.bin").stat().st_size != int(self.offsets[-1]) * 4:
            raise ValueError("incomplete spatial board buffer")
        cells = np.repeat(self.shapes.prod(1), np.diff(self.dense[1]))
        if np.any(self.indices < -1) or np.any(self.indices >= cells[:, None]):
            raise ValueError("invalid spatial candidate indices")
        self.boards = np.memmap(
            directory / "boards.bin",
            dtype=np.float32,
            mode="c",
            shape=(int(self.offsets[-1]),),
        )


def collect_spatial_dataset(configs, directory, progress=None, batch_games=8):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "manifest.json").unlink(missing_ok=True)
    offsets = [0]
    labels = []
    board_offsets = [0]
    shapes = []
    indices = []
    shards = []
    context_values = []
    contexts = []
    context_offset = 0
    rows = 0
    values = 0
    with (
        (directory / "features.bin").open("wb") as xf,
        (directory / "boards.bin").open("wb") as bf,
    ):
        for start in range(0, len(configs), batch_games):
            dense, spatial = native.collect_spatial(
                encode(configs[start : start + batch_games]), 1_000_000, side="player"
            )
            x, bounds, y, missing = dense
            boards, bbounds, shape, ix, cv, cm = spatial
            cm = cm.copy()
            cm[:, 0] += len(labels)
            cm[:, 2] += context_offset
            context_offset += len(cv)
            context_values.append(cv)
            contexts.append(cm)
            if missing or len(y) >= 1_000_000:
                raise RuntimeError("incomplete spatial teacher shard")
            assert len(shape) == len(y) and len(ix) == len(x)
            shards.append(
                dict(
                    config_start=start,
                    config_end=min(start + batch_games, len(configs)),
                    decision_start=len(labels),
                    decision_end=len(labels) + len(y),
                )
            )
            x.tofile(xf)
            boards.tofile(bf)
            offsets.extend((bounds[1:] + rows).tolist())
            labels.extend(y.tolist())
            board_offsets.extend((bbounds[1:] + values).tolist())
            shapes.extend(shape.tolist())
            indices.append(ix)
            rows += len(x)
            values += len(boards)
            if progress:
                progress(
                    dict(
                        event="collection",
                        directory=str(directory),
                        games=min(start + batch_games, len(configs)),
                        total_games=len(configs),
                        decisions=len(labels),
                        candidates=rows,
                    )
                )
    if not labels:
        raise ValueError("empty spatial teacher data")
    for name, array in [
        ("offsets", offsets),
        ("labels", labels),
        ("board_offsets", board_offsets),
        ("shapes", shapes),
    ]:
        np.save(directory / f"{name}.npy", np.asarray(array, dtype=np.int64))
    np.save(directory / "indices.npy", np.concatenate(indices))
    np.save(directory / "context_values.npy", np.concatenate(context_values))
    np.save(directory / "contexts.npy", np.concatenate(contexts))
    manifest = dict(
        schema=native.SCHEMA,
        feature_names=native.FEATURE_NAMES,
        spatial_schema=native.SPATIAL_SCHEMA,
        channel_names=native.SPATIAL_CHANNEL_NAMES,
        actor_channels=native.ACTOR_SPATIAL_CHANNELS,
        fingerprints=json.loads(native.fingerprints()),
        configs=configs,
        collection_side="player",
        decisions=len(labels),
        candidates=rows,
        shards=shards,
    )
    (directory / "manifest.json").write_text(json.dumps(manifest, indent=2))
    return SpatialData(directory)


def batches(data, ids, batch_size=8):
    from .ppo import pack

    features, offsets, labels, _ = data.dense
    for start in range(0, len(ids), batch_size):
        selected = np.asarray(ids[start : start + batch_size])
        samples = []
        for i in selected:
            h, w = map(int, data.shapes[i])
            a, b = int(offsets[i]), int(offsets[i + 1])
            contexts = [
                (
                    int(site),
                    data.context_values[
                        offset : offset + len(native.ACTOR_SPATIAL_CHANNELS) * h * w
                    ].reshape(len(native.ACTOR_SPATIAL_CHANNELS), h, w),
                )
                for owner, site, offset in data.contexts
                if owner == i
            ]
            samples.append(
                (
                    features[a:b],
                    data.boards[data.offsets[i] : data.offsets[i + 1]].reshape(
                        data.channels, h, w
                    ),
                    data.indices[a:b],
                    contexts,
                )
            )
        x, board, ix, mask = pack(samples)
        yield x, board, ix, mask, torch.from_numpy(labels[selected].copy())


def map_weights(data, enabled=False):
    """Equal aggregate loss weight per map, retaining every collected decision."""
    count = len(data.dense[2])
    if not enabled:
        return np.ones(count, dtype=np.float32)
    groups = np.full(count, -1, dtype=np.int64)
    names = {}
    cursor = 0
    for shard in data.manifest.get("shards", []):
        names_in_shard = {
            c.get("landscape", c.get("campaign"))
            for c in data.manifest["configs"][
                shard["config_start"] : shard["config_end"]
            ]
        }
        if len(names_in_shard) != 1 or shard["decision_start"] != cursor:
            raise ValueError(
                "map balancing requires contiguous homogeneous-map collection shards"
            )
        name = next(iter(names_in_shard))
        group = names.setdefault(name, len(names))
        cursor = shard["decision_end"]
        groups[shard["decision_start"] : cursor] = group
    if cursor != count or np.any(groups < 0):
        raise ValueError("incomplete map-balancing metadata")
    sizes = np.bincount(groups)
    if len(sizes) != len(names) or np.any(sizes == 0):
        raise ValueError("empty training map")
    return (count / (len(sizes) * sizes[groups])).astype(np.float32)
