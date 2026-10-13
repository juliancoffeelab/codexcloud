"""Run paired production games, saving full traces and separating decision CPU from recording."""

import argparse
import collections
import csv
import gzip
import hashlib
import itertools
import json
import os
import pathlib
import resource
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def configurations(mode, first, seeds):
    data = ROOT / "rust/data/simulation"
    enemies = ["classic", "rush", "mixed"]
    if mode == "campaign":
        content = json.loads((data / "campaign.json").read_text())
        for seed, campaign, diplomacy, enemy in itertools.product(
            range(first, first + seeds),
            content["campaigns"],
            content["diplomacy"],
            enemies,
        ):
            yield dict(
                seed=seed,
                campaign=campaign,
                diplomacy=diplomacy,
                enemy=enemy,
                maxRounds=30,
                recruitLimit=1,
            )
    else:
        armies = [r["id"] for r in json.loads((data / "rosters.json").read_text())]
        maps = [
            m["id"] for m in json.loads((data / "arena.json").read_text())["landscapes"]
        ]
        for seed, left, right, landscape, enemy, initiative in itertools.product(
            range(first, first + seeds),
            armies,
            armies,
            maps,
            enemies,
            ["player", "enemy"],
        ):
            yield dict(
                seed=seed,
                armies=dict(player=left, enemy=right),
                landscape=landscape,
                enemy=enemy,
                first=initiative,
                maxRounds=30,
                recruitLimit=1,
            )


def observation(result):
    config = result["config"]
    chapters = result.get("missions", [])
    phases = (
        [p for m in chapters for p in m["phases"]] if chapters else result["phases"]
    )
    names = collections.Counter(
        u["name"]
        for p in phases
        for u in p["losses"]
        if u["side"] == "player" and u.get("name")
    )
    if chapters:
        last = chapters[-1]["winner"]
        return dict(
            config=config,
            completed=result["completed"],
            defeated=last == "enemy",
            capped=last is None,
            chapters=len(chapters),
            rounds=sum(m["round"] for m in chapters),
            namedDeaths=dict(names),
            missions=[
                dict(id=m["id"], winner=m["winner"], round=m["round"], goals=m["goals"])
                for m in chapters
            ],
            phases=sum(p["side"] == "player" for p in phases),
            actions=sum(len(p["actions"]) for p in phases if p["side"] == "player"),
        )
    return dict(
        config=config,
        completed=result["winner"] == "player",
        defeated=result["winner"] == "enemy",
        capped=result["winner"] is None,
        rounds=result["rounds"],
        namedDeaths=dict(names),
        phases=sum(p["side"] == "player" for p in phases),
        actions=sum(len(p["actions"]) for p in phases if p["side"] == "player"),
    )


def group_key(o, mode):
    c = o["config"]
    return (
        (c["campaign"], c["diplomacy"], c["enemy"])
        if mode == "campaign"
        else (c["armies"]["player"], c["armies"]["enemy"], c["landscape"], c["enemy"])
    )


def run(binary, policy, configs, directory):
    directory.mkdir()
    started = time.perf_counter()
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    observations = []
    with (
        (directory / "native.log").open("wb") as log,
        gzip.open(directory / "details.jsonl.gz", "wb", compresslevel=1) as journal,
    ):
        child = subprocess.Popen(
            [str(binary), "--data-dir", str(ROOT / "rust/data"), "--stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=log,
            env={**os.environ, "HEX_COMMAND_AI_PROFILE": "1"},
        )
        try:
            for index, base in enumerate(configs):
                config = {**base, "player": policy}
                child.stdin.write(
                    json.dumps(config, separators=(",", ":")).encode() + b"\n"
                )
                child.stdin.flush()
                line = child.stdout.readline()
                if not line:
                    raise RuntimeError(
                        f"{policy} terminated at match {index + 1}: {child.poll()}"
                    )
                result = json.loads(line)
                if result["config"] != config:
                    raise ValueError("Native configuration mismatch")
                journal.write(line)
                observations.append(observation(result))
                if (index + 1) % 100 == 0:
                    print(f"{policy}: {index + 1}/{len(configs)}", flush=True)
                    (directory / "progress.json").write_text(
                        json.dumps(
                            dict(
                                status="running",
                                completed=len(observations),
                                expected=len(configs),
                            )
                        )
                    )
            child.stdin.close()
            if child.wait() != 0:
                raise RuntimeError(f"{policy} failed: see {directory}/native.log")
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait()
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    profiles = [
        json.loads(line[len("AI_PROFILE ") :])
        for line in (directory / "native.log").read_text().splitlines()
        if line.startswith("AI_PROFILE ")
    ]
    if len(profiles) != 1:
        raise ValueError("Native decision profile missing")
    metrics = dict(
        commandSeconds=time.perf_counter() - started,
        processCpuSeconds=after.ru_utime
        + after.ru_stime
        - before.ru_utime
        - before.ru_stime,
        decisionProfile=profiles[0],
        games=len(observations),
    )
    (directory / "metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")
    (directory / "outcomes.json").write_text(
        json.dumps(observations, separators=(",", ":")) + "\n"
    )
    (directory / "progress.json").write_text(
        json.dumps(
            dict(status="complete", completed=len(observations), expected=len(configs))
        )
    )
    return observations, metrics


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        type=pathlib.Path,
        default=ROOT / "rust/target/release/hex-command-sim",
    )
    parser.add_argument("--mode", choices=["arena", "campaign"], required=True)
    parser.add_argument("--policies", default="priority")
    parser.add_argument("--seed", type=int, default=61)
    parser.add_argument("--seeds", type=int, default=50)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()
    policies = args.policies.split(",")
    if args.seeds < 1 or args.seed < 0:
        parser.error("Seeds must be positive and first seed nonnegative")
    if len(policies) != len(set(policies)) or any(
        p not in ["priority", "battler"] for p in policies
    ):
        parser.error("Select unique priority/battler policies")
    if args.mode == "campaign" and "battler" in policies:
        parser.error("Battler is a battle-only policy")
    args.out.mkdir(parents=True, exist_ok=False)
    configs = list(configurations(args.mode, args.seed, args.seeds))
    fingerprint = json.loads(
        subprocess.check_output(
            [
                str(args.binary.resolve()),
                "--data-dir",
                str(ROOT / "rust/data"),
                "--fingerprint",
            ]
        )
    )

    metadata = dict(
        binarySha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(),
        mode=args.mode,
        firstSeed=args.seed,
        seeds=args.seeds,
        gamesPerPolicy=len(configs),
        policies=["mixed", *policies],
        fingerprints=fingerprint,
        sourceCommit=subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        workers=1,
        rawFormat="complete-native-results-jsonl-gzip",
        limits=dict(rounds=30, recruits=1),
        campaignRetries=0,
    )
    (args.out / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    runs = {
        p: run(args.binary.resolve(), p, configs, args.out / p)
        for p in ["mixed", *policies]
    }
    baseline = runs["mixed"][0]
    rows = []
    for policy in policies:
        grouped = collections.defaultdict(list)
        for old, new in zip(baseline, runs[policy][0]):
            assert {k: v for k, v in old["config"].items() if k != "player"} == {
                k: v for k, v in new["config"].items() if k != "player"
            }
            grouped[group_key(new, args.mode)].append((old, new))
        for key, pairs in sorted(grouped.items()):
            old = [p[0] for p in pairs]
            new = [p[1] for p in pairs]
            row = dict(
                policy=policy,
                stratum=key,
                games=len(pairs),
                mixedCompleted=sum(o["completed"] for o in old),
                completed=sum(o["completed"] for o in new),
                mixedDefeated=sum(o["defeated"] for o in old),
                defeated=sum(o["defeated"] for o in new),
                mixedCaps=sum(o["capped"] for o in old),
                caps=sum(o["capped"] for o in new),
                pairedImproved=sum(
                    not a["completed"] and b["completed"] for a, b in pairs
                ),
                pairedRegressed=sum(
                    a["completed"] and not b["completed"] for a, b in pairs
                ),
                mixedRounds=sum(o["rounds"] for o in old),
                rounds=sum(o["rounds"] for o in new),
                mixedNamedDeaths=dict(
                    sum(
                        (collections.Counter(o["namedDeaths"]) for o in old),
                        collections.Counter(),
                    )
                ),
                namedDeaths=dict(
                    sum(
                        (collections.Counter(o["namedDeaths"]) for o in new),
                        collections.Counter(),
                    )
                ),
            )
            if args.mode == "campaign":
                row.update(
                    mixedChapters=sum(o["chapters"] for o in old),
                    chapters=sum(o["chapters"] for o in new),
                )
            rows.append(row)
    report = dict(
        metadata=metadata,
        metrics={p: m for p, (_, m) in runs.items()},
        rows=rows,
        observedCompletionRegressions=[
            r for r in rows if r["completed"] < r["mixedCompleted"]
        ],
    )
    (args.out / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    with (args.out / "strata.csv").open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(
            {
                k: json.dumps(v, sort_keys=True)
                if isinstance(v, (dict, tuple, list))
                else v
                for k, v in row.items()
            }
            for row in rows
        )
    lines = [
        "# Paired policy comparison",
        "",
        f"{len(configs)} matched {args.mode} configurations per policy; seeds {args.seed}–{args.seed + args.seeds - 1}; 30 rounds, one recruit. Complete traces are saved under each policy.",
        "",
        "| Policy | Completed | Defeats | Caps | Player decision seconds | Player phases |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for policy, (outcomes, metrics) in runs.items():
        decision = sum(
            r["seconds"]
            for r in metrics["decisionProfile"]["rows"]
            if r["side"] == "player" and r["policy"] == policy
        )
        lines.append(
            f"| {policy} | {sum(o['completed'] for o in outcomes)} | {sum(o['defeated'] for o in outcomes)} | {sum(o['capped'] for o in outcomes)} | {decision:.6f} | {sum(o['phases'] for o in outcomes)} |"
        )
    lines += [
        "",
        "Decision timing excludes engine execution and serialization. Native processes run sequentially with one worker. Game duration and decision CPU are reported separately; different outcomes can change the number and difficulty of decisions. Read strata.csv for every recorded stratum.",
        "",
        f"Observed strata with lower completion: {len(report['observedCompletionRegressions'])}.",
    ]
    (args.out / "summary.md").write_text("\n".join(lines) + "\n")
    print(
        json.dumps(
            dict(
                output=str(args.out),
                gamesPerPolicy=len(configs),
                regressedStrata=len(report["observedCompletionRegressions"]),
            )
        )
    )


if __name__ == "__main__":
    main()
