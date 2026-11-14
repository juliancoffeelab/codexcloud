"""Supervise a PPO-only continuation, retaining atomic recovery and validation."""

import argparse
import fcntl
import json
from pathlib import Path
import subprocess
import sys
import time
from .overnight import live_pid, resources
from .recovery import atomic_json


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    p.add_argument("--seconds", type=int, default=32400)
    p.add_argument("--warm-start", type=Path, required=True)
    p.add_argument("--seed-start", type=int)
    p.add_argument("--campaign-cap", type=float, choices=[-1.0, -1.5, -2.0], default=-2)
    a = p.parse_args()
    if not 0 < a.seconds <= 43200:
        p.error("budget must be between 1 and 43200 seconds")
    root = a.root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    lock = (root / "supervisor.lock").open("a")
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    run = root / "rl"
    run.mkdir(exist_ok=True)
    children, handles = [], []
    failures = 0
    last_health = 0
    atomic_json(
        root / "control.json",
        dict(
            phase="rl",
            budget=a.seconds,
            warm_start=str(a.warm_start.resolve()),
            campaign_cap=a.campaign_cap,
        ),
    )

    def launch(module, log, args):
        output = (root / log).open("a")
        handles.append(output)
        child = subprocess.Popen(
            [sys.executable, "-m", module, "--root", str(root), *args],
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        children.append(child)
        return child

    while True:
        for child in children:
            child.poll()
        actor = live_pid("hex_command_rl.mixed_ppo", root)
        evaluator = live_pid("hex_command_rl.mixed_evaluate", root)
        complete = run / "complete.json"
        finished = json.loads(complete.read_text()) if complete.exists() else None
        # A short smoke run can be extended on the identical workload and rewards.
        if finished and finished["elapsed"] + 1 < a.seconds:
            if evaluator:
                time.sleep(5)
                continue
            complete.unlink()
            for name in ("evaluation-complete.json", "selection-complete.json"):
                (run / name).unlink(missing_ok=True)
            finished = None
        if not actor and not finished:
            if failures >= 3:
                raise RuntimeError(
                    "PPO repeatedly failed; inspect rl.log before restarting"
                )
            args = ["--seconds", str(a.seconds), "--campaign-cap", str(a.campaign_cap)]
            args += (
                ["--resume", str(run)]
                if (run / "latest.json").exists()
                else ["--warm-start", str(a.warm_start.resolve())]
            )
            if not (run / "latest.json").exists() and a.seed_start is not None:
                args += ["--seed-start", str(a.seed_start)]
            launch("hex_command_rl.mixed_ppo", "rl.log", args)
            failures += 1
        if not evaluator and not (run / "evaluation-complete.json").exists():
            (run / "evaluation-active.json").unlink(missing_ok=True)
            launch(
                "hex_command_rl.mixed_evaluate",
                "evaluation.log",
                ["--phase", "rl", "--interval", "3600"],
            )
        if time.monotonic() - last_health >= 60:
            status = resources(root)
            status["training"] = (
                json.loads((run / "status.json").read_text())
                if (run / "status.json").exists()
                else None
            )
            atomic_json(root / "health.json", status)
            with (root / "health.jsonl").open("a") as output:
                output.write(json.dumps(status) + "\n")
            print(json.dumps(status), flush=True)
            last_health = time.monotonic()
        if finished and (run / "evaluation-complete.json").exists() and not evaluator:
            atomic_json(
                root / "supervised-complete.json",
                dict(time=time.time(), training=finished),
            )
            break
        time.sleep(5)
    for output in handles:
        output.close()


if __name__ == "__main__":
    main()
