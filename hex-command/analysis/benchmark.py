"""Benchmark complete independent analyzers on identical saved journals.

Build Talon and install the Polars environment first. This runner excludes
compilation and dependency setup, and includes process startup, journal I/O,
checksums, analysis and all report output. CSV equality is checked separately.
"""

import argparse
from collections import Counter
import csv
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
from time import perf_counter

ROOT = Path(__file__).resolve().parents[1]


def parsed(value):
    if value == "":
        return None
    if value in ("True", "False"):
        return value == "True"
    try:
        return json.loads(value)
    except (ValueError, TypeError):
        return value


def normalized(value):
    if isinstance(value, dict):
        return {k: normalized(v) for k, v in sorted(value.items())}
    if isinstance(value, list):
        return [normalized(v) for v in value]
    if isinstance(value, (float, int)) and not isinstance(value, bool):
        return round(float(value), 7)
    return value


def tables(directory):
    result = {}
    for path in directory.glob("*.csv"):
        with path.open() as stream:
            rows = [
                {k: parsed(v) for k, v in row.items()} for row in csv.DictReader(stream)
            ]
        result[path.stem] = Counter(
            json.dumps(normalized(row), sort_keys=True) for row in rows
        )
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--talon-binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--unit-usage", action="store_true")
    parser.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    report = args.report.resolve()
    args.out.mkdir(parents=True, exist_ok=True)
    commands = {
        "legacy": [
            sys.executable,
            str(ROOT / ".agents/skills/combat-simulation/scripts/report-combat.py"),
        ],
        "talon": [str(args.talon_binary.resolve())],
        "polars": [sys.executable, str(ROOT / "analysis/polars/main.py")],
    }
    samples = {name: [] for name in commands}
    environment = {**os.environ, "POLARS_MAX_THREADS": str(args.threads)}
    names = list(commands)
    for i in range(args.repeats):
        for name in names[i % len(names) :] + names[: i % len(names)]:
            output = args.out / name
            command = commands[name] + [str(report), "--out", str(output)]
            if args.unit_usage:
                command += ["--unit-usage"]
            if args.baseline:
                command += ["--baseline", str(args.baseline.resolve())]
            start = perf_counter()
            with (args.out / f"{name}-{i}.log").open("w") as stream:
                process = subprocess.Popen(
                    command, stdout=stream, stderr=subprocess.STDOUT, env=environment
                )
                _, status, usage = os.wait4(process.pid, 0)
                process.returncode = os.waitstatus_to_exitcode(status)
            if process.returncode:
                raise RuntimeError(
                    f"{name} failed; inspect {args.out / f'{name}-{i}.log'}"
                )
            item = dict(
                processSeconds=perf_counter() - start,
                reportSeconds=json.loads((output / "timings.json").read_text())[
                    "reportSeconds"
                ],
                peakRssMiB=usage.ru_maxrss / 1024,
                cpuSeconds=usage.ru_utime + usage.ru_stime,
            )
            samples[name].append(item)
            print(json.dumps(dict(engine=name, repetition=i, **item)), flush=True)
    expected = tables(args.out / "legacy")
    for name in ("talon", "polars"):
        actual = tables(args.out / name)
        differences = [
            k
            for k in expected.keys() | actual.keys()
            if expected.get(k) != actual.get(k)
        ]
        if differences:
            raise AssertionError(f"{name} tables differ: {differences}")
    summary = {
        name: dict(
            processMedianSeconds=statistics.median(s["processSeconds"] for s in rows),
            reportMedianSeconds=statistics.median(s["reportSeconds"] for s in rows),
            peakRssMedianMiB=statistics.median(s["peakRssMiB"] for s in rows),
        )
        for name, rows in samples.items()
    }
    result = dict(
        summary=summary,
        samples=samples,
        csvParity=True,
        csvTables=len(expected),
        unitUsage=args.unit_usage,
        report=str(report),
        threads=args.threads,
        measurement="complete independent processes; imports, journal reads, checksums, analysis and rendering included; build/setup excluded",
    )
    (args.out / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
