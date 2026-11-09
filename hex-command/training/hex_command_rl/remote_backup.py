"""Export run-local recovery snapshots; Git publication is explicitly opt-in."""

import argparse
import fcntl
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time
from .recovery import atomic_json, resolve


def publish(destination, expected_branch):
    if not expected_branch or expected_branch in ("main", "master"):
        raise ValueError("publication requires an explicit feature branch")
    branch = subprocess.check_output(
        ["git", "branch", "--show-current"], text=True
    ).strip()
    if branch != expected_branch:
        raise ValueError(f"expected branch {expected_branch}, found {branch}")
    subprocess.run(["git", "add", "--", str(destination)], check=True)
    changed = subprocess.run(
        ["git", "diff", "--cached", "--quiet", "--", str(destination)]
    ).returncode
    if changed:
        # --only preserves unrelated staged changes in the caller's index.
        subprocess.run(
            [
                "git",
                "commit",
                "--only",
                "-m",
                "Back up native training recovery state",
                "--",
                str(destination),
            ],
            check=True,
        )
    subprocess.run(["git", "push", "origin", "HEAD"], check=True)


def backup(root, destination=None, expected_branch=None):
    root = Path(root)
    control = json.loads((root / "control.json").read_text())
    run = root / control["phase"]
    if not (run / "latest.json").exists():
        return
    latest = resolve(run)
    best_pointer = (
        "handoff-best.json" if (run / "handoff-best.json").exists() else "best.json"
    )
    best = resolve(run, best_pointer) if (run / best_pointer).exists() else latest
    destination = Path(destination) if destination is not None else root / "recovery"
    if (destination / "experiment.json").exists() or (
        destination / "report-manifest.json"
    ).exists():
        raise ValueError(
            "choose a recovery destination outside the completed experiment archive"
        )
    destination.mkdir(parents=True, exist_ok=True)
    with (root / "git.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)

        def copy(source, name):
            temporary = destination / (name + ".tmp")
            shutil.copyfile(source, temporary)
            temporary.replace(destination / name)

        for name, folder in [("latest", latest), ("best", best)]:
            copy(folder / "resume.pt", name + ".pt")
            copy(folder / "planner.json", name + "-planner.json")
        if (root / "posttune/handoff-best.json").exists():
            selected = resolve(root / "posttune", "handoff-best.json")
            copy(selected / "resume.pt", "posttune-selected.pt")
            copy(root / "posttune/handoff-complete.json", "posttune-selection.json")
        copy(root / "workload.json", "workload.json")
        for phase in ["posttune", "rl"]:
            metrics = root / phase / "metrics.jsonl"
            if metrics.exists():
                temporary = destination / (phase + "-metrics.tmp")
                temporary.write_bytes(gzip.compress(metrics.read_bytes(), mtime=0))
                temporary.replace(destination / (phase + "-metrics.jsonl.gz"))
            parameters = root / phase / "parameters.json"
            if parameters.exists():
                copy(parameters, phase + "-parameters.json")
        for source, name in [
            (root / "summary.json", "summary.json"),
            (root / "final-scores.png", "final-scores.png"),
            (root / "posttune/curves.png", "posttune-curves.png"),
            (root / "rl/curves.png", "rl-curves.png"),
            (root / "rl/selection-complete.json", "rl-selection.json"),
        ]:
            if source.exists():
                copy(source, name)
        files = {
            p.name: dict(
                bytes=p.stat().st_size,
                sha256=hashlib.sha256(p.read_bytes()).hexdigest(),
            )
            for p in destination.iterdir()
            if p.is_file() and p.name != "manifest.json"
        }
        atomic_json(
            destination / "manifest.json",
            dict(
                phase=control["phase"],
                run=str(root.resolve()),
                workload_sha256=json.loads((root / "workload.json").read_text())[
                    "workload_sha256"
                ],
                latest=json.loads((latest / "manifest.json").read_text()),
                best=json.loads((best / "manifest.json").read_text()),
                backup_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                files=files,
            ),
        )
        if expected_branch is not None:
            publish(destination, expected_branch)
        atomic_json(
            root / "backup-status.json",
            dict(
                ok=True,
                phase=control["phase"],
                time=time.time(),
                latest=str(latest),
                best=str(best),
                destination=str(destination),
                published=expected_branch is not None,
            ),
        )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument(
        "--destination",
        type=Path,
        help="Defaults to this run's local recovery directory",
    )
    parser.add_argument(
        "--branch", help="Opt in to Git publication on this exact feature branch"
    )
    parser.add_argument("--interval", type=int, default=300)
    args = parser.parse_args()
    if args.interval < 1:
        parser.error("interval must be positive")
    if args.branch and args.destination is None:
        parser.error("Git publication requires an explicit destination")
    while True:
        try:
            backup(args.root, args.destination, args.branch)
        except Exception as error:
            atomic_json(
                args.root / "backup-status.json",
                dict(ok=False, error=repr(error), time=time.time()),
            )
            print(repr(error), flush=True)
        if (args.root / "finished.json").exists():
            break
        time.sleep(args.interval)


if __name__ == "__main__":
    main()
