"""Run bounded fits on one shared corpus while giving each active job two cores."""

import argparse, json, os, signal, subprocess, sys, time
from pathlib import Path


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, default=Path("tmp/rl/representative"))
    p.add_argument("--cnn-seconds", type=int, default=1800)
    p.add_argument("--mlp-seconds", type=int, default=1200)
    p.add_argument("--tree-seconds", type=int, default=1200)
    args = p.parse_args()
    if (
        not 0 < args.cnn_seconds <= 1800
        or not 0 < args.mlp_seconds <= 1200
        or not 0 < args.tree_seconds <= 1200
    ):
        p.error("CNN <=1800 seconds; MLP/trees <=1200; positive budgets")
    root = args.root.resolve()
    runs = root / "runs"
    runs.mkdir(parents=True, exist_ok=True)
    for split in ("train", "validation"):
        if not (root / "corpus" / split / "manifest.json").exists():
            p.error("prepare and collect the complete shared corpus first")
    env = {**os.environ, "HEX_COMMAND_DATA_ROOT": str(root / "catalog")}
    processes = {}
    handles = []
    started = time.monotonic()

    def launch(label, module, extra):
        log = (runs / f"{label}.log").open("w")
        handles.append(log)
        command = [
            sys.executable,
            "-m",
            f"hex_command_rl.{module}",
            "--source-report",
            str(root / "source.json"),
            "--dataset-dir",
            str(root / "corpus"),
            "--out",
            str(runs / label),
            "--threads",
            "2",
            "--defer-evaluation",
            *extra,
        ]
        processes[label] = subprocess.Popen(
            command,
            stdout=log,
            stderr=subprocess.STDOUT,
            env=env,
            start_new_session=True,
        )
        print(
            json.dumps(
                dict(
                    event="started",
                    model=label,
                    pid=processes[label].pid,
                    command=command,
                    seconds=time.monotonic() - started,
                )
            ),
            flush=True,
        )

    def stop(signum=None, frame=None):
        for process in processes.values():
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
        if signum is not None:
            raise SystemExit(128 + signum)

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        launch(
            "cnn",
            "spatial_train",
            ["--architecture", "cnn", "--seconds", str(args.cnn_seconds)],
        )
        launch(
            "trees",
            "trees",
            [
                "--seconds",
                str(args.tree_seconds),
                "--trees",
                "1024",
                "--depth",
                "6",
                "--early-stopping-rounds",
                "64",
            ],
        )
        last = started
        while True:
            for name, process in processes.items():
                code = process.poll()
                if code not in (None, 0):
                    raise RuntimeError(
                        f"{name} failed with {code}; see {runs / name}.log"
                    )
            if processes["trees"].poll() == 0 and "mlp" not in processes:
                launch(
                    "mlp",
                    "spatial_train",
                    ["--architecture", "mlp", "--seconds", str(args.mlp_seconds)],
                )
            if len(processes) == 3 and all(
                process.poll() == 0 for process in processes.values()
            ):
                break
            if time.monotonic() - last >= 30:
                row = dict(
                    event="progress",
                    seconds=time.monotonic() - started,
                    status={
                        name: "running" if process.poll() is None else "finished"
                        for name, process in processes.items()
                    },
                )
                (runs / "progress.json").write_text(json.dumps(row))
                print(json.dumps(row), flush=True)
                last = time.monotonic()
            time.sleep(1)
        (runs / "complete.json").write_text(
            json.dumps(
                dict(
                    seconds=time.monotonic() - started,
                    budgets=dict(
                        cnn=args.cnn_seconds,
                        mlp=args.mlp_seconds,
                        trees=args.tree_seconds,
                    ),
                ),
                indent=2,
            )
        )
        print("all fits completed", flush=True)
    finally:
        stop()
        for handle in handles:
            handle.close()


if __name__ == "__main__":
    main()
