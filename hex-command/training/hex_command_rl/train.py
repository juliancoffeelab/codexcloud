"""Bounded native behavioral cloning with sampled diagnostics and held-out matches."""

import argparse
import json
import math
from pathlib import Path
import random
import resource
import time
import numpy as np
import torch
from torch.nn import functional as F
import hex_command_training as native
from .model import Planner
from .workloads import RECRUITERS, arena, encode, recruiter_pair
from .data import collect_dataset, load_dataset


def batches(data, indices, batch_size=64):
    features, offsets, labels, _ = data
    for start in range(0, len(indices), batch_size):
        ids = indices[start : start + batch_size]
        lengths = offsets[ids + 1] - offsets[ids]
        width = int(lengths.max())
        positions = torch.arange(width)[None, :]
        mask = positions < torch.from_numpy(lengths.copy())[:, None]
        gather = torch.from_numpy(offsets[ids].copy())[:, None] + positions
        x = torch.from_numpy(features)[gather.clamp_max(len(features) - 1)]
        yield x, mask, torch.from_numpy(labels[ids].copy()), ids


def metrics(model, data, indices=None):
    correct = total = loss = uniform = stops = nontrivial_correct = nontrivial = 0
    types = {name: [0, 0] for name in ("end_turn", "move", "attack", "advance")}
    predicted = {name: 0 for name in types}
    with torch.no_grad():
        for x, mask, y, _ in batches(
            data, np.arange(len(data[2])) if indices is None else indices
        ):
            logits = model(x).masked_fill(~mask, -torch.inf)
            pred = logits.argmax(-1)
            correct += int((pred == y).sum())
            total += len(y)
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
                predicted[name] += int((x[torch.arange(len(pred)), pred, k] == 1).sum())
    return dict(
        accuracy=correct / max(total, 1),
        cross_entropy=loss / max(total, 1),
        uniform_accuracy=uniform / max(total, 1),
        always_end_accuracy=stops / max(total, 1),
        nontrivial_accuracy=nontrivial_correct / max(nontrivial, 1),
        decisions=total,
        by_action=types,
        predicted_actions=predicted,
    )


def dataset_stats(data, indices):
    features, offsets, labels, missing = data
    ambiguous = duplicate = groups = candidates = 0
    for i in indices:
        rows = features[offsets[i] : offsets[i + 1]]
        counts = {}
        for row in rows:
            key = row.tobytes()
            counts[key] = counts.get(key, 0) + 1
        size = counts[rows[labels[i]].tobytes()]
        ambiguous += size > 1
        groups += size
        duplicate += len(rows) - len(counts)
        candidates += len(rows)
    return dict(
        decisions=len(labels),
        candidates=len(features),
        missing_teacher_actions=missing,
        feature_buffer_mib=features.nbytes / 1024**2,
        mean_candidates=float(np.diff(offsets).mean()),
        max_candidates=int(np.diff(offsets).max()),
        diagnostic_sample=len(indices),
        teacher_ambiguous_fraction=ambiguous / max(len(indices), 1),
        mean_identical_teacher_candidates=groups / max(len(indices), 1),
        duplicate_candidate_fraction=duplicate / max(candidates, 1),
    )


def results_summary(results):
    return dict(
        wins=sum(r["success"] for r in results),
        losses=sum(not r["success"] and not r["truncated"] for r in results),
        caps=sum(r["truncated"] for r in results),
        games=len(results),
        mean_rounds=float(np.mean([r.get("rounds", 0) for r in results])),
    )


def evaluate(configs, model=None):
    return [json.loads(r) for r in native.evaluate(encode(configs), model)]


def results_by_recruiter(configs, teacher, learned):
    keys = sorted({recruiter_pair(c) for c in configs})
    return {
        k: dict(
            baseline=results_summary(
                [r for c, r in zip(configs, teacher) if recruiter_pair(c) == k]
            ),
            learned=results_summary(
                [r for c, r in zip(configs, learned) if recruiter_pair(c) == k]
            ),
        )
        for k in keys
    }


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--seconds", type=float, default=180)
    p.add_argument("--epochs", type=int, default=100)
    p.add_argument("--train-seeds", type=int, default=16)
    p.add_argument(
        "--max-samples",
        type=int,
        help="Optional hard dataset limit; fails rather than truncates",
    )
    p.add_argument(
        "--recruiters", nargs="+", choices=RECRUITERS, default=list(RECRUITERS)
    )
    p.add_argument(
        "--teacher",
        choices=["classic", "balanced", "rotated", "situational", "predictor"],
        default="balanced",
    )
    p.add_argument(
        "--opponent",
        choices=["classic", "balanced", "rotated", "situational", "predictor"],
        default="classic",
    )
    p.add_argument(
        "--mirror-recruiters",
        action="store_true",
        help="Override enemy recruiter to match ours; default preserves the full opponent preset",
    )
    p.add_argument(
        "--collect-side", choices=["player", "enemy", "both"], default="player"
    )
    p.add_argument("--architecture", choices=["mlp", "linear"], default="mlp")
    p.add_argument(
        "--dataset-dir",
        type=Path,
        help="Reuse train/validation arrays with matching configurations, side, content and feature schema",
    )
    p.add_argument("--learning-rate", type=float, default=1e-3)
    p.add_argument("--weight-decay", type=float, default=0.0)
    p.add_argument("--train-configs", type=Path)
    p.add_argument("--validation-configs", type=Path)
    p.add_argument("--evaluation-configs", type=Path)
    p.add_argument("--eval-rounds", type=int, default=100)
    p.add_argument("--eval-seeds", type=int, default=3)
    p.add_argument(
        "--eval-every",
        type=int,
        default=5,
        help="Periodic validation interval; 0 disables",
    )
    p.add_argument("--out", type=Path, default=Path("tmp/rl/deathmatch-classic"))
    args = p.parse_args()
    if (
        not math.isfinite(args.seconds)
        or args.seconds <= 0
        or min(args.train_seeds, args.epochs, args.eval_seeds) < 1
        or (args.max_samples is not None and args.max_samples < 1)
    ):
        p.error("seconds must be finite and positive; counts must be positive")
    if (
        not math.isfinite(args.learning_rate)
        or args.learning_rate <= 0
        or not math.isfinite(args.weight_decay)
        or args.weight_decay < 0
    ):
        p.error("learning-rate must be positive; weight-decay nonnegative; both finite")
    if len(set(args.recruiters)) != len(args.recruiters):
        p.error("recruiters must be unique")
    if not 1 <= args.eval_rounds <= 1000 or args.eval_every < 0:
        p.error("eval-rounds must be in [1,1000]; eval-every cannot be negative")
    args.out.mkdir(parents=True, exist_ok=True)
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    torch.manual_seed(42)
    np.random.seed(42)
    random.seed(42)
    fingerprints = json.loads(native.fingerprints())
    started = time.monotonic()
    train_configs = arena(
        range(1, args.train_seeds + 1),
        args.teacher,
        opponent=args.opponent,
        recruiters=args.recruiters,
        mirror_recruiters=args.mirror_recruiters,
    )
    valid_configs = arena(
        [10001],
        args.teacher,
        opponent=args.opponent,
        recruiters=args.recruiters,
        mirror_recruiters=args.mirror_recruiters,
    )
    evaluation_configs = arena(
        range(20001, 20001 + args.eval_seeds),
        args.teacher,
        opponent=args.opponent,
        rounds=args.eval_rounds,
        all_pairs=True,
        recruiters=args.recruiters,
        mirror_recruiters=args.mirror_recruiters,
    )
    if args.train_configs:
        train_configs = json.loads(args.train_configs.read_text())
    if args.validation_configs:
        valid_configs = json.loads(args.validation_configs.read_text())
    if args.evaluation_configs:
        evaluation_configs = json.loads(args.evaluation_configs.read_text())
        for c in evaluation_configs:
            if "campaign" not in c:
                c["maxRounds"] = args.eval_rounds
    periodic_configs = (
        [
            c
            for c in arena(
                [15001],
                args.teacher,
                opponent=args.opponent,
                recruiters=args.recruiters,
                mirror_recruiters=args.mirror_recruiters,
            )
            if c["armies"]["player"] == "human"
        ]
        if args.eval_every
        else []
    )
    workloads = (train_configs, valid_configs, evaluation_configs, periodic_configs)
    seeds = [set(c["seed"] for c in configs) for configs in workloads]
    if any(seeds[i] & seeds[j] for i in range(4) for j in range(i)):
        p.error(
            "training, validation, periodic and final evaluation must use disjoint seeds"
        )
    if not all((train_configs, valid_configs, evaluation_configs)):
        p.error("train, validation and final workloads must be nonempty")

    def collection_progress(row):
        row["seconds"] = round(time.monotonic() - started, 2)
        print(json.dumps(row), flush=True)

    collection_side = None if args.collect_side == "both" else args.collect_side
    data_fingerprints = []
    if args.dataset_dir:
        for split, configs in [("train", train_configs), ("validation", valid_configs)]:
            manifest = json.loads(
                (args.dataset_dir / split / "manifest.json").read_text()
            )
            if (
                manifest["configs"] != configs
                or manifest.get("collection_side") != collection_side
            ):
                p.error(f"{split} dataset configurations or collection side differ")
            if manifest["fingerprints"]["content"] != fingerprints["content"]:
                p.error(f"{split} dataset content differs from current catalog")
            data_fingerprints.append(manifest["fingerprints"])
        train = load_dataset(args.dataset_dir / "train")
        valid = load_dataset(args.dataset_dir / "validation")
        if (
            args.max_samples is not None
            and max(len(train[2]), len(valid[2])) > args.max_samples
        ):
            p.error("saved datasets exceed --max-samples")
    else:
        train = collect_dataset(
            train_configs,
            args.out / "data" / "train",
            args.max_samples,
            progress=collection_progress,
            side=collection_side,
        )
        valid = collect_dataset(
            valid_configs,
            args.out / "data" / "validation",
            args.max_samples,
            progress=collection_progress,
            side=collection_side,
        )
        data_fingerprints = [fingerprints, fingerprints]
    if not len(train[2]) or not len(valid[2]) or train[3] or valid[3]:
        raise RuntimeError(
            f"empty data or missing teacher actions: train={train[3]}, validation={valid[3]}"
        )
    collection_seconds = time.monotonic() - started
    rng = np.random.default_rng(123)
    train_probe = rng.choice(len(train[2]), min(1024, len(train[2])), replace=False)
    valid_probe = rng.choice(len(valid[2]), min(1024, len(valid[2])), replace=False)
    diagnostics = dict(
        train=dataset_stats(train, train_probe),
        validation=dataset_stats(valid, valid_probe),
    )
    periodic_baseline = evaluate(periodic_configs) if periodic_configs else []
    model = Planner(architecture=args.architecture)
    optimizer = torch.optim.Adam(
        model.parameters(), lr=args.learning_rate, weight_decay=args.weight_decay
    )
    initial = metrics(model, valid)
    initial_train = metrics(model, train, train_probe)
    best = float("inf")
    best_epoch = 0
    history = []
    periodic = []
    deadline = started + args.seconds
    with (args.out / "metrics.jsonl").open("w") as log:

        def record(row):
            log.write(json.dumps(row) + "\n")
            log.flush()
            print(json.dumps(row), flush=True)

        record(
            dict(
                event="collected",
                collection_seconds=collection_seconds,
                train=diagnostics["train"],
                validation=diagnostics["validation"],
            )
        )
        for epoch in range(args.epochs):
            if time.monotonic() >= deadline:
                break
            model.train()
            optimization_start = time.monotonic()
            optimization_loss = 0.0
            examples = candidates = 0
            for x, mask, y, _ in batches(train, np.random.permutation(len(train[2]))):
                if time.monotonic() >= deadline:
                    break
                optimizer.zero_grad()
                loss = F.cross_entropy(model(x).masked_fill(~mask, -torch.inf), y)
                loss.backward()
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                optimizer.step()
                optimization_loss += float(loss.detach()) * len(y)
                examples += len(y)
                candidates += int(mask.sum())
            optimizer_seconds = time.monotonic() - optimization_start
            if not examples:
                break
            model.eval()
            validation = metrics(model, valid)
            train_metrics = metrics(model, train, train_probe)
            row = dict(
                event="epoch",
                epoch=epoch + 1,
                seconds=round(time.monotonic() - started, 3),
                validation=validation,
                train_probe=train_metrics,
                optimization_loss=optimization_loss / examples,
                optimizer_seconds=optimizer_seconds,
                examples_per_second=examples / max(optimizer_seconds, 1e-9),
                candidates_per_second=candidates / max(optimizer_seconds, 1e-9),
                peak_rss_mib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
            )
            history.append(row)
            record(row)
            if validation["cross_entropy"] < best:
                best = validation["cross_entropy"]
                best_epoch = epoch + 1
                torch.save(model.state_dict(), args.out / "best.pt")
                (args.out / "planner.json").write_text(model.export())
            if (
                args.eval_every
                and (epoch + 1) % args.eval_every == 0
                and time.monotonic() < deadline
            ):
                t = time.monotonic()
                summary = results_summary(evaluate(periodic_configs, model.export()))
                point = dict(
                    event="periodic_evaluation",
                    epoch=epoch + 1,
                    seconds=round(time.monotonic() - started, 3),
                    evaluation_seconds=time.monotonic() - t,
                    learned=summary,
                    baseline=results_summary(periodic_baseline),
                )
                periodic.append(point)
                record(point)
    if not best_epoch:
        raise RuntimeError("budget expired before first training epoch")
    training_seconds = time.monotonic() - started
    model.load_state_dict(torch.load(args.out / "best.pt", weights_only=True))
    model.eval()
    exported = model.export()
    probe = valid[0][:1024].copy()
    with torch.no_grad():
        expected = model(torch.from_numpy(probe)).numpy()
    actual = native.score(exported, probe)
    error = float(np.max(np.abs(actual - expected)))
    np.testing.assert_allclose(actual, expected, atol=2e-5, rtol=2e-5)
    evaluation_start = time.monotonic()
    teacher = evaluate(evaluation_configs)
    learned = evaluate(evaluation_configs, exported)
    grouped = results_by_recruiter(evaluation_configs, teacher, learned)
    report = dict(
        architecture=args.architecture,
        parameter_count=sum(p.numel() for p in model.parameters()),
        dataset_source=str(args.dataset_dir.resolve()) if args.dataset_dir else None,
        data_fingerprints=data_fingerprints,
        learning_rate=args.learning_rate,
        weight_decay=args.weight_decay,
        schema=native.SCHEMA,
        feature_names=native.FEATURE_NAMES,
        fingerprints=fingerprints,
        teacher=args.teacher,
        opponent=args.opponent,
        collect_side=args.collect_side,
        recruiters=args.recruiters,
        seed=42,
        train_configs=train_configs,
        validation_configs=valid_configs,
        evaluation_configs=evaluation_configs,
        periodic_configs=periodic_configs,
        initial_validation=initial,
        initial_train_probe=initial_train,
        final_validation=metrics(model, valid),
        final_train_probe=metrics(model, train, train_probe),
        diagnostics=diagnostics,
        train_probe_indices=train_probe.tolist(),
        best_epoch=best_epoch,
        training_seconds=training_seconds,
        collection_seconds=collection_seconds,
        final_evaluation_seconds=time.monotonic() - evaluation_start,
        inference_max_absolute_error=error,
        teacher_results=teacher,
        learned_results=learned,
        results_by_recruiter=grouped,
        history=history,
        periodic_history=periodic,
        versions=dict(torch=torch.__version__, numpy=np.__version__),
    )
    (args.out / "report.json").write_text(json.dumps(report, indent=2))
    print(
        json.dumps(
            dict(
                event="finished",
                training_seconds=round(training_seconds, 2),
                final_evaluation_seconds=report["final_evaluation_seconds"],
                teacher=results_summary(teacher),
                learned=results_summary(learned),
                by_recruiter=grouped,
                validation=report["final_validation"],
                parity_error=error,
            )
        ),
        flush=True,
    )


if __name__ == "__main__":
    main()
