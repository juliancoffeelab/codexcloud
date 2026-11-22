"""Resumable hierarchical imitation over losslessly compressed native shards."""

import argparse
from collections import defaultdict
import json
import os
from pathlib import Path
import random
import time
import numpy as np
import torch
import hex_command_training as native
from .checkpoint import warm_start
from .compressed_data import collect, load
from .mixed_workload import mass
from .recovery import atomic_json, save_bundle, load_resume, read_jsonl
from .spatial import SpatialPlanner
from .imitation_sequences import SEQUENCE_SCHEMA, forward_window
from .training_clock import TrainingClock


def measure(model, directory, manifest, sequence_length=8, sequence_batch_size=8):
    model.eval()
    rows = []
    with torch.no_grad():
        for name, meta in manifest["shards"].items():
            data = load(directory / name)
            loss = correct = 0.0
            n = len(data.dense[2])
            for first in range(0, len(data.streams), sequence_batch_size):
                streams = data.streams[first:first + sequence_batch_size]
                memory = None
                for step in range(0, max(map(len, streams)), sequence_length):
                    value, hits, _, memory = forward_window(
                        model, data, streams, step, memory, sequence_length
                    )
                    loss += float(value)
                    correct += hits
            rows.append(
                dict(
                    group=meta["group"],
                    loss=loss,
                    correct=correct,
                    n=n,
                    mass=mass(meta["configs"][0]),
                )
            )
    groups = {}
    for r in rows:
        k = tuple(r["group"])
        g = groups.setdefault(k, dict(loss=0.0, correct=0.0, n=0, mass=r["mass"]))
        for field in ["loss", "correct", "n"]:
            g[field] += r[field]
    total_mass = sum(g["mass"] for g in groups.values())
    return dict(
        loss=sum(g["mass"] * g["loss"] / max(1, g["n"]) for g in groups.values())
        / total_mass,
        accuracy=sum(g["mass"] * g["correct"] / max(1, g["n"]) for g in groups.values())
        / total_mass,
        groups=[
            dict(
                group=list(k),
                loss=g["loss"] / max(1, g["n"]),
                accuracy=g["correct"] / max(1, g["n"]),
                n=g["n"],
            )
            for k, g in groups.items()
        ],
    )


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    p.add_argument("--seconds", type=float, default=5400)
    p.add_argument("--resume", type=Path)
    p.add_argument(
        "--warm-start",
        type=Path,
        default=Path("training/checkpoints/campaign-cnn-imitation-v1/weights.pt"),
    )
    p.add_argument("--threads", type=int, default=2)
    p.add_argument("--sequence-length", type=int, default=8)
    p.add_argument("--sequence-batch-size", type=int, default=8)
    p.add_argument("--checkpoint-seconds", type=float, default=300)
    p.add_argument("--collection-workers", type=int, default=2)
    p.add_argument("--games-per-shard", type=int, default=4)
    a = p.parse_args()
    if (a.sequence_length < 1 or a.sequence_batch_size < 1
            or a.seconds <= 0 or a.checkpoint_seconds <= 0
            or a.collection_workers < 1 or a.games_per_shard < 1):
        p.error("budgets, sequence and collection sizes must be positive")
    sequence_training = dict(
        schema=SEQUENCE_SCHEMA, length=a.sequence_length,
        batch_size=a.sequence_batch_size,
    )
    root = a.root
    run = root / "posttune"
    run.mkdir(exist_ok=True)
    spec = json.loads((root / "workload.json").read_text())
    os.environ["HEX_COMMAND_DATA_ROOT"] = spec["catalog"]
    assert json.loads(native.fingerprints()) == spec["fingerprints"]
    torch.set_num_threads(a.threads)
    torch.set_num_interop_threads(1)
    torch.manual_seed(42)
    random.seed(42)
    rng = np.random.default_rng(42)
    model = SpatialPlanner().eval()
    warm_start(model, a.warm_start)
    optimizer = torch.optim.Adam(model.parameters(), lr=1e-4)
    old = load_resume(a.resume) if a.resume else None
    progress = dict(epoch=0, order=[], shard=0, ids=[], batch=0, updates=0,
                    decisions=0, sequence_step=0, memory=[])
    elapsed = 0.0
    best_loss = float("inf")
    if old:
        if old.get("budget_kind") != "training-time":
            raise ValueError("old wall-time checkpoints require a new warm start")
        if old.get("sequence_training") != sequence_training:
            raise ValueError("resume sequence settings differ; old independent-example checkpoints require a warm start")
        assert (
            old["workload_sha256"] == spec["workload_sha256"]
            and old["fingerprints"] == spec["fingerprints"]
        )
        model.load_state_dict(old["policy"])
        optimizer.load_state_dict(old["optimizer"])
        progress = old["progress"]
        elapsed = old["elapsed"]
        best_loss = old["best_loss"]
        torch.set_rng_state(old["torch_rng"])
        rng.bit_generator.state = old["numpy_rng"]
        random.setstate(old["python_rng"])
    start = time.monotonic()
    clock = TrainingClock(elapsed)
    prior_wall = old.get("wall_seconds", 0.0) if old else 0.0
    read_jsonl(run / "metrics.jsonl", repair=True)
    log = (run / "metrics.jsonl").open("a")

    def record(event, **kw):
        row = dict(event=event, seconds=clock.seconds(),
                   wall_seconds=prior_wall + time.monotonic() - start, **kw)
        log.write(json.dumps(row) + "\n")
        log.flush()
        print(json.dumps(row), flush=True)
        atomic_json(run / "status.json", row)

    def checkpoint(label):
        running = clock.started is not None
        clock.pause()
        state = dict(
            kind="mixed_imitation",
            policy=model.state_dict(),
            optimizer=optimizer.state_dict(),
            progress=progress,
            elapsed=clock.seconds(),
            wall_seconds=prior_wall + time.monotonic() - start,
            budget_kind="training-time",
            best_loss=best_loss,
            torch_rng=torch.get_rng_state(),
            numpy_rng=rng.bit_generator.state,
            python_rng=random.getstate(),
            fingerprints=spec["fingerprints"],
            workload_sha256=spec["workload_sha256"],
            sequence_training=sequence_training,
        )
        save_bundle(run, state, model, label)
        if running:
            clock.resume()

    checkpoint("initial")
    record("started", budget=a.seconds, resumed=bool(old), balance=spec["balance"],
           sequence_training=sequence_training)
    train = collect(spec["train"], root / "corpus/train", record,
                    workers=a.collection_workers, games_per_shard=a.games_per_shard)
    valid = collect(spec["imitation_validation"], root / "corpus/validation", record,
                    workers=a.collection_workers, games_per_shard=a.games_per_shard)
    total = sum(s["decisions"] for s in train["shards"].values())
    group_sizes = defaultdict(int)
    for s in train["shards"].values():
        group_sizes[tuple(s["group"])] += s["decisions"]
    names = list(train["shards"])
    next_save = (int(clock.seconds() // a.checkpoint_seconds) + 1) * a.checkpoint_seconds
    last_progress = time.monotonic()
    record(
        "collected",
        decisions=total,
        candidates=sum(s["candidates"] for s in train["shards"].values()),
        games=len(spec["train"]),
    )
    if not old:
        initial = measure(model, root / "corpus/validation", valid,
                          a.sequence_length, a.sequence_batch_size)
        record("validation", epoch=0, **initial)
    clock.resume()
    try:
        while clock.seconds() < a.seconds:
            if not progress["order"]:
                progress["epoch"] += 1
                progress["order"] = rng.permutation(len(names)).tolist()
                progress["shard"] = 0
            epoch_loss = epoch_rows = 0
            epoch_start = time.monotonic()
            while progress["shard"] < len(names) and clock.seconds() < a.seconds:
                name = names[progress["order"][progress["shard"]]]
                meta = train["shards"][name]
                data = load(root / "corpus/train" / name)
                n = len(data.streams)
                if not progress["ids"]:
                    progress["ids"] = rng.permutation(n).tolist()
                    progress["batch"] = 0
                weight = (
                    mass(meta["configs"][0]) * total / group_sizes[tuple(meta["group"])]
                )
                model.train()
                while progress["batch"] < n and clock.seconds() < a.seconds:
                    ids = progress["ids"][progress["batch"]:
                                          progress["batch"] + a.sequence_batch_size]
                    streams = [data.streams[i] for i in ids]
                    memory = (torch.tensor(progress["memory"], dtype=torch.float32)
                              if progress["memory"] else None)
                    optimizer.zero_grad()
                    summed_loss, _, count, memory = forward_window(
                        model, data, streams, progress["sequence_step"], memory,
                        a.sequence_length,
                    )
                    # Fixed denominator gives every decision the same loss mass,
                    # including the shorter final windows of ragged matches.
                    loss = summed_loss * weight / (a.sequence_batch_size * a.sequence_length)
                    if not torch.isfinite(loss):
                        raise RuntimeError("nonfinite imitation loss")
                    loss.backward()
                    norm = torch.nn.utils.clip_grad_norm_(
                        model.parameters(), 1.0, error_if_nonfinite=True
                    )
                    optimizer.step()
                    progress["sequence_step"] += a.sequence_length
                    progress["memory"] = memory.detach().tolist()
                    if progress["sequence_step"] >= max(map(len, streams)):
                        progress["batch"] += len(ids)
                        progress["sequence_step"] = 0
                        progress["memory"] = []
                    progress["updates"] += 1
                    progress["decisions"] += count
                    epoch_rows += count
                    epoch_loss += float(summed_loss.detach()) * weight
                    if clock.seconds() >= next_save:
                        checkpoint(f"minute-{round(next_save / 60):04d}")
                        while next_save <= clock.seconds():
                            next_save += a.checkpoint_seconds
                    if time.monotonic() - last_progress >= 30:
                        last_progress = time.monotonic()
                        record(
                            "progress",
                            **{
                                k: progress[k]
                                for k in ["epoch", "updates", "decisions"]
                            },
                            gradient_norm=float(norm),
                            loss=float(loss.detach()),
                        )
                if progress["batch"] >= n:
                    progress["ids"] = []
                    progress["shard"] += 1
            if progress["shard"] >= len(names):
                clock.pause()
                model.eval()
                v = measure(model, root / "corpus/validation", valid,
                            a.sequence_length, a.sequence_batch_size)
                record(
                    "validation",
                    epoch=progress["epoch"],
                    train_loss=epoch_loss / max(1, epoch_rows),
                    epoch_seconds=time.monotonic() - epoch_start,
                    **v,
                )
                improved = v["loss"] < best_loss
                best_loss = min(best_loss, v["loss"])
                checkpoint(f"epoch-{progress['epoch']:04d}")
                if improved:
                    atomic_json(
                        run / "best-loss.json",
                        json.loads((run / "latest.json").read_text()),
                    )
                progress["order"] = []
                progress["ids"] = []
                clock.resume()
            else:
                break
    finally:
        clock.pause()
        model.eval()
        checkpoint("last")
        record(
            "finished",
            epoch=progress["epoch"],
            updates=progress["updates"],
            decisions=progress["decisions"],
        )
        log.close()
    atomic_json(
        run / "complete.json",
        dict(elapsed=clock.seconds(), budget_kind="training-time",
             wall_seconds=prior_wall + time.monotonic() - start, progress=progress),
    )


if __name__ == "__main__":
    main()
