"""Strict saved-report regression checking with vectorized outcome alignment."""

import argparse
import json
from pathlib import Path
from time import perf_counter
import polars as pl
from journal import read_json, canonical, read_bytes, decode

RUN_METADATA = (
    "fingerprint",
    "sourceCommit",
    "elapsedSeconds",
    "detailFormat",
    "turnTiming",
)
OUTCOME_METADATA = ("elapsedMs", "detail")


def without(value, keys):
    return {k: v for k, v in value.items() if k not in keys}


def compare(baseline, current):
    start = perf_counter()
    a, b = read_json(baseline), read_json(current)
    if a.get("status") != "complete" or b.get("status") != "complete":
        raise ValueError("Both reports must be complete")

    def frame(report):
        return pl.DataFrame(
            dict(
                key=[canonical([o["kind"], o["config"]]) for o in report["outcomes"]],
                payload=[
                    canonical(without(o, OUTCOME_METADATA)) for o in report["outcomes"]
                ],
            ),
            schema={"key": pl.String, "payload": pl.String},
        ).with_row_index("position")

    left, right = frame(a), frame(b)
    if left["key"].n_unique() != left.height or right["key"].n_unique() != right.height:
        raise ValueError("Duplicate match configuration")
    if left.height != right.height:
        raise ValueError("Match configurations differ")
    joined = left.join(
        right,
        on="key",
        how="left",
        validate="1:1",
        maintain_order="left",
        suffix="_right",
    )
    if joined["position_right"].null_count():
        raise ValueError("Match configurations differ")
    if joined.filter(pl.col("position") != pl.col("position_right")).height:
        raise ValueError("Outcome array order differs")
    if canonical(without(a, (*RUN_METADATA, "outcomes"))) != canonical(
        without(b, (*RUN_METADATA, "outcomes"))
    ):
        raise ValueError("Report aggregates or configuration differ")
    if joined.filter(pl.col("payload") != pl.col("payload_right")).height:
        raise ValueError("Compact gameplay differs")
    identical = 0
    for old, now in zip(a["outcomes"], b["outcomes"]):
        x = read_bytes(baseline.parent, old["detail"])
        y = read_bytes(current.parent, now["detail"])
        left_detail = decode(x, old["detail"]["file"])
        if x == y:
            identical += 1
        elif canonical(left_detail) != canonical(decode(y, now["detail"]["file"])):
            raise ValueError("Detailed gameplay differs")
    return dict(
        equal=True,
        matches=left.height,
        byteIdenticalDetails=identical,
        outcomeOrderEqual=True,
        comparison="all gameplay fields and ordered arrays; JSON object key order ignored",
        excludedRunMetadata=RUN_METADATA,
        excludedOutcomeMetadata=OUTCOME_METADATA,
        baselineWallSeconds=a["elapsedSeconds"],
        currentWallSeconds=b["elapsedSeconds"],
        comparisonWallSeconds=perf_counter() - start,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    result = compare(args.baseline, args.current)
    text = json.dumps(result, indent=2) + "\n"
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text)
    print(text, end="")


if __name__ == "__main__":
    main()
