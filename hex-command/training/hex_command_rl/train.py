"""Bounded behavioral cloning with whole-match seed splits and native evaluation.

Run from the game root: python -m hex_command_rl.train --seconds 180
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import time
import numpy as np
import torch
from torch.nn import functional as F
import hex_command_training as native
from .model import Planner
from .workloads import arena, encode


def batches(data, indices, batch_size=64):
    features, offsets, labels, _ = data
    for start in range(0, len(indices), batch_size):
        ids = indices[start:start+batch_size]
        lengths = offsets[ids+1]-offsets[ids]
        width = int(lengths.max())
        positions = torch.arange(width)[None, :]
        mask = positions < torch.from_numpy(lengths.copy())[:, None]
        gather = torch.from_numpy(offsets[ids].copy())[:, None] + positions
        # Padding indexes must remain inside the owned feature buffer.
        x = torch.from_numpy(features)[gather.clamp_max(len(features)-1)]
        yield x, mask, torch.from_numpy(labels[ids].copy()), ids


def metrics(model, data):
    correct = total = loss = uniform = stops = nontrivial_correct = nontrivial = 0
    types = {name: [0, 0] for name in ("end_turn", "move", "attack", "advance")}
    with torch.no_grad():
        for x, mask, y, _ in batches(data, np.arange(len(data[2]))):
            logits = model(x).masked_fill(~mask, -torch.inf)
            pred = logits.argmax(-1)
            correct += int((pred == y).sum()); total += len(y)
            loss += float(F.cross_entropy(logits, y, reduction="sum"))
            uniform += float((1 / mask.sum(-1).float()).sum())
            stops += int((y == 0).sum())
            active = mask.sum(-1) > 1
            nontrivial += int(active.sum())
            nontrivial_correct += int(((pred == y) & active).sum())
            for k, name in enumerate(types):
                sel = x[torch.arange(len(y)), y, k] == 1
                types[name][0] += int(((pred == y) & sel).sum())
                types[name][1] += int(sel.sum())
    return dict(accuracy=correct/max(total, 1), cross_entropy=loss/max(total, 1),
                uniform_accuracy=uniform/max(total, 1), always_end_accuracy=stops/max(total, 1),
                nontrivial_accuracy=nontrivial_correct/max(nontrivial, 1),
                decisions=total, by_action=types)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--seconds", type=float, default=180)
    p.add_argument("--epochs", type=int, default=100)
    p.add_argument("--train-seeds", type=int, default=4)
    p.add_argument("--max-samples", type=int, default=30000)
    p.add_argument("--teacher", choices=["classic", "balanced", "rotated", "situational", "predictor"], default="balanced")
    p.add_argument("--train-configs", type=Path, help="JSON array of native match configs")
    p.add_argument("--validation-configs", type=Path)
    p.add_argument("--evaluation-configs", type=Path)
    p.add_argument("--out", type=Path, default=Path("tmp/rl/deathmatch"))
    args = p.parse_args()
    if not 0 < args.seconds <= 600 or args.train_seeds < 1 or args.epochs < 1:
        p.error("seconds must be in (0,600]; train-seeds and epochs must be positive")
    args.out.mkdir(parents=True, exist_ok=True)
    torch.set_num_threads(2); torch.set_num_interop_threads(1)
    torch.manual_seed(42); np.random.seed(42); random.seed(42)
    fingerprints = json.loads(native.fingerprints())
    started = time.monotonic()
    train_configs = arena(range(1, args.train_seeds+1), args.teacher)
    valid_configs = arena([10001], args.teacher)
    evaluation_configs = arena([20001], args.teacher)
    if args.train_configs: train_configs = json.loads(args.train_configs.read_text())
    if args.validation_configs: valid_configs = json.loads(args.validation_configs.read_text())
    if args.evaluation_configs: evaluation_configs = json.loads(args.evaluation_configs.read_text())
    seed_sets = [set(c["seed"] for c in configs) for configs in (train_configs, valid_configs, evaluation_configs)]
    if any(seed_sets[i] & seed_sets[j] for i, j in ((0, 1), (0, 2), (1, 2))):
        p.error("training, validation and evaluation must use disjoint seeds")
    if not all((train_configs, valid_configs, evaluation_configs)):
        p.error("all three workloads must be nonempty")
    train = native.collect(encode(train_configs), args.max_samples)
    valid = native.collect(encode(valid_configs), args.max_samples)
    if not len(train[2]) or not len(valid[2]):
        raise RuntimeError("empty teacher dataset")
    if train[3] or valid[3]:
        raise RuntimeError(f"teacher actions absent from candidates: train={train[3]}, validation={valid[3]}")
    print(json.dumps(dict(event="collected", train=len(train[2]), validation=len(valid[2]),
                          train_candidates=len(train[0]), seconds=time.monotonic()-started)), flush=True)
    model = Planner(); optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
    initial = metrics(model, valid)
    best = float("inf"); best_epoch = 0
    deadline = started + args.seconds
    history = []
    for epoch in range(args.epochs):
        if time.monotonic() >= deadline: break
        model.train()
        for x, mask, y, _ in batches(train, np.random.permutation(len(train[2]))):
            if time.monotonic() >= deadline: break
            optimizer.zero_grad()
            loss = F.cross_entropy(model(x).masked_fill(~mask, -torch.inf), y)
            loss.backward(); torch.nn.utils.clip_grad_norm_(model.parameters(), 1.)
            optimizer.step()
        model.eval(); report = metrics(model, valid)
        report.update(epoch=epoch+1, seconds=round(time.monotonic()-started, 2))
        history.append(report); print(json.dumps(report), flush=True)
        if report["cross_entropy"] < best:
            best = report["cross_entropy"]; best_epoch = epoch+1
            torch.save(model.state_dict(), args.out / "best.pt")
            (args.out / "planner.json").write_text(model.export())
    if not best_epoch:
        raise RuntimeError("budget expired before first training epoch; increase --seconds")
    training_seconds = time.monotonic()-started
    model.load_state_dict(torch.load(args.out / "best.pt", weights_only=True)); model.eval()
    exported = model.export()
    probe = valid[0][:1024].copy()
    with torch.no_grad(): expected = model(torch.from_numpy(probe)).numpy()
    error = float(np.max(np.abs(native.score(exported, probe)-expected)))
    np.testing.assert_allclose(native.score(exported, probe), expected, atol=2e-5, rtol=2e-5)
    teacher = [json.loads(r) for r in native.evaluate(encode(evaluation_configs), None)]
    learned = [json.loads(r) for r in native.evaluate(encode(evaluation_configs), exported)]
    digest = hashlib.sha256()
    for path in sorted(Path("rust").rglob("*.rs")):
        if "target" not in path.parts:
            digest.update(str(path).encode()); digest.update(path.read_bytes())
    report = dict(schema=native.SCHEMA, feature_names=native.FEATURE_NAMES,
                  source_sha256=digest.hexdigest(), fingerprints=fingerprints, teacher=args.teacher, seed=42,
                  train_configs=train_configs, validation_configs=valid_configs,
                  evaluation_configs=evaluation_configs, initial_validation=initial,
                  final_validation=metrics(model, valid), best_epoch=best_epoch,
                  training_seconds=training_seconds, inference_max_absolute_error=error,
                  teacher_results=teacher, learned_results=learned, history=history,
                  versions=dict(torch=torch.__version__, numpy=np.__version__))
    (args.out / "report.json").write_text(json.dumps(report, indent=2))
    print(json.dumps(dict(event="finished", training_seconds=round(training_seconds, 2),
                          teacher_wins=sum(r["success"] for r in teacher),
                          learned_wins=sum(r["success"] for r in learned),
                          evaluation_games=len(learned),
                          validation=report["final_validation"], parity_error=error)), flush=True)


if __name__ == "__main__": main()
