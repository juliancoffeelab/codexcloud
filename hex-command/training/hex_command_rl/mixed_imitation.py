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
from torch.nn import functional as F
import hex_command_training as native
from .checkpoint import warm_start
from .compressed_data import collect, load
from .mixed_workload import mass
from .recovery import atomic_json, save_bundle, load_resume
from .spatial import SpatialPlanner
from .spatial_data import batches


def measure(model, directory, manifest):
    model.eval()
    rows = []
    with torch.no_grad():
        for name, meta in manifest["shards"].items():
            data = load(directory / name)
            loss = correct = 0.0
            n = len(data.dense[2])
            for x, b, ix, mask, y in batches(data, np.arange(n)):
                scores = model(x, b, ix).masked_fill(~mask, -torch.inf)
                loss += float(F.cross_entropy(scores, y, reduction="sum"))
                correct += int((scores.argmax(-1) == y).sum())
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
    a = p.parse_args()
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
    progress = dict(epoch=0, order=[], shard=0, ids=[], batch=0, updates=0, decisions=0)
    elapsed = 0.0
    best_loss = float("inf")
    if old:
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
    deadline = start + max(0, a.seconds - elapsed)
    log = (run / "metrics.jsonl").open("a")

    def record(event, **kw):
        row = dict(event=event, seconds=elapsed + time.monotonic() - start, **kw)
        log.write(json.dumps(row) + "\n")
        log.flush()
        print(json.dumps(row), flush=True)
        atomic_json(run / "status.json", row)

    def checkpoint(label):
        state = dict(
            kind="mixed_imitation",
            policy=model.state_dict(),
            optimizer=optimizer.state_dict(),
            progress=progress,
            elapsed=elapsed + time.monotonic() - start,
            best_loss=best_loss,
            torch_rng=torch.get_rng_state(),
            numpy_rng=rng.bit_generator.state,
            python_rng=random.getstate(),
            fingerprints=spec["fingerprints"],
            workload_sha256=spec["workload_sha256"],
        )
        save_bundle(run, state, model, label)

    checkpoint("initial")
    record("started", budget=a.seconds, resumed=bool(old), balance=spec["balance"])
    train = collect(spec["train"], root / "corpus/train", record)
    valid = collect(spec["validation"], root / "corpus/validation", record)
    total = sum(s["decisions"] for s in train["shards"].values())
    group_sizes = defaultdict(int)
    for s in train["shards"].values():
        group_sizes[tuple(s["group"])] += s["decisions"]
    names = list(train["shards"])
    last_save = time.monotonic()
    record(
        "collected",
        decisions=total,
        candidates=sum(s["candidates"] for s in train["shards"].values()),
        games=len(spec["train"]),
    )
    if not old:
        initial = measure(model, root / "corpus/validation", valid)
        record("validation", epoch=0, **initial)
    try:
        while time.monotonic() < deadline:
            if not progress["order"]:
                progress["epoch"] += 1
                progress["order"] = rng.permutation(len(names)).tolist()
                progress["shard"] = 0
            epoch_loss = epoch_rows = 0
            epoch_start = time.monotonic()
            while progress["shard"] < len(names) and time.monotonic() < deadline:
                name = names[progress["order"][progress["shard"]]]
                meta = train["shards"][name]
                data = load(root / "corpus/train" / name)
                n = len(data.dense[2])
                if not progress["ids"]:
                    progress["ids"] = rng.permutation(n).tolist()
                    progress["batch"] = 0
                weight = (
                    mass(meta["configs"][0]) * total / group_sizes[tuple(meta["group"])]
                )
                model.train()
                for x, b, ix, mask, y in batches(
                    data, np.asarray(progress["ids"][progress["batch"] :]), 64
                ):
                    if time.monotonic() >= deadline:
                        break
                    optimizer.zero_grad()
                    scores = model(x, b, ix).masked_fill(~mask, -torch.inf)
                    loss = F.cross_entropy(scores, y) * weight
                    if not torch.isfinite(loss):
                        raise RuntimeError("nonfinite imitation loss")
                    loss.backward()
                    norm = torch.nn.utils.clip_grad_norm_(
                        model.parameters(), 1.0, error_if_nonfinite=True
                    )
                    optimizer.step()
                    progress["batch"] += len(y)
                    progress["updates"] += 1
                    progress["decisions"] += len(y)
                    epoch_rows += len(y)
                    epoch_loss += float(loss.detach()) * len(y)
                    if time.monotonic() - last_save >= 120:
                        checkpoint(f"update-{progress['updates']:08d}")
                        last_save = time.monotonic()
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
                model.eval()
                v = measure(model, root / "corpus/validation", valid)
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
            else:
                break
    finally:
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
        dict(elapsed=elapsed + time.monotonic() - start, progress=progress),
    )


if __name__ == "__main__":
    main()
