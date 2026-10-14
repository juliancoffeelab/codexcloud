"""Compare policy decision CPU on identical saved positions, independently of game duration."""

import argparse
import copy
import gzip
import hashlib
import json
import os
import pathlib
import subprocess
import time

from simulation_journal import decode_detail

ROOT = pathlib.Path(__file__).resolve().parents[1]


def fixtures(report, campaigns):
    saved = json.loads(report.read_text())
    result = []
    seen = set()
    readers = {}
    try:
        for outcome in saved["outcomes"]:
            c = outcome["config"]
            if (
                outcome["kind"] != "arena"
                or c["player"] not in ["mixed", "balanced"]
                or c["enemy"] != c["player"]
            ):
                continue
            c = {**c, "player": "balanced", "enemy": "balanced"}
            key = (c["armies"]["player"], c["landscape"])
            cap = outcome["winner"] is None
            tag = (key, "cap" if cap else "initial")
            if tag in seen:
                continue
            seen.add(tag)
            ref = outcome["detail"]
            path = report.parent / ref["file"]
            if path not in readers:
                readers[path] = path.open("rb")
            f = readers[path]
            f.seek(ref["offset"])
            raw = f.read(ref["bytes"])
            assert hashlib.sha256(raw).hexdigest() == ref["sha256"]
            detail = decode_detail(raw, ref["file"])

            def after(index, label):
                phase = detail["phases"][index]
                snapshot = copy.deepcopy(detail["initial"])
                snapshot.update(
                    units=phase["units"],
                    round=phase["round"],
                    turn=phase["side"],
                    gold=phase["gold"],
                    winner=None,
                    lastCombat=None,
                )
                known = [
                    int(u["id"][1:])
                    for p in detail["phases"][: index + 1]
                    for u in p["before"] + p["units"] + p["losses"]
                ]
                snapshot["nextId"] = max(snapshot["nextId"], max(known, default=0) + 1)
                owners = {x["tile"]: x["owner"] for x in phase["ownership"]}
                for tile in snapshot["tiles"]:
                    tile["owner"] = owners[tile["id"]]
                result.append(
                    dict(
                        label=f"{key[0]}/{key[1]}/{label}",
                        state=snapshot,
                        arena=c,
                        seed=c["seed"],
                        goals=detail["phases"][index + 1]["objectives"],
                        advanceTurn=True,
                        diplomacy="pragmatic",
                        recruitmentLimit=1,
                        campaign=False,
                    )
                )

            if cap:
                # Replay the last recorded player phase of a capped game, using
                # the preceding enemy boundary and its actual next-phase goals.
                candidates = [
                    i
                    for i, p in enumerate(detail["phases"][:-1])
                    if p["side"] == "enemy"
                    and detail["phases"][i + 1]["side"] == "player"
                ]
                if candidates:
                    after(candidates[-1], "late-cap")
            else:
                result.append(
                    dict(
                        label=f"{key[0]}/{key[1]}/initial",
                        state=detail["initial"],
                        arena=c,
                        seed=c["seed"],
                        goals=detail["phases"][0]["objectives"],
                        advanceTurn=False,
                        beginTurn=True,
                        diplomacy="pragmatic",
                        recruitmentLimit=1,
                        campaign=False,
                    )
                )
                for desired in (4, 10, 20):
                    candidates = [
                        i
                        for i, p in enumerate(detail["phases"][:-1])
                        if p["side"] == "enemy"
                        and p["round"] >= desired
                        and detail["phases"][i + 1]["side"] == "player"
                    ]
                    if candidates:
                        after(candidates[0], f"round-{desired}")
    finally:
        for f in readers.values():
            f.close()
    if campaigns:
        seen = set()
        with gzip.open(campaigns, "rt") as f:
            for line in f:
                detail = json.loads(line)
                c = detail["config"]
                if c["diplomacy"] != "restrained":
                    continue
                for mission in detail["missions"]:
                    if mission["id"] in seen:
                        continue
                    seen.add(mission["id"])
                    result.append(
                        dict(
                            label=mission["id"],
                            state=mission["initial"],
                            seed=c["seed"],
                            goals=None,
                            advanceTurn=False,
                            diplomacy="restrained",
                            recruitmentLimit=1,
                            campaign=True,
                        )
                    )
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--report", type=pathlib.Path, required=True)
    p.add_argument("--campaigns", type=pathlib.Path)
    p.add_argument(
        "--binary",
        type=pathlib.Path,
        default=ROOT / "rust/target/release/hex-command-sim",
    )
    p.add_argument("--repeat", type=int, default=20)
    p.add_argument("--out", type=pathlib.Path, required=True)
    args = p.parse_args()
    if args.repeat < 1:
        p.error("--repeat must be positive")
    args.out.mkdir(parents=True, exist_ok=False)
    positions = fixtures(args.report, args.campaigns)
    (args.out / "positions.json").write_text(
        json.dumps(positions, separators=(",", ":")) + "\n"
    )
    metrics = []
    for scope in ["arena", "campaign"]:
        selected = [x for x in positions if x["campaign"] == (scope == "campaign")]
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
                gzip.open(
                    args.out / (label + ".jsonl.gz"), "wb", compresslevel=1
                ) as output,
            ):
                child = subprocess.Popen(
                    [
                        str(args.binary.resolve()),
                        "--data-dir",
                        str(ROOT / "rust/data"),
                        "--phase-stdio",
                    ],
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=log,
                    env={**os.environ, "HEX_COMMAND_AI_PROFILE": "1"},
                )
                try:
                    for _ in range(args.repeat):
                        for position in selected:
                            request = {
                                k: v
                                for k, v in position.items()
                                if k not in ["label", "campaign"]
                            }
                            request["policy"] = policy
                            request["blocks"] = dict(recruiter="balanced", campaign="mission", planner=planner)
                            child.stdin.write(
                                json.dumps(request, separators=(",", ":")).encode()
                                + b"\n"
                            )
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
            profile = [
                json.loads(line[len("AI_PROFILE ") :])
                for line in (args.out / (label + ".log")).read_text().splitlines()
                if line.startswith("AI_PROFILE ")
            ]
            if len(profile) != 1:
                raise ValueError("Native profile missing")
            seconds = sum(
                row["seconds"] for row in profile[0]["rows"] if row["policy"] == policy
            )
            metrics.append(
                dict(
                    scope=scope,
                    policy=policy,
                    planner=planner,
                    positions=len(selected),
                    repetitions=args.repeat,
                    phases=phases,
                    actions=actions,
                    decisionSeconds=seconds,
                    secondsPerPhase=seconds / phases,
                    clock=profile[0]["clock"],
                    commandSeconds=time.perf_counter() - started,
                )
            )
    result = dict(
        method="identical saved tactical states: initial turn-start, reconstructed mid/late arena boundaries with recorded next-phase objectives, and complete campaign chapter starts; one own phase per policy",
        fingerprints=json.loads(
            subprocess.check_output(
                [
                    str(args.binary.resolve()),
                    "--data-dir",
                    str(ROOT / "rust/data"),
                    "--fingerprint",
                ]
            )
        ),
        metrics=metrics,
    )
    (args.out / "timing.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
