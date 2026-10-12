"""Export all checksummed gameplay records from a report to JSONL in schedule order.

uv run --with pyarrow scripts/export-simulation-details.py REPORT.json OUTPUT.jsonl
The Arrow dependency is loaded only for Arrow journals. No matches are executed.
"""
import argparse
import json
from pathlib import Path
from importlib.util import module_from_spec, spec_from_file_location
from simulation_journal import decode_detail


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    spec = spec_from_file_location("compare_reports", Path(__file__).with_name("compare-simulation-reports.py"))
    compare = module_from_spec(spec)
    spec.loader.exec_module(compare)
    report = json.loads(args.report.read_text())
    paths = {args.report.resolve(), *(args.report.parent / outcome["detail"]["file"] for outcome in report["outcomes"])}
    if args.output.resolve() in {path.resolve() for path in paths}:
        raise ValueError("Export must not overwrite the report or its journals")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = args.output.with_name(args.output.name + ".tmp")
    readers = {}
    try:
        with temporary.open("w") as output:
            for outcome in report["outcomes"]:
                reference = outcome["detail"]
                raw = compare.read_detail(args.report.parent, reference, readers)
                value = decode_detail(raw, reference["file"])
                output.write(json.dumps(value, separators=(",", ":"), allow_nan=False) + "\n")
        temporary.replace(args.output)
    finally:
        for reader in readers.values():
            reader.close()
    print(f"Saved {len(report['outcomes'])} complete gameplay records to {args.output}")


if __name__ == "__main__":
    main()
