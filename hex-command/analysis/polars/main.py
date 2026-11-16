"""Independent Polars implementation of the complete saved-run analyzer."""

import argparse
import json
from pathlib import Path
from time import perf_counter
import analysis
import report
import store


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path)
    parser.add_argument("--baseline", action="append", default=[], type=Path)
    parser.add_argument("--out", required=True, type=Path)
    for flag in [
        "unit-usage",
        "battlefield-presence",
        "support-effects",
        "battlefield-value",
        "leader-audit",
    ]:
        parser.add_argument("--" + flag, action="store_true")
    args = parser.parse_args()
    start = perf_counter()
    current = store.load(
        args.inputs,
        unit_usage=args.unit_usage,
        presence=args.battlefield_presence,
        support=args.support_effects,
        value=args.battlefield_value,
        leaders=args.leader_audit,
    )
    previous = store.load(
        args.baseline, presence=args.unit_usage, value=args.unit_usage
    )
    tables = analysis.build(
        current,
        previous,
        unit_usage=args.unit_usage,
        presence=args.battlefield_presence,
        support=args.support_effects,
        value=args.battlefield_value,
        leaders=args.leader_audit,
    )
    # The single table-to-object boundary is final presentation.
    tables = {
        name: value.to_dicts() if hasattr(value, "to_dicts") else value
        for name, value in tables.items()
    }
    report.write(args.out, current, previous, tables, perf_counter() - start)
    seconds = perf_counter() - start
    (args.out / "timings.json").write_text(
        json.dumps(
            dict(reportSeconds=seconds, physicalMatches=len(current["outcomes"])),
            indent=2,
        )
        + "\n"
    )
    print(
        f"{len(current['outcomes'])} physical matches; {seconds:.3f}s. {args.out}/summary.md"
    )


if __name__ == "__main__":
    main()
