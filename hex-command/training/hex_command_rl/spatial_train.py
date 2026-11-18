"""Time-bounded native CNN imitation, including fresh spatial data collection."""

import argparse, json, math, random, resource, subprocess, sys, time, uuid
from pathlib import Path
import numpy as np
import torch
from torch.nn import functional as F
import hex_command_training as native
from .data import load_dataset
from .spatial import SpatialPlanner
from .spatial_data import SpatialData, collect_spatial_dataset, batches, map_weights
from .model import Planner
from .train import batches as dense_batches
from .trees import save_json
from .train import evaluate, results_summary, results_by_recruiter
from .checkpoint import warm_start
from .recovery import atomic_torch_save


def model_batches(model, data, ids):
    ids = np.asarray(ids)
    cursor = 0
    if isinstance(model, SpatialPlanner):
        for x, board, ix, mask, y in batches(data, ids):
            selected = ids[cursor : cursor + len(y)]
            cursor += len(y)
            yield (x, board, ix), mask, y, selected
    else:
        for x, mask, y, selected in dense_batches(data.dense, ids):
            yield (x,), mask, y, selected


def measure(model, data, ids=None, balance_maps=False):
    training = model.training
    model.eval()
    loss = correct = total = nontrivial = nontrivial_correct = 0
    weighted_loss = weighted_correct = weight_sum = 0.0
    weights = map_weights(data, balance_maps)
    actions = {name: [0, 0] for name in ("end_turn", "move", "attack", "advance")}
    predicted = {name: 0 for name in actions}
    with torch.no_grad():
        for inputs, mask, y, selected_ids in model_batches(
            model, data, np.arange(len(data.dense[2])) if ids is None else ids
        ):
            x = inputs[0]
            scores = model(*inputs).masked_fill(~mask, -torch.inf)
            pred = scores.argmax(-1)
            losses = F.cross_entropy(scores, y, reduction="none")
            w = torch.from_numpy(weights[selected_ids].copy())
            total += len(y)
            correct += int((pred == y).sum())
            loss += float(losses.sum())
            weighted_loss += float((losses * w).sum())
            weighted_correct += float(((pred == y) * w).sum())
            weight_sum += float(w.sum())
            active = mask.sum(-1) > 1
            nontrivial += int(active.sum())
            nontrivial_correct += int(((pred == y) & active).sum())
            for k, name in enumerate(actions):
                selected = x[torch.arange(len(y)), y, k] == 1
                actions[name][0] += int(((pred == y) & selected).sum())
                actions[name][1] += int(selected.sum())
                predicted[name] += int((x[torch.arange(len(y)), pred, k] == 1).sum())
    model.train(training)
    return dict(
        accuracy=correct / max(total, 1),
        cross_entropy=loss / max(total, 1),
        decisions=total,
        map_balanced_accuracy=weighted_correct / max(weight_sum, 1e-12),
        map_balanced_cross_entropy=weighted_loss / max(weight_sum, 1e-12),
        nontrivial_accuracy=nontrivial_correct / max(nontrivial, 1),
        by_action=actions,
        predicted_actions=predicted,
    )


def worker(args):
    torch.set_num_threads(args.threads)
    torch.set_num_interop_threads(1)
    torch.manual_seed(42)
    np.random.seed(42)
    random.seed(42)
    started = time.monotonic()
    deadline = started + args.seconds
    source = json.loads(args.source_report.read_text())
    fingerprints = json.loads(native.fingerprints())
    if (
        source.get("fingerprints", {}).get("content", fingerprints["content"])
        != fingerprints["content"]
    ):
        raise ValueError("source catalog content differs")
    balance_maps = source.get("balance_maps", False)
    if source["collect_side"] != "player":
        raise ValueError("CNN source report must use player-side collection")
    configs = [
        source[k] for k in ("train_configs", "validation_configs", "evaluation_configs")
    ]
    seeds = [{c["seed"] for c in v} for v in configs]
    if any(seeds[i] & seeds[j] for i in range(3) for j in range(i)):
        raise ValueError("workload seeds overlap")
    log = (args.out / "metrics.jsonl").open("w")

    def record(row):
        row["seconds"] = time.monotonic() - started
        log.write(json.dumps(row) + "\n")
        log.flush()
        print(json.dumps(row), flush=True)

    data = []
    dense_equal = []
    for split, cases in zip(("train", "validation"), configs):
        path = (args.dataset_dir or args.out / "data") / split
        current = (
            SpatialData(path)
            if args.dataset_dir
            else collect_spatial_dataset(cases, path, progress=record)
        )
        manifest = current.manifest
        if (
            manifest["configs"] != cases
            or manifest["collection_side"] != "player"
            or manifest["fingerprints"]["content"] != fingerprints["content"]
        ):
            raise ValueError("spatial dataset provenance differs")
        reference_path = args.source_report.parent / "data" / split
        equal = None
        if reference_path.exists():
            reference = load_dataset(reference_path)
            for a, b in zip(current.dense[1:3], reference[1:3]):
                np.testing.assert_array_equal(a, b)
            if current.dense[0].shape != reference[0].shape:
                raise ValueError("source candidate matrix dimensions differ")
            for start in range(0, len(reference[0]), 65536):
                np.testing.assert_array_equal(
                    current.dense[0][start : start + 65536],
                    reference[0][start : start + 65536],
                )
            equal = True
        dense_equal.append(equal)
        data.append(current)
    train, valid = data
    collection_seconds = time.monotonic() - started
    record(
        dict(
            event="collected",
            collection_seconds=collection_seconds,
            train_decisions=len(train.dense[2]),
            validation_decisions=len(valid.dense[2]),
            source_dense_equal=dense_equal,
        )
    )
    model = SpatialPlanner(args.dropout) if args.architecture == "cnn" else Planner()
    checkpoint = warm_start(model, args.warm_start) if args.warm_start else None
    sample_weights = map_weights(train, balance_maps)
    optimizer = torch.optim.Adam(model.parameters(), lr=args.learning_rate)
    record(
        dict(
            event="initialization",
            warm_start=checkpoint,
            feature_names=native.FEATURE_NAMES,
            channel_names=native.SPATIAL_CHANNEL_NAMES
            if args.architecture == "cnn"
            else None,
        )
    )
    probe = np.random.default_rng(123).choice(
        len(train.dense[2]), min(1024, len(train.dense[2])), replace=False
    )
    initial = measure(model, valid, balance_maps=balance_maps)
    record(dict(event="initial", validation=initial))
    best = math.inf
    history = []
    best_epoch = 0
    for epoch in range(1, args.epochs + 1):
        if time.monotonic() >= deadline:
            break
        model.train()
        optimization_start = time.monotonic()
        loss_sum = examples = candidates = 0
        for inputs, mask, y, selected_ids in model_batches(
            model, train, np.random.permutation(len(train.dense[2]))
        ):
            if time.monotonic() >= deadline:
                break
            optimizer.zero_grad()
            scores = model(*inputs).masked_fill(~mask, -torch.inf)
            loss = (
                F.cross_entropy(scores, y, reduction="none")
                * torch.from_numpy(sample_weights[selected_ids].copy())
            ).mean()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            loss_sum += float(loss.detach()) * len(y)
            examples += len(y)
            candidates += int(mask.sum())
        if not examples:
            break
        optimizer_seconds = time.monotonic() - optimization_start
        validation = measure(model, valid, balance_maps=balance_maps)
        train_probe = measure(model, train, probe, balance_maps=balance_maps)
        row = dict(
            event="epoch",
            epoch=epoch,
            validation=validation,
            train_probe=train_probe,
            optimization_loss=loss_sum / examples,
            optimizer_seconds=optimizer_seconds,
            examples=examples,
            examples_per_second=examples / optimizer_seconds,
            candidates_per_second=candidates / optimizer_seconds,
            peak_rss_mib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
        )
        record(row)
        history.append(row)
        selection = (
            validation["map_balanced_cross_entropy"]
            if balance_maps
            else validation["cross_entropy"]
        )
        if selection < best:
            best = selection
            best_epoch = epoch
            weights = f"weights-{epoch:04d}.pt"
            model_path = f"planner-{epoch:04d}.json"
            atomic_torch_save(model.state_dict(), args.out / weights)
            save_json(args.out / model_path, json.loads(model.export()))
            save_json(
                args.out / "best.json",
                dict(
                    run_id=args.run_id,
                    epoch=epoch,
                    architecture=args.architecture,
                    balance_maps=balance_maps,
                    threads=args.threads,
                    weights=weights,
                    model=model_path,
                    validation=validation,
                    collection_seconds=collection_seconds,
                    source_dense_equal=dense_equal,
                    initial_validation=initial,
                    dropout=args.dropout if args.architecture == "cnn" else 0.0,
                    learning_rate=args.learning_rate,
                    parameter_count=sum(p.numel() for p in model.parameters()),
                    fingerprints=fingerprints,
                    dataset_source=str(
                        (args.dataset_dir or args.out / "data").resolve()
                    ),
                    warm_start=checkpoint,
                ),
            )
    log.close()
    save_json(
        args.out / "training-complete.json",
        dict(
            run_id=args.run_id,
            seconds=time.monotonic() - started,
            best_epoch=best_epoch,
            history=history,
        ),
    )


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--architecture", choices=["cnn", "mlp"], default="cnn")
    p.add_argument("--threads", type=int, default=2)
    p.add_argument("--defer-evaluation", action="store_true")
    p.add_argument("--seconds", type=float, default=1200)
    p.add_argument("--epochs", type=int, default=1000000)
    p.add_argument(
        "--source-report",
        type=Path,
        default=Path("tmp/rl/deathmatch-classic-20m/report.json"),
    )
    p.add_argument("--out", type=Path, default=Path("tmp/rl/deathmatch-classic-cnn"))
    p.add_argument("--dataset-dir", type=Path)
    p.add_argument("--dropout", type=float, default=0.1)
    p.add_argument("--learning-rate", type=float)
    p.add_argument(
        "--warm-start",
        type=Path,
        help="Current or legacy .pt state dictionary; new inputs get zero weights",
    )
    p.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    p.add_argument("--run-id", default="", help=argparse.SUPPRESS)
    args = p.parse_args()
    if args.learning_rate is None:
        args.learning_rate = 1e-4 if args.warm_start else 1e-3
    if (
        not math.isfinite(args.seconds)
        or not 0 < args.seconds <= (1800 if args.architecture == "cnn" else 1200)
        or not math.isfinite(args.dropout)
        or not 0 <= args.dropout < 1
        or not math.isfinite(args.learning_rate)
        or args.learning_rate <= 0
        or args.epochs < 1
        or not 1 <= args.threads <= 4
    ):
        p.error(
            "seconds: CNN <=1800, MLP <=1200; threads 1..4; dropout [0,1); positive rate/counts; all finite"
        )
    args.out.mkdir(parents=True, exist_ok=True)
    if args.worker:
        worker(args)
        return
    run_id = str(uuid.uuid4())
    started = time.monotonic()
    cutoff = False
    child = subprocess.Popen(
        [
            sys.executable,
            "-m",
            "hex_command_rl.spatial_train",
            "--worker",
            "--run-id",
            run_id,
            *sys.argv[1:],
        ]
    )
    try:
        code = child.wait(timeout=args.seconds)
        if code:
            raise RuntimeError(f"CNN trainer exited with {code}")
    except subprocess.TimeoutExpired:
        cutoff = True
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
    except BaseException:
        child.kill()
        child.wait()
        raise
    training_seconds = time.monotonic() - started
    if not (args.out / "best.json").exists():
        raise RuntimeError("budget ended before a completed CNN checkpoint")
    best = json.loads((args.out / "best.json").read_text())
    if best["run_id"] != run_id:
        raise RuntimeError("no CNN checkpoint from this run")
    torch.set_num_threads(args.threads)
    model = SpatialPlanner(best["dropout"]) if args.architecture == "cnn" else Planner()
    model.load_state_dict(torch.load(args.out / best["weights"], weights_only=True))
    model.eval()
    exported = (args.out / best["model"]).read_text()
    save_json(args.out / "planner.json", json.loads(exported))
    valid = SpatialData(Path(best["dataset_source"]) / "validation")
    count = min(32, len(valid.dense[2]))
    end = int(valid.dense[1][count])
    board_end = int(valid.offsets[count])
    with torch.no_grad():
        expected = np.concatenate(
            [
                model(*inputs)[mask].numpy()
                for inputs, mask, y, _ in model_batches(model, valid, np.arange(count))
            ]
        )
    actual = (
        native.score_spatial(
            exported,
            valid.dense[0][:end],
            valid.dense[1][: count + 1],
            np.asarray(valid.boards[:board_end]),
            valid.offsets[: count + 1],
            valid.shapes[:count],
            valid.indices[:end],
            valid.context_values[
                : sum(
                    len(native.ACTOR_SPATIAL_CHANNELS)
                    * int(np.prod(valid.shapes[owner]))
                    for owner, _, _ in valid.contexts
                    if owner < count
                )
            ],
            valid.contexts[valid.contexts[:, 0] < count],
        )
        if args.architecture == "cnn"
        else native.score(exported, valid.dense[0][:end])
    )
    error = float(np.max(np.abs(actual - expected)))
    np.testing.assert_allclose(actual, expected, atol=3e-5, rtol=2e-5)
    source = json.loads(args.source_report.read_text())
    evaluation_started = time.monotonic()
    teacher = (
        source["teacher_results"]
        if args.defer_evaluation
        else evaluate(source["evaluation_configs"])
    )
    learned = (
        None
        if args.defer_evaluation
        else evaluate(source["evaluation_configs"], exported)
    )
    if teacher != source["teacher_results"]:
        raise RuntimeError("teacher outcomes changed")
    history = [
        json.loads(line)
        for line in (args.out / "metrics.jsonl").read_text().splitlines()
        if json.loads(line)["event"] == "epoch"
    ]
    report = dict(
        architecture="hex_cnn" if args.architecture == "cnn" else "mlp",
        schema=native.SPATIAL_SCHEMA if args.architecture == "cnn" else native.SCHEMA,
        feature_names=native.FEATURE_NAMES,
        channel_names=native.SPATIAL_CHANNEL_NAMES
        if args.architecture == "cnn"
        else None,
        fingerprints=json.loads(native.fingerprints()),
        source_report=str(args.source_report.resolve()),
        source_fingerprints=source.get("fingerprints"),
        seed=42,
        learning_rate=best["learning_rate"],
        parameter_count=best["parameter_count"],
        dropout=best["dropout"],
        best_epoch=best["epoch"],
        warm_start=best["warm_start"],
        training_seconds=training_seconds,
        training_cutoff=cutoff,
        training_limit_seconds=args.seconds,
        collection_seconds=best["collection_seconds"],
        dataset_source=best["dataset_source"],
        source_dense_equal=best["source_dense_equal"],
        initial_validation=best["initial_validation"],
        balance_maps=best["balance_maps"],
        threads=best["threads"],
        evaluation_deferred=args.defer_evaluation,
        final_validation=measure(model, valid, balance_maps=best["balance_maps"]),
        history=history,
        inference_max_absolute_error=error,
        train_configs=source["train_configs"],
        validation_configs=source["validation_configs"],
        evaluation_configs=source["evaluation_configs"],
        teacher_results=teacher,
        learned_results=learned,
        results_by_recruiter={}
        if learned is None
        else results_by_recruiter(source["evaluation_configs"], teacher, learned),
        final_evaluation_seconds=time.monotonic() - evaluation_started,
    )
    save_json(args.out / "report.json", report)
    print(
        json.dumps(
            dict(
                event="finished",
                training_seconds=training_seconds,
                cutoff=cutoff,
                best_epoch=best["epoch"],
                validation=report["final_validation"],
                learned=None if learned is None else results_summary(learned),
                teacher=results_summary(teacher),
                by_recruiter=report["results_by_recruiter"],
                parity_error=error,
            )
        ),
        flush=True,
    )


if __name__ == "__main__":
    main()
