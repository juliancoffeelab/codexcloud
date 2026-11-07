"""Compare planner decision CPU using frozen native planner-entry checkpoints.

--report selects source configurations, not historical journal snapshots. Capture
runs fresh matches once. Use --fixtures positions.json to reuse the exact inputs
with another binary; its engine fingerprint is recorded separately.
"""

import argparse
import gzip
import hashlib
import json
import os
import pathlib
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
FORMAT = "hex-phase-fixtures-v1"


def configurations(report, campaigns):
    """Select representative configurations; all game interpretation stays native."""
    seen = set()
    result = []
    source = json.loads(report.read_text())
    candidates = [outcome["config"] for outcome in source["outcomes"]]
    if campaigns:
        with gzip.open(campaigns, "rt") as stream:
            candidates.extend(json.loads(line)["config"] for line in stream if line.strip())
    for config in candidates:
        # One seed for each distinct matchup/route, not one fresh simulation per seed.
        key = json.dumps({k: v for k, v in config.items() if k != "seed"}, sort_keys=True)
        if key not in seen:
            seen.add(key)
            result.append(config)
    if not result:
        raise ValueError("Requested source contains no match configurations")
    return result


def capture(binary, data_dir, configs, rounds, side):
    artifacts = []
    # Independent of measurement processes: capture CPU is never counted as decisions.
    with subprocess.Popen(
        [str(binary), "--data-dir", str(data_dir), "--capture-stdio"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=None, text=True,
    ) as child:
        try:
            for config in configs:
                selected = [r for r in rounds if r <= config["maxRounds"]]
                if not selected:
                    raise ValueError("No requested capture rounds are within the source match cap")
                request = dict(config=config, rounds=selected, side=side)
                child.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
                child.stdin.flush()
                raw = child.stdout.readline()
                if not raw:
                    raise RuntimeError("Native checkpoint capture stopped before producing a result")
                artifact = json.loads(raw)
                if artifact["format"] != FORMAT or not artifact["positions"]:
                    raise ValueError("Native capture produced no requested planner-entry positions")
                artifacts.append(artifact)
            child.stdin.close()
            if child.wait() != 0:
                raise RuntimeError("Native checkpoint capture failed")
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait()
    return dict(format=FORMAT, selection=dict(rounds=rounds, side=side), captures=artifacts)


def positions(artifact):
    if artifact.get("format") != FORMAT or not artifact.get("captures"):
        raise ValueError("Expected a nonempty native phase-fixture artifact")
    result = []
    for captured in artifact["captures"]:
        if captured["format"] != FORMAT or not captured["positions"]:
            raise ValueError("Empty or unsupported native capture")
        for position in captured["positions"]:
            request = position["request"]
            if request.get("beginTurn") or request.get("advanceTurn"):
                raise ValueError("A planner-entry fixture must not repeat turn preparation")
            result.append(position)
    return result


def recruitment_budget(value):
    if value == "source":
        return None
    try:
        count = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("Use a nonnegative recruit count or 'source'") from error
    if not 0 <= count <= 1_000_000:
        raise argparse.ArgumentTypeError("Recruit count must be between 0 and 1000000")
    return count


def main():
    p = argparse.ArgumentParser(description=__doc__)
    inputs = p.add_mutually_exclusive_group(required=True)
    inputs.add_argument("--report", type=pathlib.Path)
    inputs.add_argument("--fixtures", type=pathlib.Path, help="Reuse a previously captured positions.json")
    p.add_argument("--campaigns", type=pathlib.Path, help="Additional configuration source (.jsonl.gz)")
    p.add_argument("--binary", type=pathlib.Path, default=ROOT / "rust/target/release/hex-command-sim")
    p.add_argument("--data-dir", type=pathlib.Path, default=ROOT / "rust/data")
    p.add_argument("--rounds", type=int, nargs="+", default=[1, 4, 10, 20])
    p.add_argument("--side", choices=["player", "enemy"], default="player")
    p.add_argument("--recruits", type=recruitment_budget, default=1, help="Comparison budget (default 1); source retains captured policy defaults")
    p.add_argument("--repeat", type=int, default=20)
    p.add_argument("--out", type=pathlib.Path, required=True)
    args = p.parse_args()
    if args.repeat < 1:
        p.error("--repeat must be positive")
    if not args.rounds or len(args.rounds) > 32 or len(set(args.rounds)) != len(args.rounds) or min(args.rounds) < 1:
        p.error("--rounds needs 1–32 distinct positive numbers")
    if args.fixtures and args.campaigns:
        p.error("--campaigns is a capture source and cannot be used with --fixtures")
    binary = args.binary.resolve()
    data_dir = args.data_dir.resolve()
    artifact = (json.loads(args.fixtures.read_text()) if args.fixtures else
                capture(binary, data_dir, configurations(args.report, args.campaigns), args.rounds, args.side))
    samples = positions(artifact)
    args.out.mkdir(parents=True, exist_ok=False)
    encoded = json.dumps(artifact, separators=(",", ":")) + "\n"
    (args.out / "positions.json").write_text(encoded)
    metrics = []
    for scope in ["arena", "campaign"]:
        selected = [x for x in samples if x["campaign"] == (scope == "campaign")]
        if not selected:
            continue
        for planner in ["full", "queued"]:
            policy = "balanced"
            label = f"{scope}-{planner}"
            started = time.perf_counter()
            phases = 0
            actions = 0
            with (
                (args.out / (label + ".log")).open("wb") as log,
                gzip.open(args.out / (label + ".jsonl.gz"), "wb", compresslevel=1) as output,
            ):
                child = subprocess.Popen(
                    [str(binary), "--data-dir", str(data_dir), "--phase-stdio"],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
                    env={**os.environ, "HEX_COMMAND_AI_PROFILE": "1"},
                )
                try:
                    for _ in range(args.repeat):
                        for position in selected:
                            request = {**position["request"], "policy": policy,
                                       "blocks": dict(recruiter="balanced", campaign="mission", planner=planner)}
                            if args.recruits is not None:
                                request["recruitmentLimit"] = args.recruits
                            child.stdin.write(json.dumps(request, separators=(",", ":")).encode() + b"\n")
                            child.stdin.flush()
                            raw = child.stdout.readline()
                            if not raw:
                                raise RuntimeError(f"{label} stopped; see native log")
                            value = json.loads(raw)
                            output.write(raw)
                            phases += 1
                            actions += len(value["records"]["actions"])
                    child.stdin.close()
                    if child.wait() != 0:
                        raise RuntimeError(f"{label} failed; see native log")
                finally:
                    if child.poll() is None:
                        child.terminate()
                        child.wait()
            profile = [json.loads(line[len("AI_PROFILE "):])
                       for line in (args.out / (label + ".log")).read_text().splitlines()
                       if line.startswith("AI_PROFILE ")]
            if len(profile) != 1:
                raise ValueError("Native profile missing")
            seconds = sum(row["seconds"] for row in profile[0]["rows"] if row["policy"] == policy)
            metrics.append(dict(scope=scope, policy=policy, planner=planner, positions=len(selected),
                                repetitions=args.repeat, phases=phases, actions=actions,
                                decisionSeconds=seconds, secondsPerPhase=seconds / phases,
                                clock=profile[0]["clock"], commandSeconds=time.perf_counter() - started))
    result = dict(
        method="identical frozen native planner-entry checkpoints; no Python state reconstruction or replayed turn preparation",
        recruitmentBudget="source" if args.recruits is None else args.recruits,
        fixtureSha256=hashlib.sha256(encoded.encode()).hexdigest(),
        sourceFingerprints=[c["fingerprints"] for c in artifact["captures"]],
        fingerprints=json.loads(subprocess.check_output([str(binary), "--data-dir", str(data_dir), "--fingerprint"])),
        coverage=[dict(config=c["config"], requestedRounds=c["requestedRounds"], side=c["side"], positions=[p["label"] for p in c["positions"]]) for c in artifact["captures"]],
        metrics=metrics,
    )
    (args.out / "timing.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
