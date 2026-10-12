"""Verify complete saved gameplay, ignoring only JSON object key order and run metadata.

python3 scripts/compare-simulation-reports.py BASELINE/report.json CURRENT/report.json

Unlike a columnar projection, this checks missing-versus-null fields, ordered
arrays, endpoint hashes, complete commands/states and each journal checksum.
It does not execute matches. Only one pair of detail records is held at a time.
"""

import argparse
import hashlib
import json
from pathlib import Path
from time import perf_counter

RUN_METADATA = ("fingerprint", "sourceCommit", "elapsedSeconds")
OUTCOME_METADATA = ("elapsedMs", "detail")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def without(value, fields):
    return {key: item for key, item in value.items() if key not in fields}


def outcomes(report):
    result = {}
    for outcome in report["outcomes"]:
        key = canonical([outcome["kind"], outcome["config"]])
        if key in result:
            raise ValueError(f"Duplicate match configuration: {key}")
        result[key] = outcome
    return result


def read_detail(directory, reference, readers):
    name = reference["file"]
    if Path(name).name != name or name in ("", ".", ".."):
        raise ValueError("Detail reference must name an adjacent journal")
    if reference["offset"] < 0 or reference["bytes"] <= 0:
        raise ValueError("Invalid detail byte range")
    path = directory / name
    if path not in readers:
        readers[path] = path.open("rb")
    reader = readers[path]
    reader.seek(reference["offset"])
    data = reader.read(reference["bytes"])
    if len(data) != reference["bytes"] or hashlib.sha256(data).hexdigest() != reference["sha256"]:
        raise ValueError(f"Invalid detail checksum or byte range: {path}")
    return data


def compare(baseline, current):
    started = perf_counter()
    a, b = (json.loads(path.read_text()) for path in (baseline, current))
    if a.get("status") != "complete" or b.get("status") != "complete":
        raise ValueError("Both reports must be complete")
    left, right = outcomes(a), outcomes(b)
    if left.keys() != right.keys():
        raise ValueError("Match configurations differ")
    if canonical(without(a, (*RUN_METADATA, "outcomes"))) != canonical(without(b, (*RUN_METADATA, "outcomes"))):
        raise ValueError("Report aggregates or configuration differ")
    readers = {}
    byte_identical = 0
    try:
        for key, outcome in left.items():
            other = right[key]
            if canonical(without(outcome, OUTCOME_METADATA)) != canonical(without(other, OUTCOME_METADATA)):
                raise ValueError(f"Compact gameplay differs: {key}")
            old = read_detail(baseline.parent, outcome["detail"], readers)
            new = read_detail(current.parent, other["detail"], readers)
            if old == new:
                byte_identical += 1
            elif canonical(json.loads(old)) != canonical(json.loads(new)):
                raise ValueError(f"Detailed gameplay differs: {key}")
    finally:
        for reader in readers.values():
            reader.close()
    return {
        "equal": True, "matches": len(left), "byteIdenticalDetails": byte_identical,
        "comparison": "all gameplay fields and ordered arrays; JSON object key order ignored",
        "excludedRunMetadata": RUN_METADATA, "excludedOutcomeMetadata": OUTCOME_METADATA,
        "baselineWallSeconds": a["elapsedSeconds"], "currentWallSeconds": b["elapsedSeconds"],
        "comparisonWallSeconds": perf_counter() - started,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    result = compare(args.baseline, args.current)
    output = json.dumps(result, indent=2) + "\n"
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(output)
    print(output, end="")


if __name__ == "__main__":
    main()
