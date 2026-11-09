"""Choose a robust RL starting policy on a larger, separate validation panel."""

import argparse
import json
import os
from pathlib import Path
import time
from .mixed_evaluate import evaluate, reliability
from .recovery import atomic_json, read_jsonl


def candidates(run):
    result = set()
    if (run / "evaluations.jsonl").exists():
        for row in read_jsonl(run / "evaluations.jsonl"):
            result.add(row["checkpoint"])
    # Completed epochs are less sensitive to which shard was trained most recently.
    for p in (run / "checkpoints").glob("epoch-*"):
        result.add(str(p.relative_to(run)))
    if (run / "complete.json").exists():
        result.add(json.loads((run / "latest.json").read_text())["path"])
    return sorted(result, key=lambda p: int(p.rsplit("-", 1)[-1]))


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    a = p.parse_args()
    root = a.root
    run = root / "posttune"
    spec = json.loads((root / "workload.json").read_text())
    os.environ["HEX_COMMAND_DATA_ROOT"] = spec["catalog"]
    os.environ["HEX_COMMAND_EVALUATION_LOCK"] = str(root / "gameplay.lock")
    cache = run / "handoff-validation"
    cache.mkdir(exist_ok=True)
    read_jsonl(run / "handoff-results.jsonl", repair=True)
    best = None
    best_key = (-1.0, -1.0, -1.0)
    # Atomic result files are authoritative if a crash happened before logging
    # or before updating the handoff pointer.
    for path in cache.glob("*.json"):
        r = json.loads(path.read_text())
        key = tuple(r["reliability_key"])
        if key > best_key:
            best_key = key
            best = r
            folder = run / r["checkpoint"]
            m = json.loads((folder / "manifest.json").read_text())
            atomic_json(
                run / "handoff-best.json",
                dict(path=r["checkpoint"], label=m["label"], files=m["files"]),
            )
    while True:
        pending = []
        for relative in candidates(run):
            folder = run / relative
            output = cache / (folder.name + ".json")
            if not output.exists():
                pending.append((relative, folder, output))
        for relative, folder, output in pending:
            r = evaluate(spec["validation"], str(folder / "planner.json"))
            key, strata = reliability(r)
            r.update(checkpoint=relative, reliability_key=list(key), strata=strata)
            atomic_json(output, r)
            compact = {k: v for k, v in r.items() if k != "rows"}
            with (run / "handoff-results.jsonl").open("a") as f:
                f.write(json.dumps(compact) + "\n")
            if key > best_key:
                best_key = key
                best = compact
                m = json.loads((folder / "manifest.json").read_text())
                atomic_json(
                    run / "handoff-best.json",
                    dict(path=relative, label=m["label"], files=m["files"]),
                )
            print(
                json.dumps(
                    dict(
                        checkpoint=relative,
                        reliability=list(key),
                        strata=strata,
                        runs=r["runs"],
                        best=best["checkpoint"],
                    )
                ),
                flush=True,
            )
        if (
            (run / "complete.json").exists()
            and (run / "evaluation-complete.json").exists()
            and not any(
                not (cache / ((run / r).name + ".json")).exists()
                for r in candidates(run)
            )
        ):
            atomic_json(
                run / "handoff-complete.json",
                dict(
                    best=best["checkpoint"],
                    reliability=list(best_key),
                    strata=best["strata"],
                    criterion="Maximize minimum balanced result across arena/campaign × Classic/Predictor; then overall score and fewer caps",
                    validation_runs=len(spec["validation"]),
                    test_seeds_used=False,
                ),
            )
            break
        time.sleep(5)


if __name__ == "__main__":
    main()


def reliable_choice(records, tolerance=0.025):
    """Treat tiny worst-stratum differences as ties, preserving overall strength."""
    maximum = max(r["reliability_key"][0] for r in records)
    eligible = [
        r
        for r in records
        if r["reliability_key"][0] >= maximum - tolerance - 1e-12
        and (maximum <= 0 or r["reliability_key"][0] > 0)
    ]
    return max(
        eligible,
        key=lambda r: (
            r["reliability_key"][1],
            r["reliability_key"][2],
            r["reliability_key"][0],
        ),
    )


def finalize_selection(run):
    run = Path(run)
    records = [
        json.loads(p.read_text()) for p in (run / "handoff-validation").glob("*.json")
    ]
    selected = reliable_choice(records)
    folder = run / selected["checkpoint"]
    m = json.loads((folder / "manifest.json").read_text())
    atomic_json(
        run / "handoff-best.json",
        dict(path=selected["checkpoint"], label=m["label"], files=m["files"]),
    )
    atomic_json(
        run / "handoff-complete.json",
        dict(
            best=selected["checkpoint"],
            reliability=selected["reliability_key"],
            strata=selected["strata"],
            criterion="Worst stratum within 2.5 percentage points of best; then highest overall score and fewer caps; positive strata required when available",
            candidates=len(records),
            validation_runs=selected["runs"],
            test_seeds_used=False,
        ),
    )
    return selected


def refine_rl_selection(root):
    """Recheck a bounded shortlist on validation, leaving test seeds untouched."""
    root = Path(root)
    run = root / "rl"
    spec = json.loads((root / "workload.json").read_text())
    history = read_jsonl(run / "evaluations.jsonl")
    shortlist = sorted(history, key=lambda r: reliability(r)[0], reverse=True)[:5]
    shortlist += sorted(history, key=lambda r: r["score"], reverse=True)[:3]
    paths = [r["checkpoint"] for r in shortlist] + [
        history[0]["checkpoint"],
        json.loads((run / "latest.json").read_text())["path"],
    ]
    cache = run / "selection-validation"
    cache.mkdir(exist_ok=True)
    records = []
    seen = set()
    import hashlib

    for relative in dict.fromkeys(paths):
        folder = run / relative
        digest = hashlib.sha256((folder / "planner.json").read_bytes()).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        output = cache / (folder.name + ".json")
        if output.exists():
            r = json.loads(output.read_text())
        else:
            r = evaluate(spec["validation"], str(folder / "planner.json"))
            key, strata = reliability(r)
            r.update(
                checkpoint=relative,
                reliability_key=list(key),
                strata=strata,
                model_sha256=digest,
            )
            atomic_json(output, r)
        records.append(r)
        print(
            json.dumps(
                dict(
                    event="rl_selection_validation",
                    checkpoint=relative,
                    reliability=r["reliability_key"],
                    strata=r["strata"],
                )
            ),
            flush=True,
        )
    chosen = reliable_choice(records)
    folder = run / chosen["checkpoint"]
    manifest = json.loads((folder / "manifest.json").read_text())
    atomic_json(
        run / "best.json",
        dict(
            path=chosen["checkpoint"], label=manifest["label"], files=manifest["files"]
        ),
    )
    atomic_json(
        run / "selection-complete.json",
        dict(
            checkpoint=chosen["checkpoint"],
            reliability_key=chosen["reliability_key"],
            strata=chosen["strata"],
            candidates=len(records),
            validation_runs=len(spec["validation"]),
            test_seeds_used=False,
            criterion="Worst stratum within 2.5 percentage points of best; then overall score and fewer caps; starting policy eligible",
        ),
    )
    return chosen
